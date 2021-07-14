use crate::{error::Error, padding::padded_size};
use byteorder::{LittleEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::io::{Read, Seek, SeekFrom};

#[repr(u16)]
#[derive(Clone, Debug, TryFromPrimitive)]
pub enum FilterType {
    ReservedFilter = 0,
    GzipDeflateFilter = 1,
    ShuffleFilter = 2,
    Fletch32Filter = 3,
    SzipFilter = 4,
    NbitFilter = 5,
    ScaleoffsetFilter = 6,
}

#[derive(Clone, Debug)]
pub struct FilterPipeline {
    pub filter_type: FilterType,
    pub name: String,
}

pub fn parse_filter_pipeline_message(
    input: &mut (impl Read + Seek),
) -> Result<Vec<FilterPipeline>, Error> {
    let version = input.read_u8()?;
    if version != 1 {
        return Err(Error::OxifiveError(format!(
            "Unsupported filter pipeline version: {}",
            version
        )));
    }
    let filter_count = input.read_u8()? as usize;
    let _reserved_0 = input.read_u16::<LittleEndian>()?;
    let _reserved_1 = input.read_u32::<LittleEndian>()?;
    let mut filters = vec![];
    for _ in 0..filter_count {
        let filter_id = input.read_u16::<LittleEndian>()?;
        let filter_type = FilterType::try_from(filter_id)?;
        let name_length = input.read_u16::<LittleEndian>()? as usize;
        let name_length_padded = padded_size(name_length);
        let _flags = input.read_u16::<LittleEndian>()?;
        let client_data_value_count = input.read_u16::<LittleEndian>()?;

        let mut name_bytes_padded = vec![0; name_length_padded as usize];
        input.read_exact(&mut name_bytes_padded)?;
        assert!(name_bytes_padded[name_length - 1] == 0);
        let name_bytes = name_bytes_padded[0..name_length - 1].to_vec();
        let name = String::from_utf8(name_bytes)?;

        let mut client_data_values = vec![];
        for _ in 0..client_data_value_count {
            client_data_values.push(input.read_u32::<LittleEndian>()?);
        }
        if client_data_value_count % 2 == 1 {
            input.seek(SeekFrom::Current(4))?;
        }
        filters.push(FilterPipeline { filter_type, name });
    }

    assert!(filters.len() == filter_count);

    Ok(filters)
}
