use crate::{
    error::Error,
    read::{dataset::Dataset, group::Group},
    ReadSeek,
};

#[derive(Clone, Debug)]
pub enum Object<R> {
    Group(Group<R>),
    Dataset(Dataset<R>),
}

impl<R: ReadSeek> Object<R> {
    pub fn object(&self, name: &str) -> Result<Object<R>, Error> {
        match self {
            Object::Group(group) => group.object(name),
            _ => Err(Error::OxifiveError(format!("Not a group"))),
        }
    }

    pub fn group(&self, name: &str) -> Result<Group<R>, Error> {
        match self {
            Object::Group(group) => group.group(name),
            _ => Err(Error::OxifiveError(format!("Not a group"))),
        }
    }

    pub fn dataset(&self, name: &str) -> Result<Dataset<R>, Error> {
        match self {
            Object::Group(group) => group.dataset(name),
            _ => Err(Error::OxifiveError(format!("Not a group"))),
        }
    }
}
