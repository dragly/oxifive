use std::ops::Index;

use crate::error::Error;
use crate::read::{
    data_object::{self, DataObject},
    dataset::Dataset,
    group::Group,
    io::ReadSeek,
    superblock::{self, SuperBlockVersion0},
    symbol_table::SymbolTableEntry,
};

use crate::read::link::Link;

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
        self.as_mut_group().group(self, name)
    }

    pub fn dataset(&mut self, name: &str) -> Result<Dataset, Error> {
        self.as_mut_group().dataset(self, name)
    }

    pub fn keys(&self) -> Vec<String> {
        self.as_group().keys()
    }

    pub fn object(&mut self, name: &str) -> Result<DataObject, Error> {
        self.as_mut_group().object(self, name)
    }

    pub fn as_mut_group(&mut self) -> Group {
        Group {
            data_object: self.data_object.clone(),
        }
    }

    pub fn as_group(&self) -> Group {
        Group {
            data_object: self.data_object.clone(),
        }
    }
}

impl Index<&String> for FileReader {
    type Output = Link;
    fn index(&self, index: &String) -> &Self::Output {
        &self.data_object.links[index]
    }
}
