use crate::relic::device::usb::UsbDevice;
use crate::vault::pouch::{AppKey, DeploymentId, Pouch, VaultPouch};
use flecsd_axum_server::models;
use flecsd_axum_server::models::InstanceStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

const APPS_FILE_NAME: &str = "apps.json";
pub type AppStatus = models::AppStatus;

pub type InstanceId = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PortRange {
    start: u16,
    end: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum PortMapping {
    Range { from: PortRange, to: PortRange },
    Single { from: u16, to: u16 },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EnvironmentVariable {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Network {
    ip_addr: IpAddr,
    mac_addr: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Instance {
    id: InstanceId,
    name: String,
    desired: InstanceStatus,
    networks: Vec<Network>,
    environment: Vec<EnvironmentVariable>,
    ports: Vec<PortMapping>,
    usb_devices: Vec<UsbDevice>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AppData {
    desired: AppStatus,
    instances: Vec<Instance>,
}

// TODO: Implement version handling
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct App {
    app_key: AppKey,
    properties: HashMap<DeploymentId, AppData>,
}

pub struct AppPouch {
    path: PathBuf,
    apps: HashMap<AppKey, App>,
}

impl Pouch for AppPouch {
    type Gems = HashMap<AppKey, App>;

    fn gems(&self) -> &Self::Gems {
        &self.apps
    }

    fn gems_mut(&mut self) -> &mut Self::Gems {
        &mut self.apps
    }
}

impl VaultPouch for AppPouch {
    fn close(&mut self) -> crate::vault::Result<()> {
        let file = fs::File::create(self.path.join(APPS_FILE_NAME))?;
        let content: Vec<_> = self.apps.values().collect();
        serde_json::to_writer_pretty(file, &content)?;
        Ok(())
    }

    fn open(&mut self) -> crate::vault::Result<()> {
        let file = fs::File::open(self.path.join(APPS_FILE_NAME))?;
        let apps: Vec<App> = serde_json::from_reader(file)?;
        self.apps = apps
            .into_iter()
            .map(|app| (app.app_key.clone(), app))
            .collect();
        Ok(())
    }
}

impl AppPouch {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            apps: HashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::fs;
    use std::net::Ipv4Addr;
    use std::path::Path;
    use std::str::FromStr;

    const TEST_PATH: &str = "/tmp/flecs-tests/app-pouch/";

    fn prepare_path(path: &Path) {
        println!("Preparing {:?}", path);
        let _ = fs::remove_dir_all(path);
        assert!(!path.try_exists().unwrap());
        fs::create_dir_all(path).unwrap();
        assert!(path.try_exists().unwrap());
    }

    fn create_test_json() -> Value {
        serde_json::json!([
            {
                "app_key": {
                    "name": "test-app-1",
                    "version": "1.2.3"
                },
                "properties": {
                    "test-deployment-id-1": {
                        "desired": "installed",
                        "instances": [
                            {
                                "id": "test-instance-id-1",
                                "name": "Some Test Instance #1",
                                "desired": "running",
                                "networks": [
                                    {
                                        "ip_addr": "12.34.56.78",
                                        "mac_addr": "60:2D:2F:45:BA:72",
                                        "name": "Some Test Network #1"
                                    }
                                ],
                                "environment": [
                                    {
                                        "name": "VariableWithValue",
                                        "value": "TestValue"
                                    },
                                    {
                                        "name": "VariableWithoutValue"
                                    }
                                ],
                                "ports": [
                                    {
                                        "from": 10,
                                        "to": 20
                                    },
                                    {
                                        "from": {
                                            "start": 2000,
                                            "end": 3000,
                                        },
                                        "to": {
                                            "start": 5000,
                                            "end": 6000,
                                        }
                                    }
                                ],
                                "usb_devices": [
                                    {
                                        "vid": 123,
                                        "pid": 456,
                                        "port": "test-port-1",
                                        "device": "test-device-1",
                                        "vendor": "test-vendor-1"
                                    }
                                ]
                            }
                        ]
                    }
                }
            }
        ])
    }

    fn create_test_app() -> App {
        App {
            app_key: AppKey {
                name: "test-app-1".to_string(),
                version: "1.2.3".to_string(),
            },
            properties: HashMap::from([(
                "test-deployment-id-1".to_string(),
                AppData {
                    desired: AppStatus::Installed,
                    instances: vec![Instance {
                        id: "test-instance-id-1".to_string(),
                        name: "Some Test Instance #1".to_string(),
                        desired: InstanceStatus::Running,
                        networks: vec![Network {
                            ip_addr: IpAddr::V4(Ipv4Addr::from_str("12.34.56.78").unwrap()),
                            mac_addr: "60:2D:2F:45:BA:72".to_string(),
                            name: "Some Test Network #1".to_string(),
                        }],
                        environment: vec![
                            EnvironmentVariable {
                                name: "VariableWithValue".to_string(),
                                value: Some("TestValue".to_string()),
                            },
                            EnvironmentVariable {
                                name: "VariableWithoutValue".to_string(),
                                value: None,
                            },
                        ],
                        ports: vec![
                            PortMapping::Single { from: 10, to: 20 },
                            PortMapping::Range {
                                from: PortRange {
                                    start: 2000,
                                    end: 3000,
                                },
                                to: PortRange {
                                    start: 5000,
                                    end: 6000,
                                },
                            },
                        ],
                        usb_devices: vec![UsbDevice {
                            vid: 123,
                            pid: 456,
                            port: "test-port-1".to_string(),
                            device: "test-device-1".to_string(),
                            vendor: "test-vendor-1".to_string(),
                        }],
                    }],
                },
            )]),
        }
    }

    #[test]
    fn open_app_pouch() {
        let path = Path::new(TEST_PATH).join("open_pouch");
        prepare_path(&path);
        let json = create_test_json();
        fs::write(
            path.join(APPS_FILE_NAME),
            serde_json::to_string_pretty(&json).unwrap(),
        )
        .unwrap();
        let mut app_pouch = AppPouch {
            apps: HashMap::default(),
            path,
        };
        app_pouch.open().unwrap();
    }

    #[test]
    fn close_app_pouch() {
        let path = Path::new(TEST_PATH).join("close_pouch");
        prepare_path(&path);
        let json = create_test_json();
        let app = create_test_app();
        let mut app_pouch = AppPouch {
            apps: HashMap::from([(app.app_key.clone(), app)]),
            path: path.clone(),
        };
        app_pouch.close().unwrap();
        let file = fs::File::open(path.join(APPS_FILE_NAME)).unwrap();
        let content: Value = serde_json::from_reader(file).unwrap();
        assert_eq!(content, json);
    }

    #[test]
    fn app_gems() {
        let app = create_test_app();
        let gems = HashMap::from([(app.app_key.clone(), app)]);
        let mut app_pouch = AppPouch {
            apps: gems.clone(),
            path: PathBuf::from(TEST_PATH),
        };
        assert_eq!(&gems, app_pouch.gems());
        assert_eq!(&gems, app_pouch.gems_mut());
    }
}
