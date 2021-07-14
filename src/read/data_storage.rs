use crate::error::Error;
use byteorder::{LittleEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::io::{Cursor, Read, Seek};

#[repr(u8)]
#[derive(Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
enum LayoutClass {
    Compact = 0,
    Contiguous = 1,
    Chunked = 2,
}

#[derive(Clone, Debug)]
pub enum DataStorage {
    Contiguous { address: u64, size: u64 },
    Chunked { chunk_shape: Vec<u32>, address: u64 },
}

fn parse_chunked(input: &mut (impl Read + Seek)) -> Result<DataStorage, Error> {
    let dimensions = input.read_u8()? as usize;
    let address = input.read_u64::<LittleEndian>()?;
    let mut chunk_shape = vec![];
    for _ in 0..dimensions {
        chunk_shape.push(input.read_u32::<LittleEndian>()?);
    }
    if dimensions != chunk_shape.len() {
        return Err(Error::OxifiveError(format!(
            "Wrong number of dimensions {} compared to chunks {}",
            dimensions,
            chunk_shape.len()
        )));
    }
    Ok(DataStorage::Chunked {
        chunk_shape,
        address,
    })
}

fn parse_contiguous(input: &mut (impl Read + Seek)) -> Result<DataStorage, Error> {
    Ok(DataStorage::Contiguous {
        address: input.read_u64::<LittleEndian>()?,
        size: input.read_u64::<LittleEndian>()?,
    })
}

pub fn parse_data_storage_message(input: &mut Cursor<Vec<u8>>) -> Result<DataStorage, Error> {
    let version = input.read_u8()?;
    if version != 3 {
        return Err(Error::OxifiveError(format!(
            "Unsupported data storage version {}",
            version
        )));
    }
    let layout_class = LayoutClass::try_from(input.read_u8()?)?;
    match layout_class {
        LayoutClass::Contiguous => parse_contiguous(input),
        LayoutClass::Chunked => parse_chunked(input),
        _ => {
            return Err(Error::OxifiveError(format!(
                "Only chunked data storage is supported, found layout_class {:?}",
                layout_class
            )));
        }
    }
}
