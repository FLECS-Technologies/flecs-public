pub use super::{Error, Result};
use anyhow::anyhow;
use flecsd_axum_server::models::{SystemDistro, SystemInfo, SystemKernel};
use platform_info::{PlatformInfo, PlatformInfoAPI, UNameAPI};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use tracing::error;

pub fn try_create_system_info() -> Result<SystemInfo> {
    let info = PlatformInfo::new().map_err(|e| anyhow!(e))?;
    Ok(SystemInfo {
        arch: machine_to_arch(info.machine().to_str().unwrap_or_default()).to_string(),
        distro: read_distro(),
        kernel: read_kernel(&info),
        platform: platform_from_version(info.version().to_str().unwrap_or_default()).to_string(),
    })
}

fn platform_from_version(version: &str) -> &str {
    if version.contains("weidmueller") {
        "weidmueller"
    } else {
        ""
    }
}

fn read_distro() -> SystemDistro {
    fs::read_to_string("/etc/os-release")
        .or_else(|_| fs::read_to_string("/usr/lib/os-release"))
        .map(|s| distro_from_str(&s))
        .unwrap_or_else(|e| {
            error!("Could not read distro info from /etc/os-release or /usr/lib/os-release: {e}");
            SystemDistro {
                name: String::default(),
                version: String::default(),
                id: String::default(),
                codename: String::default(),
            }
        })
}

fn distro_from_str(value: &str) -> SystemDistro {
    let info: OsReleaseInfo = value.into();
    SystemDistro {
        id: info.get_value("ID").unwrap_or_default(),
        version: info.get_value("VERSION_ID").unwrap_or_default(),
        name: info.get_value("PRETTY_NAME").unwrap_or_default(),
        codename: info.get_value("VERSION_CODENAME").unwrap_or_default(),
    }
}

fn read_kernel(info: &impl UNameAPI) -> SystemKernel {
    SystemKernel {
        build: info.release().to_str().unwrap_or_default().to_string(),
        machine: info.machine().to_str().unwrap_or_default().to_string(),
        version: info.version().to_str().unwrap_or_default().to_string(),
    }
}

impl From<&str> for OsReleaseInfo {
    fn from(value: &str) -> Self {
        let regex = Regex::new(r#"^(.+)=(?:"(.*)"|(.*))$"#).unwrap();

        let mut map = HashMap::default();

        for line in value.lines() {
            if let Some(captures) = regex.captures(line) {
                let key = captures.get(1).map(|m| m.as_str().to_string());
                let value = captures
                    .get(2)
                    .or_else(|| captures.get(3))
                    .map(|m| m.as_str().to_string());
                if let (Some(key), Some(value)) = (key, value) {
                    map.insert(key, value);
                }
            }
        }
        Self(map)
    }
}

struct OsReleaseInfo(HashMap<String, String>);

impl OsReleaseInfo {
    pub fn get_value(&self, key: &str) -> Option<String> {
        self.0.get(&key.to_string()).cloned()
    }
}

fn machine_to_arch(machine: &str) -> &'static str {
    match machine {
        "aarch64" => "arm64",
        "armv7l" => "armhf",
        "x86" => "i386",
        "x86_64" => "amd64",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    #[test]
    fn parse_distro() {
        let expected = SystemDistro {
            id: "endeavouros".to_string(),
            name: "EndeavourOS".to_string(),
            codename: String::new(),
            version: String::new(),
        };

        let value = r#"NAME="EndeavourOS"
PRETTY_NAME="EndeavourOS"
ID="endeavouros"
ID_LIKE="arch"
BUILD_ID="2023.11.17"
ANSI_COLOR="38;2;23;147;209"
HOME_URL="https://endeavouros.com"
DOCUMENTATION_URL="https://discovery.endeavouros.com"
SUPPORT_URL="https://forum.endeavouros.com"
BUG_REPORT_URL="https://forum.endeavouros.com/c/general-system/endeavouros-installation"
PRIVACY_POLICY_URL="https://endeavouros.com/privacy-policy-2"
LOGO="endeavouros""#;

        assert_eq!(distro_from_str(value), expected);
    }

    #[test]
    fn test_os_release_info_from_str() {
        let input = "KEY=\"value\"\nANOTHER_KEY=value";
        let os_release_info: OsReleaseInfo = input.into();

        assert_eq!(
            os_release_info.get_value("KEY"),
            Some(String::from("value"))
        );
        assert_eq!(
            os_release_info.get_value("ANOTHER_KEY"),
            Some(String::from("value"))
        );
        assert_eq!(os_release_info.get_value("NON_EXISTENT_KEY"), None);
    }

    #[test]
    fn test_os_release_info_from_str_empty_input() {
        let os_release_info: OsReleaseInfo = "".into();
        assert!(os_release_info.0.is_empty());
    }

    #[test]
    fn test_os_release_info_from_str_multiple_lines() {
        let input = "KEY=\"value\"\nANOTHER_KEY=value\nYET_ANOTHER=\"quoted value\"";
        let os_release_info: OsReleaseInfo = input.into();

        assert_eq!(
            os_release_info.get_value("KEY"),
            Some(String::from("value"))
        );
        assert_eq!(
            os_release_info.get_value("ANOTHER_KEY"),
            Some(String::from("value"))
        );
        assert_eq!(
            os_release_info.get_value("YET_ANOTHER"),
            Some(String::from("quoted value"))
        );
    }

    #[test]
    fn test_os_release_info_from_str_unquoted_values() {
        let input = "KEY=value\nANOTHER_KEY=\"quoted value\"";
        let os_release_info: OsReleaseInfo = input.into();

        assert_eq!(
            os_release_info.get_value("KEY"),
            Some(String::from("value"))
        );
        assert_eq!(
            os_release_info.get_value("ANOTHER_KEY"),
            Some(String::from("quoted value"))
        );
    }

    #[test]
    fn test_os_release_info_from_str_empty_values() {
        let input = "KEY=\"\"\nANOTHER_KEY=";
        let os_release_info: OsReleaseInfo = input.into();

        assert_eq!(os_release_info.get_value("KEY"), Some(String::from("")));
        assert_eq!(
            os_release_info.get_value("ANOTHER_KEY"),
            Some(String::from(""))
        );
    }

    #[test]
    fn test_os_release_info_from_str_invalid_input() {
        let input = "INVALID_INPUT";
        let os_release_info: OsReleaseInfo = input.into();

        assert!(os_release_info.0.is_empty());
    }

    #[test]
    fn test_get_value_existing_key() {
        let mut map = HashMap::new();
        map.insert("EXISTING_KEY".to_string(), "value".to_string());
        let os_release_info = OsReleaseInfo(map);

        assert_eq!(
            os_release_info.get_value("EXISTING_KEY"),
            Some(String::from("value"))
        );
    }

    #[test]
    fn test_get_value_non_existent_key() {
        let mut map = HashMap::new();
        map.insert("EXISTING_KEY".to_string(), "value".to_string());
        let os_release_info = OsReleaseInfo(map);

        assert_eq!(os_release_info.get_value("NON_EXISTENT_KEY"), None);
    }

    #[test]
    fn test_get_value_empty_map() {
        let os_release_info = OsReleaseInfo(HashMap::new());

        assert_eq!(os_release_info.get_value("ANY_KEY"), None);
    }

    #[test]
    fn test_platform() {
        assert_eq!(
            platform_from_version("some-weidmueller-version"),
            "weidmueller"
        );
        assert_eq!(platform_from_version("weidmueller"), "weidmueller");
        assert_eq!(
            platform_from_version("8.5.2-beta-weidmueller"),
            "weidmueller"
        );
        assert_eq!(platform_from_version("8.5.2-beta"), "");
        assert_eq!(platform_from_version(""), "");
    }

    struct ValidUNameAPI;
    struct InvalidUNameAPI;

    const SYSNAME: &str = "Valid Sysname";
    const NODENAME: &str = "Valid Nodename";
    const RELEASE: &str = "Valid Release";
    const VERSION: &str = "Valid Version";
    const MACHINE: &str = "Valid Machine";
    const OSNAME: &str = "Valid OSName";
    const INVALID_BYTES: [u8; 4] = [0x66, 0x6f, 0x80, 0x6f];

    impl UNameAPI for ValidUNameAPI {
        fn sysname(&self) -> &OsStr {
            SYSNAME.as_ref()
        }

        fn nodename(&self) -> &OsStr {
            NODENAME.as_ref()
        }

        fn release(&self) -> &OsStr {
            RELEASE.as_ref()
        }

        fn version(&self) -> &OsStr {
            VERSION.as_ref()
        }

        fn machine(&self) -> &OsStr {
            MACHINE.as_ref()
        }

        fn osname(&self) -> &OsStr {
            OSNAME.as_ref()
        }
    }

    impl UNameAPI for InvalidUNameAPI {
        fn sysname(&self) -> &OsStr {
            OsStr::from_bytes(&INVALID_BYTES)
        }

        fn nodename(&self) -> &OsStr {
            OsStr::from_bytes(&INVALID_BYTES)
        }

        fn release(&self) -> &OsStr {
            OsStr::from_bytes(&INVALID_BYTES)
        }

        fn version(&self) -> &OsStr {
            OsStr::from_bytes(&INVALID_BYTES)
        }

        fn machine(&self) -> &OsStr {
            OsStr::from_bytes(&INVALID_BYTES)
        }

        fn osname(&self) -> &OsStr {
            OsStr::from_bytes(&INVALID_BYTES)
        }
    }

    #[test]
    fn test_kernel_valid() {
        let result = read_kernel(&ValidUNameAPI);
        assert_eq!(
            result,
            SystemKernel {
                build: RELEASE.to_string(),
                machine: MACHINE.to_string(),
                version: VERSION.to_string(),
            }
        )
    }

    #[test]
    fn test_kernel_invalid() {
        let result = read_kernel(&InvalidUNameAPI);
        assert_eq!(
            result,
            SystemKernel {
                build: String::default(),
                machine: String::default(),
                version: String::default(),
            }
        )
    }

    #[test]
    fn test_machine_to_arch() {
        assert_eq!(machine_to_arch("aarch64"), "arm64");
        assert_eq!(machine_to_arch("armv7l"), "armhf");
        assert_eq!(machine_to_arch("x86"), "i386");
        assert_eq!(machine_to_arch("x86_64"), "amd64");
        assert_eq!(machine_to_arch("1234"), "unknown");
        assert_eq!(machine_to_arch(""), "unknown");
    }
}
