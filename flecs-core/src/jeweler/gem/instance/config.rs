use crate::forge::vec::VecExtension;
use crate::jeweler::gem::manifest::{EnvironmentVariable, PortMapping, PortRange, VolumeMount};
use crate::jeweler::network::NetworkId;
use crate::jeweler::volume::VolumeId;
use crate::relic::device::usb::{UsbDevice, UsbDeviceReader, UsbDeviceReaderExtension};
use bollard::models::{DeviceMapping, MountTypeEnum, PortBinding};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use tracing::warn;

#[cfg(not(test))]
const USB_DEVICE_PATH: &str = "/dev/bus/usb/";
#[cfg(test)]
const USB_DEVICE_PATH: &str = "/tmp/flecs-tests/dev/bus/usb/";

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct InstancePortMapping {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tcp: Vec<PortMapping>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub udp: Vec<PortMapping>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub sctp: Vec<PortMapping>,
}

impl InstancePortMapping {
    pub fn is_empty(&self) -> bool {
        self.tcp.is_empty() && self.udp.is_empty() && self.sctp.is_empty()
    }

    pub fn clear(&mut self) {
        self.tcp.clear();
        self.udp.clear();
        self.sctp.clear();
    }

    /// Returns Ok(true) if an element was updated, Ok(false) if an element was added
    fn update_port_mapping_vec(
        existing_port_mappings: &mut Vec<PortMapping>,
        port_mapping: PortMapping,
    ) -> anyhow::Result<bool> {
        for existing_port_mapping in existing_port_mappings.iter_mut() {
            if port_mapping.are_host_ports_equal(existing_port_mapping) {
                *existing_port_mapping = port_mapping;
                return Ok(true);
            }
            if port_mapping.do_host_ports_overlap(existing_port_mapping) {
                return Err(anyhow::anyhow!("New port mapping {port_mapping} overlaps with existing port mapping {existing_port_mapping}"));
            }
        }
        existing_port_mappings.push(port_mapping);
        Ok(false)
    }

    fn delete_port_mapping_vec(
        existing_port_mappings: &mut Vec<PortMapping>,
        host_port: u16,
    ) -> Option<PortMapping> {
        existing_port_mappings.extract_first_element_with(|mapping| {
            mapping.are_host_ports_equal(&PortMapping::Single(host_port, 0))
        })
    }

    fn get_port_mapping_vec(
        existing_port_mappings: &[PortMapping],
        host_port: u16,
    ) -> Option<PortMapping> {
        existing_port_mappings
            .iter()
            .find(|mapping| mapping.are_host_ports_equal(&PortMapping::Single(host_port, 0)))
            .cloned()
    }

    fn get_port_mapping_range_vec(
        existing_port_mappings: &[PortMapping],
        host_ports: PortRange,
    ) -> Option<PortMapping> {
        existing_port_mappings
            .iter()
            .find(|mapping| {
                mapping.are_host_ports_equal(&PortMapping::Range {
                    from: host_ports,
                    to: host_ports,
                })
            })
            .cloned()
    }

    fn delete_port_mapping_range_vec(
        existing_port_mappings: &mut Vec<PortMapping>,
        host_ports: PortRange,
    ) -> Option<PortMapping> {
        existing_port_mappings.extract_first_element_with(|mapping| {
            mapping.are_host_ports_equal(&PortMapping::Range {
                from: host_ports,
                to: host_ports,
            })
        })
    }

    pub fn update_port_mapping(
        &mut self,
        mapping: PortMapping,
        protocol: TransportProtocol,
    ) -> anyhow::Result<bool> {
        Self::update_port_mapping_vec(
            match protocol {
                TransportProtocol::Tcp => &mut self.tcp,
                TransportProtocol::Udp => &mut self.udp,
                TransportProtocol::Sctp => &mut self.sctp,
            },
            mapping,
        )
    }

    pub fn delete_port_mapping(
        &mut self,
        host_port: u16,
        protocol: TransportProtocol,
    ) -> Option<PortMapping> {
        Self::delete_port_mapping_vec(
            match protocol {
                TransportProtocol::Tcp => &mut self.tcp,
                TransportProtocol::Udp => &mut self.udp,
                TransportProtocol::Sctp => &mut self.sctp,
            },
            host_port,
        )
    }

    pub fn get_port_mapping(
        &self,
        host_port: u16,
        protocol: TransportProtocol,
    ) -> Option<PortMapping> {
        Self::get_port_mapping_vec(
            match protocol {
                TransportProtocol::Tcp => &self.tcp,
                TransportProtocol::Udp => &self.udp,
                TransportProtocol::Sctp => &self.sctp,
            },
            host_port,
        )
    }

    pub fn get_port_mapping_range(
        &self,
        host_port_range: PortRange,
        protocol: TransportProtocol,
    ) -> Option<PortMapping> {
        Self::get_port_mapping_range_vec(
            match protocol {
                TransportProtocol::Tcp => &self.tcp,
                TransportProtocol::Udp => &self.udp,
                TransportProtocol::Sctp => &self.sctp,
            },
            host_port_range,
        )
    }

    pub fn delete_port_mapping_range(
        &mut self,
        host_ports: PortRange,
        protocol: TransportProtocol,
    ) -> Option<PortMapping> {
        Self::delete_port_mapping_range_vec(
            match protocol {
                TransportProtocol::Tcp => &mut self.tcp,
                TransportProtocol::Udp => &mut self.udp,
                TransportProtocol::Sctp => &mut self.sctp,
            },
            host_ports,
        )
    }
}

#[derive(Debug, Default, Hash, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct UsbPathConfig {
    pub port: String,
    pub bus_num: u16,
    pub dev_num: u16,
}

impl TryFrom<(&UsbDevice, &dyn UsbDeviceReader)> for UsbPathConfig {
    type Error = crate::Error;

    fn try_from((device, reader): (&UsbDevice, &dyn UsbDeviceReader)) -> Result<Self, Self::Error> {
        Ok(Self {
            dev_num: reader.get_dev_num(&device.port)?,
            bus_num: reader.get_bus_num(&device.port)?,
            port: device.port.clone(),
        })
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct InstanceConfig {
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub volume_mounts: HashMap<VolumeId, VolumeMount>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub environment_variables: Vec<EnvironmentVariable>,
    #[serde(skip_serializing_if = "InstancePortMapping::is_empty", default)]
    pub port_mapping: InstancePortMapping,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub network_addresses: HashMap<NetworkId, IpAddr>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub usb_devices: HashMap<String, UsbPathConfig>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum TransportProtocol {
    Tcp,
    Udp,
    Sctp,
}

impl InstanceConfig {
    pub fn generate_port_bindings(&self) -> HashMap<String, Option<Vec<PortBinding>>> {
        let result = self
            .port_mapping
            .tcp
            .iter()
            .flat_map(|port_mapping| {
                port_mapping_to_port_bindings(port_mapping, TransportProtocol::Tcp).into_iter()
            })
            .chain(self.port_mapping.udp.iter().flat_map(|port_mapping| {
                port_mapping_to_port_bindings(port_mapping, TransportProtocol::Udp).into_iter()
            }))
            .chain(self.port_mapping.sctp.iter().flat_map(|port_mapping| {
                port_mapping_to_port_bindings(port_mapping, TransportProtocol::Sctp).into_iter()
            }))
            .collect();
        result
    }

    pub fn generate_usb_device_mappings(&self) -> Vec<DeviceMapping> {
        let mut device_mappings = Vec::new();
        for (_, usb_device) in self.usb_devices.iter() {
            match DeviceMapping::try_from(usb_device) {
                Err(e) => warn!(
                    "Could not generate device mapping for usb device {}: {e}",
                    usb_device.port
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

fn new_port_bindings(
    host_port: u16,
    container_port: u16,
    transport_protocol: TransportProtocol,
) -> (String, Option<Vec<PortBinding>>) {
    (
        format!(
            "{container_port}/{}",
            match transport_protocol {
                TransportProtocol::Tcp => "tcp",
                TransportProtocol::Udp => "udp",
                TransportProtocol::Sctp => "sctp",
            }
        ),
        Some(vec![PortBinding {
            host_port: Some(host_port.to_string()),
            host_ip: None,
        }]),
    )
}

fn port_mapping_to_port_bindings(
    port_mapping: &PortMapping,
    transport_protocol: TransportProtocol,
) -> HashMap<String, Option<Vec<PortBinding>>> {
    match port_mapping {
        PortMapping::Single(host, container) => {
            HashMap::from([new_port_bindings(*host, *container, transport_protocol)])
        }
        PortMapping::Range { from, to } => HashMap::from_iter(
            from.range()
                .zip(to.range())
                .map(|(host, container)| new_port_bindings(host, container, transport_protocol)),
        ),
    }
}

impl TryFrom<&UsbPathConfig> for DeviceMapping {
    type Error = crate::Error;

    fn try_from(value: &UsbPathConfig) -> Result<Self, Self::Error> {
        let path = PathBuf::from(format!(
            "{USB_DEVICE_PATH}{:03}/{:03}",
            value.bus_num, value.dev_num
        ));
        anyhow::ensure!(
            path.try_exists()?,
            "Path {:?} for device {} does not exist",
            path,
            value.port
        );
        Ok(DeviceMapping {
            path_in_container: Some(path.to_string_lossy().to_string()),
            path_on_host: Some(path.to_string_lossy().to_string()),
            cgroup_permissions: Some("rwm".to_string()),
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::jeweler::gem::manifest::PortRange;
    use crate::relic::device::usb::tests::prepare_usb_device_test_path;
    use crate::relic::device::usb::MockUsbDeviceReader;
    use bollard::models::Mount;
    use std::fs;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use std::path::{Path, PathBuf};

    #[test]
    fn new_port_bindings_test() {
        assert_eq!(
            new_port_bindings(10, 20, TransportProtocol::Tcp),
            (
                "20/tcp".to_string(),
                Some(vec![PortBinding {
                    host_port: Some(10.to_string()),
                    host_ip: None,
                }])
            )
        );
        assert_eq!(
            new_port_bindings(10, 20, TransportProtocol::Udp),
            (
                "20/udp".to_string(),
                Some(vec![PortBinding {
                    host_port: Some(10.to_string()),
                    host_ip: None,
                }])
            )
        );
        assert_eq!(
            new_port_bindings(10, 20, TransportProtocol::Sctp),
            (
                "20/sctp".to_string(),
                Some(vec![PortBinding {
                    host_port: Some(10.to_string()),
                    host_ip: None,
                }])
            )
        );
    }

    #[test]
    fn port_mapping_to_port_bindings_single() {
        let port_mapping = PortMapping::Single(60, 600);
        assert_eq!(
            port_mapping_to_port_bindings(&port_mapping, TransportProtocol::Tcp),
            HashMap::from([(
                "600/tcp".to_string(),
                Some(vec![PortBinding {
                    host_port: Some(60.to_string()),
                    host_ip: None,
                }])
            )])
        );
        assert_eq!(
            port_mapping_to_port_bindings(&port_mapping, TransportProtocol::Udp),
            HashMap::from([(
                "600/udp".to_string(),
                Some(vec![PortBinding {
                    host_port: Some(60.to_string()),
                    host_ip: None,
                }])
            )])
        );
        assert_eq!(
            port_mapping_to_port_bindings(&port_mapping, TransportProtocol::Sctp),
            HashMap::from([(
                "600/sctp".to_string(),
                Some(vec![PortBinding {
                    host_port: Some(60.to_string()),
                    host_ip: None,
                }])
            )])
        );
    }

    #[test]
    fn port_mapping_to_port_bindings_range() {
        let port_mapping = PortMapping::Range {
            from: PortRange::try_new(100, 102).unwrap(),
            to: PortRange::try_new(40, 42).unwrap(),
        };
        assert_eq!(
            port_mapping_to_port_bindings(&port_mapping, TransportProtocol::Tcp),
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
        );
        assert_eq!(
            port_mapping_to_port_bindings(&port_mapping, TransportProtocol::Udp),
            HashMap::from([
                (
                    "40/udp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(100.to_string()),
                        host_ip: None,
                    }])
                ),
                (
                    "41/udp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(101.to_string()),
                        host_ip: None,
                    }])
                ),
                (
                    "42/udp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(102.to_string()),
                        host_ip: None,
                    }])
                ),
            ])
        );
        assert_eq!(
            port_mapping_to_port_bindings(&port_mapping, TransportProtocol::Sctp),
            HashMap::from([
                (
                    "40/sctp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(100.to_string()),
                        host_ip: None,
                    }])
                ),
                (
                    "41/sctp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(101.to_string()),
                        host_ip: None,
                    }])
                ),
                (
                    "42/sctp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(102.to_string()),
                        host_ip: None,
                    }])
                ),
            ])
        );
    }

    #[test]
    fn generate_port_bindings_test() {
        let config = InstanceConfig {
            port_mapping: InstancePortMapping {
                tcp: vec![
                    PortMapping::Range {
                        from: PortRange::try_new(100, 102).unwrap(),
                        to: PortRange::try_new(40, 42).unwrap(),
                    },
                    PortMapping::Single(60, 600),
                ],
                udp: vec![
                    PortMapping::Range {
                        from: PortRange::try_new(100, 102).unwrap(),
                        to: PortRange::try_new(40, 42).unwrap(),
                    },
                    PortMapping::Single(60, 600),
                ],
                sctp: vec![
                    PortMapping::Range {
                        from: PortRange::try_new(100, 102).unwrap(),
                        to: PortRange::try_new(40, 42).unwrap(),
                    },
                    PortMapping::Single(60, 600),
                ],
            },
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
                ),
                (
                    "40/udp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(100.to_string()),
                        host_ip: None,
                    }])
                ),
                (
                    "41/udp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(101.to_string()),
                        host_ip: None,
                    }])
                ),
                (
                    "42/udp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(102.to_string()),
                        host_ip: None,
                    }])
                ),
                (
                    "600/udp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(60.to_string()),
                        host_ip: None,
                    }])
                ),
                (
                    "40/sctp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(100.to_string()),
                        host_ip: None,
                    }])
                ),
                (
                    "41/sctp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(101.to_string()),
                        host_ip: None,
                    }])
                ),
                (
                    "42/sctp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(102.to_string()),
                        host_ip: None,
                    }])
                ),
                (
                    "600/sctp".to_string(),
                    Some(vec![PortBinding {
                        host_port: Some(60.to_string()),
                        host_ip: None,
                    }])
                ),
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

    #[test]
    fn try_usb_path_config_to_device_mapping_ok() {
        let bus_path = PathBuf::from(format!("{USB_DEVICE_PATH}010"));
        prepare_path(&bus_path);
        fs::write(bus_path.join("020"), b"test-dev").unwrap();
        let usb_path_config = UsbPathConfig {
            port: "try_usb_path_config_to_device_mapping_ok".to_string(),
            bus_num: 10,
            dev_num: 20,
        };
        assert_eq!(
            DeviceMapping::try_from(&usb_path_config).unwrap(),
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
    fn try_usb_path_config_to_device_mapping_err() {
        let bus_path = PathBuf::from(format!("{USB_DEVICE_PATH}020"));
        prepare_path(&bus_path);
        let usb_path_config = UsbPathConfig {
            port: "try_usb_path_config_to_device_mapping_err".to_string(),
            bus_num: 20,
            dev_num: 20,
        };
        assert!(DeviceMapping::try_from(&usb_path_config).is_err())
    }

    #[test]
    fn generate_usb_device_mappings_ok() {
        prepare_usb_device_test_path("generate_usb_device_mappings_ok_1");
        let bus_path = PathBuf::from(format!("{USB_DEVICE_PATH}111"));
        prepare_path(&bus_path);
        fs::write(bus_path.join("999"), b"test-dev-1").unwrap();
        prepare_usb_device_test_path("generate_usb_device_mappings_ok_2");
        let bus_path = PathBuf::from(format!("{USB_DEVICE_PATH}222"));
        prepare_path(&bus_path);
        fs::write(bus_path.join("888"), b"test-dev-2").unwrap();
        prepare_usb_device_test_path("generate_usb_device_mappings_ok_3");
        let bus_path = PathBuf::from(format!("{USB_DEVICE_PATH}333"));
        prepare_path(&bus_path);
        fs::write(bus_path.join("777"), b"test-dev-3").unwrap();
        let device_mappings = InstanceConfig {
            usb_devices: HashMap::from([
                (
                    "generate_usb_device_mappings_ok_1".to_string(),
                    UsbPathConfig {
                        port: "generate_usb_device_mappings_ok_1".to_string(),
                        bus_num: 111,
                        dev_num: 999,
                    },
                ),
                (
                    "generate_usb_device_mappings_ok_2".to_string(),
                    UsbPathConfig {
                        port: "generate_usb_device_mappings_ok_2".to_string(),
                        bus_num: 222,
                        dev_num: 888,
                    },
                ),
                (
                    "generate_usb_device_mappings_ok_3".to_string(),
                    UsbPathConfig {
                        port: "generate_usb_device_mappings_ok_3".to_string(),
                        bus_num: 333,
                        dev_num: 777,
                    },
                ),
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
        prepare_usb_device_test_path("generate_usb_device_mappings_partly_ok_1");
        let bus_path = PathBuf::from(format!("{USB_DEVICE_PATH}121"));
        prepare_path(&bus_path);
        fs::write(bus_path.join("919"), b"test-dev-1").unwrap();
        prepare_usb_device_test_path("generate_usb_device_mappings_partly_ok_3");
        let bus_path = PathBuf::from(format!("{USB_DEVICE_PATH}343"));
        prepare_path(&bus_path);
        fs::write(bus_path.join("747"), b"test-dev-3").unwrap();
        let device_mappings = InstanceConfig {
            usb_devices: HashMap::from([
                (
                    "generate_usb_device_mappings_partly_ok_1".to_string(),
                    UsbPathConfig {
                        port: "generate_usb_device_mappings_partly_ok_1".to_string(),
                        bus_num: 121,
                        dev_num: 919,
                    },
                ),
                (
                    "generate_usb_device_mappings_partly_ok_2".to_string(),
                    UsbPathConfig {
                        port: "generate_usb_device_mappings_partly_ok_2".to_string(),
                        bus_num: 123,
                        dev_num: 456,
                    },
                ),
                (
                    "generate_usb_device_mappings_partly_ok_3".to_string(),
                    UsbPathConfig {
                        port: "generate_usb_device_mappings_partly_ok_3".to_string(),
                        bus_num: 343,
                        dev_num: 747,
                    },
                ),
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
    fn usb_path_config_from_usb_device_ok() {
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_get_usb_value()
            .withf(|value_name, _| value_name == "devnum")
            .times(1)
            .returning(|_, _| Ok("919".to_string()));
        reader
            .expect_get_usb_value()
            .withf(|value_name, _| value_name == "busnum")
            .times(1)
            .returning(|_, _| Ok("121".to_string()));
        let usb_device = UsbDevice {
            device: String::default(),
            vendor: String::default(),
            port: "usb_path_config_from_usb_device_ok".to_string(),
            pid: 0,
            vid: 0,
        };
        let reader: &dyn UsbDeviceReader = &reader;
        assert_eq!(
            UsbPathConfig::try_from((&usb_device, reader)).unwrap(),
            UsbPathConfig {
                dev_num: 919,
                bus_num: 121,
                port: "usb_path_config_from_usb_device_ok".to_string(),
            }
        );
    }

    #[test]
    fn usb_path_config_from_usb_device_err_devnum() {
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_get_usb_value()
            .withf(|value_name, _| value_name == "devnum")
            .times(1)
            .returning(|_, _| Ok("aaa".to_string()));
        reader
            .expect_get_usb_value()
            .withf(|value_name, _| value_name == "busnum")
            .returning(|_, _| Ok("121".to_string()));
        let usb_device = UsbDevice {
            device: String::default(),
            vendor: String::default(),
            port: "usb_path_config_from_usb_device_err_devnum".to_string(),
            pid: 0,
            vid: 0,
        };
        let reader: &dyn UsbDeviceReader = &reader;
        assert!(UsbPathConfig::try_from((&usb_device, reader)).is_err());
    }

    #[test]
    fn usb_path_config_from_usb_device_err_busnum() {
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_get_usb_value()
            .withf(|value_name, _| value_name == "devnum")
            .times(1)
            .returning(|_, _| Ok("100".to_string()));
        reader
            .expect_get_usb_value()
            .withf(|value_name, _| value_name == "busnum")
            .times(1)
            .returning(|_, _| Ok("xxx".to_string()));
        let usb_device = UsbDevice {
            device: String::default(),
            vendor: String::default(),
            port: "usb_path_config_from_usb_device_err_busnum".to_string(),
            pid: 0,
            vid: 0,
        };
        let reader: &dyn UsbDeviceReader = &reader;
        assert!(UsbPathConfig::try_from((&usb_device, reader)).is_err());
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

    #[test]
    fn instance_port_mapping_clear() {
        let mut instance_port_mapping = InstancePortMapping {
            tcp: vec![PortMapping::Single(1, 2), PortMapping::Single(3, 4)],
            sctp: vec![PortMapping::Single(1, 2)],
            udp: vec![PortMapping::Single(1, 2)],
        };
        instance_port_mapping.clear();
        assert!(instance_port_mapping.tcp.is_empty());
        assert!(instance_port_mapping.sctp.is_empty());
        assert!(instance_port_mapping.udp.is_empty());
    }

    #[test]
    fn instance_port_mapping_is_empty_true() {
        assert!(InstancePortMapping {
            tcp: vec![],
            sctp: vec![],
            udp: vec![],
        }
        .is_empty());
    }

    #[test]
    fn instance_port_mapping_is_empty_false() {
        assert!(!InstancePortMapping {
            tcp: vec![PortMapping::Single(1, 2)],
            sctp: vec![],
            udp: vec![],
        }
        .is_empty());
        assert!(!InstancePortMapping {
            tcp: vec![],
            sctp: vec![PortMapping::Single(1, 2)],
            udp: vec![],
        }
        .is_empty());
        assert!(!InstancePortMapping {
            tcp: vec![],
            sctp: vec![],
            udp: vec![PortMapping::Single(1, 2)],
        }
        .is_empty());
    }

    #[test]
    fn instance_port_mapping_update_port_mapping_vec_replace() {
        let mut vec = vec![
            PortMapping::Single(1, 2),
            PortMapping::Single(10, 20),
            PortMapping::Single(100, 20),
        ];
        assert!(matches!(
            InstancePortMapping::update_port_mapping_vec(&mut vec, PortMapping::Single(10, 100)),
            Ok(true)
        ));
        assert_eq!(
            vec,
            vec![
                PortMapping::Single(1, 2),
                PortMapping::Single(10, 100),
                PortMapping::Single(100, 20),
            ]
        );
    }

    #[test]
    fn instance_port_mapping_update_port_mapping_vec_add() {
        let mut vec = vec![
            PortMapping::Single(1, 2),
            PortMapping::Single(10, 20),
            PortMapping::Single(100, 20),
        ];
        assert!(matches!(
            InstancePortMapping::update_port_mapping_vec(&mut vec, PortMapping::Single(20, 100)),
            Ok(false)
        ));
        assert_eq!(
            vec,
            vec![
                PortMapping::Single(1, 2),
                PortMapping::Single(10, 20),
                PortMapping::Single(100, 20),
                PortMapping::Single(20, 100),
            ]
        );
    }

    #[test]
    fn instance_port_mapping_update_port_mapping_vec_overlap() {
        let original = vec![
            PortMapping::Single(1, 2),
            PortMapping::Range {
                from: PortRange::new(50..=70),
                to: PortRange::new(100..=120),
            },
            PortMapping::Single(100, 20),
        ];
        let mut vec = original.clone();
        assert!(InstancePortMapping::update_port_mapping_vec(
            &mut vec,
            PortMapping::Single(60, 100)
        )
        .is_err());
        assert_eq!(vec, original);
    }

    #[test]
    fn instance_port_mapping_delete_port_mapping_vec_some() {
        let mut vec = vec![
            PortMapping::Single(1, 2),
            PortMapping::Range {
                from: PortRange::new(50..=70),
                to: PortRange::new(100..=120),
            },
            PortMapping::Single(100, 20),
        ];
        assert_eq!(
            InstancePortMapping::delete_port_mapping_vec(&mut vec, 100),
            Some(PortMapping::Single(100, 20))
        );
        assert_eq!(
            vec,
            vec![
                PortMapping::Single(1, 2),
                PortMapping::Range {
                    from: PortRange::new(50..=70),
                    to: PortRange::new(100..=120),
                },
            ]
        );
    }

    #[test]
    fn instance_port_mapping_delete_port_mapping_vec_none() {
        let original = vec![
            PortMapping::Single(1, 2),
            PortMapping::Range {
                from: PortRange::new(50..=70),
                to: PortRange::new(100..=120),
            },
            PortMapping::Single(100, 20),
        ];
        let mut vec = original.clone();
        assert!(InstancePortMapping::delete_port_mapping_vec(&mut vec, 30).is_none());
        assert_eq!(vec, original);
    }

    #[test]
    fn instance_port_mapping_get_port_mapping_vec_some() {
        let vec = vec![
            PortMapping::Single(1, 2),
            PortMapping::Range {
                from: PortRange::new(50..=70),
                to: PortRange::new(100..=120),
            },
            PortMapping::Single(100, 20),
        ];
        assert_eq!(
            InstancePortMapping::get_port_mapping_vec(&vec, 100),
            Some(PortMapping::Single(100, 20))
        );
    }

    #[test]
    fn instance_port_mapping_get_port_mapping_vec_none() {
        let vec = vec![
            PortMapping::Single(1, 2),
            PortMapping::Range {
                from: PortRange::new(50..=70),
                to: PortRange::new(100..=120),
            },
            PortMapping::Single(100, 20),
        ];
        assert!(InstancePortMapping::get_port_mapping_vec(&vec, 30).is_none());
    }

    #[test]
    fn instance_port_mapping_get_port_mapping_range_vec_some() {
        let vec = vec![
            PortMapping::Single(1, 2),
            PortMapping::Range {
                from: PortRange::new(50..=70),
                to: PortRange::new(100..=120),
            },
            PortMapping::Single(100, 20),
        ];
        assert_eq!(
            InstancePortMapping::get_port_mapping_range_vec(&vec, PortRange::new(50..=70)),
            Some(PortMapping::Range {
                from: PortRange::new(50..=70),
                to: PortRange::new(100..=120),
            })
        );
    }

    #[test]
    fn instance_port_mapping_get_port_mapping_range_vec_none() {
        let vec = vec![
            PortMapping::Single(1, 2),
            PortMapping::Range {
                from: PortRange::new(50..=70),
                to: PortRange::new(100..=120),
            },
            PortMapping::Single(100, 20),
        ];
        assert!(
            InstancePortMapping::get_port_mapping_range_vec(&vec, PortRange::new(50..=80))
                .is_none()
        );
    }

    #[test]
    fn instance_port_mapping_delete_port_mapping_range_vec_some() {
        let mut vec = vec![
            PortMapping::Single(1, 2),
            PortMapping::Range {
                from: PortRange::new(50..=70),
                to: PortRange::new(100..=120),
            },
            PortMapping::Single(100, 20),
        ];
        assert_eq!(
            InstancePortMapping::delete_port_mapping_range_vec(&mut vec, PortRange::new(50..=70)),
            Some(PortMapping::Range {
                from: PortRange::new(50..=70),
                to: PortRange::new(100..=120),
            })
        );
        assert_eq!(
            vec,
            vec![PortMapping::Single(1, 2), PortMapping::Single(100, 20)]
        )
    }

    #[test]
    fn instance_port_mapping_delete_port_mapping_range_vec_none() {
        let original = vec![
            PortMapping::Single(1, 2),
            PortMapping::Range {
                from: PortRange::new(50..=70),
                to: PortRange::new(100..=120),
            },
            PortMapping::Single(100, 20),
        ];
        let mut vec = original.clone();
        assert!(InstancePortMapping::delete_port_mapping_range_vec(
            &mut vec,
            PortRange::new(50..=80)
        )
        .is_none());
        assert_eq!(vec, original);
    }

    #[test]
    fn instance_port_mapping_update_port_mapping() {
        let mut instance_port_mapping = InstancePortMapping::default();
        assert!(!instance_port_mapping
            .update_port_mapping(PortMapping::Single(20, 30), TransportProtocol::Tcp)
            .unwrap());
        assert_eq!(instance_port_mapping.tcp, vec![PortMapping::Single(20, 30)]);
        assert!(instance_port_mapping.udp.is_empty());
        assert!(instance_port_mapping.sctp.is_empty());
        assert!(!instance_port_mapping
            .update_port_mapping(PortMapping::Single(40, 50), TransportProtocol::Udp)
            .unwrap());
        assert_eq!(instance_port_mapping.tcp, vec![PortMapping::Single(20, 30)]);
        assert_eq!(instance_port_mapping.udp, vec![PortMapping::Single(40, 50)]);
        assert!(instance_port_mapping.sctp.is_empty());
        assert!(!instance_port_mapping
            .update_port_mapping(PortMapping::Single(99, 77), TransportProtocol::Sctp)
            .unwrap());
        assert_eq!(instance_port_mapping.tcp, vec![PortMapping::Single(20, 30)]);
        assert_eq!(instance_port_mapping.udp, vec![PortMapping::Single(40, 50)]);
        assert_eq!(
            instance_port_mapping.sctp,
            vec![PortMapping::Single(99, 77)]
        );
    }

    #[test]
    fn instance_port_mapping_delete_port_mapping() {
        let mut instance_port_mapping = InstancePortMapping {
            tcp: vec![PortMapping::Single(3, 50)],
            udp: vec![PortMapping::Single(4, 50)],
            sctp: vec![PortMapping::Single(5, 50)],
        };
        assert_eq!(
            instance_port_mapping.delete_port_mapping(3, TransportProtocol::Tcp),
            Some(PortMapping::Single(3, 50))
        );
        assert!(instance_port_mapping.tcp.is_empty());
        assert_eq!(instance_port_mapping.udp, vec![PortMapping::Single(4, 50)]);
        assert_eq!(instance_port_mapping.sctp, vec![PortMapping::Single(5, 50)]);
        assert_eq!(
            instance_port_mapping.delete_port_mapping(4, TransportProtocol::Udp),
            Some(PortMapping::Single(4, 50))
        );
        assert!(instance_port_mapping.tcp.is_empty());
        assert!(instance_port_mapping.udp.is_empty());
        assert_eq!(instance_port_mapping.sctp, vec![PortMapping::Single(5, 50)]);
        assert_eq!(
            instance_port_mapping.delete_port_mapping(5, TransportProtocol::Sctp),
            Some(PortMapping::Single(5, 50))
        );
        assert!(instance_port_mapping.tcp.is_empty());
        assert!(instance_port_mapping.udp.is_empty());
        assert!(instance_port_mapping.sctp.is_empty());
    }

    #[test]
    fn instance_port_mapping_get_port_mapping() {
        let instance_port_mapping = InstancePortMapping {
            tcp: vec![PortMapping::Single(3, 50)],
            udp: vec![PortMapping::Single(4, 50)],
            sctp: vec![PortMapping::Single(5, 50)],
        };
        assert_eq!(
            instance_port_mapping.get_port_mapping(3, TransportProtocol::Tcp),
            Some(PortMapping::Single(3, 50))
        );
        assert_eq!(
            instance_port_mapping.get_port_mapping(4, TransportProtocol::Udp),
            Some(PortMapping::Single(4, 50))
        );
        assert_eq!(
            instance_port_mapping.get_port_mapping(5, TransportProtocol::Sctp),
            Some(PortMapping::Single(5, 50))
        );
    }

    #[test]
    fn instance_port_mapping_get_port_mapping_range() {
        let instance_port_mapping = InstancePortMapping {
            tcp: vec![PortMapping::Range {
                from: PortRange::new(5..=7),
                to: PortRange::new(7..=9),
            }],
            udp: vec![PortMapping::Range {
                from: PortRange::new(2..=3),
                to: PortRange::new(4..=5),
            }],
            sctp: vec![PortMapping::Range {
                from: PortRange::new(10..=15),
                to: PortRange::new(20..=25),
            }],
        };
        assert_eq!(
            instance_port_mapping
                .get_port_mapping_range(PortRange::new(5..=7), TransportProtocol::Tcp),
            Some(PortMapping::Range {
                from: PortRange::new(5..=7),
                to: PortRange::new(7..=9),
            })
        );
        assert_eq!(
            instance_port_mapping
                .get_port_mapping_range(PortRange::new(2..=3), TransportProtocol::Udp),
            Some(PortMapping::Range {
                from: PortRange::new(2..=3),
                to: PortRange::new(4..=5),
            })
        );
        assert_eq!(
            instance_port_mapping
                .get_port_mapping_range(PortRange::new(10..=15), TransportProtocol::Sctp),
            Some(PortMapping::Range {
                from: PortRange::new(10..=15),
                to: PortRange::new(20..=25),
            })
        );
    }

    #[test]
    fn instance_port_mapping_delete_port_mapping_range() {
        let mut instance_port_mapping = InstancePortMapping {
            tcp: vec![PortMapping::Range {
                from: PortRange::new(5..=7),
                to: PortRange::new(7..=9),
            }],
            udp: vec![PortMapping::Range {
                from: PortRange::new(2..=3),
                to: PortRange::new(4..=5),
            }],
            sctp: vec![PortMapping::Range {
                from: PortRange::new(10..=15),
                to: PortRange::new(20..=25),
            }],
        };
        assert_eq!(
            instance_port_mapping
                .delete_port_mapping_range(PortRange::new(5..=7), TransportProtocol::Tcp),
            Some(PortMapping::Range {
                from: PortRange::new(5..=7),
                to: PortRange::new(7..=9),
            })
        );
        assert!(instance_port_mapping.tcp.is_empty());
        assert_eq!(
            instance_port_mapping.udp,
            vec![PortMapping::Range {
                from: PortRange::new(2..=3),
                to: PortRange::new(4..=5),
            }]
        );
        assert_eq!(
            instance_port_mapping.sctp,
            vec![PortMapping::Range {
                from: PortRange::new(10..=15),
                to: PortRange::new(20..=25),
            }]
        );
        assert_eq!(
            instance_port_mapping
                .delete_port_mapping_range(PortRange::new(2..=3), TransportProtocol::Udp),
            Some(PortMapping::Range {
                from: PortRange::new(2..=3),
                to: PortRange::new(4..=5),
            })
        );
        assert!(instance_port_mapping.tcp.is_empty());
        assert!(instance_port_mapping.udp.is_empty());
        assert_eq!(
            instance_port_mapping.sctp,
            vec![PortMapping::Range {
                from: PortRange::new(10..=15),
                to: PortRange::new(20..=25),
            }]
        );
        assert_eq!(
            instance_port_mapping
                .delete_port_mapping_range(PortRange::new(10..=15), TransportProtocol::Sctp),
            Some(PortMapping::Range {
                from: PortRange::new(10..=15),
                to: PortRange::new(20..=25),
            })
        );
        assert!(instance_port_mapping.tcp.is_empty());
        assert!(instance_port_mapping.udp.is_empty());
        assert!(instance_port_mapping.sctp.is_empty());
    }
}
