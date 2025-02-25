pub use super::Result;
use crate::jeweler::network::NetworkId;
use crate::relic::network::Ipv4NetworkAccess;
use crate::vault::pouch::Pouch;
use crate::vault::Vault;
use std::net::Ipv4Addr;
use std::sync::Arc;

#[derive(Debug, Eq, PartialEq)]
pub enum ReserveIpv4AddressResult {
    UnknownNetwork(NetworkId),
    NoFreeIpAddress,
    Reserved(Ipv4Addr),
}

pub async fn reserve_ipv4_address(
    vault: Arc<Vault>,
    network_id: NetworkId,
) -> Result<ReserveIpv4AddressResult> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::vault::VaultConfig;

    async fn test_vault(deployment: MockedDeployment) -> Arc<Vault> {
        let vault = Vault::new(VaultConfig::default());
        vault
            .reservation()
            .reserve_deployment_pouch_mut()
            .grab()
            .await
            .deployment_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut()
            .insert("TestDeployment".to_string(), Arc::new(deployment));
        Arc::new(vault)
    }

    #[tokio::test]
    async fn reserve_ipv4_address_err_no_deployment() {
        assert!(
            reserve_ipv4_address(Arc::new(Vault::new(VaultConfig::default())), String::new())
                .await
                .is_err()
        )
    }

    #[tokio::test]
    async fn reserve_ipv4_address_err_network() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_network()
            .once()
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let vault = test_vault(deployment).await;
        assert!(reserve_ipv4_address(vault, String::new()).await.is_err())
    }

    #[tokio::test]
    async fn reserve_ipv4_address_ok_unknown_network() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_network().once().returning(|_| Ok(None));
        let vault = test_vault(deployment).await;
        assert_eq!(
            reserve_ipv4_address(vault, "TestNetwork".to_string())
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
        deployment
            .expect_network()
            .times(6)
            .returning(move |_| Ok(Some(bollard_network.clone())));
        let vault = test_vault(deployment).await;
        assert_eq!(
            reserve_ipv4_address(vault.clone(), "TestNetwork".to_string())
                .await
                .unwrap(),
            ReserveIpv4AddressResult::Reserved(Ipv4Addr::new(90, 70, 23, 2))
        );
        assert_eq!(
            reserve_ipv4_address(vault.clone(), "TestNetwork".to_string())
                .await
                .unwrap(),
            ReserveIpv4AddressResult::Reserved(Ipv4Addr::new(90, 70, 23, 3))
        );
        assert_eq!(
            reserve_ipv4_address(vault.clone(), "TestNetwork".to_string())
                .await
                .unwrap(),
            ReserveIpv4AddressResult::Reserved(Ipv4Addr::new(90, 70, 23, 4))
        );
        assert_eq!(
            reserve_ipv4_address(vault.clone(), "TestNetwork".to_string())
                .await
                .unwrap(),
            ReserveIpv4AddressResult::Reserved(Ipv4Addr::new(90, 70, 23, 5))
        );
        assert_eq!(
            reserve_ipv4_address(vault.clone(), "TestNetwork".to_string())
                .await
                .unwrap(),
            ReserveIpv4AddressResult::Reserved(Ipv4Addr::new(90, 70, 23, 6))
        );
        assert_eq!(
            reserve_ipv4_address(vault.clone(), "TestNetwork".to_string())
                .await
                .unwrap(),
            ReserveIpv4AddressResult::NoFreeIpAddress
        );
    }
}
