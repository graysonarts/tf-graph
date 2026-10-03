use std::{
    fmt::Display,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::GraphError;

pub(crate) enum TerraformState {
    Remote(TerraformRemoteState),
    Local(TerraformLocalState),
}

impl From<TerraformRemoteStateJson> for TerraformState {
    fn from(json: TerraformRemoteStateJson) -> Self {
        TerraformState::Remote(json.into())
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum TerraformStateJson {
    Remote(TerraformRemoteStateJson),
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
pub(crate) struct TerraformRemoteStateJson {
    pub version: i32,
    pub backend: TerraformBackendJson,
}

pub(crate) struct TerraformRemoteState {
    pub version: i32,
    pub backend: TerraformBackend,
}

impl From<TerraformRemoteStateJson> for TerraformRemoteState {
    fn from(json: TerraformRemoteStateJson) -> Self {
        TerraformRemoteState {
            version: json.version,
            backend: json.backend.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "config", rename_all = "lowercase")]
pub(crate) enum TerraformBackendJson {
    S3(S3BackendConfigJson),
}

pub(crate) enum TerraformBackend {
    S3(S3BackendConfig),
}

impl From<TerraformBackendJson> for TerraformBackend {
    fn from(json: TerraformBackendJson) -> Self {
        match json {
            TerraformBackendJson::S3(config) => TerraformBackend::S3(config.into()),
        }
    }
}

impl Display for TerraformBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TerraformBackend::S3(config) => write!(f, "{}", config),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct S3BackendConfigJson {
    pub bucket: String,
    pub region: String,
    pub key: String,
}

#[derive(Debug)]
pub(crate) struct S3BackendConfig {
    pub bucket: String,
    pub region: String,
    pub key: String,
    pub environment: Option<String>,
}

impl From<S3BackendConfigJson> for S3BackendConfig {
    fn from(json: S3BackendConfigJson) -> Self {
        let (environment, key) = if json.key.contains("/") {
            let parts: Vec<_> = json.key.splitn(2, '/').collect();
            (Some(parts[0].to_string()), parts[1])
        } else {
            (None, json.key.as_str())
        };
        S3BackendConfig {
            bucket: json.bucket,
            region: json.region,
            key: key.to_string(),
            environment: environment,
        }
    }
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
            let state: TerraformRemoteStateJson = serde_json::from_reader(reader)?;
            Ok(TerraformState::Remote(state.into()))
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
