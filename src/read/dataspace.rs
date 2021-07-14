use crate::error::Error;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

#[derive(Clone, Debug)]
pub struct Dataspace {
    pub shape: Vec<u64>,
}

pub fn parse_dataspace_message(input: &mut impl Read) -> Result<Dataspace, Error> {
    let version = input.read_u8()?;
    let dimensions = match version {
        1 => {
            let dimensionality = input.read_u8()?;
            let _flags = input.read_u8()?;
            let _reserved_0 = input.read_u8()?;
            let _reserved_1 = input.read_u32::<LittleEndian>()?;
            dimensionality
        }
        2 => {
            let dimensionality = input.read_u8()?;
            let _flags = input.read_u8()?;
            let _space_type = input.read_u8()?;
            dimensionality
        }
        _ => {
            return Err(Error::OxifiveError(format!(
                "Unsupported dataspace version: {}",
                version
            )));
        }
    };
    let mut shape = Vec::new();
    for _ in 0..dimensions {
        shape.push(input.read_u64::<LittleEndian>()?);
    }
    Ok(Dataspace { shape })
}
