use crate::error::Error;
use crate::read::io::ReadSeek;
use crate::read::node::BTreeNodeType;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::SeekFrom;

#[derive(Clone, Debug)]
pub struct GroupBTreeNode {
    pub node_type: BTreeNodeType,
    pub node_level: u8,
    pub entries_used: u16,
    pub left_sibling: u64,
    pub right_sibling: u64,
    pub keys: Vec<u64>,
    pub addresses: Vec<u64>,
}

pub fn parse_group_btree(
    input: &mut impl ReadSeek,
    group: u64,
) -> Result<Vec<GroupBTreeNode>, Error> {
    let root_node = parse_group_btree_node(input, group)?;

    let mut nodes = HashMap::<u8, Vec<GroupBTreeNode>>::new();
    let mut node_level = root_node.node_level;
    nodes.insert(node_level, vec![root_node]);
    while node_level != 0 {
        let mut next_nodes = vec![];
        for parent_node in &nodes[&node_level] {
            for &group in &parent_node.addresses {
                next_nodes.push(parse_group_btree_node(input, group)?);
            }
        }
        let next_node_level = next_nodes[0].node_level;
        nodes.insert(next_node_level, next_nodes);
        node_level = next_node_level;
    }
    let nodes_flat: Vec<GroupBTreeNode> = nodes.values().flatten().cloned().collect();
    Ok(nodes_flat)
}

pub fn parse_group_btree_node(
    input: &mut impl ReadSeek,
    offset: u64,
) -> Result<GroupBTreeNode, Error> {
    input.seek(SeekFrom::Start(offset))?;
    let signature = {
        let mut bytes = [0; 4];
        input.read_exact(&mut bytes)?;
        String::from_utf8(bytes.into())?
    };
    if signature != "TREE" {
        return Err(Error::OxifiveError(format!(
            "Wrong BTreeNode signature: {}",
            signature
        )));
    }
    let node_type = BTreeNodeType::try_from(input.read_u8()?)?;
    if node_type != BTreeNodeType::Group {
        return Err(Error::OxifiveError(
            "Only group nodes are supported".to_string(),
        ));
    }
    let node_level = input.read_u8()?;
    let entries_used = input.read_u16::<LittleEndian>()?;
    let left_sibling = input.read_u64::<LittleEndian>()?;
    let right_sibling = input.read_u64::<LittleEndian>()?;

    let mut keys = vec![];
    let mut addresses = vec![];

    for _ in 0..entries_used {
        let key = input.read_u64::<LittleEndian>()?;
        let group = input.read_u64::<LittleEndian>()?;
        keys.push(key);
        addresses.push(group);
    }

    Ok(GroupBTreeNode {
        node_type,
        node_level,
        entries_used,
        left_sibling,
        right_sibling,
        keys,
        addresses,
    })
}
