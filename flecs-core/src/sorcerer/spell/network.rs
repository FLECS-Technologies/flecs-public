use crate::jeweler::deployment::Deployment;
use crate::jeweler::network::{NetworkConfig, NetworkId, NetworkKind};
use crate::quest::Quest;
use crate::relic::network::Ipv4NetworkAccess;
use std::sync::Arc;

const IPVLAN_PREFIX: &str = "flecs-ipvlan_l2";

pub async fn create_vlan_for_network_adapter(
    deployment: Arc<dyn Deployment>,
    adapter_name: String,
    network: Ipv4NetworkAccess,
) -> crate::Result<NetworkId> {
    let vlan_name = format!("{IPVLAN_PREFIX}-{}", adapter_name);
    if deployment.network(vlan_name.clone()).await?.is_none() {
        deployment
            .create_network(
                Quest::new_synced(format!("Create vlan for {adapter_name}")),
                NetworkConfig {
                    kind: NetworkKind::IpvlanL2,
                    name: vlan_name.clone(),
                    cidr_subnet: Some(network.network()),
                    gateway: Some(network.gateway()),
                    parent_adapter: Some(adapter_name),
                    options: None,
                },
            )
            .await?;
    }
    Ok(vlan_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::jeweler::network::Network;
    use crate::relic::network::Ipv4Network;
    use core::default::Default;
    use mockall::predicate;
    use std::net::Ipv4Addr;
    use std::str::FromStr;

    #[tokio::test]
    async fn create_vlan_for_network_adapter_new() {
        let adapter_name = "test_adapter".to_string();
        let expected_vlan_name = "flecs-ipvlan_l2-test_adapter".to_string();
        let gateway = Ipv4Addr::new(36, 84, 123, 25);
        let network = Ipv4Network::from_str("36.84.123.0/24").unwrap();
        let network_access = Ipv4NetworkAccess::try_new(network, gateway).unwrap();
        let expected_network_config = NetworkConfig {
            parent_adapter: Some(adapter_name.clone()),
            name: expected_vlan_name.clone(),
            kind: NetworkKind::IpvlanL2,
            gateway: Some(gateway),
            cidr_subnet: Some(network),
            options: None,
        };
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_create_network()
            .once()
            .with(predicate::always(), predicate::eq(expected_network_config))
            .returning(|_, _| {
                Ok(Network {
                    id: Some("1234abcd".to_string()),
                    ..Default::default()
                })
            });
        deployment
            .expect_network()
            .once()
            .with(predicate::eq(expected_vlan_name.clone()))
            .returning(|_| Ok(None));
        assert_eq!(
            create_vlan_for_network_adapter(Arc::new(deployment), adapter_name, network_access)
                .await
                .unwrap(),
            expected_vlan_name
        );
    }

    #[tokio::test]
    async fn create_vlan_for_network_adapter_exists() {
        let adapter_name = "test_adapter".to_string();
        let expected_vlan_name = "flecs-ipvlan_l2-test_adapter".to_string();
        let gateway = Ipv4Addr::new(36, 84, 123, 25);
        let network = Ipv4Network::from_str("36.84.123.0/24").unwrap();
        let network_access = Ipv4NetworkAccess::try_new(network, gateway).unwrap();
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_network()
            .once()
            .with(predicate::eq(expected_vlan_name.clone()))
            .returning(|_| Ok(Some(Default::default())));
        assert_eq!(
            create_vlan_for_network_adapter(Arc::new(deployment), adapter_name, network_access)
                .await
                .unwrap(),
            expected_vlan_name
        );
    }

    #[tokio::test]
    async fn create_vlan_for_network_adapter_err_create() {
        let adapter_name = "test_adapter".to_string();
        let expected_vlan_name = "flecs-ipvlan_l2-test_adapter".to_string();
        let gateway = Ipv4Addr::new(36, 84, 123, 25);
        let network = Ipv4Network::from_str("36.84.123.0/24").unwrap();
        let network_access = Ipv4NetworkAccess::try_new(network, gateway).unwrap();
        let expected_network_config = NetworkConfig {
            parent_adapter: Some(adapter_name.clone()),
            name: expected_vlan_name.clone(),
            kind: NetworkKind::IpvlanL2,
            gateway: Some(gateway),
            cidr_subnet: Some(network),
            options: None,
        };
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_create_network()
            .once()
            .with(predicate::always(), predicate::eq(expected_network_config))
            .returning(|_, _| Err(anyhow::anyhow!("TestError").into()));
        deployment
            .expect_network()
            .once()
            .with(predicate::eq(expected_vlan_name))
            .returning(|_| Ok(None));
        assert!(create_vlan_for_network_adapter(
            Arc::new(deployment),
            adapter_name,
            network_access
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn create_vlan_for_network_adapter_err_network() {
        let adapter_name = "test_adapter".to_string();
        let expected_vlan_name = "flecs-ipvlan_l2-test_adapter".to_string();
        let gateway = Ipv4Addr::new(36, 84, 123, 25);
        let network = Ipv4Network::from_str("36.84.123.0/24").unwrap();
        let network_access = Ipv4NetworkAccess::try_new(network, gateway).unwrap();
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_network()
            .once()
            .with(predicate::eq(expected_vlan_name))
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        assert!(create_vlan_for_network_adapter(
            Arc::new(deployment),
            adapter_name,
            network_access
        )
        .await
        .is_err());
    }
}
