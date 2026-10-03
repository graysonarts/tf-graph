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

    pub fn environment(&self) -> Option<&str> {
        match self {
            Node::Unprocessed(_) => None,
            Node::Processed(full) => full
                .path
                .file_name()
                .and_then(|f| f.to_str())
                .map(|f| f.split('.').next().unwrap()),
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
            environment: None,
        }
    }
}

impl NodeFull {
    pub fn with_environment(self, environment: &str) -> NodeFull {
        NodeFull {
            environment: Some(environment.to_string()),
            ..self
        }
    }
}

#[derive(Debug, Builder, Clone)]
pub struct NodeFull {
    pub id: NodeId,
    pub path: PathBuf,
    pub node: String,
    pub environment: Option<String>,
}
