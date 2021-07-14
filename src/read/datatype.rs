use crate::error::Error;
use byteorder::{LittleEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::io::Read;

#[repr(u8)]
#[derive(Clone, Debug, PartialEq, TryFromPrimitive)]
pub enum DatatypeEncoding {
    FixedPoint = 0,
    FloatingPoint = 1,
    Time = 2,
    String = 3,
    Bitfield = 4,
    Opaque = 5,
    Compound = 6,
    Reference = 7,
    Enumerated = 8,
    VariableLength = 9,
    Array = 10,
}

#[derive(Clone, Debug)]
pub struct Datatype {
    pub class_and_version: u8,
    pub class_bit_field_0: u8,
    pub class_bit_field_1: u8,
    pub class_bit_field_2: u8,
    pub size: u32,
    pub encoding: DatatypeEncoding,
}

pub fn parse_datatype_message(input: &mut impl Read) -> Result<Datatype, Error> {
    let class_and_version = input.read_u8()?;
    let datatype = Datatype {
        class_and_version,
        class_bit_field_0: input.read_u8()?,
        class_bit_field_1: input.read_u8()?,
        class_bit_field_2: input.read_u8()?,
        size: input.read_u32::<LittleEndian>()?,
        encoding: DatatypeEncoding::try_from(class_and_version & 0x0F)?,
    };

    Ok(datatype)
}
