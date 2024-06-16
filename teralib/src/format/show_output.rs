use std::{fmt::Display, path::PathBuf};

use serde::Deserialize;
use serde_json::Value;

use super::tfstate::S3BackendConfig;

#[derive(Debug, Deserialize)]
pub(crate) struct TerraformShow {
    pub format_version: String,
    pub values: TerraformValues,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TerraformValues {
    pub root_module: TerraformRootModule,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TerraformRootModule {
    pub resources: Option<Vec<TerraformResource>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TerraformResource {
    pub mode: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,
    pub values: Value,
}

impl TerraformShow {
    pub fn dependencies(&self) -> Vec<String> {
        let maybe_resources = self.values.root_module.resources.as_ref();

        if let Some(ref resources) = maybe_resources {
            resources
                .iter()
                .filter_map(|r| {
                    if r.mode != "data" || r.type_ != "terraform_remote_state" {
                        return None;
                    }

                    let backend: TerraformShowBackend = serde_json::from_value(r.values.clone())
                        .expect("Unable to deserialize backend");

                    Some(format!("{}", backend))
                })
                .collect()
        } else {
            vec![]
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "backend", content = "config", rename_all = "lowercase")]
pub(crate) enum TerraformShowBackend {
    S3(S3BackendConfig),
    Local(LocalConfig),
}

impl Display for TerraformShowBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TerraformShowBackend::S3(config) => write!(f, "{}", config),
            TerraformShowBackend::Local(config) => write!(f, "{}", config),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct LocalConfig {
    pub path: PathBuf,
}

impl Display for LocalConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = self.path.canonicalize().unwrap();
        let path = match path.is_dir() {
            true => path.to_string_lossy().to_string(),
            false => path.parent().unwrap().to_string_lossy().to_string(),
        };
        write!(f, "local:{}", path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_terraform_show() {
        let test_file = include_str!("../../../test.json");

        let results: TerraformShow =
            serde_json::from_str(test_file).expect("Unable to deserialize show");

        assert_eq!(results.format_version, "1.0");
        assert_eq!(
            results
                .values
                .root_module
                .resources
                .as_ref()
                .expect("Resources should be set")
                .len(),
            23
        );

        let dependencies = results.dependencies();
        assert_eq!(dependencies.len(), 2);
    }
}
