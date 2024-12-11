use crate::jeweler::gem::manifest::{EnvironmentVariable, PortMapping, VolumeMount};
use crate::jeweler::network::NetworkId;
use crate::jeweler::volume::VolumeId;
use bollard::models::PortBinding;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct MacAddress {
    data: [u8; 8],
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct NetworkAddress {
    ip_addr: IpAddr,
    mac_address: MacAddress,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct InstanceConfig {
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub volume_mounts: HashMap<VolumeId, VolumeMount>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub environment_variables: Vec<EnvironmentVariable>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub port_mapping: Vec<PortMapping>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub network_addresses: HashMap<NetworkId, Option<NetworkAddress>>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::gem::manifest::PortRange;

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
}
