use crate::relic::network::{NetworkAdapter, NetworkAdapterReader};
use crate::sorcerer::systemus::Systemus;
use crate::sorcerer::Sorcerer;
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Default)]
pub struct SystemusImpl {}

impl Sorcerer for SystemusImpl {}

#[async_trait]
impl Systemus for SystemusImpl {
    fn read_network_adapters(
        &self,
        network_adapter_reader: &dyn NetworkAdapterReader,
    ) -> anyhow::Result<HashMap<String, NetworkAdapter>> {
        Ok(network_adapter_reader.try_read_network_adapters()?)
    }

    fn read_network_adapter(
        &self,
        network_adapter_reader: &dyn NetworkAdapterReader,
        network_id: &str,
    ) -> anyhow::Result<Option<NetworkAdapter>> {
        Ok(network_adapter_reader
            .try_read_network_adapters()?
            .remove(network_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relic::network::{
        full_network_adapter, minimal_network_adapter, test_adapters, MockNetworkAdapterReader,
    };

    #[tokio::test]
    async fn read_network_adapters_ok() {
        let test_adapters = test_adapters();
        let expected_adapters = test_adapters.clone();
        let systemus = SystemusImpl::default();
        let mut network_adapter_reader = MockNetworkAdapterReader::default();
        network_adapter_reader
            .expect_try_read_network_adapters()
            .once()
            .returning(move || Ok(test_adapters.clone()));
        assert_eq!(
            systemus
                .read_network_adapters(&network_adapter_reader)
                .unwrap(),
            expected_adapters
        );
    }

    #[tokio::test]
    async fn read_network_adapters_err() {
        let systemus = SystemusImpl::default();
        let mut network_adapter_reader = MockNetworkAdapterReader::default();
        network_adapter_reader
            .expect_try_read_network_adapters()
            .once()
            .returning(|| {
                Err(crate::relic::network::Error::InvalidNetwork(
                    "TestError".to_string(),
                ))
            });
        assert!(systemus
            .read_network_adapters(&network_adapter_reader)
            .is_err());
    }

    #[tokio::test]
    async fn read_network_adapter_ok_none() {
        let test_adapters = test_adapters();
        let systemus = SystemusImpl::default();
        let mut network_adapter_reader = MockNetworkAdapterReader::default();
        network_adapter_reader
            .expect_try_read_network_adapters()
            .once()
            .returning(move || Ok(test_adapters.clone()));
        assert!(matches!(
            systemus.read_network_adapter(&network_adapter_reader, "TestAdapterUnknown"),
            Ok(None)
        ));
    }

    #[tokio::test]
    async fn read_network_adapter_ok_some() {
        let test_adapters = test_adapters();
        let systemus = SystemusImpl::default();
        let mut network_adapter_reader = MockNetworkAdapterReader::default();
        network_adapter_reader
            .expect_try_read_network_adapters()
            .times(2)
            .returning(move || Ok(test_adapters.clone()));
        assert_eq!(
            systemus
                .read_network_adapter(&network_adapter_reader, "TestAdapterFull")
                .unwrap(),
            Some(full_network_adapter())
        );
        assert_eq!(
            systemus
                .read_network_adapter(&network_adapter_reader, "TestAdapterMinimal")
                .unwrap(),
            Some(minimal_network_adapter())
        );
    }

    #[tokio::test]
    async fn read_network_adapter_err() {
        let systemus = SystemusImpl::default();
        let mut network_adapter_reader = MockNetworkAdapterReader::default();
        network_adapter_reader
            .expect_try_read_network_adapters()
            .once()
            .returning(|| {
                Err(crate::relic::network::Error::InvalidNetwork(
                    "TestError".to_string(),
                ))
            });
        assert!(systemus
            .read_network_adapter(&network_adapter_reader, "TestAdapterFull")
            .is_err());
    }
}
