mod format;
mod node;

use cwd_block::{with_cwd, WithWorkingDirectory};
use format::tfstate::TerraformState;

use self::node::*;
use std::{collections::HashMap, path::Path, process::Command};

#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("An error occurred while running the terraform command: {0}")]
    TerraformCommand(#[from] std::io::Error),

    #[error("Unable to find a terraform state file for {0}")]
    NoStateFile(String),

    #[error("Unable to parse the terraform json")]
    UnableToParse(#[from] serde_json::Error),

    #[error("Unable to convert the output to a string")]
    InvalidUTF8(#[from] std::string::FromUtf8Error),

    #[cfg(debug)]
    #[error("This error has not been defined yet")]
    ToBeDefined,
}

#[derive(Debug, Default)]
pub struct TFGraph {
    pub roots: HashMap<NodeId, Node>,
    pub backends: HashMap<String, NodeId>,
    pub dependency_list: HashMap<NodeId, Vec<NodeId>>,
}

impl TFGraph {
    fn next_id(&self) -> NodeId {
        self.roots.len() as NodeId
    }
    pub fn with_root<P: AsRef<Path>>(mut self, path: P) -> Self {
        let next_id = self.next_id();
        let node = Node::new(next_id, path.as_ref().to_path_buf());
        self.roots.insert(node.id(), node);
        self
    }

    pub fn build(mut self) -> Result<Self, GraphError> {
        for (id, node) in self.roots.iter() {
            let state = TerraformState::from_root(node.path())?;
            tracing::info!("state {}", state);

            self.backends.insert(state.to_string(), *id);
        }

        for (id, node) in self.roots.iter() {
            with_cwd(node.path(), || -> Result<_, GraphError> {
                let json = run_terraform_json(".")?;
                let results: format::show_output::TerraformShow =
                    serde_json::from_str(&json).expect("Unable to deserialize show");
                let dependencies = results.dependencies();
                let dependencies: Vec<_> = dependencies
                    .iter()
                    .map(|d| {
                        tracing::info!("Looking for backend {:?}", d);
                        let id = self.backends.get(d).expect("Unable to find backend");
                        *id
                    })
                    .collect();
                tracing::info!("{:?}\n{:#?}", node, dependencies);
                self.dependency_list.insert(*id, dependencies);
                Ok(())
            })?;
        }
        Ok(self)
    }
}

fn run_terraform_json<P: AsRef<Path>>(path: P) -> Result<String, GraphError> {
    let command = Command::new("terraform")
        .args(["show", "-json"])
        .current_dir(path.as_ref())
        .output()?;
    let output = String::from_utf8(command.stdout)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use format::show_output::TerraformShow;
    use test_log::test;

    use super::*;

    #[test]
    fn test_graph_builder() {
        let graph = TFGraph::default()
            .with_root("../test_infrastructure/child")
            .with_root("../test_infrastructure/parent")
            .with_root("../test_infrastructure/cycle")
            .build()
            .expect("Unable to construct graph");
    }

    #[test]
    fn test_run_terraform_json() {
        let json =
            run_terraform_json("../test_infrastructure/child").expect("Unable to run terraform");
        let results: TerraformShow =
            serde_json::from_str(&json).expect("Unable to deserialize show");

        assert_eq!(results.format_version, "1.0");
        assert_eq!(
            results
                .values
                .root_module
                .resources
                .expect("resources should exist")
                .len(),
            2
        );
    }
}
