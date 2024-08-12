use crate::generated::manifest_2_0_0;
use crate::generated::manifest_3_0_0;
use crate::generated::manifest_3_0_0::error::ConversionError;
use std::num::ParseIntError;
use std::str::FromStr;
use std::string::ToString;

impl TryFrom<&manifest_2_0_0::FlecsAppManifest> for manifest_3_0_0::FlecsAppManifest {
    type Error = ConversionError;

    fn try_from(value: &manifest_2_0_0::FlecsAppManifest) -> Result<Self, Self::Error> {
        let capabilities = value.capabilities.clone().map(|c| {
            c.iter()
                .map(manifest_3_0_0::FlecsAppManifestCapabilitiesItem::from)
                .collect()
        });
        let conffiles: Result<Vec<_>, _> = value
            .conffiles
            .iter()
            .map(manifest_3_0_0::FlecsAppManifestConffilesItem::try_from)
            .collect();
        let conffiles = conffiles?;
        let devices: Result<Vec<_>, _> = value
            .devices
            .iter()
            .map(manifest_3_0_0::FlecsAppManifestDevicesItem::try_from)
            .collect();
        let devices = devices?;
        let editors = match &value.editor {
            None => Vec::new(),
            Some(e) => TryInto::<Vec<manifest_3_0_0::FlecsAppManifestEditorsItem>>::try_into(e)?,
        };
        let env: Result<Vec<_>, _> = value
            .env
            .iter()
            .map(manifest_3_0_0::FlecsAppManifestEnvItem::try_from)
            .collect();
        let env = env?;
        let labels: Result<Vec<_>, _> = value
            .labels
            .iter()
            .map(manifest_3_0_0::FlecsAppManifestLabelsItem::try_from)
            .collect();
        let labels = labels?;
        let ports: Result<Vec<_>, _> = value
            .ports
            .iter()
            .map(manifest_3_0_0::FlecsAppManifestPortsItem::try_from)
            .collect();
        let ports = ports?;
        let volumes: Result<Vec<_>, _> = value
            .volumes
            .iter()
            .map(manifest_3_0_0::FlecsAppManifestVolumesItem::try_from)
            .collect();
        let volumes = volumes?;
        Self::builder()
            .app(value.app.as_str())
            .args(value.args.clone())
            .capabilities(capabilities)
            .conffiles(conffiles)
            .devices(devices)
            .editors(editors)
            .env(env)
            .image(value.image.as_str())
            .interactive(value.interactive)
            .labels(labels)
            .multi_instance(value.multi_instance)
            .minimum_flecs_version(None)
            .ports(ports)
            .revision(value.revision.clone())
            .volumes(volumes)
            .version(value.version.clone())
            .try_into()
    }
}

impl From<&manifest_2_0_0::FlecsAppManifestCapabilitiesItem>
    for manifest_3_0_0::FlecsAppManifestCapabilitiesItem
{
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

impl TryFrom<&manifest_2_0_0::FlecsAppManifestConffilesItem>
    for manifest_3_0_0::FlecsAppManifestConffilesItem
{
    type Error = ConversionError;

    fn try_from(
        value: &manifest_2_0_0::FlecsAppManifestConffilesItem,
    ) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}
impl TryFrom<&manifest_2_0_0::FlecsAppManifestDevicesItem>
    for manifest_3_0_0::FlecsAppManifestDevicesItem
{
    type Error = ConversionError;

    fn try_from(value: &manifest_2_0_0::FlecsAppManifestDevicesItem) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&manifest_2_0_0::FlecsAppManifestEditor>
    for Vec<manifest_3_0_0::FlecsAppManifestEditorsItem>
{
    type Error = ConversionError;

    fn try_from(value: &manifest_2_0_0::FlecsAppManifestEditor) -> Result<Self, Self::Error> {
        if value.as_str().is_empty() {
            return Ok(Vec::new());
        }
        let port: std::num::NonZeroU16 = value.as_str()[1..]
            .parse()
            .map_err(|e: ParseIntError| ConversionError::from(e.to_string()))?;
        Ok(vec![manifest_3_0_0::FlecsAppManifestEditorsItem {
            name: String::new(),
            port,
            supports_reverse_proxy: false,
        }])
    }
}
impl TryFrom<&manifest_2_0_0::FlecsAppManifestEnvItem> for manifest_3_0_0::FlecsAppManifestEnvItem {
    type Error = ConversionError;

    fn try_from(value: &manifest_2_0_0::FlecsAppManifestEnvItem) -> Result<Self, Self::Error> {
        manifest_3_0_0::FlecsAppManifestEnvItem::from_str(value.as_str())
    }
}

impl TryFrom<&manifest_2_0_0::FlecsAppManifestLabelsItem>
    for manifest_3_0_0::FlecsAppManifestLabelsItem
{
    type Error = ConversionError;

    fn try_from(value: &manifest_2_0_0::FlecsAppManifestLabelsItem) -> Result<Self, Self::Error> {
        manifest_3_0_0::FlecsAppManifestLabelsItem::from_str(value.as_str())
    }
}

impl TryFrom<&manifest_2_0_0::FlecsAppManifestPortsItem>
    for manifest_3_0_0::FlecsAppManifestPortsItem
{
    type Error = ConversionError;

    fn try_from(value: &manifest_2_0_0::FlecsAppManifestPortsItem) -> Result<Self, Self::Error> {
        manifest_3_0_0::FlecsAppManifestPortsItem::from_str(value.as_str())
    }
}

impl TryFrom<&manifest_2_0_0::FlecsAppManifestVolumesItem>
    for manifest_3_0_0::FlecsAppManifestVolumesItem
{
    type Error = ConversionError;

    fn try_from(value: &manifest_2_0_0::FlecsAppManifestVolumesItem) -> Result<Self, Self::Error> {
        manifest_3_0_0::FlecsAppManifestVolumesItem::from_str(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capability() {
        assert_eq!(
            manifest_3_0_0::FlecsAppManifestCapabilitiesItem::Docker,
            manifest_3_0_0::FlecsAppManifestCapabilitiesItem::from(
                &manifest_2_0_0::FlecsAppManifestCapabilitiesItem::Docker
            )
        );
        assert_eq!(
            manifest_3_0_0::FlecsAppManifestCapabilitiesItem::NetAdmin,
            manifest_3_0_0::FlecsAppManifestCapabilitiesItem::from(
                &manifest_2_0_0::FlecsAppManifestCapabilitiesItem::NetAdmin
            )
        );
        assert_eq!(
            manifest_3_0_0::FlecsAppManifestCapabilitiesItem::IpcLock,
            manifest_3_0_0::FlecsAppManifestCapabilitiesItem::from(
                &manifest_2_0_0::FlecsAppManifestCapabilitiesItem::IpcLock
            )
        );
        assert_eq!(
            manifest_3_0_0::FlecsAppManifestCapabilitiesItem::NetRaw,
            manifest_3_0_0::FlecsAppManifestCapabilitiesItem::from(
                &manifest_2_0_0::FlecsAppManifestCapabilitiesItem::NetRaw
            )
        );
        assert_eq!(
            manifest_3_0_0::FlecsAppManifestCapabilitiesItem::SysNice,
            manifest_3_0_0::FlecsAppManifestCapabilitiesItem::from(
                &manifest_2_0_0::FlecsAppManifestCapabilitiesItem::SysNice
            )
        );
    }

    #[test]
    fn conffile() {
        assert_eq!(
            manifest_3_0_0::FlecsAppManifestConffilesItem::from_str(
                "default.conf:/etc/my-app/default.conf:rw,no_init"
            )
            .unwrap(),
            manifest_3_0_0::FlecsAppManifestConffilesItem::try_from(
                &manifest_2_0_0::FlecsAppManifestConffilesItem::from_str(
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
            manifest_3_0_0::FlecsAppManifestDevicesItem::from_str("/dev/net/tun").unwrap(),
            manifest_3_0_0::FlecsAppManifestDevicesItem::try_from(
                &manifest_2_0_0::FlecsAppManifestDevicesItem::from_str("/dev/net/tun").unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn editors() {
        assert_eq!(
            vec![manifest_3_0_0::FlecsAppManifestEditorsItem {
                name: "".to_string(),
                port: std::num::NonZeroU16::try_from(1234).unwrap(),
                supports_reverse_proxy: false,
            }],
            TryInto::<Vec<manifest_3_0_0::FlecsAppManifestEditorsItem>>::try_into(
                &manifest_2_0_0::FlecsAppManifestEditor::from_str(":1234").unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn env() {
        assert_eq!(
            manifest_3_0_0::FlecsAppManifestEnvItem::from_str("tech.flecs.some-app_value=any")
                .unwrap(),
            manifest_3_0_0::FlecsAppManifestEnvItem::try_from(
                &manifest_2_0_0::FlecsAppManifestEnvItem::from_str("tech.flecs.some-app_value=any")
                    .unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn label() {
        assert_eq!(
            manifest_3_0_0::FlecsAppManifestLabelsItem::from_str(
                "tech.flecs.some-label=Some custom label value"
            )
            .unwrap(),
            manifest_3_0_0::FlecsAppManifestLabelsItem::try_from(
                &manifest_2_0_0::FlecsAppManifestLabelsItem::from_str(
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
            manifest_3_0_0::FlecsAppManifestPortsItem::from_str("5001-5008:6001-6008").unwrap(),
            manifest_3_0_0::FlecsAppManifestPortsItem::try_from(
                &manifest_2_0_0::FlecsAppManifestPortsItem::from_str("5001-5008:6001-6008")
                    .unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn volume() {
        assert_eq!(
            manifest_3_0_0::FlecsAppManifestVolumesItem::from_str("/etc/my-app:/etc/my-app")
                .unwrap(),
            manifest_3_0_0::FlecsAppManifestVolumesItem::try_from(
                &manifest_2_0_0::FlecsAppManifestVolumesItem::from_str("/etc/my-app:/etc/my-app")
                    .unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn complete_conversion_2_3() {
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

        let expected_manifest_v3 = manifest_3_0_0::FlecsAppManifest {
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

        assert_eq!(
            expected_manifest_v3,
            manifest_3_0_0::FlecsAppManifest::try_from(&manifest_v2).unwrap()
        )
    }
}
