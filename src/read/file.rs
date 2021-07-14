use crate::error::Error;
use crate::read::{
    data_object::{self, DataObject},
    group::Group,
    superblock::{self, SuperBlockVersion0},
    symbol_table::SymbolTableEntry,
    io::ReadSeek,
};

pub struct FileReader {
    pub superblock: SuperBlockVersion0,
    pub root_entry: SymbolTableEntry,
    pub data_object: DataObject,
    pub input: Box<dyn ReadSeek>,
}

impl FileReader {
    pub fn read(mut input: Box<dyn ReadSeek>) -> Result<Self, Error> {
        let superblock = superblock::parse_superblock(&mut input)?;
        log::info!("{:#?}", superblock);
        let root_entry = SymbolTableEntry::read(&mut input)?;
        log::info!("{:#?}", root_entry);
        let offset_to_data_objects = root_entry.object_header_address;
        let data_object = data_object::parse_data_object(&mut input, offset_to_data_objects)?;
        Ok(Self {
            superblock,
            root_entry,
            data_object,
            input: Box::new(input),
        })
    }

    pub fn group(&mut self, name: &str) -> Result<Group, Error> {
        let group = Group {
            data_object: self.data_object.clone(),
        };
        group.group(self, name)
    }
}
