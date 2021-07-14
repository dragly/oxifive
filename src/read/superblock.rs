use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

use crate::error::Error;

#[derive(Clone, Debug)]
pub struct SuperBlockVersion0 {
    pub format_signature: [u8; 8],
    pub superblock_version: u8,
    pub free_storage_version: u8,
    pub root_group_version: u8,
    pub reserved_0: u8,
    pub shared_header_version: u8,
    pub offset_size: u8,
    pub length_size: u8,
    pub reserved_1: u8,
    pub group_leaf_node_k: u16,
    pub group_internal_node_k: u16,
    pub file_consistency_flags: u32,
    pub base_address: u64,
    pub free_space_address: u64,
    pub end_of_file_address: u64,
    pub driver_information_address: u64,
}

pub fn parse_superblock(input: &mut (impl Read + ?Sized)) -> Result<SuperBlockVersion0, Error> {
    let mut format_signature = [0; 8];
    input.read_exact(&mut format_signature)?;
    if format_signature != [137, 72, 68, 70, 13, 10, 26, 10] {
        return Err(Error::OxifiveError(format!(
            "Wrong header, found {:#?}",
            format_signature
        )));
    }

    let superblock_version = input.read_u8()?;
    if superblock_version != 0 {
        return Err(Error::OxifiveError(format!(
            "Only superblock version 0 is supported, but found {}",
            superblock_version
        )));
    }

    Ok(SuperBlockVersion0 {
        format_signature,
        superblock_version,
        free_storage_version: input.read_u8()?,
        root_group_version: input.read_u8()?,
        reserved_0: input.read_u8()?,
        shared_header_version: input.read_u8()?,
        offset_size: input.read_u8()?,
        length_size: input.read_u8()?,
        reserved_1: input.read_u8()?,
        group_leaf_node_k: input.read_u16::<LittleEndian>()?,
        group_internal_node_k: input.read_u16::<LittleEndian>()?,
        file_consistency_flags: input.read_u32::<LittleEndian>()?,
        base_address: input.read_u64::<LittleEndian>()?,
        free_space_address: input.read_u64::<LittleEndian>()?,
        end_of_file_address: input.read_u64::<LittleEndian>()?,
        driver_information_address: input.read_u64::<LittleEndian>()?,
    })
}
