use std::path::PathBuf;

use derive_builder::Builder;

pub(crate) type NodeId = u8;

#[derive(Debug)]
pub enum Node {
    Unprocessed(NodeHeader),
    Processed(NodeFull),
}

impl Node {
    pub fn new(id: NodeId, path: PathBuf) -> Node {
        Node::Unprocessed(NodeHeader { id, path })
    }

    pub fn id(&self) -> NodeId {
        match self {
            Node::Unprocessed(header) => header.id,
            Node::Processed(full) => full.id,
        }
    }

    pub fn path(&self) -> &PathBuf {
        match self {
            Node::Unprocessed(header) => &header.path,
            Node::Processed(full) => &full.path,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Node::Unprocessed(header) => {
                header.path.file_name().as_ref().unwrap().to_str().unwrap()
            }
            Node::Processed(full) => &full.node,
        }
    }
}

#[derive(Debug)]
pub struct NodeHeader {
    pub id: NodeId,
    pub path: PathBuf,
}

impl NodeHeader {
    pub fn with_name(self, name: &str) -> NodeFull {
        NodeFull {
            id: self.id,
            path: self.path,
            node: name.to_string(),
        }
    }
}

#[derive(Debug, Builder, Clone)]
pub struct NodeFull {
    pub id: NodeId,
    pub path: PathBuf,
    pub node: String,
}
