use crate::error::Error;
use std::convert::TryFrom;

use byteorder::{LittleEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;
use std::io::{Read, Seek, SeekFrom};

#[repr(u8)]
#[derive(Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum BTreeNodeType {
    Group = 0,
    RawDataChunk = 1,
}

#[derive(Clone, Debug)]
pub struct BTreeNodeKey {
    pub chunk_size: u32,
    pub filter_mask: u32,
    pub chunk_offsets: Vec<u64>,
    pub chunk_address: u64,
}

#[derive(Clone, Debug)]
pub struct BTreeNode {
    pub node_type: BTreeNodeType,
    pub node_level: u8,
    pub entries_used: u16,
    pub left_sibling: u64,
    pub right_sibling: u64,
    pub keys: Vec<BTreeNodeKey>,
    //pub addresses: Vec<u64>,
}

pub fn parse_node(
    input: &mut (impl Read + Seek + ?Sized),
    offset: u64,
    dimensions: usize,
) -> Result<BTreeNode, Error> {
    input.seek(SeekFrom::Start(offset))?;
    let signature = {
        let mut bytes = [0; 4];
        input.read_exact(&mut bytes)?;
        String::from_utf8(bytes.into())?
    };
    if signature != "TREE" {
        return Err(Error::OxifiveError(format!(
            "Wrong BTreeNode signature: {}",
            signature
        )));
    }
    let node_type = BTreeNodeType::try_from(input.read_u8()?)?;
    if node_type != BTreeNodeType::RawDataChunk {
        return Err(Error::OxifiveError(
            "Only raw data chunk nodes are supported".to_string(),
        ));
    }
    let node_level = input.read_u8()?;
    let entries_used = input.read_u16::<LittleEndian>()?;
    let left_sibling = input.read_u64::<LittleEndian>()?;
    let right_sibling = input.read_u64::<LittleEndian>()?;

    let mut keys = vec![];
    //let mut addresses = vec![];

    for _ in 0..entries_used {
        let chunk_size = input.read_u32::<LittleEndian>()?;
        let filter_mask = input.read_u32::<LittleEndian>()?;
        let mut chunk_offsets = vec![];
        for _ in 0..dimensions {
            chunk_offsets.push(input.read_u64::<LittleEndian>()?);
        }
        let chunk_address = input.read_u64::<LittleEndian>()?;
        keys.push(BTreeNodeKey {
            chunk_size,
            filter_mask,
            chunk_offsets,
            chunk_address,
        });
        //addresses.push(chunk_address);
    }

    Ok(BTreeNode {
        node_type,
        node_level,
        entries_used,
        left_sibling,
        right_sibling,
        keys,
        //addresses,
    })
}
