use crate::error::Error;
use crate::read::group_btree::parse_group_btree;
use crate::read::io::ReadSeek;
use crate::read::local_heap::LocalHeap;
use crate::read::object::Object;
use crate::read::symbol_table::{SymbolTableEntry, SymbolTableNode};
use crate::FileReader;
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::io::{Cursor, Read, SeekFrom};

use super::data_object::parse_data_object;

bitflags! {
    pub struct LinkFlags : u8 {
        const SIZE_OF_LINK_BIT_A = 0b0000_0001;
        const SIZE_OF_LINK_BIT_B = 0b0000_0010;
        const CREATION_ORDER_FIELD_PRESENT = 0b0000_0100;
        const LINK_TYPE_FIELD_PRESENT = 0b0000_1000;
        const LINK_NAME_CHARACTER_SET_FIELD_PRESENT = 0b0001_0000;
    }
}

impl LinkFlags {
    pub fn new(bits: u8) -> Self {
        LinkFlags { bits }
    }
}

#[repr(u8)]
#[derive(Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum LinkType {
    Hard = 0,
    Soft = 1,
    External = 64,
}

#[derive(Clone, Debug)]
pub enum LinkTarget {
    Soft { name: String },
    Hard { address: u64 },
}

#[derive(Clone, Debug)]
pub struct Link {
    pub version: u8,
    pub flags: LinkFlags,
    pub name: String,
    pub target: LinkTarget,
}

impl Link {
    pub fn follow(&self, file: &mut FileReader) -> Result<Object, Error> {
        let data_address = match self.target {
            LinkTarget::Hard { address } => address,
            _ => {
                return Err(Error::OxifiveError(format!(
                    "Link '{}' is not a hard link and soft links are not yet supported",
                    self.name
                )))
            }
        };
        let data_object = parse_data_object(&mut file.input, data_address)?;
        if data_object.is_group() {
            Ok(Object::Group(data_object.as_group()))
        } else {
            Ok(Object::Dataset(data_object.as_dataset()))
        }
    }
}

#[repr(u8)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkNameEncoding {
    Ascii,
    Utf8,
}

struct SymbolTableMessage {
    btree_address: u64,
    heap_address: u64,
}

pub fn parse_symbol_table_message(
    input: &mut impl ReadSeek,
    message_cursor: &mut impl ReadSeek,
) -> Result<Vec<Link>, Error> {
    let symbol_table_message = SymbolTableMessage {
        btree_address: message_cursor.read_u64::<LittleEndian>()?,
        heap_address: message_cursor.read_u64::<LittleEndian>()?,
    };
    let mut links = vec![];
    // TODO verify value < usize
    let btree_nodes = parse_group_btree(input, symbol_table_message.btree_address)?;
    let heap = LocalHeap::read(input, symbol_table_message.heap_address)?;

    for address in btree_nodes.iter().map(|n| n.addresses.clone()).flatten() {
        input.seek(SeekFrom::Start(address))?;
        let symbol_table_node = SymbolTableNode::read(input)?;
        let mut symbol_table = vec![];
        for _ in 0..symbol_table_node.symbols {
            symbol_table.push(SymbolTableEntry::read(input)?);
        }
        for symbol_table_entry in symbol_table {
            let link_name = heap.object_name(input, symbol_table_entry.link_name_offset)?;
            match symbol_table_entry.cache_type {
                0 | 1 => {
                    links.push(Link {
                        version: 0,
                        flags: LinkFlags::empty(),
                        name: link_name.clone(),
                        target: LinkTarget::Hard {
                            address: symbol_table_entry.object_header_address,
                        },
                    });
                }
                2 => {
                    let mut scratch_cursor = Cursor::new(symbol_table_entry.scratch);
                    let offset = scratch_cursor.read_u32::<LittleEndian>()?;
                    let link_target = heap.object_name(input, offset.into())?;
                    links.push(Link {
                        version: 0,
                        flags: LinkFlags::empty(),
                        name: link_name.clone(),
                        target: LinkTarget::Soft { name: link_target },
                    });
                }
                _ => {
                    return Err(Error::OxifiveError("Unsupported cache type".to_string()));
                }
            }
        }
    }

    Ok(links)
}

pub fn parse_link_message(input: &mut Cursor<Vec<u8>>) -> Result<Link, Error> {
    let version = input.read_u8()?;
    let flags = LinkFlags::new(input.read_u8()?);
    let link_type = if flags.contains(LinkFlags::LINK_TYPE_FIELD_PRESENT) {
        LinkType::try_from(input.read_u8()?)?
    } else {
        LinkType::Hard
    };
    if flags.contains(LinkFlags::CREATION_ORDER_FIELD_PRESENT) {
        let _creation_order_field = input.read_u64::<LittleEndian>()?;
    }
    let link_name_character_set_value =
        if flags.contains(LinkFlags::LINK_NAME_CHARACTER_SET_FIELD_PRESENT) {
            input.read_u8()?
        } else {
            0
        };
    let link_name_encoding = if link_name_character_set_value == 0 {
        LinkNameEncoding::Ascii
    } else {
        LinkNameEncoding::Utf8
    };
    let size_of_length_of_link_name_bits =
        (flags & (LinkFlags::SIZE_OF_LINK_BIT_A | LinkFlags::SIZE_OF_LINK_BIT_B)).bits();
    let length_of_link_name = match size_of_length_of_link_name_bits {
        0 => input.read_u8()? as u64,
        1 => input.read_u16::<LittleEndian>()? as u64,
        2 => input.read_u32::<LittleEndian>()? as u64,
        3 => input.read_u64::<LittleEndian>()?,
        _ => {
            return Err(Error::OxifiveError(
                "Unknown size of link name bits value".to_string(),
            ))
        }
    };

    let name = {
        let mut bytes = vec![0; length_of_link_name as usize];
        input.read_exact(&mut bytes)?;
        match link_name_encoding {
            LinkNameEncoding::Ascii => std::str::from_utf8(&bytes)?.to_string(),
            LinkNameEncoding::Utf8 => std::str::from_utf8(&bytes)?.to_string(),
        }
    };

    let target = {
        match link_type {
            LinkType::Soft => {
                let length_of_soft_link_value = input.read_u16::<LittleEndian>()? as usize;
                let mut soft_link_target = vec![0; length_of_soft_link_value];
                input.read_exact(&mut soft_link_target)?;
                LinkTarget::Soft {
                    name: String::from_utf8(soft_link_target)?,
                }
            }
            LinkType::Hard => LinkTarget::Hard {
                address: input.read_u64::<LittleEndian>()?,
            },
            _ => {
                return Err(Error::OxifiveError(format!(
                    "Unsupported link type {:?}",
                    link_type
                )));
            }
        }
    };

    log::info!("Link name {}", name);
    Ok(Link {
        version,
        flags,
        name,
        target,
    })
}
