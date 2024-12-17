pub use super::{Error, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct VolumeMount {
    pub name: String,
    pub container_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct BindMount {
    pub host_path: PathBuf,
    pub container_path: PathBuf,
}

impl BindMount {
    pub fn default_docker_socket_bind_mount() -> Self {
        Self {
            host_path: PathBuf::from("/run/docker.sock"),
            container_path: PathBuf::from("/run/docker.sock"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Mount {
    Bind(BindMount),
    Volume(VolumeMount),
}

impl FromStr for Mount {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('/') {
            Ok(Self::Bind(BindMount::from_str(s)?))
        } else {
            Ok(Self::Volume(VolumeMount::from_str(s)?))
        }
    }
}

impl TryFrom<&flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestVolumesItem>
    for Mount
{
    type Error = Error;

    fn try_from(
        value: &flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestVolumesItem,
    ) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl FromStr for BindMount {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits = s.split(':').collect::<Vec<&str>>();
        anyhow::ensure!(
            splits.len() == 2,
            "Expected host and container path split by ':', received {s}"
        );
        let host_path = PathBuf::from(splits[0]);
        anyhow::ensure!(
            host_path.is_absolute(),
            "Expected host path to be an absolute path, is {}",
            splits[0]
        );
        let container_path = PathBuf::from(splits[1]);
        anyhow::ensure!(
            container_path.is_absolute(),
            "Expected container path to be an absolute path, is {}",
            splits[1]
        );

        Ok(Self {
            container_path,
            host_path,
        })
    }
}

impl FromStr for VolumeMount {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits = s.split(':').collect::<Vec<&str>>();
        anyhow::ensure!(
            splits.len() == 2,
            "Expected volume name and container path split by ':', received '{s}'",
        );
        let container_path = PathBuf::from(splits[1]);
        anyhow::ensure!(
            container_path.is_absolute(),
            "Expected container path to be an absolute path, is {}",
            splits[1]
        );
        const REGEX_PATTERN: &str = r"^[a-zA-Z0-9\-_.]+[a-zA-Z0-9]$";
        let regex = Regex::new(REGEX_PATTERN)?;
        anyhow::ensure!(
            regex.is_match(splits[0]),
            "Volume name '{}' did not match regex pattern {REGEX_PATTERN}",
            splits[0]
        );

        Ok(Self {
            container_path,
            name: splits[0].to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_mount_from_str_ok() {
        assert_eq!(
            BindMount {
                host_path: PathBuf::from("/some/host/path"),
                container_path: PathBuf::from("/some/container/path")
            },
            BindMount::from_str("/some/host/path:/some/container/path").unwrap()
        )
    }

    #[test]
    fn bind_mount_from_str_too_many_paths() {
        assert!(
            BindMount::from_str("/some/host/path:/some/container/path:/some/unexpected/path")
                .is_err()
        )
    }

    #[test]
    fn bind_mount_from_str_only_host_path() {
        assert!(BindMount::from_str("/some/host/path").is_err())
    }

    #[test]
    fn bind_mount_from_str_relative_host_path() {
        assert!(BindMount::from_str("local/some/host/path:/some/container/path").is_err())
    }

    #[test]
    fn bind_mount_from_str_relative_container_path() {
        assert!(BindMount::from_str("/some/host/path:../some/container/path").is_err())
    }

    #[test]
    fn volume_mount_from_str_ok() {
        assert_eq!(
            VolumeMount {
                name: "MyVolume".to_string(),
                container_path: PathBuf::from("/some/container/path")
            },
            VolumeMount::from_str("MyVolume:/some/container/path").unwrap()
        )
    }

    #[test]
    fn volume_mount_from_str_too_many_parts() {
        assert!(BindMount::from_str("MyVolume:/some/container/path:invalid_addition").is_err())
    }

    #[test]
    fn volume_mount_from_str_only_name() {
        assert!(BindMount::from_str("MyVolume").is_err())
    }

    #[test]
    fn volume_mount_from_str_relative_container_path() {
        assert!(BindMount::from_str("MyVolume:../some/container/path").is_err())
    }

    #[test]
    fn volume_mount_from_str_invalid_volume_name() {
        assert!(BindMount::from_str("MyVolume-:/some/container/path").is_err())
    }

    #[test]
    fn default_docker_socket_bind_mount() {
        assert_eq!(
            BindMount::default_docker_socket_bind_mount().host_path,
            PathBuf::from("/run/docker.sock")
        );
        assert_eq!(
            BindMount::default_docker_socket_bind_mount().container_path,
            PathBuf::from("/run/docker.sock")
        );
    }
}
