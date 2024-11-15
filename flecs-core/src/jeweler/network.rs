use super::Result;
use crate::quest::SyncQuest;
use async_trait::async_trait;
use std::fmt::{Display, Formatter};
use std::net::Ipv4Addr;

pub type NetworkId = String;
pub type Network = bollard::models::Network;
#[derive(Copy, Clone, Eq, PartialEq, Default)]
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

#[derive(Default)]
pub struct NetworkConfig {
    pub kind: NetworkKind,
    pub name: String,
    pub cidr_subnet: Option<crate::relic::network::Ipv4Network>,
    pub gateway: Option<Ipv4Addr>,
    pub parent_adapter: Option<String>,
}

#[async_trait]
pub trait NetworkDeployment {
    async fn create_network(&self, quest: SyncQuest, config: NetworkConfig) -> Result<NetworkId>;
    async fn delete_network(&self, id: NetworkId) -> Result<()>;
    async fn network(&self, id: NetworkId) -> Result<Network>;
    async fn networks(&self, quest: SyncQuest) -> Result<Vec<Network>>;
    async fn connect_network(
        &self,
        quest: SyncQuest,
        id: NetworkId,
        address: Ipv4Addr,
        container: &str,
    ) -> Result<()>;
    async fn disconnect_network(
        &self,
        quest: SyncQuest,
        id: NetworkId,
        container: &str,
    ) -> Result<()>;
}
