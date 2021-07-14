use crate::error::Error;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

#[derive(Clone, Debug)]
pub struct SymbolTableNode {
    pub signature: [u8; 4],
    pub version: u8,
    pub reserved: u8,
    pub symbols: u16,
}

#[derive(Clone, Debug)]
pub struct SymbolTableEntry {
    pub link_name_offset: u64,
    pub object_header_address: u64,
    pub cache_type: u32,
    pub reserved: u32,
    pub scratch: [u8; 16],
}

impl SymbolTableNode {
    pub fn read(input: &mut impl Read) -> Result<Self, Error> {
        Ok(SymbolTableNode {
            signature: {
                let mut result = [0; 4];
                input.read_exact(&mut result)?;
                result
            },
            version: input.read_u8()?,
            reserved: input.read_u8()?,
            symbols: input.read_u16::<LittleEndian>()?,
        })
    }
}

impl SymbolTableEntry {
    pub fn read(input: &mut impl Read) -> Result<SymbolTableEntry, Error> {
        Ok(SymbolTableEntry {
            link_name_offset: input.read_u64::<LittleEndian>()?,
            object_header_address: input.read_u64::<LittleEndian>()?,
            cache_type: input.read_u32::<LittleEndian>()?,
            reserved: input.read_u32::<LittleEndian>()?,
            scratch: {
                let mut result = [0; 16];
                input.read_exact(&mut result)?;
                result
            },
        })
    }
}
