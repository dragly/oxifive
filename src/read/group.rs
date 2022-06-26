use std::io::Read;
use std::ops::Index;
use std::sync::{Arc, Mutex};

use crate::error::Error;
use crate::read::{
    data_object::{parse_data_object, DataObject},
    dataset::Dataset,
    file::FileReader,
    link::Link,
    link::LinkTarget,
};
use crate::{Object, ReadSeek};

#[derive(Clone, Debug)]
pub struct Group<R> {
    pub data_object: DataObject,
    pub input: Arc<Mutex<R>>,
}

impl<R: ReadSeek> Group<R> {
    pub fn keys(&self) -> Vec<String> {
        self.data_object
            .links
            .keys()
            .map(|key| key.clone())
            .collect()
    }

    pub fn object(&self, name: &str) -> Result<Object<R>, Error> {
        let link = self.data_object.links.get(name).unwrap();
        let data_address = match link.target {
            LinkTarget::Hard { address } => address,
            _ => {
                return Err(Error::OxifiveError(format!(
                    "Link '{}' is not a hard link and soft links are not yet supported",
                    link.name
                )))
            }
        };
        let data_object = parse_data_object(&mut *self.input.lock().unwrap(), data_address)?;
        if data_object.is_group() {
            Ok(Object::Group(Group {
                data_object,
                input: self.input.clone(),
            }))
        } else {
            Ok(Object::Dataset(Dataset {
                data_object,
                input: self.input.clone(),
            }))
        }
    }

    pub fn group(&self, name: &str) -> Result<Group<R>, Error> {
        match self.object(name)? {
            Object::Group(group) => Ok(group),
            _ => Err(Error::OxifiveError(format!("{name} is not a group"))),
        }
    }

    pub fn dataset(&self, name: &str) -> Result<Dataset<R>, Error> {
        match self.object(name)? {
            Object::Dataset(dataset) => Ok(dataset),
            _ => Err(Error::OxifiveError(format!("{name} is not a dataset"))),
        }
    }
}
