use crate::error::Error;
use crate::read::{
    data_object::{self, parse_data_object, DataObject},
    dataset::Dataset,
    file::FileReader,
    link::LinkTarget,
};

#[derive(Debug, Clone)]
pub struct Group {
    pub data_object: DataObject,
}

impl Group {
    pub fn group(&self, file: &mut FileReader, name: &str) -> Result<Group, Error> {
        let data_link = self
            .data_object
            .links
            .get(name)
            .ok_or_else(|| Error::OxifiveError(format!("Group '{}' not found", name)))?
            .clone();
        let data_address = match data_link.target {
            LinkTarget::Hard { address } => address,
            _ => return Err(Error::OxifiveError(format!("{} is not a hard link", name))),
        };
        let data_object = parse_data_object(&mut file.input, data_address)?;
        Ok(Group { data_object })
    }

    pub fn dataset(&self, file: &mut FileReader, name: &str) -> Result<Dataset, Error> {
        let pointcloud_link = self.data_object.links[name].clone();
        let pointcloud_address = match pointcloud_link.target {
            LinkTarget::Hard { address } => address,
            _ => return Err(Error::OxifiveError("Data is not a hard link".to_string())),
        };
        let data_object = data_object::parse_data_object(&mut file.input, pointcloud_address)?;
        Ok(Dataset { data_object })
    }
}
