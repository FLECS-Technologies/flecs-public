mod systemus_impl;

pub use super::Result;
use crate::relic::network::{NetworkAdapter, NetworkAdapterReader};
use crate::sorcerer::Sorcerer;
use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use std::collections::HashMap;
pub use systemus_impl::SystemusImpl;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Systemus: Sorcerer {
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
