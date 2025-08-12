use super::Result;
use crate::lore::NetworkLoreRef;
use crate::quest::SyncQuest;
use anyhow::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::net::Ipv4Addr;

pub type NetworkId = String;
pub type Network = bollard::models::Network;
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub enum NetworkKind {
    #[default]
    None,
    Internal,
    Bridge,
    MACVLAN,
    IpvlanL2,
    IpvlanL3,
    Unknown,
}

impl Display for NetworkKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NetworkKind::None => {
                    "none"
                }
                NetworkKind::Internal => {
                    "internal"
                }
                NetworkKind::Bridge => {
                    "bridge"
                }
                NetworkKind::MACVLAN => {
                    "macvlan"
                }
                NetworkKind::IpvlanL2 => {
                    "ipvlan_l2"
                }
                NetworkKind::IpvlanL3 => {
                    "ipvlan_l3"
                }
                NetworkKind::Unknown => {
                    "unknown"
                }
            }
        )
    }
}

impl From<&str> for NetworkKind {
    fn from(value: &str) -> Self {
        match value {
            "none" | "None" => NetworkKind::None,
            "internal" | "Internal" => NetworkKind::Internal,
            "bridge" | "Bridge" => NetworkKind::Bridge,
            "macvlan" | "MACVLAN" => NetworkKind::MACVLAN,
            "ipvlan_l2" | "IpvlanL2" => NetworkKind::IpvlanL2,
            "ipvlan_l3" | "IpvlanL3" => NetworkKind::IpvlanL3,
            _ => NetworkKind::Unknown,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NetworkConfig {
    pub kind: NetworkKind,
    pub name: String,
    pub cidr_subnet: Option<crate::relic::network::Ipv4Network>,
    pub gateway: Option<Ipv4Addr>,
    pub parent_adapter: Option<String>,
    pub options: Option<HashMap<String, String>>,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum CreateNetworkError {
    #[error("Network config invalid at {location}: {reason}")]
    NetworkConfigInvalid { location: String, reason: String },
    #[error("Network already exists")]
    ExactNetworkExists(Network),
    #[error("Network with same name but different config already exists")]
    DifferentNetworkExists(Network),
    #[error("Failed to create network: {0}")]
    Other(String),
}

impl From<Error> for CreateNetworkError {
    fn from(value: Error) -> Self {
        Self::Other(value.to_string())
    }
}

impl From<crate::relic::network::Error> for CreateNetworkError {
    fn from(value: crate::relic::network::Error) -> Self {
        Self::Other(value.to_string())
    }
}

// TODO: Move to DockerDeployment
#[async_trait]
pub trait NetworkDeployment {
    async fn create_network(
        &self,
        quest: SyncQuest,
        config: NetworkConfig,
    ) -> Result<Network, CreateNetworkError>;
    async fn default_network(&self, lore: NetworkLoreRef) -> Result<Network, CreateNetworkError>;
    async fn delete_network(&self, id: NetworkId) -> Result<()>;
    async fn network(&self, id: NetworkId) -> Result<Option<Network>>;
    async fn networks(&self, quest: SyncQuest) -> Result<Vec<Network>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_kind_from_str() {
        assert_eq!(NetworkKind::from("none"), NetworkKind::None);
        assert_eq!(NetworkKind::from("None"), NetworkKind::None);
        assert_eq!(NetworkKind::from("internal"), NetworkKind::Internal);
        assert_eq!(NetworkKind::from("Internal"), NetworkKind::Internal);
        assert_eq!(NetworkKind::from("bridge"), NetworkKind::Bridge);
        assert_eq!(NetworkKind::from("Bridge"), NetworkKind::Bridge);
        assert_eq!(NetworkKind::from("macvlan"), NetworkKind::MACVLAN);
        assert_eq!(NetworkKind::from("MACVLAN"), NetworkKind::MACVLAN);
        assert_eq!(NetworkKind::from("ipvlan_l2"), NetworkKind::IpvlanL2);
        assert_eq!(NetworkKind::from("IpvlanL2"), NetworkKind::IpvlanL2);
        assert_eq!(NetworkKind::from("ipvlan_l3"), NetworkKind::IpvlanL3);
        assert_eq!(NetworkKind::from("IpvlanL3"), NetworkKind::IpvlanL3);
        assert_eq!(NetworkKind::from("08ih208h5"), NetworkKind::Unknown);
    }
}
