mod config_file;
mod device;
mod environment_variable;
mod label;
mod mount;
mod port;

use crate::vault::pouch::AppKey;
pub use crate::{Error, Result};
pub use config_file::*;
pub use device::*;
pub use environment_variable::*;
pub use label::*;
pub use mount::*;
pub use port::*;
use serde::Serialize;
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub struct AppManifest {
    #[serde(skip_serializing)]
    pub key: AppKey,
    #[serde(skip_serializing)]
    pub config_files: Vec<ConfigFile>,
    #[serde(skip_serializing)]
    pub mounts: Vec<Mount>,
    #[serde(skip_serializing)]
    pub environment_variables: Vec<EnvironmentVariable>,
    #[serde(skip_serializing)]
    pub devices: Vec<Device>,
    #[serde(skip_serializing)]
    pub labels: Vec<Label>,
    #[serde(skip_serializing)]
    pub ports: Vec<PortMapping>,
    #[serde(flatten)]
    original: flecs_app_manifest::AppManifest,
}

impl AppManifest {
    pub fn arguments(&self) -> &Vec<String> {
        &self.original.args
    }

    pub fn capabilities(
        &self,
    ) -> Vec<flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestCapabilitiesItem> {
        self.original.capabilities.clone().unwrap_or_default()
    }

    pub fn multi_instance(&self) -> bool {
        self.original.multi_instance.unwrap_or_default()
    }

    pub fn interactive(&self) -> bool {
        self.original.interactive.unwrap_or_default()
    }

    pub fn revision(&self) -> Option<&String> {
        self.original.revision.as_ref()
    }

    pub fn editors(
        &self,
    ) -> &Vec<flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestEditorsItem> {
        &self.original.editors
    }

    pub fn image(&self) -> &str {
        self.original.image.as_str()
    }

    pub fn image_with_tag(&self) -> String {
        format!("{}:{}", self.original.image.as_str(), self.key.version)
    }

    pub fn minimum_flecs_version(&self) -> Option<&str> {
        self.original
            .minimum_flecs_version
            .as_ref()
            .map(|version| version.as_str())
    }

    pub fn volume_mounts(&self) -> Vec<VolumeMount> {
        self.mounts
            .iter()
            .filter_map(|mount| match mount {
                Mount::Volume(volume_mount) => Some(volume_mount),
                _ => None,
            })
            .cloned()
            .collect()
    }

    pub fn bind_mounts(&self) -> Vec<BindMount> {
        self.mounts
            .iter()
            .filter_map(|mount| match mount {
                Mount::Bind(bind_mount) => Some(bind_mount),
                _ => None,
            })
            .cloned()
            .collect()
    }
}

impl TryFrom<flecs_app_manifest::AppManifestVersion> for AppManifest {
    type Error = Error;

    fn try_from(value: flecs_app_manifest::AppManifestVersion) -> Result<Self, Self::Error> {
        flecs_app_manifest::AppManifest::try_from(value)?.try_into()
    }
}

impl TryFrom<flecs_app_manifest::AppManifest> for AppManifest {
    type Error = Error;

    fn try_from(value: flecs_app_manifest::AppManifest) -> Result<Self, Self::Error> {
        let mut environment_variable_names: HashSet<String> = HashSet::new();
        let mut environment_variables: Vec<EnvironmentVariable> = Vec::new();
        for environment in value.env.iter() {
            let env = EnvironmentVariable::from_str(environment.as_str())?;
            if !environment_variable_names.insert(env.name.clone()) {
                anyhow::bail!(
                    "Duplicate environment variable with name '{}' detected",
                    env.name,
                )
            }
            environment_variables.push(env);
        }
        Ok(Self {
            key: AppKey {
                name: value.app.to_string(),
                version: value.version.clone(),
            },
            config_files: value
                .conffiles
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<ConfigFile>, _>>()?,
            mounts: value
                .volumes
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<Mount>, _>>()?,
            environment_variables,
            devices: value
                .devices
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<Device>, _>>()?,
            labels: value
                .labels
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<Label>, _>>()?,
            ports: value
                .ports
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<PortMapping>, _>>()?,
            original: value,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    pub fn create_test_manifest_numbered(
        name_number: u8,
        version_number: u8,
        revision: Option<String>,
    ) -> Arc<AppManifest> {
        Arc::new(
            AppManifest::try_from(flecs_app_manifest::AppManifestVersion::V3_0_0(
                create_test_manifest_numbered_raw(name_number, version_number, revision),
            ))
            .unwrap(),
        )
    }

    pub fn create_test_manifest_numbered_raw(
        name_number: u8,
        version_number: u8,
        revision: Option<String>,
    ) -> flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifest {
        flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifest {
            app: FromStr::from_str(&format!("some.test.app-{name_number}")).unwrap(),
            args: vec![],
            capabilities: None,
            conffiles: vec![],
            devices: vec![],
            editors: vec![],
            env: vec![],
            image: FromStr::from_str("flecs.azurecr.io/io.anyviz.cloudadapter").unwrap(),
            interactive: None,
            labels: vec![],
            minimum_flecs_version: None,
            multi_instance: None,
            ports: vec![],
            revision,
            version: FromStr::from_str(&format!("1.2.{version_number}")).unwrap(),
            volumes: vec![],
        }
    }

    pub fn create_test_manifest(revision: Option<String>) -> Arc<AppManifest> {
        create_test_manifest_numbered(0, 0, revision)
    }

    pub fn create_test_manifest_raw(
        revision: Option<String>,
    ) -> flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifest {
        create_test_manifest_numbered_raw(0, 0, revision)
    }

    pub fn create_test_manifest_full(multi_instance: Option<bool>) -> AppManifest {
        AppManifest::try_from(flecs_app_manifest::AppManifestVersion::V3_0_0(
            flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifest {
                app: FromStr::from_str("some.test.app").unwrap(),
                args: vec![
                    "--launch-arg1".to_string(),
                    "--launch-arg2=value".to_string(),
                ],
                capabilities: Some(vec![
                    flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestCapabilitiesItem::Docker,
                    flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestCapabilitiesItem::NetAdmin,
                    flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestCapabilitiesItem::SysNice,
                ]),
                conffiles: vec![
                    FromStr::from_str("default.conf:/etc/my-app/default.conf").unwrap(),
                    FromStr::from_str("default.conf:/etc/my-app/default.conf:rw").unwrap(),
                    FromStr::from_str("default.conf:/etc/my-app/default.conf:ro").unwrap(),
                ],
                devices: vec![
                    FromStr::from_str("/dev/dev1").unwrap(),
                    FromStr::from_str("/dev/usb/dev1").unwrap(),
                ],
                editors: vec![
                    flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestEditorsItem {
                        name: "Editor#1".to_string(),
                        port: std::num::NonZeroU16::new(123).unwrap(),
                        supports_reverse_proxy: false,
                    },
                    flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestEditorsItem {
                        name: "Editor#2".to_string(),
                        port: std::num::NonZeroU16::new(789).unwrap(),
                        supports_reverse_proxy: true,
                    },
                ],
                env: vec![
                    FromStr::from_str("ENV_VAR_1=value-1").unwrap(),
                    FromStr::from_str("ENV_VAR_2=value-2").unwrap(),
                ],
                image: FromStr::from_str("flecs.azurecr.io/some.test.app").unwrap(),
                interactive: None,
                labels: vec![
                    FromStr::from_str("my.label-one=value-1").unwrap(),
                    FromStr::from_str("my.label-two=").unwrap(),
                    FromStr::from_str("my.label-three").unwrap(),
                ],
                minimum_flecs_version: Some(FromStr::from_str("3.0.0").unwrap()),
                multi_instance,
                ports: vec![
                    FromStr::from_str("8001:8001").unwrap(),
                    FromStr::from_str("5000").unwrap(),
                    FromStr::from_str("5001-5008:6001-6008").unwrap(),
                    FromStr::from_str("6001-6008").unwrap(),
                ],
                revision: Some("5".to_string()),
                version: FromStr::from_str("1.2.1").unwrap(),
                volumes: vec![
                    FromStr::from_str("my-app-etc:/etc/my-app").unwrap(),
                    FromStr::from_str("/etc/my-app:/etc/my-app").unwrap(),
                ],
            },
        ))
        .unwrap()
    }

    #[test]
    fn manifest_key() {
        let manifest = create_test_manifest_full(None);
        assert_eq!(
            manifest.key,
            AppKey {
                name: "some.test.app".to_string(),
                version: "1.2.1".to_string()
            }
        )
    }

    #[test]
    fn config_files() {
        let manifest = create_test_manifest_full(None);

        assert_eq!(
            manifest.config_files,
            vec![
                ConfigFile {
                    host_file_name: "default.conf".to_string(),
                    container_file_path: PathBuf::from("/etc/my-app/default.conf"),
                    read_only: false,
                },
                ConfigFile {
                    host_file_name: "default.conf".to_string(),
                    container_file_path: PathBuf::from("/etc/my-app/default.conf"),
                    read_only: false,
                },
                ConfigFile {
                    host_file_name: "default.conf".to_string(),
                    container_file_path: PathBuf::from("/etc/my-app/default.conf"),
                    read_only: true,
                },
            ]
        )
    }

    #[test]
    fn mounts() {
        let manifest = create_test_manifest_full(None);

        assert_eq!(
            manifest.mounts,
            vec![
                Mount::Volume(VolumeMount {
                    name: "my-app-etc".to_string(),
                    container_path: PathBuf::from("/etc/my-app"),
                }),
                Mount::Bind(BindMount {
                    host_path: PathBuf::from("/etc/my-app"),
                    container_path: PathBuf::from("/etc/my-app"),
                }),
            ]
        )
    }

    #[test]
    fn environment_variables() {
        let manifest = create_test_manifest_full(None);

        assert_eq!(
            manifest.environment_variables,
            vec![
                EnvironmentVariable {
                    name: "ENV_VAR_1".to_string(),
                    value: Some("value-1".to_string()),
                },
                EnvironmentVariable {
                    name: "ENV_VAR_2".to_string(),
                    value: Some("value-2".to_string()),
                },
            ]
        )
    }

    #[test]
    fn devices() {
        let manifest = create_test_manifest_full(None);

        assert_eq!(
            manifest.devices,
            vec![
                Device {
                    path: PathBuf::from("/dev/dev1"),
                },
                Device {
                    path: PathBuf::from("/dev/usb/dev1"),
                },
            ]
        )
    }

    #[test]
    fn labels() {
        let manifest = create_test_manifest_full(None);

        assert_eq!(
            manifest.labels,
            vec![
                Label {
                    label: "my.label-one".to_string(),
                    value: Some("value-1".to_string()),
                },
                Label {
                    label: "my.label-two".to_string(),
                    value: Some(String::new()),
                },
                Label {
                    label: "my.label-three".to_string(),
                    value: None,
                },
            ]
        )
    }

    #[test]
    fn ports() {
        let manifest = create_test_manifest_full(None);

        assert_eq!(
            manifest.ports,
            vec![
                PortMapping::Single(8001, 8001),
                PortMapping::Single(5000, 5000),
                PortMapping::Range {
                    from: PortRange::try_new(5001, 5008).unwrap(),
                    to: PortRange::try_new(6001, 6008).unwrap()
                },
                PortMapping::Range {
                    from: PortRange::try_new(6001, 6008).unwrap(),
                    to: PortRange::try_new(6001, 6008).unwrap()
                },
            ]
        )
    }

    #[test]
    fn arguments() {
        let manifest = create_test_manifest_full(None);

        assert_eq!(
            manifest.arguments(),
            &vec![
                "--launch-arg1".to_string(),
                "--launch-arg2=value".to_string(),
            ]
        )
    }

    #[test]
    fn capabilities() {
        let manifest = create_test_manifest_full(None);

        assert_eq!(
            manifest.capabilities(),
            vec![
                flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestCapabilitiesItem::Docker,
                flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestCapabilitiesItem::NetAdmin,
                flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestCapabilitiesItem::SysNice,
            ]
        )
    }

    #[test]
    fn multi_instance() {
        let manifest = create_test_manifest_full(None);

        assert!(!manifest.multi_instance())
    }

    #[test]
    fn interactive() {
        let manifest = create_test_manifest_full(None);

        assert!(!manifest.interactive())
    }

    #[test]
    fn revision() {
        let manifest = create_test_manifest_full(None);

        assert_eq!(manifest.revision(), Some(&"5".to_string()))
    }

    #[test]
    fn editors() {
        let manifest = create_test_manifest_full(None);
        let editors = vec![
            flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestEditorsItem {
                name: "Editor#1".to_string(),
                port: std::num::NonZeroU16::new(123).unwrap(),
                supports_reverse_proxy: false,
            },
            flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestEditorsItem {
                name: "Editor#2".to_string(),
                port: std::num::NonZeroU16::new(789).unwrap(),
                supports_reverse_proxy: true,
            },
        ];

        assert_eq!(manifest.editors(), &editors)
    }

    #[test]
    fn image() {
        let manifest = create_test_manifest_full(None);

        assert_eq!(manifest.image(), "flecs.azurecr.io/some.test.app")
    }

    #[test]
    fn image_with_tag() {
        let manifest = create_test_manifest_full(None);

        assert_eq!(
            manifest.image_with_tag(),
            "flecs.azurecr.io/some.test.app:1.2.1"
        )
    }

    #[test]
    fn minimum_flecs_version() {
        let manifest = create_test_manifest_full(None);

        assert_eq!(manifest.minimum_flecs_version(), Some("3.0.0"))
    }

    #[test]
    fn volume_mounts() {
        let manifest = create_test_manifest_full(None);

        assert_eq!(
            manifest.volume_mounts(),
            vec!(VolumeMount {
                name: "my-app-etc".to_string(),
                container_path: PathBuf::from("/etc/my-app"),
            })
        )
    }

    #[test]
    fn bind_mounts() {
        let manifest = create_test_manifest_full(None);

        assert_eq!(
            manifest.bind_mounts(),
            vec!(BindMount {
                host_path: PathBuf::from("/etc/my-app"),
                container_path: PathBuf::from("/etc/my-app"),
            })
        )
    }

    #[test]
    fn try_from_duplicate_environment_err() {
        assert!(
            AppManifest::try_from(flecs_app_manifest::AppManifestVersion::V3_0_0(
                flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifest {
                    app: FromStr::from_str("some.test.app").unwrap(),
                    args: Vec::new(),
                    capabilities: None,
                    conffiles: Vec::new(),
                    devices: Vec::new(),
                    editors: Vec::new(),
                    env: vec![
                        FromStr::from_str("ENV_VAR_1=value-1").unwrap(),
                        FromStr::from_str("ENV_VAR_1=value-2").unwrap(),
                    ],
                    image: FromStr::from_str("flecs.azurecr.io/some.test.app").unwrap(),
                    interactive: None,
                    labels: Vec::new(),
                    minimum_flecs_version: None,
                    multi_instance: None,
                    ports: Vec::new(),
                    revision: None,
                    version: FromStr::from_str("1.2.1").unwrap(),
                    volumes: Vec::new(),
                },
            ))
            .is_err()
        )
    }
}
