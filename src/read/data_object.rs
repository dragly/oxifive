use crate::read::data_storage::{parse_data_storage_message, DataStorage};
use crate::read::dataspace::{parse_dataspace_message, Dataspace};
use crate::read::datatype::{parse_datatype_message, Datatype};
use crate::read::filter_pipeline::{parse_filter_pipeline_message, FilterPipeline};
use crate::read::io::ReadSeek;
use crate::read::link::{parse_link_message, parse_symbol_table_message};
use crate::{
    error::Error,
    read::link::Link,
    read::message::{MessageHeaderV1, MessageHeaderV2, MessageType},
};
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use std::convert::TryFrom;
use std::{
    collections::HashMap,
    io::{Cursor, Read, Seek, SeekFrom},
};

bitflags! {
    struct ObjectHeaderFlags: u8 {
        const SIZE_OF_CHUNK_BIT_A = 0b000001;
        const SIZE_OF_CHUNK_BIT_B = 0b000010;
        const ATTRIBUTE_CREATION_ORDER_TRACKED = 0b000100;
        const ATTRIBUTE_CREATION_ORDER_INDEXED = 0b001000;
        const NON_DEFAULT_ATTRIBUTE_STORAGE_PHASE_CHANGE = 0b010000;
        const ACCESS_MODIFICATION_CHANGE_AND_BIRTH_TRACKED = 0b100000;
    }
}

#[derive(Clone, Debug)]
struct ObjectHeaderTimes {
    access: u32,
    modification: u32,
    change: u32,
    birth: u32,
}

#[derive(Clone, Debug)]
struct ObjectHeaderV1 {
    version: u8,
    reserved: u8,
    total_header_messages: u16,
    object_reference_count: u32,
    object_header_size: u32,
    padding: u32,
}

#[derive(Clone, Debug)]
struct ObjectHeader {
    signature: [u8; 4],
    version: u8,
    flags: ObjectHeaderFlags,
    times: Option<ObjectHeaderTimes>,
    size_of_chunk_0: u64, // TODO look for better way of filling out struct with all info over time...
}

#[derive(Clone, Debug)]
pub struct DataObject {
    pub links: HashMap<String, Link>,
    pub data: Vec<DataStorage>,
    pub datatypes: Vec<Datatype>,
    pub dataspaces: Vec<Dataspace>,
    pub filter_pipelines: Vec<FilterPipeline>,
}

fn parse_message(
    mut input: &mut impl ReadSeek,
    current_message_data: Vec<u8>,
    message_type: MessageType,
    data_object: &mut DataObject,
    chunks: &mut Vec<Vec<u8>>,
) -> Result<(), Error> {
    let mut current_message_cursor = Cursor::new(current_message_data);

    match message_type {
        MessageType::ObjectContinuation => {
            let offset = current_message_cursor.read_u64::<LittleEndian>()?;
            let size = current_message_cursor.read_u64::<LittleEndian>()?;
            log::info!("Cont {} {}", offset, size);
            input.seek(SeekFrom::Start(offset))?;
            let mut chunk_signature = vec![0; 4];
            input.read_exact(&mut chunk_signature)?;
            if chunk_signature != b"OCHK" {
                return Err(Error::OxifiveError(format!(
                    "Unexpected chunk signature {:?}",
                    &chunk_signature
                )));
            }
            let mut continuation_chunk = vec![0; (size - 4) as usize];
            input.read_exact(&mut continuation_chunk)?;
            chunks.push(continuation_chunk);
        }
        MessageType::Link => {
            let link = parse_link_message(&mut current_message_cursor)?;
            data_object.links.insert(link.name.clone(), link);
        }
        MessageType::DataStorage => {
            data_object
                .data
                .push(parse_data_storage_message(&mut current_message_cursor)?);
        }
        MessageType::Datatype => {
            data_object
                .datatypes
                .push(parse_datatype_message(&mut current_message_cursor)?);
        }
        MessageType::Dataspace => {
            data_object
                .dataspaces
                .push(parse_dataspace_message(&mut current_message_cursor)?);
        }
        MessageType::DataStorageFilterPipeline => {
            data_object
                .filter_pipelines
                .extend(parse_filter_pipeline_message(&mut current_message_cursor)?);
        }
        MessageType::SymbolTable => {
            let links = parse_symbol_table_message(&mut input, &mut current_message_cursor)?;
            data_object
                .links
                .extend(links.iter().map(|l| (l.name.clone(), l.clone())));
        }
        MessageType::Fillvalue => {
            // TODO this should not just be ignored
        }
        MessageType::ObjectModificationTime => {
            // TODO this should not just be ignored
        }
        MessageType::Nil => {
            // TODO this should not just be ignored
        }
        _ => {
            return Err(Error::OxifiveError(format!(
                "Unsupported message type {:?}",
                message_type
            )));
        }
    }
    Ok(())
}

fn parse_v1_objects(version_hint: u8, input: &mut impl ReadSeek) -> Result<DataObject, Error> {
    let object_header = {
        let version = version_hint;

        if version != 1 {
            return Err(Error::OxifiveError(format!(
                "Unsupported data object header version found: {}",
                version
            )));
        }

        ObjectHeaderV1 {
            version,
            reserved: input.read_u8()?,
            total_header_messages: input.read_u16::<LittleEndian>()?,
            object_reference_count: input.read_u32::<LittleEndian>()?,
            object_header_size: input.read_u32::<LittleEndian>()?,
            padding: input.read_u32::<LittleEndian>()?,
        }
    };

    log::info!("{:#?}", object_header);

    let mut chunks = vec![{
        let mut result = vec![0; object_header.object_header_size as usize];
        input.read_exact(&mut result)?;
        result
    }];
    let mut data_object = DataObject {
        links: HashMap::new(),
        data: Vec::new(),
        datatypes: Vec::new(),
        dataspaces: Vec::new(),
        filter_pipelines: Vec::new(),
    };
    let mut current_chunk_index = 0;
    while current_chunk_index < chunks.len() {
        let current_chunk = chunks[current_chunk_index].clone();
        let mut data_cursor = Cursor::new(current_chunk);
        for _ in 0..object_header.total_header_messages {
            let message_header = MessageHeaderV1 {
                // TODO verify safety of cast from u16 to u8
                message_type: MessageType::try_from(data_cursor.read_u16::<LittleEndian>()? as u8)?,
                size: data_cursor.read_u16::<LittleEndian>()?,
                flags: data_cursor.read_u8()?,
                reserved: {
                    let mut result = [0; 3];
                    data_cursor.read_exact(&mut result)?;
                    result
                },
            };
            log::info!("{:#?}", message_header);

            let current_message_data = {
                let mut result = vec![0; message_header.size as usize];
                data_cursor.read_exact(&mut result)?;
                result
            };

            parse_message(
                input,
                current_message_data,
                message_header.message_type,
                &mut data_object,
                &mut chunks,
            )?;
        }
        current_chunk_index += 1;
    }

    Ok(data_object)
}

fn parse_v2_objects(version_hint: u8, input: &mut impl ReadSeek) -> Result<DataObject, Error> {
    let object_header = {
        let signature = [
            version_hint,
            input.read_u8()?,
            input.read_u8()?,
            input.read_u8()?,
        ];
        let version = input.read_u8()?;
        let flags = ObjectHeaderFlags {
            bits: input.read_u8()?,
        };

        if version != 2 {
            return Err(Error::OxifiveError(format!(
                "Unsupported data object header version found: {}",
                version
            )));
        }

        if flags.contains(ObjectHeaderFlags::NON_DEFAULT_ATTRIBUTE_STORAGE_PHASE_CHANGE) {
            return Err(Error::OxifiveError(
                "Non-default attribute storage phase change values are not supported".to_string(),
            ));
        }

        let times =
            if flags.contains(ObjectHeaderFlags::ACCESS_MODIFICATION_CHANGE_AND_BIRTH_TRACKED) {
                Some(ObjectHeaderTimes {
                    access: input.read_u32::<LittleEndian>()?,
                    modification: input.read_u32::<LittleEndian>()?,
                    change: input.read_u32::<LittleEndian>()?,
                    birth: input.read_u32::<LittleEndian>()?,
                })
            } else {
                None
            };

        let size_of_chunk_field_bits_value = (flags
            & (ObjectHeaderFlags::SIZE_OF_CHUNK_BIT_A | ObjectHeaderFlags::SIZE_OF_CHUNK_BIT_B))
            .bits();
        let size_of_chunk_0 = match size_of_chunk_field_bits_value {
            0 => input.read_u8()? as u64,
            1 => input.read_u16::<LittleEndian>()? as u64,
            2 => input.read_u32::<LittleEndian>()? as u64,
            3 => input.read_u64::<LittleEndian>()?,
            _ => {
                return Err(Error::OxifiveError(
                    "Unknown size of chunk field bits value".to_string(),
                ))
            }
        };
        ObjectHeader {
            signature,
            version,
            flags,
            times,
            size_of_chunk_0,
        }
    };

    log::info!("{:#?}", object_header);

    // TODO verify that platform supports u64 if necessary
    let mut chunks = vec![{
        let mut result = vec![0; object_header.size_of_chunk_0 as usize];
        input.read_exact(&mut result)?;
        result
    }];

    let mut links = HashMap::new();
    let mut data = Vec::new();
    let mut datatypes = Vec::new();
    let mut dataspaces = Vec::new();
    let mut filter_pipelines = Vec::new();
    let mut current_chunk_index = 0;
    while current_chunk_index < chunks.len() {
        let current_chunk = chunks[current_chunk_index].clone();
        let current_chunk_len = current_chunk.len();
        let mut data_cursor = Cursor::new(current_chunk);
        let header_size = 4;
        while data_cursor.position() + header_size < current_chunk_len as u64 {
            let message_header = MessageHeaderV2 {
                message_type: MessageType::try_from(data_cursor.read_u8()?)?,
                size: data_cursor.read_u16::<LittleEndian>()?,
                flags: data_cursor.read_u8()?,
            };
            log::info!("{:#?}", message_header);

            if object_header
                .flags
                .contains(ObjectHeaderFlags::ATTRIBUTE_CREATION_ORDER_TRACKED)
            {
                data_cursor.seek(SeekFrom::Current(2))?;
            }

            let mut current_message_data = vec![0; message_header.size as usize];
            data_cursor.read_exact(&mut current_message_data)?;

            let mut current_message_cursor = Cursor::new(current_message_data);

            match message_header.message_type {
                MessageType::ObjectContinuation => {
                    let offset = current_message_cursor.read_u64::<LittleEndian>()?;
                    let size = current_message_cursor.read_u64::<LittleEndian>()?;
                    log::info!("Cont {} {}", offset, size);
                    input.seek(SeekFrom::Start(offset))?;
                    let mut chunk_signature = vec![0; 4];
                    input.read_exact(&mut chunk_signature)?;
                    if chunk_signature != b"OCHK" {
                        return Err(Error::OxifiveError(format!(
                            "Unexpected chunk signature {:?}",
                            &chunk_signature
                        )));
                    }
                    let mut continuation_chunk = vec![0; (size - 4) as usize];
                    input.read_exact(&mut continuation_chunk)?;
                    chunks.push(continuation_chunk);
                }
                MessageType::Link => {
                    let link = parse_link_message(&mut current_message_cursor)?;
                    links.insert(link.name.clone(), link);
                }
                MessageType::DataStorage => {
                    data.push(parse_data_storage_message(&mut current_message_cursor)?);
                }
                MessageType::Datatype => {
                    datatypes.push(parse_datatype_message(&mut current_message_cursor)?);
                }
                MessageType::Dataspace => {
                    dataspaces.push(parse_dataspace_message(&mut current_message_cursor)?);
                }
                MessageType::DataStorageFilterPipeline => {
                    filter_pipelines
                        .extend(parse_filter_pipeline_message(&mut current_message_cursor)?);
                }
                _ => {
                    // TODO handle all message types
                }
            }
        }
        current_chunk_index += 1;
    }

    Ok(DataObject {
        links,
        data,
        datatypes,
        dataspaces,
        filter_pipelines,
    })
}

pub fn parse_data_object(input: &mut impl ReadSeek, offset: u64) -> Result<DataObject, Error> {
    input.seek(SeekFrom::Start(offset))?;
    let version_hint = input.read_u8()?;
    log::info!("Version hint: {:#?}", version_hint);
    if version_hint == 1 {
        parse_v1_objects(version_hint, input)
    } else if version_hint == b'O' {
        parse_v2_objects(version_hint, input)
    } else {
        return Err(Error::OxifiveError(format!(
            "Unsupported data object version hint found: {}",
            version_hint
        )));
    }
}
