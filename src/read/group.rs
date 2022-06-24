use std::ops::Index;

use crate::error::Error;
use crate::read::{
    data_object::{parse_data_object, DataObject},
    dataset::Dataset,
    file::FileReader,
    link::Link,
    link::LinkTarget,
};

#[derive(Clone, Debug)]
pub struct Group {
    pub data_object: DataObject,
}

impl Group {
    pub fn keys(&self) -> Vec<String> {
        self.data_object
            .links
            .keys()
            .map(|key| key.clone())
            .collect()
    }

    pub fn object(&self, file: &mut FileReader, name: &str) -> Result<DataObject, Error> {
        let data_link = self
            .data_object
            .links
            .get(name)
            .ok_or_else(|| Error::OxifiveError(format!("Link '{}' not found", name)))?
            .clone();
        let data_address = match data_link.target {
            LinkTarget::Hard { address } => address,
            _ => return Err(Error::OxifiveError(format!("Link '{}' is not a hard link", name))),
        };
        Ok(parse_data_object(&mut file.input, data_address)?)
    }

    pub fn group(&self, file: &mut FileReader, name: &str) -> Result<Group, Error> {
        Ok(Group { data_object: self.object(file, name)? })
    }

    pub fn dataset(&self, file: &mut FileReader, name: &str) -> Result<Dataset, Error> {
        Ok(Dataset { data_object: self.object(file, name)? })
    }
}

impl Index<&String> for Group {
    type Output = Link;
    fn index(&self, index: &String) -> &Self::Output {
        &self.data_object.links[index]
    }
}
