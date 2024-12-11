use crate::jeweler::gem::manifest::{EnvironmentVariable, PortMapping, VolumeMount};
use crate::jeweler::network::NetworkId;
use crate::jeweler::volume::VolumeId;
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
