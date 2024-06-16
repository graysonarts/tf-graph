use std::{
    fmt::Display,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use serde::{Deserialize};

use crate::GraphError;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum TerraformState {
    Remote(TerraformRemoteState),
    Local(TerraformLocalState),
}

impl Display for TerraformState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TerraformState::Remote(state) => write!(f, "{}", state.backend),
            TerraformState::Local(state) => write!(f, "local:{}", state.name),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct TerraformLocalState {
    pub name: String,
}

impl From<PathBuf> for TerraformLocalState {
    fn from(path: PathBuf) -> Self {
        let path = path.canonicalize().unwrap();
        let path = match path.is_dir() {
            true => path.to_string_lossy().to_string(),
            false => path.parent().unwrap().to_string_lossy().to_string(),
        };
        TerraformLocalState { name: path }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct TerraformRemoteState {
    pub version: i32,
    pub backend: TerraformBackend,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "config", rename_all = "lowercase")]
pub(crate) enum TerraformBackend {
    S3(S3BackendConfig),
}

impl Display for TerraformBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TerraformBackend::S3(config) => write!(f, "{}", config),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct S3BackendConfig {
    pub bucket: String,
    pub region: String,
    pub key: String,
}

impl Display for S3BackendConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "s3:{}/{}/{}", self.region, self.bucket, self.key)
    }
}

impl TerraformState {
    pub fn from_root<P: AsRef<Path>>(path: P) -> Result<Self, GraphError> {
        let remote_path = path.as_ref().join(".terraform/terraform.tfstate");
        let local_path = path.as_ref().join("terraform.tfstate");

        if remote_path.exists() {
            let file = File::open(remote_path)?;
            let reader = BufReader::new(file);
            let state: TerraformRemoteState = serde_json::from_reader(reader)?;
            Ok(TerraformState::Remote(state))
        } else if local_path.exists() {
            Ok(TerraformState::Local(local_path.canonicalize()?.into()))
        } else {
            Err(GraphError::NoStateFile(
                path.as_ref().to_string_lossy().to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_terraform_state() {
        let results = TerraformState::from_root("../test_infrastructure/child")
            .expect("Unable to read terraform state");

        assert!(matches!(results, TerraformState::Local(_)));
        assert_eq!(
            results.to_string(),
            "local:/Users/grayson/code/tf-graph/test_infrastructure/child"
        );
    }

    #[test]
    fn test_remote_terraform_state() {
        let results = TerraformState::from_root("/Users/grayson/code/elka/infrastructure/company")
            .expect("Unable to find terraform state");
        println!("REMOTE: {}", results);
        assert!(matches!(results, TerraformState::Remote(_)));
        assert_eq!(
            results.to_string(),
            "s3:us-east-1/terraform.elka.ai/terraform.tfstate"
        );
    }
}
