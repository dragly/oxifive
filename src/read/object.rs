use crate::read::{
    group::Group,
    dataset::Dataset
};

#[derive(Clone, Debug)]
pub enum Object {
    Group(Group),
    Dataset(Dataset),
}
