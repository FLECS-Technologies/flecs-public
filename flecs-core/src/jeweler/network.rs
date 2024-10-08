use super::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::net::IpAddr;

pub type NetworkId = String;
pub struct Network {
    // TBD
}
#[derive(Default)]
pub struct NetworkConfig {
    // TBD
}

#[async_trait]
pub trait NetworkDeployment {
    async fn create_network(&self, config: NetworkConfig) -> Result<NetworkId>;
    async fn delete_network(&self, id: NetworkId) -> Result<()>;
    async fn network(&self, id: NetworkId) -> Result<Network>;
    async fn networks(&self) -> Result<HashMap<NetworkId, Network>>;
    async fn connect_network(&self, id: NetworkId, address: IpAddr) -> Result<()>;
    async fn disconnect_network(&self, id: NetworkId, address: IpAddr) -> Result<()>;
}
