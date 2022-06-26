use std::io::Read;
use std::ops::Index;
use std::sync::{Arc, Mutex};

use crate::Object;
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

pub struct FileReader<R> {
    pub superblock: SuperBlockVersion0,
    pub root_entry: SymbolTableEntry,
    pub data_object: DataObject,
    input: Arc<Mutex<R>>,
}

impl<R: ReadSeek> FileReader<R> {
    pub fn new(input: Arc<Mutex<R>>) -> Result<Self, Error> {
        let reader = &mut *input.lock().unwrap();
        let superblock = superblock::parse_superblock(reader)?;
        log::info!("{:#?}", superblock);
        let root_entry = SymbolTableEntry::read(reader)?;
        log::info!("{:#?}", root_entry);
        let offset_to_data_objects = root_entry.object_header_address;
        let data_object = data_object::parse_data_object(reader, offset_to_data_objects)?;
        Ok(Self {
            superblock,
            root_entry,
            data_object,
            input: input.clone(),
        })
    }

    pub fn keys(&self) -> Vec<String> {
        self.as_group().keys()
    }

    pub fn as_mut_group(&mut self) -> Group<R> {
        Group {
            data_object: self.data_object.clone(),
            input: self.input.clone(),
        }
    }

    pub fn as_group(&self) -> Group<R> {
        Group {
            data_object: self.data_object.clone(),
            input: self.input.clone(),
        }
    }

    pub fn object(&self, index: &str) -> Result<Object<R>, Error> {
        self.as_group().object(index)
    }

    pub fn group(&self, index: &str) -> Result<Group<R>, Error> {
        self.as_group().group(index)
    }

    pub fn dataset(&self, index: &str) -> Result<Dataset<R>, Error> {
        self.as_group().dataset(index)
    }
}

