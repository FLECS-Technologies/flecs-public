use crate::jeweler::gem::manifest::{EnvironmentVariable, PortMapping, VolumeMount};
use crate::jeweler::network::NetworkId;
use crate::jeweler::volume::VolumeId;
use crate::relic::device::usb::UsbDevice;
use bollard::models::{DeviceMapping, MountTypeEnum, PortBinding};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::path::PathBuf;
use tracing::warn;

#[cfg(not(test))]
const USB_DEVICE_PATH: &str = "/dev/bus/usb/";
#[cfg(test)]
const USB_DEVICE_PATH: &str = "/tmp/flecs-tests/dev/bus/usb/";

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct InstanceConfig {
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub volume_mounts: HashMap<VolumeId, VolumeMount>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub environment_variables: Vec<EnvironmentVariable>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub port_mapping: Vec<PortMapping>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub network_addresses: HashMap<NetworkId, IpAddr>,
    #[serde(skip_serializing_if = "HashSet::is_empty", default)]
    pub usb_devices: HashSet<UsbDevice>,
}

impl InstanceConfig {
    pub fn generate_port_bindings(&self) -> HashMap<String, Option<Vec<PortBinding>>> {
        let result = self
            .port_mapping
            .iter()
            .flat_map(|port_mapping| port_mapping_to_port_bindings(port_mapping).into_iter())
            .collect();
        result
    }

    pub fn generate_usb_device_mappings(&self) -> Vec<DeviceMapping> {
        let mut device_mappings = Vec::new();
        for usb_device in self.usb_devices.iter() {
            match try_usb_device_to_device_mapping(usb_device) {
                Err(e) => warn!(
                    "Could not generate device mapping for usb device {}: {e}",
                    usb_device.device
                ),
                Ok(mapping) => device_mappings.push(mapping),
            }
        }
        device_mappings
    }

    pub fn generate_volume_mounts(&self) -> Vec<bollard::models::Mount> {
        self.volume_mounts
            .iter()
            .map(|(id, volume_mount)| bollard::models::Mount {
                typ: Some(MountTypeEnum::VOLUME),
                source: Some(id.clone()),
                target: Some(volume_mount.container_path.to_string_lossy().to_string()),
                ..Default::default()
            })
            .collect()
    }

    pub fn generate_network_config(&self) -> bollard::container::NetworkingConfig<String> {
        let endpoints_config = self
            .network_addresses
            .iter()
            .map(|(id, address)| {
                let endpoint_ipam_config = match address {
                    IpAddr::V4(address) => bollard::models::EndpointIpamConfig {
                        ipv4_address: Some(address.to_string()),
                        ..Default::default()
                    },
                    IpAddr::V6(address) => bollard::models::EndpointIpamConfig {
                        ipv6_address: Some(address.to_string()),
                        ..Default::default()
                    },
                };
                (
                    id.clone(),
                    bollard::models::EndpointSettings {
                        ip_address: Some(address.to_string()),
                        ipam_config: Some(endpoint_ipam_config),
                        ..Default::default()
                    },
                )
            })
            .collect();
        bollard::container::NetworkingConfig { endpoints_config }
    }
}

fn new_port_bindings(host_port: u16, container_port: u16) -> (String, Option<Vec<PortBinding>>) {
    (
        format!("{}/tcp", container_port),
        Some(vec![PortBinding {
            host_port: Some(host_port.to_string()),
            host_ip: None,
        }]),
    )
}

fn port_mapping_to_port_bindings(
    port_mapping: &PortMapping,
) -> HashMap<String, Option<Vec<PortBinding>>> {
    match port_mapping {
        PortMapping::Single(host, container) => {
            HashMap::from([new_port_bindings(*host, *container)])
        }
        PortMapping::Range { from, to } => HashMap::from_iter(
            from.range()
                .zip(to.range())
                .map(|(host, container)| new_port_bindings(host, container)),
        ),
    }
}

fn try_usb_device_to_device_mapping(usb_device: &UsbDevice) -> crate::Result<DeviceMapping> {
    match (usb_device.get_bus_num(), usb_device.get_dev_num()) {
        (None, None) => Err(anyhow::anyhow!(
            "Could not get bus and dev num for {}",
            usb_device.device
        )),
        (None, _) => Err(anyhow::anyhow!(
            "Could not get bus num for {}",
            usb_device.device
        )),
        (_, None) => Err(anyhow::anyhow!(
            "Could not get dev num for {}",
            usb_device.device
        )),
        (Some(bus_num), Some(dev_num)) => {
            let path = PathBuf::from(format!("{USB_DEVICE_PATH}{bus_num:03}/{dev_num:03}"));
            anyhow::ensure!(
                path.try_exists()?,
                "Path {:?} for device {} does not exist",
                path,
                usb_device.device
            );
            Ok(DeviceMapping {
                path_in_container: Some(path.to_string_lossy().to_string()),
                path_on_host: Some(path.to_string_lossy().to_string()),
                cgroup_permissions: Some("rwm".to_string()),
            })
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::jeweler::gem::manifest::PortRange;
    use crate::relic::device::usb::tests::prepare_usb_device_test_path;
    use bollard::models::Mount;
    use std::fs;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use std::path::{Path, PathBuf};

    #[test]
    fn new_port_bindings_test() {
        assert_eq!(
            new_port_bindings(10, 20),
            (
                "20/tcp".to_string(),
                Some(vec![PortBinding {
                    host_port: Some(10.to_string()),
                    host_ip: None,
                }])
            )
        )
    }

    #[test]
    fn port_mapping_to_port_bindings_single() {
        let port_mapping = PortMapping::Single(60, 600);
        assert_eq!(
            port_mapping_to_port_bindings(&port_mapping),
            HashMap::from([(
                "600/tcp".to_string(),
                Some(vec![PortBinding {
                    host_port: Some(60.to_string()),
                    host_ip: None,
                }])
            )])
        )
    }

    #[test]
    fn port_mapping_to_port_bindings_range() {
        let port_mapping = PortMapping::Range {
            from: PortRange::try_new(100, 102).unwrap(),
            to: PortRange::try_new(40, 42).unwrap(),
        };
        assert_eq!(
            port_mapping_to_port_bindings(&port_mapping),
            HashMap::from([
                (
                    "40/tcp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(100.to_string()),
                        host_ip: None,
                    }])
                ),
                (
                    "41/tcp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(101.to_string()),
                        host_ip: None,
                    }])
                ),
                (
                    "42/tcp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(102.to_string()),
                        host_ip: None,
                    }])
                ),
            ])
        )
    }

    #[test]
    fn generate_port_bindings_test() {
        let config = InstanceConfig {
            port_mapping: vec![
                PortMapping::Range {
                    from: PortRange::try_new(100, 102).unwrap(),
                    to: PortRange::try_new(40, 42).unwrap(),
                },
                PortMapping::Single(60, 600),
            ],
            ..InstanceConfig::default()
        };
        assert_eq!(
            config.generate_port_bindings(),
            HashMap::from([
                (
                    "40/tcp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(100.to_string()),
                        host_ip: None,
                    }])
                ),
                (
                    "41/tcp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(101.to_string()),
                        host_ip: None,
                    }])
                ),
                (
                    "42/tcp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(102.to_string()),
                        host_ip: None,
                    }])
                ),
                (
                    "600/tcp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(60.to_string()),
                        host_ip: None,
                    }])
                )
            ])
        )
    }

    #[test]
    fn generate_volume_mounts_test() {
        let config = InstanceConfig {
            volume_mounts: HashMap::from([
                (
                    "VolumeId1".to_string(),
                    VolumeMount {
                        name: "Volume1".to_string(),
                        container_path: PathBuf::from("/path/to/volume-1"),
                    },
                ),
                (
                    "VolumeId2".to_string(),
                    VolumeMount {
                        name: "Volume2".to_string(),
                        container_path: PathBuf::from("/path/to/volume-2"),
                    },
                ),
            ]),
            ..InstanceConfig::default()
        };
        let resulting_mounts = config.generate_volume_mounts();
        assert_eq!(resulting_mounts.len(), 2);
        let expected_mounts = vec![
            Mount {
                typ: Some(MountTypeEnum::VOLUME),
                source: Some("VolumeId1".to_string()),
                target: Some("/path/to/volume-1".to_string()),
                ..Mount::default()
            },
            Mount {
                typ: Some(MountTypeEnum::VOLUME),
                source: Some("VolumeId2".to_string()),
                target: Some("/path/to/volume-2".to_string()),
                ..Mount::default()
            },
        ];
        for expected_mount in expected_mounts {
            assert!(resulting_mounts.contains(&expected_mount))
        }
    }

    pub(crate) fn prepare_path(path: &Path) {
        println!("Preparing {:?}", path);
        let _ = fs::remove_dir_all(path);
        assert!(!path.try_exists().unwrap());
        fs::create_dir_all(path).unwrap();
        assert!(path.try_exists().unwrap());
    }

    fn test_usb_device(port: &str) -> UsbDevice {
        UsbDevice {
            vid: 0,
            pid: 0,
            port: port.to_string(),
            device: "".to_string(),
            vendor: "".to_string(),
        }
    }

    #[test]
    fn try_usb_device_to_device_mapping_ok() {
        let path = prepare_usb_device_test_path("try_usb_device_to_device_mapping_ok");
        let bus_path = PathBuf::from(format!("{USB_DEVICE_PATH}010"));
        prepare_path(&bus_path);
        fs::write(bus_path.join("020"), b"test-dev").unwrap();
        fs::write(path.join("devnum"), b"20").unwrap();
        fs::write(path.join("busnum"), b"10").unwrap();
        let usb_device = test_usb_device("try_usb_device_to_device_mapping_ok");
        assert_eq!(
            try_usb_device_to_device_mapping(&usb_device).unwrap(),
            DeviceMapping {
                cgroup_permissions: Some("rwm".to_string()),
                path_on_host: Some(
                    Path::new(&format!("{USB_DEVICE_PATH}010/020"))
                        .to_string_lossy()
                        .to_string()
                ),
                path_in_container: Some(
                    Path::new(&format!("{USB_DEVICE_PATH}010/020"))
                        .to_string_lossy()
                        .to_string()
                ),
            }
        )
    }

    #[test]
    fn try_usb_device_to_device_mapping_device_does_not_exist() {
        let path =
            prepare_usb_device_test_path("try_usb_device_to_device_mapping_device_does_not_exist");
        let bus_path = PathBuf::from(format!("{USB_DEVICE_PATH}080"));
        prepare_path(&bus_path);
        fs::write(path.join("devnum"), b"81").unwrap();
        fs::write(path.join("busnum"), b"80").unwrap();
        let usb_device = test_usb_device("try_usb_device_to_device_mapping_device_does_not_exist");
        assert!(try_usb_device_to_device_mapping(&usb_device).is_err())
    }

    #[test]
    fn try_usb_device_to_device_mapping_no_bus_num() {
        let path = prepare_usb_device_test_path("try_usb_device_to_device_mapping_no_bus_num");
        fs::write(path.join("devnum"), b"40").unwrap();
        let usb_device = test_usb_device("try_usb_device_to_device_mapping_no_bus_num");
        assert!(try_usb_device_to_device_mapping(&usb_device).is_err())
    }

    #[test]
    fn try_usb_device_to_device_mapping_no_dev_num() {
        let path = prepare_usb_device_test_path("try_usb_device_to_device_mapping_no_dev_num");
        fs::write(path.join("busnum"), b"70").unwrap();
        let usb_device = test_usb_device("try_usb_device_to_device_mapping_no_dev_num");
        assert!(try_usb_device_to_device_mapping(&usb_device).is_err())
    }

    #[test]
    fn try_usb_device_to_device_mapping_no_num() {
        prepare_usb_device_test_path("try_usb_device_to_device_mapping_no_num");
        let usb_device = test_usb_device("try_usb_device_to_device_mapping_no_num");
        assert!(try_usb_device_to_device_mapping(&usb_device).is_err())
    }

    #[test]
    fn generate_usb_device_mappings_ok() {
        let path = prepare_usb_device_test_path("generate_usb_device_mappings_ok_1");
        let bus_path = PathBuf::from(format!("{USB_DEVICE_PATH}111"));
        prepare_path(&bus_path);
        fs::write(bus_path.join("999"), b"test-dev-1").unwrap();
        fs::write(path.join("devnum"), b"999").unwrap();
        fs::write(path.join("busnum"), b"111").unwrap();
        let path = prepare_usb_device_test_path("generate_usb_device_mappings_ok_2");
        let bus_path = PathBuf::from(format!("{USB_DEVICE_PATH}222"));
        prepare_path(&bus_path);
        fs::write(bus_path.join("888"), b"test-dev-2").unwrap();
        fs::write(path.join("devnum"), b"888").unwrap();
        fs::write(path.join("busnum"), b"222").unwrap();
        let path = prepare_usb_device_test_path("generate_usb_device_mappings_ok_3");
        let bus_path = PathBuf::from(format!("{USB_DEVICE_PATH}333"));
        prepare_path(&bus_path);
        fs::write(bus_path.join("777"), b"test-dev-3").unwrap();
        fs::write(path.join("devnum"), b"777").unwrap();
        fs::write(path.join("busnum"), b"333").unwrap();
        let device_mappings = InstanceConfig {
            usb_devices: HashSet::from([
                test_usb_device("generate_usb_device_mappings_ok_1"),
                test_usb_device("generate_usb_device_mappings_ok_2"),
                test_usb_device("generate_usb_device_mappings_ok_3"),
            ]),
            ..InstanceConfig::default()
        }
        .generate_usb_device_mappings();
        assert_eq!(device_mappings.len(), 3);
        assert!(device_mappings.contains(&DeviceMapping {
            cgroup_permissions: Some("rwm".to_string()),
            path_on_host: Some(
                Path::new(&format!("{USB_DEVICE_PATH}111/999"))
                    .to_string_lossy()
                    .to_string()
            ),
            path_in_container: Some(
                Path::new(&format!("{USB_DEVICE_PATH}111/999"))
                    .to_string_lossy()
                    .to_string()
            ),
        }));
        assert!(device_mappings.contains(&DeviceMapping {
            cgroup_permissions: Some("rwm".to_string()),
            path_on_host: Some(
                Path::new(&format!("{USB_DEVICE_PATH}222/888"))
                    .to_string_lossy()
                    .to_string()
            ),
            path_in_container: Some(
                Path::new(&format!("{USB_DEVICE_PATH}222/888"))
                    .to_string_lossy()
                    .to_string()
            ),
        }));
        assert!(device_mappings.contains(&DeviceMapping {
            cgroup_permissions: Some("rwm".to_string()),
            path_on_host: Some(
                Path::new(&format!("{USB_DEVICE_PATH}333/777"))
                    .to_string_lossy()
                    .to_string()
            ),
            path_in_container: Some(
                Path::new(&format!("{USB_DEVICE_PATH}333/777"))
                    .to_string_lossy()
                    .to_string()
            ),
        }));
    }

    #[test]
    fn generate_usb_device_mappings_partly_ok() {
        let path = prepare_usb_device_test_path("generate_usb_device_mappings_partly_ok_1");
        let bus_path = PathBuf::from(format!("{USB_DEVICE_PATH}121"));
        prepare_path(&bus_path);
        fs::write(bus_path.join("919"), b"test-dev-1").unwrap();
        fs::write(path.join("devnum"), b"919").unwrap();
        fs::write(path.join("busnum"), b"121").unwrap();
        let path = prepare_usb_device_test_path("generate_usb_device_mappings_partly_ok_3");
        let bus_path = PathBuf::from(format!("{USB_DEVICE_PATH}343"));
        prepare_path(&bus_path);
        fs::write(bus_path.join("747"), b"test-dev-3").unwrap();
        fs::write(path.join("devnum"), b"747").unwrap();
        fs::write(path.join("busnum"), b"343").unwrap();
        let device_mappings = InstanceConfig {
            usb_devices: HashSet::from([
                test_usb_device("generate_usb_device_mappings_partly_ok_1"),
                test_usb_device("generate_usb_device_mappings_partly_ok_2"),
                test_usb_device("generate_usb_device_mappings_partly_ok_3"),
            ]),
            ..InstanceConfig::default()
        }
        .generate_usb_device_mappings();
        assert_eq!(device_mappings.len(), 2);
        assert!(device_mappings.contains(&DeviceMapping {
            cgroup_permissions: Some("rwm".to_string()),
            path_on_host: Some(
                Path::new(&format!("{USB_DEVICE_PATH}121/919"))
                    .to_string_lossy()
                    .to_string()
            ),
            path_in_container: Some(
                Path::new(&format!("{USB_DEVICE_PATH}121/919"))
                    .to_string_lossy()
                    .to_string()
            ),
        }));
        assert!(device_mappings.contains(&DeviceMapping {
            cgroup_permissions: Some("rwm".to_string()),
            path_on_host: Some(
                Path::new(&format!("{USB_DEVICE_PATH}343/747"))
                    .to_string_lossy()
                    .to_string()
            ),
            path_in_container: Some(
                Path::new(&format!("{USB_DEVICE_PATH}343/747"))
                    .to_string_lossy()
                    .to_string()
            ),
        }));
    }

    #[test]
    fn generate_network_config_empty() {
        let config = InstanceConfig {
            network_addresses: HashMap::from([]),
            ..InstanceConfig::default()
        };
        assert_eq!(
            config.generate_network_config(),
            bollard::container::NetworkingConfig {
                endpoints_config: HashMap::from([]),
            }
        )
    }

    #[test]
    fn generate_network_config_single_ipv4() {
        let config = InstanceConfig {
            network_addresses: HashMap::from([(
                "test-network".to_string(),
                IpAddr::V4(Ipv4Addr::new(160, 80, 40, 20)),
            )]),
            ..InstanceConfig::default()
        };
        assert_eq!(
            config.generate_network_config(),
            bollard::container::NetworkingConfig {
                endpoints_config: HashMap::from([(
                    "test-network".to_string(),
                    bollard::models::EndpointSettings {
                        ip_address: Some("160.80.40.20".to_string()),
                        ipam_config: Some(bollard::models::EndpointIpamConfig {
                            ipv4_address: Some("160.80.40.20".to_string()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }
                )]),
            }
        )
    }

    #[test]
    fn generate_network_config_single_ipv6() {
        let config = InstanceConfig {
            network_addresses: HashMap::from([(
                "test-network".to_string(),
                IpAddr::V6(Ipv6Addr::new(
                    0xab, 0xcd, 0xef, 0x12, 0x34, 0x45, 0x67, 0x89,
                )),
            )]),
            ..InstanceConfig::default()
        };
        assert_eq!(
            config.generate_network_config(),
            bollard::container::NetworkingConfig {
                endpoints_config: HashMap::from([(
                    "test-network".to_string(),
                    bollard::models::EndpointSettings {
                        ip_address: Some("ab:cd:ef:12:34:45:67:89".to_string()),
                        ipam_config: Some(bollard::models::EndpointIpamConfig {
                            ipv6_address: Some("ab:cd:ef:12:34:45:67:89".to_string()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }
                )]),
            }
        )
    }

    #[test]
    fn generate_network_config_multiple() {
        let config = InstanceConfig {
            network_addresses: HashMap::from([
                (
                    "test-network1".to_string(),
                    IpAddr::V6(Ipv6Addr::new(
                        0xab, 0xcd, 0xef, 0x12, 0x34, 0x45, 0x67, 0x89,
                    )),
                ),
                (
                    "test-network2".to_string(),
                    IpAddr::V4(Ipv4Addr::new(160, 80, 40, 20)),
                ),
                (
                    "test-network3".to_string(),
                    IpAddr::V6(Ipv6Addr::new(
                        0xab, 0xcd, 0xef, 0x12, 0x34, 0x45, 0x67, 0x11,
                    )),
                ),
                (
                    "test-network4".to_string(),
                    IpAddr::V4(Ipv4Addr::new(160, 80, 40, 200)),
                ),
            ]),
            ..InstanceConfig::default()
        };
        assert_eq!(
            config.generate_network_config(),
            bollard::container::NetworkingConfig {
                endpoints_config: HashMap::from([
                    (
                        "test-network1".to_string(),
                        bollard::models::EndpointSettings {
                            ip_address: Some("ab:cd:ef:12:34:45:67:89".to_string()),
                            ipam_config: Some(bollard::models::EndpointIpamConfig {
                                ipv6_address: Some("ab:cd:ef:12:34:45:67:89".to_string()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }
                    ),
                    (
                        "test-network2".to_string(),
                        bollard::models::EndpointSettings {
                            ip_address: Some("160.80.40.20".to_string()),
                            ipam_config: Some(bollard::models::EndpointIpamConfig {
                                ipv4_address: Some("160.80.40.20".to_string()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }
                    ),
                    (
                        "test-network3".to_string(),
                        bollard::models::EndpointSettings {
                            ip_address: Some("ab:cd:ef:12:34:45:67:11".to_string()),
                            ipam_config: Some(bollard::models::EndpointIpamConfig {
                                ipv6_address: Some("ab:cd:ef:12:34:45:67:11".to_string()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }
                    ),
                    (
                        "test-network4".to_string(),
                        bollard::models::EndpointSettings {
                            ip_address: Some("160.80.40.200".to_string()),
                            ipam_config: Some(bollard::models::EndpointIpamConfig {
                                ipv4_address: Some("160.80.40.200".to_string()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }
                    ),
                ]),
            }
        )
    }
}
