use crate::jeweler::network::NetworkId;
use crate::relic::network::Ipv4NetworkAccess;
use crate::sorcerer::systemus::{ReserveIpv4AddressResult, Systemus};
use crate::sorcerer::Sorcerer;
use crate::vault::pouch::Pouch;
use crate::vault::Vault;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Default)]
pub struct SystemusImpl {}

impl Sorcerer for SystemusImpl {}

#[async_trait]
impl Systemus for SystemusImpl {
    async fn reserve_ipv4_address(
        &self,
        vault: Arc<Vault>,
        network_id: NetworkId,
    ) -> anyhow::Result<ReserveIpv4AddressResult> {
        // TODO: Which deployment to query?
        let deployment = {
            let grab = vault.reservation().reserve_deployment_pouch().grab().await;
            let Some(deployment) = grab
                .deployment_pouch
                .as_ref()
                .expect("Vault reservations should never fail")
                .gems()
                .values()
                .next()
                .cloned()
            else {
                anyhow::bail!("No deployment available");
            };
            deployment
        };
        let Some(network) = deployment.network(network_id.clone()).await? else {
            return Ok(ReserveIpv4AddressResult::UnknownNetwork(network_id));
        };
        match crate::sorcerer::spell::instance::make_ipv4_reservation(
            vault,
            Ipv4NetworkAccess::try_from(network)?,
        )
        .await
        {
            None => Ok(ReserveIpv4AddressResult::NoFreeIpAddress),
            Some(address) => Ok(ReserveIpv4AddressResult::Reserved(address)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::vault::tests::{create_empty_test_vault, create_test_vault_with_deployment};
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn reserve_ipv4_address_err_no_deployment() {
        assert!(SystemusImpl::default()
            .reserve_ipv4_address(create_empty_test_vault(), String::new())
            .await
            .is_err())
    }

    #[tokio::test]
    async fn reserve_ipv4_address_err_network() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_network()
            .once()
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let vault = create_test_vault_with_deployment(Arc::new(deployment));
        assert!(SystemusImpl::default()
            .reserve_ipv4_address(vault, String::new())
            .await
            .is_err())
    }

    #[tokio::test]
    async fn reserve_ipv4_address_ok_unknown_network() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment.expect_network().once().returning(|_| Ok(None));
        let vault = create_test_vault_with_deployment(Arc::new(deployment));
        assert_eq!(
            SystemusImpl::default()
                .reserve_ipv4_address(vault, "TestNetwork".to_string())
                .await
                .unwrap(),
            ReserveIpv4AddressResult::UnknownNetwork("TestNetwork".to_string())
        )
    }

    #[tokio::test]
    async fn reserve_ipv4_address_ok() {
        let bollard_network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![bollard::models::IpamConfig {
                    subnet: Some("90.70.23.0/29".to_string()),
                    gateway: Some("90.70.23.1".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_network()
            .times(6)
            .returning(move |_| Ok(Some(bollard_network.clone())));
        let vault = create_test_vault_with_deployment(Arc::new(deployment));
        assert_eq!(
            SystemusImpl::default()
                .reserve_ipv4_address(vault.clone(), "TestNetwork".to_string())
                .await
                .unwrap(),
            ReserveIpv4AddressResult::Reserved(Ipv4Addr::new(90, 70, 23, 2))
        );
        assert_eq!(
            SystemusImpl::default()
                .reserve_ipv4_address(vault.clone(), "TestNetwork".to_string())
                .await
                .unwrap(),
            ReserveIpv4AddressResult::Reserved(Ipv4Addr::new(90, 70, 23, 3))
        );
        assert_eq!(
            SystemusImpl::default()
                .reserve_ipv4_address(vault.clone(), "TestNetwork".to_string())
                .await
                .unwrap(),
            ReserveIpv4AddressResult::Reserved(Ipv4Addr::new(90, 70, 23, 4))
        );
        assert_eq!(
            SystemusImpl::default()
                .reserve_ipv4_address(vault.clone(), "TestNetwork".to_string())
                .await
                .unwrap(),
            ReserveIpv4AddressResult::Reserved(Ipv4Addr::new(90, 70, 23, 5))
        );
        assert_eq!(
            SystemusImpl::default()
                .reserve_ipv4_address(vault.clone(), "TestNetwork".to_string())
                .await
                .unwrap(),
            ReserveIpv4AddressResult::Reserved(Ipv4Addr::new(90, 70, 23, 6))
        );
        assert_eq!(
            SystemusImpl::default()
                .reserve_ipv4_address(vault.clone(), "TestNetwork".to_string())
                .await
                .unwrap(),
            ReserveIpv4AddressResult::NoFreeIpAddress
        );
    }
}
