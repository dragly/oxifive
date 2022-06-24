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

    pub fn get(&self, name: &str) -> Result<&Link, Error> {
        match self.data_object.links.get(name) {
            Some(l) => Ok(l),
            None => Err(Error::OxifiveError(format!("Unknown key name '{name}'"))),
        }
    }
}

impl Index<&str> for Group {
    type Output = Link;
    fn index(&self, index: &str) -> &Self::Output {
        &self.data_object.links[index]
    }
}
