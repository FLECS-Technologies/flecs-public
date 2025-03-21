mod systemus_impl;

pub use super::Result;
use crate::jeweler::network::NetworkId;
use crate::relic::network::{NetworkAdapter, NetworkAdapterReader};
use crate::sorcerer::Sorcerer;
use crate::vault::Vault;
use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;
pub use systemus_impl::SystemusImpl;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Systemus: Sorcerer {
    async fn reserve_ipv4_address(
        &self,
        vault: Arc<Vault>,
        network_id: NetworkId,
    ) -> Result<ReserveIpv4AddressResult>;
    fn read_network_adapters(
        &self,
        network_adapter_reader: &dyn NetworkAdapterReader,
    ) -> Result<HashMap<String, NetworkAdapter>>;
    fn read_network_adapter(
        &self,
        network_adapter_reader: &dyn NetworkAdapterReader,
        network_id: &str,
    ) -> Result<Option<NetworkAdapter>>;
}

#[cfg(test)]
impl Sorcerer for MockSystemus {}

#[derive(Debug, Eq, PartialEq)]
pub enum ReserveIpv4AddressResult {
    UnknownNetwork(NetworkId),
    NoFreeIpAddress,
    Reserved(Ipv4Addr),
}
