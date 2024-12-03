pub use super::{Error, Result};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Device {
    pub path: PathBuf,
}

impl FromStr for Device {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        anyhow::ensure!(
            s.starts_with("/dev/"),
            "Device path has to start with /dev/"
        );
        Ok(Self {
            path: FromStr::from_str(s)?,
        })
    }
}

impl TryFrom<&flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestDevicesItem>
    for Device
{
    type Error = Error;

    fn try_from(
        value: &flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestDevicesItem,
    ) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_from_str_ok() {
        assert_eq!(
            Device::from_str("/dev/test.device").unwrap(),
            Device {
                path: PathBuf::from("/dev/test.device")
            }
        );
    }

    #[test]
    fn device_from_str_wrong_path() {
        assert!(Device::from_str("/not_dev_dir/test.device").is_err());
    }

    #[test]
    fn device_from_device_item() {
        let item =
            flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifestDevicesItem::from_str(
                "/dev/test.device",
            )
            .unwrap();
        assert_eq!(
            Device::try_from(&item).unwrap(),
            Device {
                path: PathBuf::from("/dev/test.device")
            }
        );
    }
}
