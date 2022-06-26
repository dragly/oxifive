use crate::read::{dataset::Dataset, group::Group, link::Link};
use std::ops::Index;

#[derive(Clone, Debug)]
pub enum Object {
    Group(Group),
    Dataset(Dataset),
}

impl Index<&str> for Object {
    type Output = Link;
    fn index(&self, index: &str) -> &Self::Output {
        match self {
            Object::Group(group) => &group.data_object.links[index],
            Object::Dataset(_) => unimplemented!("Indexing datasets is not yet implemented"),
        }
    }
}
