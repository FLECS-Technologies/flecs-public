use crate::generated::{manifest_2_0_0, manifest_3_0_0, manifest_3_1_0, manifest_3_2_0};
use std::num::ParseIntError;
use std::str::FromStr;
use std::string::ToString;

impl TryFrom<&manifest_3_0_0::FlecsAppManifest> for manifest_3_2_0::Single {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(manifest: &manifest_3_0_0::FlecsAppManifest) -> Result<Self, Self::Error> {
        let conffiles = if manifest.conffiles.is_empty() {
            None
        } else {
            Some((&manifest.conffiles).try_into()?)
        };
        let args = if manifest.args.is_empty() {
            None
        } else {
            Some(manifest_3_2_0::Args::from(manifest.args.clone()))
        };
        let devices = if manifest.devices.is_empty() {
            None
        } else {
            Some(manifest_3_2_0::Devices::try_from(&manifest.devices)?)
        };
        let editors = if manifest.editors.is_empty() {
            None
        } else {
            Some(manifest_3_2_0::Editors::from(manifest.editors.clone()))
        };
        let env = if manifest.env.is_empty() {
            None
        } else {
            Some(manifest_3_2_0::Env::try_from(&manifest.env)?)
        };
        let labels = if manifest.labels.is_empty() {
            None
        } else {
            Some(manifest_3_2_0::Labels::try_from(&manifest.labels)?)
        };
        let minimum_flecs_version = if let Some(version) = manifest.minimum_flecs_version.as_ref() {
            Some(version.try_into()?)
        } else {
            None
        };
        let ports = if manifest.ports.is_empty() {
            None
        } else {
            Some(manifest_3_2_0::Ports::try_from(&manifest.ports)?)
        };
        let volumes = if manifest.volumes.is_empty() {
            None
        } else {
            Some(manifest_3_2_0::Volumes::try_from(&manifest.volumes)?)
        };
        Ok(manifest_3_2_0::Single {
            app: (&manifest.app).try_into()?,
            args,
            capabilities: manifest
                .capabilities
                .as_ref()
                .map(manifest_3_2_0::Capabilities::from),
            conffiles,
            devices,
            editors,
            env,
            hostname: None,
            image: (&manifest.image).try_into()?,
            interactive: manifest.interactive.map(manifest_3_2_0::Interactive::from),
            labels,
            minimum_flecs_version,
            multi_instance: manifest
                .multi_instance
                .map(manifest_3_2_0::MultiInstance::from),
            ports,
            revision: manifest
                .revision
                .clone()
                .map(manifest_3_2_0::Revision::from),
            schema: None,
            version: manifest.version.clone().into(),
            volumes,
            depends: None,
            provides: None,
            recommends: None,
        })
    }
}

impl TryFrom<&manifest_3_0_0::FlecsAppManifestApp> for manifest_3_2_0::App {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(app: &manifest_3_0_0::FlecsAppManifestApp) -> Result<Self, Self::Error> {
        Self::try_from(app.as_str())
    }
}

impl From<&Vec<manifest_3_0_0::FlecsAppManifestCapabilitiesItem>> for manifest_3_2_0::Capabilities {
    fn from(capabilities: &Vec<manifest_3_0_0::FlecsAppManifestCapabilitiesItem>) -> Self {
        let capabilities: Vec<manifest_3_2_0::CapabilitiesItem> = capabilities
            .iter()
            .map(manifest_3_2_0::CapabilitiesItem::from)
            .collect();
        Self::from(capabilities)
    }
}

impl From<&manifest_3_0_0::FlecsAppManifestCapabilitiesItem> for manifest_3_2_0::CapabilitiesItem {
    fn from(value: &manifest_3_0_0::FlecsAppManifestCapabilitiesItem) -> Self {
        match value {
            manifest_3_0_0::FlecsAppManifestCapabilitiesItem::Docker => Self::Docker,
            manifest_3_0_0::FlecsAppManifestCapabilitiesItem::NetAdmin => Self::NetAdmin,
            manifest_3_0_0::FlecsAppManifestCapabilitiesItem::SysNice => Self::SysNice,
            manifest_3_0_0::FlecsAppManifestCapabilitiesItem::IpcLock => Self::IpcLock,
            manifest_3_0_0::FlecsAppManifestCapabilitiesItem::NetRaw => Self::NetRaw,
        }
    }
}

impl TryFrom<&Vec<manifest_3_0_0::FlecsAppManifestConffilesItem>> for manifest_3_2_0::Conffiles {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(
        conffiles: &Vec<manifest_3_0_0::FlecsAppManifestConffilesItem>,
    ) -> Result<Self, Self::Error> {
        let conffiles: Result<Vec<manifest_3_2_0::ConffilesItem>, _> = conffiles
            .iter()
            .map(manifest_3_2_0::ConffilesItem::try_from)
            .collect();
        Ok(Self::from(conffiles?))
    }
}

impl TryFrom<&manifest_3_0_0::FlecsAppManifestConffilesItem> for manifest_3_2_0::ConffilesItem {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(
        value: &manifest_3_0_0::FlecsAppManifestConffilesItem,
    ) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl TryFrom<&Vec<manifest_3_0_0::FlecsAppManifestDevicesItem>> for manifest_3_2_0::Devices {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(
        devices: &Vec<manifest_3_0_0::FlecsAppManifestDevicesItem>,
    ) -> Result<Self, Self::Error> {
        let devices: Result<Vec<manifest_3_2_0::DevicesItem>, _> = devices
            .iter()
            .map(manifest_3_2_0::DevicesItem::try_from)
            .collect();
        Ok(Self::from(devices?))
    }
}

impl TryFrom<&manifest_3_0_0::FlecsAppManifestDevicesItem> for manifest_3_2_0::DevicesItem {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_0_0::FlecsAppManifestDevicesItem) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl From<Vec<manifest_3_0_0::FlecsAppManifestEditorsItem>> for manifest_3_2_0::Editors {
    fn from(value: Vec<manifest_3_0_0::FlecsAppManifestEditorsItem>) -> Self {
        Self::from(
            value
                .into_iter()
                .map(manifest_3_2_0::EditorsItem::from)
                .collect::<Vec<_>>(),
        )
    }
}
impl From<manifest_3_0_0::FlecsAppManifestEditorsItem> for manifest_3_2_0::EditorsItem {
    fn from(value: manifest_3_0_0::FlecsAppManifestEditorsItem) -> Self {
        Self {
            name: value.name,
            port: value.port,
            supports_reverse_proxy: value.supports_reverse_proxy,
        }
    }
}

impl TryFrom<&Vec<manifest_3_0_0::FlecsAppManifestEnvItem>> for manifest_3_2_0::Env {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(
        devices: &Vec<manifest_3_0_0::FlecsAppManifestEnvItem>,
    ) -> Result<Self, Self::Error> {
        let devices: Result<Vec<manifest_3_2_0::EnvItem>, _> = devices
            .iter()
            .map(manifest_3_2_0::EnvItem::try_from)
            .collect();
        Ok(Self::from(devices?))
    }
}

impl TryFrom<&manifest_3_0_0::FlecsAppManifestEnvItem> for manifest_3_2_0::EnvItem {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_0_0::FlecsAppManifestEnvItem) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl TryFrom<&manifest_3_0_0::FlecsAppManifestImage> for manifest_3_2_0::Image {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_0_0::FlecsAppManifestImage) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl TryFrom<&Vec<manifest_3_0_0::FlecsAppManifestLabelsItem>> for manifest_3_2_0::Labels {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(
        labels: &Vec<manifest_3_0_0::FlecsAppManifestLabelsItem>,
    ) -> Result<Self, Self::Error> {
        let labels: Result<Vec<manifest_3_2_0::LabelsItem>, _> = labels
            .iter()
            .map(manifest_3_2_0::LabelsItem::try_from)
            .collect();
        Ok(Self::from(labels?))
    }
}

impl TryFrom<&manifest_3_0_0::FlecsAppManifestLabelsItem> for manifest_3_2_0::LabelsItem {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_0_0::FlecsAppManifestLabelsItem) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl TryFrom<&manifest_3_0_0::FlecsAppManifestMinimumFlecsVersion>
    for manifest_3_2_0::MinimumFlecsVersion
{
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(
        value: &manifest_3_0_0::FlecsAppManifestMinimumFlecsVersion,
    ) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl TryFrom<&Vec<manifest_3_0_0::FlecsAppManifestPortsItem>> for manifest_3_2_0::Ports {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(
        ports: &Vec<manifest_3_0_0::FlecsAppManifestPortsItem>,
    ) -> Result<Self, Self::Error> {
        let ports: Result<Vec<manifest_3_2_0::PortsItem>, _> = ports
            .iter()
            .map(manifest_3_2_0::PortsItem::try_from)
            .collect();
        Ok(Self(ports?))
    }
}

impl TryFrom<&manifest_3_0_0::FlecsAppManifestPortsItem> for manifest_3_2_0::PortsItem {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_0_0::FlecsAppManifestPortsItem) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl TryFrom<&Vec<manifest_3_0_0::FlecsAppManifestVolumesItem>> for manifest_3_2_0::Volumes {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(
        volumes: &Vec<manifest_3_0_0::FlecsAppManifestVolumesItem>,
    ) -> Result<Self, Self::Error> {
        let volumes: Result<Vec<manifest_3_2_0::VolumesItem>, _> = volumes
            .iter()
            .map(manifest_3_2_0::VolumesItem::try_from)
            .collect();
        Ok(Self(volumes?))
    }
}

impl TryFrom<&manifest_3_0_0::FlecsAppManifestVolumesItem> for manifest_3_2_0::VolumesItem {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_0_0::FlecsAppManifestVolumesItem) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl TryFrom<&manifest_2_0_0::FlecsAppManifest> for manifest_3_2_0::Single {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(manifest: &manifest_2_0_0::FlecsAppManifest) -> Result<Self, Self::Error> {
        let conffiles = if manifest.conffiles.is_empty() {
            None
        } else {
            Some((&manifest.conffiles).try_into()?)
        };
        let args = if manifest.args.is_empty() {
            None
        } else {
            Some(manifest_3_2_0::Args::from(manifest.args.clone()))
        };
        let devices = if manifest.devices.is_empty() {
            None
        } else {
            Some(manifest_3_2_0::Devices::try_from(&manifest.devices)?)
        };
        let editors = match manifest.editor.as_ref() {
            None => None,
            Some(editor) => {
                let editors = manifest_3_2_0::Editors::try_from(editor)?;
                if (*editors).is_empty() {
                    None
                } else {
                    Some(editors)
                }
            }
        };
        let env = if manifest.env.is_empty() {
            None
        } else {
            Some(manifest_3_2_0::Env::try_from(&manifest.env)?)
        };
        let labels = if manifest.labels.is_empty() {
            None
        } else {
            Some(manifest_3_2_0::Labels::try_from(&manifest.labels)?)
        };
        let ports = if manifest.ports.is_empty() {
            None
        } else {
            Some(manifest_3_2_0::Ports::try_from(&manifest.ports)?)
        };
        let volumes = if manifest.volumes.is_empty() {
            None
        } else {
            Some(manifest_3_2_0::Volumes::try_from(&manifest.volumes)?)
        };
        Ok(manifest_3_2_0::Single {
            app: (&manifest.app).try_into()?,
            args,
            capabilities: manifest
                .capabilities
                .as_ref()
                .map(manifest_3_2_0::Capabilities::from),
            conffiles,
            depends: None,
            devices,
            editors,
            env,
            hostname: manifest
                .hostname
                .clone()
                .map(manifest_3_2_0::Hostname::from),
            image: (&manifest.image).try_into()?,
            interactive: manifest.interactive.map(manifest_3_2_0::Interactive::from),
            labels,
            minimum_flecs_version: None,
            multi_instance: manifest
                .multi_instance
                .map(manifest_3_2_0::MultiInstance::from),
            ports,
            provides: None,
            recommends: None,
            revision: manifest
                .revision
                .clone()
                .map(manifest_3_2_0::Revision::from),
            schema: None,
            version: manifest.version.clone().into(),
            volumes,
        })
    }
}

impl TryFrom<&manifest_2_0_0::FlecsAppManifestApp> for manifest_3_2_0::App {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(app: &manifest_2_0_0::FlecsAppManifestApp) -> Result<Self, Self::Error> {
        Self::try_from(app.as_str())
    }
}

impl TryFrom<&manifest_2_0_0::FlecsAppManifestImage> for manifest_3_2_0::Image {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(app: &manifest_2_0_0::FlecsAppManifestImage) -> Result<Self, Self::Error> {
        Self::try_from(app.as_str())
    }
}

impl From<&manifest_2_0_0::FlecsAppManifestCapabilitiesItem> for manifest_3_2_0::CapabilitiesItem {
    fn from(value: &manifest_2_0_0::FlecsAppManifestCapabilitiesItem) -> Self {
        match value {
            manifest_2_0_0::FlecsAppManifestCapabilitiesItem::Docker => Self::Docker,
            manifest_2_0_0::FlecsAppManifestCapabilitiesItem::NetAdmin => Self::NetAdmin,
            manifest_2_0_0::FlecsAppManifestCapabilitiesItem::SysNice => Self::SysNice,
            manifest_2_0_0::FlecsAppManifestCapabilitiesItem::IpcLock => Self::IpcLock,
            manifest_2_0_0::FlecsAppManifestCapabilitiesItem::NetRaw => Self::NetRaw,
        }
    }
}

impl From<&Vec<manifest_2_0_0::FlecsAppManifestCapabilitiesItem>> for manifest_3_2_0::Capabilities {
    fn from(capabilities: &Vec<manifest_2_0_0::FlecsAppManifestCapabilitiesItem>) -> Self {
        let capabilities: Vec<manifest_3_2_0::CapabilitiesItem> = capabilities
            .iter()
            .map(manifest_3_2_0::CapabilitiesItem::from)
            .collect();
        Self::from(capabilities)
    }
}

impl TryFrom<&manifest_2_0_0::FlecsAppManifestConffilesItem> for manifest_3_2_0::ConffilesItem {
    type Error = manifest_3_2_0::error::ConversionError;

    fn try_from(
        value: &manifest_2_0_0::FlecsAppManifestConffilesItem,
    ) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl TryFrom<&Vec<manifest_2_0_0::FlecsAppManifestConffilesItem>> for manifest_3_2_0::Conffiles {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(
        conffiles: &Vec<manifest_2_0_0::FlecsAppManifestConffilesItem>,
    ) -> Result<Self, Self::Error> {
        let conffiles: Result<Vec<manifest_3_2_0::ConffilesItem>, _> = conffiles
            .iter()
            .map(manifest_3_2_0::ConffilesItem::try_from)
            .collect();
        Ok(Self::from(conffiles?))
    }
}

impl TryFrom<&manifest_2_0_0::FlecsAppManifestDevicesItem> for manifest_3_2_0::DevicesItem {
    type Error = manifest_3_2_0::error::ConversionError;

    fn try_from(value: &manifest_2_0_0::FlecsAppManifestDevicesItem) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&Vec<manifest_2_0_0::FlecsAppManifestDevicesItem>> for manifest_3_2_0::Devices {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(
        devices: &Vec<manifest_2_0_0::FlecsAppManifestDevicesItem>,
    ) -> Result<Self, Self::Error> {
        let devices: Result<Vec<manifest_3_2_0::DevicesItem>, _> = devices
            .iter()
            .map(manifest_3_2_0::DevicesItem::try_from)
            .collect();
        Ok(Self::from(devices?))
    }
}

impl TryFrom<&manifest_2_0_0::FlecsAppManifestEditor> for manifest_3_2_0::Editors {
    type Error = manifest_3_2_0::error::ConversionError;

    fn try_from(value: &manifest_2_0_0::FlecsAppManifestEditor) -> Result<Self, Self::Error> {
        if value.as_str().is_empty() {
            return Ok(Self::from(Vec::<manifest_3_2_0::EditorsItem>::new()));
        }
        let port: std::num::NonZeroU16 =
            value.as_str()[1..].parse().map_err(|e: ParseIntError| {
                manifest_3_2_0::error::ConversionError::from(e.to_string())
            })?;
        Ok(Self::from(vec![manifest_3_2_0::EditorsItem {
            name: String::new(),
            port,
            supports_reverse_proxy: false,
        }]))
    }
}
impl TryFrom<&manifest_2_0_0::FlecsAppManifestEnvItem> for manifest_3_2_0::EnvItem {
    type Error = manifest_3_2_0::error::ConversionError;

    fn try_from(value: &manifest_2_0_0::FlecsAppManifestEnvItem) -> Result<Self, Self::Error> {
        manifest_3_2_0::EnvItem::from_str(&value.as_str().replacen(':', "=", 1))
    }
}

impl TryFrom<&Vec<manifest_2_0_0::FlecsAppManifestEnvItem>> for manifest_3_2_0::Env {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(env: &Vec<manifest_2_0_0::FlecsAppManifestEnvItem>) -> Result<Self, Self::Error> {
        let env: Result<Vec<manifest_3_2_0::EnvItem>, _> =
            env.iter().map(manifest_3_2_0::EnvItem::try_from).collect();
        Ok(Self::from(env?))
    }
}

impl TryFrom<&manifest_2_0_0::FlecsAppManifestLabelsItem> for manifest_3_2_0::LabelsItem {
    type Error = manifest_3_2_0::error::ConversionError;

    fn try_from(value: &manifest_2_0_0::FlecsAppManifestLabelsItem) -> Result<Self, Self::Error> {
        manifest_3_2_0::LabelsItem::from_str(value.as_str())
    }
}

impl TryFrom<&Vec<manifest_2_0_0::FlecsAppManifestLabelsItem>> for manifest_3_2_0::Labels {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(
        labels: &Vec<manifest_2_0_0::FlecsAppManifestLabelsItem>,
    ) -> Result<Self, Self::Error> {
        let labels: Result<Vec<manifest_3_2_0::LabelsItem>, _> = labels
            .iter()
            .map(manifest_3_2_0::LabelsItem::try_from)
            .collect();
        Ok(Self::from(labels?))
    }
}

impl TryFrom<&manifest_2_0_0::FlecsAppManifestPortsItem> for manifest_3_2_0::PortsItem {
    type Error = manifest_3_2_0::error::ConversionError;

    fn try_from(value: &manifest_2_0_0::FlecsAppManifestPortsItem) -> Result<Self, Self::Error> {
        manifest_3_2_0::PortsItem::from_str(value.as_str())
    }
}

impl TryFrom<&Vec<manifest_2_0_0::FlecsAppManifestPortsItem>> for manifest_3_2_0::Ports {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(
        ports: &Vec<manifest_2_0_0::FlecsAppManifestPortsItem>,
    ) -> Result<Self, Self::Error> {
        let ports: Result<Vec<manifest_3_2_0::PortsItem>, _> = ports
            .iter()
            .map(manifest_3_2_0::PortsItem::try_from)
            .collect();
        Ok(Self::from(ports?))
    }
}

impl TryFrom<&manifest_2_0_0::FlecsAppManifestVolumesItem> for manifest_3_2_0::VolumesItem {
    type Error = manifest_3_2_0::error::ConversionError;

    fn try_from(value: &manifest_2_0_0::FlecsAppManifestVolumesItem) -> Result<Self, Self::Error> {
        manifest_3_2_0::VolumesItem::from_str(value.as_str())
    }
}

impl TryFrom<&Vec<manifest_2_0_0::FlecsAppManifestVolumesItem>> for manifest_3_2_0::Volumes {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(
        labels: &Vec<manifest_2_0_0::FlecsAppManifestVolumesItem>,
    ) -> Result<Self, Self::Error> {
        let labels: Result<Vec<manifest_3_2_0::VolumesItem>, _> = labels
            .iter()
            .map(manifest_3_2_0::VolumesItem::try_from)
            .collect();
        Ok(Self::from(labels?))
    }
}

impl TryFrom<&manifest_3_1_0::FlecsAppManifest> for manifest_3_2_0::FlecsAppManifest {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::FlecsAppManifest) -> Result<Self, Self::Error> {
        Ok(match value {
            manifest_3_1_0::FlecsAppManifest::Single(single) => {
                manifest_3_2_0::FlecsAppManifest::Single(single.try_into()?)
            }
            manifest_3_1_0::FlecsAppManifest::Multi(multi) => {
                manifest_3_2_0::FlecsAppManifest::Multi(multi.try_into()?)
            }
        })
    }
}

impl TryFrom<&manifest_3_1_0::Single> for manifest_3_2_0::Single {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::Single) -> Result<Self, Self::Error> {
        Ok(Self {
            app: (&value.app).try_into()?,
            args: value.args.as_ref().map(manifest_3_2_0::Args::from),
            capabilities: value
                .capabilities
                .as_ref()
                .map(manifest_3_2_0::Capabilities::from),
            conffiles: value
                .conffiles
                .as_ref()
                .map(manifest_3_2_0::Conffiles::try_from)
                .transpose()?,
            depends: None,
            devices: value
                .devices
                .as_ref()
                .map(manifest_3_2_0::Devices::try_from)
                .transpose()?,
            editors: value.editors.as_ref().map(manifest_3_2_0::Editors::from),
            env: value
                .env
                .as_ref()
                .map(manifest_3_2_0::Env::try_from)
                .transpose()?,
            hostname: value.hostname.as_ref().map(manifest_3_2_0::Hostname::from),
            image: (&value.image).try_into()?,
            interactive: value
                .interactive
                .as_ref()
                .map(manifest_3_2_0::Interactive::from),
            labels: value
                .labels
                .as_ref()
                .map(manifest_3_2_0::Labels::try_from)
                .transpose()?,
            minimum_flecs_version: value
                .minimum_flecs_version
                .as_ref()
                .map(manifest_3_2_0::MinimumFlecsVersion::try_from)
                .transpose()?,
            multi_instance: value
                .multi_instance
                .as_ref()
                .map(manifest_3_2_0::MultiInstance::from),
            ports: value
                .ports
                .as_ref()
                .map(manifest_3_2_0::Ports::try_from)
                .transpose()?,
            provides: None,
            recommends: None,
            revision: value.revision.as_ref().map(manifest_3_2_0::Revision::from),
            // TODO: How do we set the correct schema?
            schema: None,
            version: (&value.version).into(),
            volumes: value
                .volumes
                .as_ref()
                .map(manifest_3_2_0::Volumes::try_from)
                .transpose()?,
        })
    }
}

impl TryFrom<&manifest_3_1_0::Multi> for manifest_3_2_0::Multi {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::Multi) -> Result<Self, Self::Error> {
        Ok(Self {
            app: (&value.app).try_into()?,
            depends: None,
            minimum_flecs_version: value
                .minimum_flecs_version
                .as_ref()
                .map(manifest_3_2_0::MinimumFlecsVersion::try_from)
                .transpose()?,
            provides: None,
            recommends: None,
            revision: value.revision.as_ref().map(manifest_3_2_0::Revision::from),
            // TODO: How do we set the correct schema?
            schema: None,
            version: (&value.version).into(),
            deployment: (&value.deployment).into(),
        })
    }
}

impl TryFrom<&manifest_3_1_0::App> for manifest_3_2_0::App {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::App) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl From<&manifest_3_1_0::Args> for manifest_3_2_0::Args {
    fn from(value: &manifest_3_1_0::Args) -> Self {
        Self(value.0.clone())
    }
}

impl From<&manifest_3_1_0::Capabilities> for manifest_3_2_0::Capabilities {
    fn from(value: &manifest_3_1_0::Capabilities) -> Self {
        Self(
            value
                .0
                .iter()
                .map(manifest_3_2_0::CapabilitiesItem::from)
                .collect(),
        )
    }
}

impl From<&manifest_3_1_0::CapabilitiesItem> for manifest_3_2_0::CapabilitiesItem {
    fn from(value: &manifest_3_1_0::CapabilitiesItem) -> Self {
        match value {
            manifest_3_1_0::CapabilitiesItem::Docker => Self::Docker,
            manifest_3_1_0::CapabilitiesItem::NetAdmin => Self::NetAdmin,
            manifest_3_1_0::CapabilitiesItem::SysNice => Self::SysNice,
            manifest_3_1_0::CapabilitiesItem::IpcLock => Self::IpcLock,
            manifest_3_1_0::CapabilitiesItem::NetRaw => Self::NetRaw,
        }
    }
}

impl TryFrom<&manifest_3_1_0::Conffiles> for manifest_3_2_0::Conffiles {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::Conffiles) -> Result<Self, Self::Error> {
        Ok(Self(
            value
                .0
                .iter()
                .map(manifest_3_2_0::ConffilesItem::try_from)
                .collect::<Result<_, _>>()?,
        ))
    }
}

impl TryFrom<&manifest_3_1_0::ConffilesItem> for manifest_3_2_0::ConffilesItem {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::ConffilesItem) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl From<&manifest_3_1_0::Deployment> for manifest_3_2_0::Deployment {
    fn from(value: &manifest_3_1_0::Deployment) -> Self {
        Self {
            compose: (&value.compose).into(),
        }
    }
}

impl From<&manifest_3_1_0::DeploymentCompose> for manifest_3_2_0::DeploymentCompose {
    fn from(value: &manifest_3_1_0::DeploymentCompose) -> Self {
        Self {
            yaml: value.yaml.clone(),
        }
    }
}

impl TryFrom<&manifest_3_1_0::Devices> for manifest_3_2_0::Devices {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::Devices) -> Result<Self, Self::Error> {
        Ok(Self(
            value
                .0
                .iter()
                .map(manifest_3_2_0::DevicesItem::try_from)
                .collect::<Result<_, _>>()?,
        ))
    }
}

impl TryFrom<&manifest_3_1_0::DevicesItem> for manifest_3_2_0::DevicesItem {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::DevicesItem) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl From<&manifest_3_1_0::Editors> for manifest_3_2_0::Editors {
    fn from(value: &manifest_3_1_0::Editors) -> Self {
        Self(
            value
                .0
                .iter()
                .map(manifest_3_2_0::EditorsItem::from)
                .collect(),
        )
    }
}

impl From<&manifest_3_1_0::EditorsItem> for manifest_3_2_0::EditorsItem {
    fn from(value: &manifest_3_1_0::EditorsItem) -> Self {
        Self {
            name: value.name.clone(),
            port: value.port,
            supports_reverse_proxy: value.supports_reverse_proxy,
        }
    }
}

impl TryFrom<&manifest_3_1_0::Env> for manifest_3_2_0::Env {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::Env) -> Result<Self, Self::Error> {
        Ok(Self(
            value
                .0
                .iter()
                .map(manifest_3_2_0::EnvItem::try_from)
                .collect::<Result<_, _>>()?,
        ))
    }
}

impl TryFrom<&manifest_3_1_0::EnvItem> for manifest_3_2_0::EnvItem {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::EnvItem) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl From<&manifest_3_1_0::Hostname> for manifest_3_2_0::Hostname {
    fn from(value: &manifest_3_1_0::Hostname) -> Self {
        Self(value.0.clone())
    }
}

impl TryFrom<&manifest_3_1_0::Image> for manifest_3_2_0::Image {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::Image) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl From<&manifest_3_1_0::Interactive> for manifest_3_2_0::Interactive {
    fn from(value: &manifest_3_1_0::Interactive) -> Self {
        Self(value.0)
    }
}

impl TryFrom<&manifest_3_1_0::Labels> for manifest_3_2_0::Labels {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::Labels) -> Result<Self, Self::Error> {
        Ok(Self(
            value
                .0
                .iter()
                .map(manifest_3_2_0::LabelsItem::try_from)
                .collect::<Result<_, _>>()?,
        ))
    }
}

impl TryFrom<&manifest_3_1_0::LabelsItem> for manifest_3_2_0::LabelsItem {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::LabelsItem) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl TryFrom<&manifest_3_1_0::MinimumFlecsVersion> for manifest_3_2_0::MinimumFlecsVersion {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::MinimumFlecsVersion) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl From<&manifest_3_1_0::MultiInstance> for manifest_3_2_0::MultiInstance {
    fn from(value: &manifest_3_1_0::MultiInstance) -> Self {
        Self(value.0)
    }
}

impl TryFrom<&manifest_3_1_0::Ports> for manifest_3_2_0::Ports {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::Ports) -> Result<Self, Self::Error> {
        Ok(Self(
            value
                .0
                .iter()
                .map(manifest_3_2_0::PortsItem::try_from)
                .collect::<Result<_, _>>()?,
        ))
    }
}

impl TryFrom<&manifest_3_1_0::PortsItem> for manifest_3_2_0::PortsItem {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::PortsItem) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl From<&manifest_3_1_0::Revision> for manifest_3_2_0::Revision {
    fn from(value: &manifest_3_1_0::Revision) -> Self {
        Self(value.0.clone())
    }
}

impl From<&manifest_3_1_0::Schema> for manifest_3_2_0::Schema {
    fn from(value: &manifest_3_1_0::Schema) -> Self {
        Self(value.0.clone())
    }
}

impl From<&manifest_3_1_0::Version> for manifest_3_2_0::Version {
    fn from(value: &manifest_3_1_0::Version) -> Self {
        Self(value.0.clone())
    }
}

impl TryFrom<&manifest_3_1_0::Volumes> for manifest_3_2_0::Volumes {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::Volumes) -> Result<Self, Self::Error> {
        Ok(Self(
            value
                .0
                .iter()
                .map(manifest_3_2_0::VolumesItem::try_from)
                .collect::<Result<_, _>>()?,
        ))
    }
}

impl TryFrom<&manifest_3_1_0::VolumesItem> for manifest_3_2_0::VolumesItem {
    type Error = manifest_3_2_0::error::ConversionError;
    fn try_from(value: &manifest_3_1_0::VolumesItem) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capability() {
        assert_eq!(
            manifest_3_2_0::CapabilitiesItem::Docker,
            manifest_3_2_0::CapabilitiesItem::from(
                &manifest_2_0_0::FlecsAppManifestCapabilitiesItem::Docker
            )
        );
        assert_eq!(
            manifest_3_2_0::CapabilitiesItem::NetAdmin,
            manifest_3_2_0::CapabilitiesItem::from(
                &manifest_2_0_0::FlecsAppManifestCapabilitiesItem::NetAdmin
            )
        );
        assert_eq!(
            manifest_3_2_0::CapabilitiesItem::IpcLock,
            manifest_3_2_0::CapabilitiesItem::from(
                &manifest_2_0_0::FlecsAppManifestCapabilitiesItem::IpcLock
            )
        );
        assert_eq!(
            manifest_3_2_0::CapabilitiesItem::NetRaw,
            manifest_3_2_0::CapabilitiesItem::from(
                &manifest_2_0_0::FlecsAppManifestCapabilitiesItem::NetRaw
            )
        );
        assert_eq!(
            manifest_3_2_0::CapabilitiesItem::SysNice,
            manifest_3_2_0::CapabilitiesItem::from(
                &manifest_2_0_0::FlecsAppManifestCapabilitiesItem::SysNice
            )
        );
        assert_eq!(
            manifest_3_2_0::CapabilitiesItem::Docker,
            manifest_3_2_0::CapabilitiesItem::from(
                &manifest_3_0_0::FlecsAppManifestCapabilitiesItem::Docker
            )
        );
        assert_eq!(
            manifest_3_2_0::CapabilitiesItem::NetAdmin,
            manifest_3_2_0::CapabilitiesItem::from(
                &manifest_3_0_0::FlecsAppManifestCapabilitiesItem::NetAdmin
            )
        );
        assert_eq!(
            manifest_3_2_0::CapabilitiesItem::IpcLock,
            manifest_3_2_0::CapabilitiesItem::from(
                &manifest_3_0_0::FlecsAppManifestCapabilitiesItem::IpcLock
            )
        );
        assert_eq!(
            manifest_3_2_0::CapabilitiesItem::NetRaw,
            manifest_3_2_0::CapabilitiesItem::from(
                &manifest_3_0_0::FlecsAppManifestCapabilitiesItem::NetRaw
            )
        );
        assert_eq!(
            manifest_3_2_0::CapabilitiesItem::SysNice,
            manifest_3_2_0::CapabilitiesItem::from(
                &manifest_3_0_0::FlecsAppManifestCapabilitiesItem::SysNice
            )
        );
    }

    #[test]
    fn conffile() {
        assert_eq!(
            manifest_3_2_0::ConffilesItem::from_str(
                "default.conf:/etc/my-app/default.conf:rw,no_init"
            )
            .unwrap(),
            manifest_3_2_0::ConffilesItem::try_from(
                &manifest_2_0_0::FlecsAppManifestConffilesItem::from_str(
                    "default.conf:/etc/my-app/default.conf:rw,no_init"
                )
                .unwrap()
            )
            .unwrap()
        );
        assert_eq!(
            manifest_3_2_0::ConffilesItem::from_str(
                "default.conf:/etc/my-app/default.conf:rw,no_init"
            )
            .unwrap(),
            manifest_3_2_0::ConffilesItem::try_from(
                &manifest_3_0_0::FlecsAppManifestConffilesItem::from_str(
                    "default.conf:/etc/my-app/default.conf:rw,no_init"
                )
                .unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn device() {
        assert_eq!(
            manifest_3_2_0::DevicesItem::from_str("/dev/net/tun").unwrap(),
            manifest_3_2_0::DevicesItem::try_from(
                &manifest_2_0_0::FlecsAppManifestDevicesItem::from_str("/dev/net/tun").unwrap()
            )
            .unwrap()
        );
        assert_eq!(
            manifest_3_2_0::DevicesItem::from_str("/dev/net/tun").unwrap(),
            manifest_3_2_0::DevicesItem::try_from(
                &manifest_3_0_0::FlecsAppManifestDevicesItem::from_str("/dev/net/tun").unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn editors() {
        assert_eq!(
            manifest_3_2_0::Editors::from(vec![manifest_3_2_0::EditorsItem {
                name: "".to_string(),
                port: std::num::NonZeroU16::try_from(1234).unwrap(),
                supports_reverse_proxy: false,
            }]),
            TryInto::<manifest_3_2_0::Editors>::try_into(
                &manifest_2_0_0::FlecsAppManifestEditor::from_str(":1234").unwrap()
            )
            .unwrap()
        );
        assert_eq!(
            manifest_3_2_0::Editors::from(vec![
                manifest_3_2_0::EditorsItem {
                    name: "TestEditor#1".to_string(),
                    port: std::num::NonZeroU16::try_from(1234).unwrap(),
                    supports_reverse_proxy: false,
                },
                manifest_3_2_0::EditorsItem {
                    name: "TestEditor#2".to_string(),
                    port: std::num::NonZeroU16::try_from(5678).unwrap(),
                    supports_reverse_proxy: true,
                }
            ]),
            manifest_3_2_0::Editors::from(vec![
                manifest_3_0_0::FlecsAppManifestEditorsItem {
                    name: "TestEditor#1".to_string(),
                    port: std::num::NonZeroU16::try_from(1234).unwrap(),
                    supports_reverse_proxy: false,
                },
                manifest_3_0_0::FlecsAppManifestEditorsItem {
                    name: "TestEditor#2".to_string(),
                    port: std::num::NonZeroU16::try_from(5678).unwrap(),
                    supports_reverse_proxy: true,
                }
            ])
        );
    }

    #[test]
    fn env() {
        assert_eq!(
            manifest_3_2_0::EnvItem::from_str("tech.flecs.some-app_value=any").unwrap(),
            manifest_3_2_0::EnvItem::try_from(
                &manifest_2_0_0::FlecsAppManifestEnvItem::from_str("tech.flecs.some-app_value=any")
                    .unwrap()
            )
            .unwrap()
        );
        assert_eq!(
            manifest_3_2_0::EnvItem::from_str("tech.flecs.some-app_value=any").unwrap(),
            manifest_3_2_0::EnvItem::try_from(
                &manifest_2_0_0::FlecsAppManifestEnvItem::from_str("tech.flecs.some-app_value:any")
                    .unwrap()
            )
            .unwrap()
        );
        assert_eq!(
            manifest_3_2_0::EnvItem::from_str("tech.flecs.some-app_value=a:n:y").unwrap(),
            manifest_3_2_0::EnvItem::try_from(
                &manifest_2_0_0::FlecsAppManifestEnvItem::from_str(
                    "tech.flecs.some-app_value:a:n:y"
                )
                .unwrap()
            )
            .unwrap()
        );
        assert_eq!(
            manifest_3_2_0::EnvItem::from_str("tech.flecs.some-app_value=any").unwrap(),
            manifest_3_2_0::EnvItem::try_from(
                &manifest_3_0_0::FlecsAppManifestEnvItem::from_str("tech.flecs.some-app_value=any")
                    .unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn label() {
        assert_eq!(
            manifest_3_2_0::LabelsItem::from_str("tech.flecs.some-label=Some custom label value")
                .unwrap(),
            manifest_3_2_0::LabelsItem::try_from(
                &manifest_2_0_0::FlecsAppManifestLabelsItem::from_str(
                    "tech.flecs.some-label=Some custom label value"
                )
                .unwrap()
            )
            .unwrap()
        );
        assert_eq!(
            manifest_3_2_0::LabelsItem::from_str("tech.flecs.some-label=Some custom label value")
                .unwrap(),
            manifest_3_2_0::LabelsItem::try_from(
                &manifest_3_0_0::FlecsAppManifestLabelsItem::from_str(
                    "tech.flecs.some-label=Some custom label value"
                )
                .unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn port() {
        assert_eq!(
            manifest_3_2_0::PortsItem::from_str("5001-5008:6001-6008").unwrap(),
            manifest_3_2_0::PortsItem::try_from(
                &manifest_2_0_0::FlecsAppManifestPortsItem::from_str("5001-5008:6001-6008")
                    .unwrap()
            )
            .unwrap()
        );
        assert_eq!(
            manifest_3_2_0::PortsItem::from_str("5001-5008:6001-6008").unwrap(),
            manifest_3_2_0::PortsItem::try_from(
                &manifest_3_0_0::FlecsAppManifestPortsItem::from_str("5001-5008:6001-6008")
                    .unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn volume() {
        assert_eq!(
            manifest_3_2_0::VolumesItem::from_str("/etc/my-app:/etc/my-app").unwrap(),
            manifest_3_2_0::VolumesItem::try_from(
                &manifest_2_0_0::FlecsAppManifestVolumesItem::from_str("/etc/my-app:/etc/my-app")
                    .unwrap()
            )
            .unwrap()
        );
        assert_eq!(
            manifest_3_2_0::VolumesItem::from_str("/etc/my-app:/etc/my-app").unwrap(),
            manifest_3_2_0::VolumesItem::try_from(
                &manifest_3_0_0::FlecsAppManifestVolumesItem::from_str("/etc/my-app:/etc/my-app")
                    .unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn complete_conversion_2_to_3_2() {
        let manifest_v2 = manifest_2_0_0::FlecsAppManifest {
            app: manifest_2_0_0::FlecsAppManifestApp::from_str("io.some.app").unwrap(),
            args: vec![],
            capabilities: Some(vec![
                manifest_2_0_0::FlecsAppManifestCapabilitiesItem::IpcLock,
                manifest_2_0_0::FlecsAppManifestCapabilitiesItem::Docker,
            ]),
            conffiles: vec![
                manifest_2_0_0::FlecsAppManifestConffilesItem::from_str(
                    "some.conf:/etc/my-app/new.conf:rw,no_init",
                )
                .unwrap(),
                manifest_2_0_0::FlecsAppManifestConffilesItem::from_str(
                    "default.conf:/etc/my-app/default.conf:rw,no_init",
                )
                .unwrap(),
            ],
            devices: vec![
                manifest_2_0_0::FlecsAppManifestDevicesItem::from_str("/dev/net/tun").unwrap(),
                manifest_2_0_0::FlecsAppManifestDevicesItem::from_str("/dev/bus/usb").unwrap(),
            ],
            editor: Some(manifest_2_0_0::FlecsAppManifestEditor::from_str(":15945").unwrap()),
            env: vec![
                manifest_2_0_0::FlecsAppManifestEnvItem::from_str("MY_ENV=value").unwrap(),
                manifest_2_0_0::FlecsAppManifestEnvItem::from_str("tech.flecs.some-app_value=any")
                    .unwrap(),
            ],
            hostname: Some("TestHostname".to_string()),
            image: manifest_2_0_0::FlecsAppManifestImage::from_str(
                "flecs.azurecr.io/tech.flecs.plunder",
            )
            .unwrap(),
            interactive: Some(false),
            labels: vec![
                manifest_2_0_0::FlecsAppManifestLabelsItem::from_str("tech.flecs").unwrap(),
                manifest_2_0_0::FlecsAppManifestLabelsItem::from_str(
                    "tech.flecs.some-label=Some custom label value",
                )
                .unwrap(),
            ],
            multi_instance: Some(true),
            ports: vec![
                manifest_2_0_0::FlecsAppManifestPortsItem::from_str("8001:8001").unwrap(),
                manifest_2_0_0::FlecsAppManifestPortsItem::from_str("5000").unwrap(),
                manifest_2_0_0::FlecsAppManifestPortsItem::from_str("5001-5008:6001-6008").unwrap(),
                manifest_2_0_0::FlecsAppManifestPortsItem::from_str("6001-6008").unwrap(),
            ],
            revision: Some("34".to_string()),
            version: "6.8.8-bunny".to_string(),
            volumes: vec![
                manifest_2_0_0::FlecsAppManifestVolumesItem::from_str("my-app-etc:/etc/my-app")
                    .unwrap(),
                manifest_2_0_0::FlecsAppManifestVolumesItem::from_str("/etc/my-app:/etc/my-app")
                    .unwrap(),
            ],
        };

        let expected_manifest_v3_2 = manifest_3_2_0::Single {
            app: manifest_3_2_0::App::from_str("io.some.app").unwrap(),
            args: None,
            capabilities: Some(
                vec![
                    manifest_3_2_0::CapabilitiesItem::IpcLock,
                    manifest_3_2_0::CapabilitiesItem::Docker,
                ]
                .into(),
            ),
            conffiles: Some(
                vec![
                    manifest_3_2_0::ConffilesItem::from_str(
                        "some.conf:/etc/my-app/new.conf:rw,no_init",
                    )
                    .unwrap(),
                    manifest_3_2_0::ConffilesItem::from_str(
                        "default.conf:/etc/my-app/default.conf:rw,no_init",
                    )
                    .unwrap(),
                ]
                .into(),
            ),
            depends: None,
            devices: Some(
                vec![
                    manifest_3_2_0::DevicesItem::from_str("/dev/net/tun").unwrap(),
                    manifest_3_2_0::DevicesItem::from_str("/dev/bus/usb").unwrap(),
                ]
                .into(),
            ),
            editors: Some(
                vec![manifest_3_2_0::EditorsItem {
                    name: "".to_string(),
                    port: std::num::NonZeroU16::try_from(15945).unwrap(),
                    supports_reverse_proxy: false,
                }]
                .into(),
            ),
            env: Some(
                vec![
                    manifest_3_2_0::EnvItem::from_str("MY_ENV=value").unwrap(),
                    manifest_3_2_0::EnvItem::from_str("tech.flecs.some-app_value=any").unwrap(),
                ]
                .into(),
            ),
            hostname: Some("TestHostname".to_string().into()),
            image: manifest_3_2_0::Image::from_str("flecs.azurecr.io/tech.flecs.plunder").unwrap(),
            interactive: Some(false.into()),
            labels: Some(
                vec![
                    manifest_3_2_0::LabelsItem::from_str("tech.flecs").unwrap(),
                    manifest_3_2_0::LabelsItem::from_str(
                        "tech.flecs.some-label=Some custom label value",
                    )
                    .unwrap(),
                ]
                .into(),
            ),
            minimum_flecs_version: None,
            multi_instance: Some(true.into()),
            ports: Some(
                vec![
                    manifest_3_2_0::PortsItem::from_str("8001:8001").unwrap(),
                    manifest_3_2_0::PortsItem::from_str("5000").unwrap(),
                    manifest_3_2_0::PortsItem::from_str("5001-5008:6001-6008").unwrap(),
                    manifest_3_2_0::PortsItem::from_str("6001-6008").unwrap(),
                ]
                .into(),
            ),
            provides: None,
            recommends: None,
            revision: Some("34".to_string().into()),
            schema: None,
            version: "6.8.8-bunny".to_string().into(),
            volumes: Some(
                vec![
                    manifest_3_2_0::VolumesItem::from_str("my-app-etc:/etc/my-app").unwrap(),
                    manifest_3_2_0::VolumesItem::from_str("/etc/my-app:/etc/my-app").unwrap(),
                ]
                .into(),
            ),
        };

        assert_eq!(
            expected_manifest_v3_2,
            manifest_3_2_0::Single::try_from(&manifest_v2).unwrap()
        )
    }

    #[test]
    fn complete_conversion_3_to_3_2() {
        let manifest_v3 = manifest_3_0_0::FlecsAppManifest {
            app: manifest_3_0_0::FlecsAppManifestApp::from_str("io.some.app").unwrap(),
            args: vec![],
            capabilities: Some(vec![
                manifest_3_0_0::FlecsAppManifestCapabilitiesItem::IpcLock,
                manifest_3_0_0::FlecsAppManifestCapabilitiesItem::Docker,
            ]),
            conffiles: vec![
                manifest_3_0_0::FlecsAppManifestConffilesItem::from_str(
                    "some.conf:/etc/my-app/new.conf:rw,no_init",
                )
                .unwrap(),
                manifest_3_0_0::FlecsAppManifestConffilesItem::from_str(
                    "default.conf:/etc/my-app/default.conf:rw,no_init",
                )
                .unwrap(),
            ],
            devices: vec![
                manifest_3_0_0::FlecsAppManifestDevicesItem::from_str("/dev/net/tun").unwrap(),
                manifest_3_0_0::FlecsAppManifestDevicesItem::from_str("/dev/bus/usb").unwrap(),
            ],
            editors: vec![manifest_3_0_0::FlecsAppManifestEditorsItem {
                name: "".to_string(),
                port: std::num::NonZeroU16::try_from(15945).unwrap(),
                supports_reverse_proxy: false,
            }],
            env: vec![
                manifest_3_0_0::FlecsAppManifestEnvItem::from_str("MY_ENV=value").unwrap(),
                manifest_3_0_0::FlecsAppManifestEnvItem::from_str("tech.flecs.some-app_value=any")
                    .unwrap(),
            ],
            image: manifest_3_0_0::FlecsAppManifestImage::from_str(
                "flecs.azurecr.io/tech.flecs.plunder",
            )
            .unwrap(),
            interactive: Some(false),
            labels: vec![
                manifest_3_0_0::FlecsAppManifestLabelsItem::from_str("tech.flecs").unwrap(),
                manifest_3_0_0::FlecsAppManifestLabelsItem::from_str(
                    "tech.flecs.some-label=Some custom label value",
                )
                .unwrap(),
            ],
            minimum_flecs_version: None,
            multi_instance: Some(true),
            ports: vec![
                manifest_3_0_0::FlecsAppManifestPortsItem::from_str("8001:8001").unwrap(),
                manifest_3_0_0::FlecsAppManifestPortsItem::from_str("5000").unwrap(),
                manifest_3_0_0::FlecsAppManifestPortsItem::from_str("5001-5008:6001-6008").unwrap(),
                manifest_3_0_0::FlecsAppManifestPortsItem::from_str("6001-6008").unwrap(),
            ],
            revision: Some("34".to_string()),
            version: "6.8.8-bunny".to_string(),
            volumes: vec![
                manifest_3_0_0::FlecsAppManifestVolumesItem::from_str("my-app-etc:/etc/my-app")
                    .unwrap(),
                manifest_3_0_0::FlecsAppManifestVolumesItem::from_str("/etc/my-app:/etc/my-app")
                    .unwrap(),
            ],
        };

        let expected_manifest_v3_2 = manifest_3_2_0::Single {
            app: manifest_3_2_0::App::from_str("io.some.app").unwrap(),
            args: None,
            capabilities: Some(
                vec![
                    manifest_3_2_0::CapabilitiesItem::IpcLock,
                    manifest_3_2_0::CapabilitiesItem::Docker,
                ]
                .into(),
            ),
            conffiles: Some(
                vec![
                    manifest_3_2_0::ConffilesItem::from_str(
                        "some.conf:/etc/my-app/new.conf:rw,no_init",
                    )
                    .unwrap(),
                    manifest_3_2_0::ConffilesItem::from_str(
                        "default.conf:/etc/my-app/default.conf:rw,no_init",
                    )
                    .unwrap(),
                ]
                .into(),
            ),
            depends: None,
            devices: Some(
                vec![
                    manifest_3_2_0::DevicesItem::from_str("/dev/net/tun").unwrap(),
                    manifest_3_2_0::DevicesItem::from_str("/dev/bus/usb").unwrap(),
                ]
                .into(),
            ),
            editors: Some(
                vec![manifest_3_2_0::EditorsItem {
                    name: "".to_string(),
                    port: std::num::NonZeroU16::try_from(15945).unwrap(),
                    supports_reverse_proxy: false,
                }]
                .into(),
            ),
            env: Some(
                vec![
                    manifest_3_2_0::EnvItem::from_str("MY_ENV=value").unwrap(),
                    manifest_3_2_0::EnvItem::from_str("tech.flecs.some-app_value=any").unwrap(),
                ]
                .into(),
            ),
            hostname: None,
            image: manifest_3_2_0::Image::from_str("flecs.azurecr.io/tech.flecs.plunder").unwrap(),
            interactive: Some(false.into()),
            labels: Some(
                vec![
                    manifest_3_2_0::LabelsItem::from_str("tech.flecs").unwrap(),
                    manifest_3_2_0::LabelsItem::from_str(
                        "tech.flecs.some-label=Some custom label value",
                    )
                    .unwrap(),
                ]
                .into(),
            ),
            minimum_flecs_version: None,
            multi_instance: Some(true.into()),
            ports: Some(
                vec![
                    manifest_3_2_0::PortsItem::from_str("8001:8001").unwrap(),
                    manifest_3_2_0::PortsItem::from_str("5000").unwrap(),
                    manifest_3_2_0::PortsItem::from_str("5001-5008:6001-6008").unwrap(),
                    manifest_3_2_0::PortsItem::from_str("6001-6008").unwrap(),
                ]
                .into(),
            ),
            provides: None,
            recommends: None,
            revision: Some("34".to_string().into()),
            schema: None,
            version: "6.8.8-bunny".to_string().into(),
            volumes: Some(
                vec![
                    manifest_3_2_0::VolumesItem::from_str("my-app-etc:/etc/my-app").unwrap(),
                    manifest_3_2_0::VolumesItem::from_str("/etc/my-app:/etc/my-app").unwrap(),
                ]
                .into(),
            ),
        };

        assert_eq!(
            expected_manifest_v3_2,
            manifest_3_2_0::Single::try_from(&manifest_v3).unwrap()
        )
    }
}
