pub use super::{Error, Result};
use serde::Serialize;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::log::warn;
#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub enum ConfigFileProperty {
    ReadOnly(bool),
    Unknown(String),
}

impl From<&str> for ConfigFileProperty {
    fn from(value: &str) -> Self {
        match value {
            "ro" => ConfigFileProperty::ReadOnly(true),
            "rw" => ConfigFileProperty::ReadOnly(false),
            x => ConfigFileProperty::Unknown(x.to_string()),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub struct ConfigFile {
    pub host_file_name: String,
    pub container_file_path: PathBuf,
    pub read_only: bool,
}

impl ConfigFile {
    fn new(
        host_file_name: String,
        container_file_path: PathBuf,
        properties: Vec<ConfigFileProperty>,
    ) -> Self {
        let mut config_file = Self {
            host_file_name,
            container_file_path,
            read_only: false,
        };
        for property in properties {
            match property {
                ConfigFileProperty::ReadOnly(value) => {
                    config_file.read_only = value;
                }
                ConfigFileProperty::Unknown(s) => {
                    warn!("Ignoring invalid conffile property {s}");
                }
            }
        }
        config_file
    }
}

impl FromStr for ConfigFile {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits = s.split(':').collect::<Vec<&str>>();
        anyhow::ensure!(
            splits.len() >= 2,
            "Expected host file name, container file path and an arbitrary number of parameters split by ':', received {s}"
        );
        anyhow::ensure!(
            !splits[0].contains('/'),
            "Expected host file name without any path, received {}",
            splits[0]
        );
        let container_path = PathBuf::from(splits[1]);
        anyhow::ensure!(
            container_path.is_absolute(),
            "Expected container file path to be an absolute path, is {}",
            splits[1]
        );
        let properties = splits
            .iter()
            .skip(2)
            .map(|s| ConfigFileProperty::from(*s))
            .collect();
        Ok(Self::new(splits[0].to_string(), container_path, properties))
    }
}

impl TryFrom<&flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestConffilesItem>
    for ConfigFile
{
    type Error = Error;

    fn try_from(
        value: &flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestConffilesItem,
    ) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use TryInto;
    #[test]
    fn config_file_property_from_string() {
        assert_eq!(ConfigFileProperty::ReadOnly(false), "rw".into());
        assert_eq!(ConfigFileProperty::ReadOnly(true), "ro".into());
        assert_eq!(
            ConfigFileProperty::Unknown("Something".to_string()),
            "Something".into()
        );
    }

    #[test]
    fn new_config_file() {
        let host_file_name = "example.config".to_string();
        let container_file_path = PathBuf::from("/some/path/with.config");
        let config_file_properties = vec![
            ConfigFileProperty::ReadOnly(true),
            ConfigFileProperty::Unknown("XY".to_string()),
        ];
        assert_eq!(
            ConfigFile::new(
                host_file_name.clone(),
                container_file_path.clone(),
                config_file_properties
            ),
            ConfigFile {
                host_file_name,
                container_file_path,
                read_only: true
            }
        );
    }

    #[test]
    fn new_config_file_no_property() {
        let host_file_name = "example.config".to_string();
        let container_file_path = PathBuf::from("/some/path/with.config");
        assert_eq!(
            ConfigFile::new(
                host_file_name.clone(),
                container_file_path.clone(),
                Vec::new()
            ),
            ConfigFile {
                host_file_name,
                container_file_path,
                read_only: false
            }
        );
    }

    #[test]
    fn config_file_from_str_ok() {
        let s = "my.config:/some/container/path.config:ro";
        assert_eq!(
            ConfigFile::from_str(s).unwrap(),
            ConfigFile {
                host_file_name: "my.config".to_string(),
                container_file_path: PathBuf::from("/some/container/path.config"),
                read_only: true
            }
        );
    }

    #[test]
    fn config_file_from_str_no_container_path() {
        let s = "my.config";
        assert!(ConfigFile::from_str(s).is_err());
    }

    #[test]
    fn config_file_from_str_host_file_is_path() {
        let s = "path/my.config:/some/container/path.config";
        assert!(ConfigFile::from_str(s).is_err());
    }

    #[test]
    fn config_file_from_str_relative_container_path() {
        let s = "my.config:../some/container/path.config";
        assert!(ConfigFile::from_str(s).is_err());
    }

    #[test]
    fn try_config_file_conffile_item_ok() {
        let item =
            flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestConffilesItem::from_str(
                "my.config:/some/container/path.config:ro",
            )
            .unwrap();
        let result: ConfigFile = (&item).try_into().unwrap();
        assert_eq!(
            result,
            ConfigFile {
                host_file_name: "my.config".to_string(),
                container_file_path: PathBuf::from("/some/container/path.config"),
                read_only: true
            }
        );
    }

    #[test]
    fn try_config_file_conffile_item_relative_container_path() {
        let item =
            flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestConffilesItem::from_str(
                "my.config:../some/container/path.config",
            )
            .unwrap();
        let result: Result<ConfigFile> = (&item).try_into();
        assert!(result.is_err());
    }
}
