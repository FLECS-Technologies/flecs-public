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
use std::ops::Deref;

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
    pub fn arguments(&self) -> Vec<String> {
        match &self.original.args {
            None => Vec::new(),
            Some(args) => args.deref().clone(),
        }
    }

    pub fn capabilities(
        &self,
    ) -> HashSet<flecs_app_manifest::generated::manifest_3_1_0::CapabilitiesItem> {
        self.original
            .capabilities
            .as_ref()
            .map(|capabilities| HashSet::from_iter(capabilities.deref().iter().cloned()))
            .unwrap_or_default()
    }

    pub fn multi_instance(&self) -> bool {
        match self.original.multi_instance.as_ref() {
            None => false,
            Some(multi_instance) => multi_instance.0,
        }
    }

    pub fn interactive(&self) -> bool {
        match self.original.interactive.as_ref() {
            None => false,
            Some(interactive) => interactive.0,
        }
    }

    pub fn revision(&self) -> Option<&String> {
        self.original.revision.as_deref()
    }

    pub fn editors(&self) -> Vec<flecs_app_manifest::generated::manifest_3_1_0::EditorsItem> {
        match &self.original.editors {
            None => Vec::new(),
            Some(editors) => editors.0.clone(),
        }
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

    pub fn hostname(&self) -> Option<String> {
        self.original.hostname.as_ref().map(ToString::to_string)
    }

    pub fn schema(&self) -> Option<&String> {
        self.original.schema.as_deref()
    }
}

impl TryFrom<flecs_app_manifest::AppManifestVersion> for AppManifest {
    type Error = Error;

    fn try_from(value: flecs_app_manifest::AppManifestVersion) -> Result<Self, Self::Error> {
        flecs_app_manifest::AppManifest::try_from(value)?.try_into()
    }
}

fn try_from_option<'s, S, D, E>(source: Option<&'s Vec<S>>) -> Result<Vec<D>, E>
where
    D: TryFrom<&'s S, Error = E>,
{
    match source {
        None => Ok(Vec::new()),
        Some(source) => source
            .iter()
            .map(D::try_from)
            .collect::<Result<Vec<_>, _>>(),
    }
}

impl TryFrom<flecs_app_manifest::AppManifest> for AppManifest {
    type Error = Error;

    fn try_from(value: flecs_app_manifest::AppManifest) -> Result<Self, Self::Error> {
        let mut environment_variable_names: HashSet<String> = HashSet::new();
        let mut environment_variables: Vec<EnvironmentVariable> = Vec::new();
        match &value.env {
            Some(env) => {
                for environment in env.deref() {
                    let env = EnvironmentVariable::try_from(environment)?;
                    if !environment_variable_names.insert(env.name.clone()) {
                        anyhow::bail!(
                            "Duplicate environment variable with name '{}' detected",
                            env.name,
                        )
                    }
                    environment_variables.push(env);
                }
            }
            None => {}
        }
        Ok(Self {
            key: AppKey {
                name: value.app.to_string(),
                version: value.version.to_string(),
            },
            config_files: try_from_option(value.conffiles.as_deref())?,
            mounts: try_from_option(value.volumes.as_deref())?,
            environment_variables,
            devices: try_from_option(value.devices.as_deref())?,
            labels: try_from_option(value.labels.as_deref())?,
            ports: try_from_option(value.ports.as_deref())?,
            original: value,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::sync::Arc;

    pub fn create_test_manifest_numbered(
        name_number: u8,
        version_number: u8,
        revision: Option<String>,
    ) -> Arc<AppManifest> {
        Arc::new(
            AppManifest::try_from(flecs_app_manifest::AppManifestVersion::V3_1_0(
                create_test_manifest_numbered_raw(name_number, version_number, revision),
            ))
            .unwrap(),
        )
    }

    pub fn create_test_manifest_numbered_raw(
        name_number: u8,
        version_number: u8,
        revision: Option<String>,
    ) -> flecs_app_manifest::generated::manifest_3_1_0::FlecsAppManifest {
        flecs_app_manifest::generated::manifest_3_1_0::FlecsAppManifest::Single(
            flecs_app_manifest::generated::manifest_3_1_0::Single {
                app: FromStr::from_str(&format!("some.test.app-{name_number}")).unwrap(),
                args: None,
                capabilities: None,
                conffiles: None,
                devices: None,
                editors: None,
                env: None,
                hostname: None,
                image: FromStr::from_str("flecs.azurecr.io/io.anyviz.cloudadapter").unwrap(),
                interactive: None,
                labels: None,
                minimum_flecs_version: None,
                multi_instance: None,
                ports: None,
                revision: revision.map(From::from),
                schema: None,
                version: FromStr::from_str(&format!("1.2.{version_number}")).unwrap(),
                volumes: None,
            },
        )
    }

    pub fn create_test_manifest(revision: Option<String>) -> Arc<AppManifest> {
        create_test_manifest_numbered(0, 0, revision)
    }

    pub fn create_test_manifest_raw(
        revision: Option<String>,
    ) -> flecs_app_manifest::generated::manifest_3_1_0::FlecsAppManifest {
        create_test_manifest_numbered_raw(0, 0, revision)
    }

    pub fn create_test_manifest_full(multi_instance: Option<bool>) -> AppManifest {
        AppManifest::try_from(flecs_app_manifest::AppManifestVersion::V3_1_0(
            flecs_app_manifest::generated::manifest_3_1_0::FlecsAppManifest::Single(
                flecs_app_manifest::generated::manifest_3_1_0::Single {
                    app: FromStr::from_str("some.test.app").unwrap(),
                    args: Some(
                        vec![
                            "--launch-arg1".to_string(),
                            "--launch-arg2=value".to_string(),
                        ]
                        .into(),
                    ),
                    capabilities: Some(
                        vec![
                    flecs_app_manifest::generated::manifest_3_1_0::CapabilitiesItem::Docker,
                    flecs_app_manifest::generated::manifest_3_1_0::CapabilitiesItem::NetAdmin,
                    flecs_app_manifest::generated::manifest_3_1_0::CapabilitiesItem::SysNice,
                ]
                        .into(),
                    ),
                    conffiles: Some(
                        vec![
                            FromStr::from_str("default.conf:/etc/my-app/default.conf").unwrap(),
                            FromStr::from_str("default.conf:/etc/my-app/default.conf:rw").unwrap(),
                            FromStr::from_str("default.conf:/etc/my-app/default.conf:ro").unwrap(),
                        ]
                        .into(),
                    ),
                    devices: Some(
                        vec![
                            FromStr::from_str("/dev/dev1").unwrap(),
                            FromStr::from_str("/dev/usb/dev1").unwrap(),
                        ]
                        .into(),
                    ),
                    editors: Some(
                        vec![
                            flecs_app_manifest::generated::manifest_3_1_0::EditorsItem {
                                name: "Editor#1".to_string(),
                                port: std::num::NonZeroU16::new(123).unwrap(),
                                supports_reverse_proxy: false,
                            },
                            flecs_app_manifest::generated::manifest_3_1_0::EditorsItem {
                                name: "Editor#2".to_string(),
                                port: std::num::NonZeroU16::new(789).unwrap(),
                                supports_reverse_proxy: true,
                            },
                        ]
                        .into(),
                    ),
                    env: Some(
                        vec![
                            FromStr::from_str("ENV_VAR_1=value-1").unwrap(),
                            FromStr::from_str("ENV_VAR_2=value-2").unwrap(),
                        ]
                        .into(),
                    ),
                    hostname: Some("TestHostName".parse().unwrap()),
                    image: FromStr::from_str("flecs.azurecr.io/some.test.app").unwrap(),
                    interactive: None,
                    labels: Some(
                        vec![
                            FromStr::from_str("my.label-one=value-1").unwrap(),
                            FromStr::from_str("my.label-two=").unwrap(),
                            FromStr::from_str("my.label-three").unwrap(),
                        ]
                        .into(),
                    ),
                    minimum_flecs_version: Some(FromStr::from_str("3.0.0").unwrap()),
                    multi_instance: multi_instance.map(|b| b.into()),
                    ports: Some(
                        vec![
                            FromStr::from_str("8001:8001").unwrap(),
                            FromStr::from_str("5000").unwrap(),
                            FromStr::from_str("5001-5008:6001-6008").unwrap(),
                            FromStr::from_str("6001-6008").unwrap(),
                        ]
                        .into(),
                    ),
                    revision: Some(FromStr::from_str("5").unwrap()),
                    schema: Some(FromStr::from_str("/path/to/manifest/schema.json").unwrap()),
                    version: FromStr::from_str("1.2.1").unwrap(),
                    volumes: Some(
                        vec![
                            FromStr::from_str("my-app-etc:/etc/my-app").unwrap(),
                            FromStr::from_str("/etc/my-app:/etc/my-app").unwrap(),
                        ]
                        .into(),
                    ),
                },
            ),
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
            vec![
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
            HashSet::from([
                flecs_app_manifest::generated::manifest_3_1_0::CapabilitiesItem::Docker,
                flecs_app_manifest::generated::manifest_3_1_0::CapabilitiesItem::NetAdmin,
                flecs_app_manifest::generated::manifest_3_1_0::CapabilitiesItem::SysNice,
            ])
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
            flecs_app_manifest::generated::manifest_3_1_0::EditorsItem {
                name: "Editor#1".to_string(),
                port: std::num::NonZeroU16::new(123).unwrap(),
                supports_reverse_proxy: false,
            },
            flecs_app_manifest::generated::manifest_3_1_0::EditorsItem {
                name: "Editor#2".to_string(),
                port: std::num::NonZeroU16::new(789).unwrap(),
                supports_reverse_proxy: true,
            },
        ];

        assert_eq!(manifest.editors(), editors)
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
    fn schema() {
        let manifest = create_test_manifest_full(None);

        assert_eq!(
            manifest.schema(),
            Some(&"/path/to/manifest/schema.json".to_string())
        )
    }

    #[test]
    fn try_from_duplicate_environment_err() {
        assert!(
            AppManifest::try_from(flecs_app_manifest::AppManifestVersion::V3_1_0(
                flecs_app_manifest::generated::manifest_3_1_0::FlecsAppManifest::Single(
                    flecs_app_manifest::generated::manifest_3_1_0::Single {
                        app: FromStr::from_str("some.test.app").unwrap(),
                        args: None,
                        capabilities: None,
                        conffiles: None,
                        devices: None,
                        editors: None,
                        env: Some(
                            vec![
                                FromStr::from_str("ENV_VAR_1=value-1").unwrap(),
                                FromStr::from_str("ENV_VAR_1=value-2").unwrap(),
                            ]
                            .into()
                        ),
                        hostname: None,
                        image: FromStr::from_str("flecs.azurecr.io/some.test.app").unwrap(),
                        interactive: None,
                        labels: None,
                        minimum_flecs_version: None,
                        multi_instance: None,
                        ports: None,
                        revision: None,
                        schema: None,
                        version: FromStr::from_str("1.2.1").unwrap(),
                        volumes: None,
                    },
                )
            ))
            .is_err()
        )
    }
}
