use crate::jeweler::deployment::DeploymentId;
use crate::jeweler::network::{Network, NetworkConfig, NetworkId};
use crate::quest::Quest;
use crate::relic::network::Ipv4NetworkAccess;
use crate::sorcerer::deploymento::{
    CreateNetworkError, Deploymento, GetDeploymentNetworkError, ReserveIpv4AddressError,
};
use crate::sorcerer::Sorcerer;
use crate::vault::Vault;
use async_trait::async_trait;
use std::net::Ipv4Addr;
use std::sync::Arc;

#[derive(Default)]
pub struct DeploymentoImpl;

#[async_trait]
impl Deploymento for DeploymentoImpl {
    async fn create_network(
        &self,
        vault: Arc<Vault>,
        deployment_id: DeploymentId,
        config: NetworkConfig,
    ) -> anyhow::Result<Network, CreateNetworkError> {
        let deployment = deployment_id.clone();
        match crate::sorcerer::spell::deployment::query_deployment(
            vault,
            deployment_id.clone(),
            |deployment| async move {
                deployment
                    .create_network(
                        Quest::new_synced(format!(
                            "Create network {} at {deployment_id}",
                            config.name
                        )),
                        config,
                    )
                    .await
            },
        )
        .await
        {
            None => Err(CreateNetworkError::DeploymentNotFound(deployment)),
            Some(network) => Ok(network?),
        }
    }

    async fn get_deployment_networks(
        &self,
        vault: Arc<Vault>,
        deployment_id: DeploymentId,
    ) -> anyhow::Result<Option<Vec<Network>>> {
        let result = crate::sorcerer::spell::deployment::query_deployment(
            vault,
            deployment_id.clone(),
            |deployment| async move {
                deployment
                    .networks(Quest::new_synced(format!(
                        "Get networks of {deployment_id}"
                    )))
                    .await
            },
        )
        .await;
        match result {
            None => Ok(None),
            Some(Err(e)) => Err(e),
            Some(Ok(networks)) => Ok(Some(networks)),
        }
    }

    async fn get_deployment_network(
        &self,
        vault: Arc<Vault>,
        deployment_id: DeploymentId,
        network_id: NetworkId,
    ) -> anyhow::Result<Network, GetDeploymentNetworkError> {
        let result = {
            let network_id = network_id.clone();
            crate::sorcerer::spell::deployment::query_deployment(
                vault,
                deployment_id.clone(),
                |deployment| async move { deployment.network(network_id).await },
            )
            .await
        };
        match result {
            None => Err(GetDeploymentNetworkError::DeploymentNotFound(deployment_id)),
            Some(Err(e)) => Err(GetDeploymentNetworkError::Other {
                network_id,
                reason: e.to_string(),
            }),
            Some(Ok(None)) => Err(GetDeploymentNetworkError::NetworkNotFound(network_id)),
            Some(Ok(Some(network))) => Ok(network),
        }
    }

    async fn reserve_ipv4_address(
        &self,
        vault: Arc<Vault>,
        deployment_id: DeploymentId,
        network_id: NetworkId,
    ) -> anyhow::Result<Ipv4Addr, ReserveIpv4AddressError> {
        let network = self
            .get_deployment_network(vault.clone(), deployment_id, network_id.clone())
            .await?;
        let network = match Ipv4NetworkAccess::try_from(network) {
            Ok(network) => network,
            Err(e) => {
                return Err(ReserveIpv4AddressError::Other {
                    network_id,
                    reason: e.to_string(),
                })
            }
        };
        match crate::sorcerer::spell::instance::make_ipv4_reservation(vault, network).await {
            None => Err(ReserveIpv4AddressError::NoFreeIpAddress),
            Some(address) => Ok(address),
        }
    }
}

impl Sorcerer for DeploymentoImpl {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::gem::deployment::docker::tests::MockedDockerDeployment;
    use crate::jeweler::gem::deployment::Deployment;
    use crate::jeweler::network::NetworkKind;
    use crate::vault::tests::{create_empty_test_vault, create_test_vault_with_deployment};
    use mockall::predicate;

    #[tokio::test]
    async fn get_deployment_networks_unknown_deployment() {
        let vault = create_empty_test_vault();
        assert!(DeploymentoImpl
            .get_deployment_networks(vault, "UnknownDeployment".to_string())
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn get_deployment_networks_err() {
        let mut deployment = MockedDockerDeployment::new();
        deployment.expect_is_default().return_const(true);
        deployment
            .expect_networks()
            .once()
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        deployment
            .expect_id()
            .return_const("TestDeployment".to_string());
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_test_vault_with_deployment(deployment);
        assert!(DeploymentoImpl
            .get_deployment_networks(vault, "TestDeployment".to_string())
            .await
            .is_err());
    }

    #[tokio::test]
    async fn get_deployment_networks_ok() {
        let mut deployment = MockedDockerDeployment::new();
        deployment.expect_is_default().return_const(true);
        deployment
            .expect_networks()
            .once()
            .returning(|_| Ok(vec![Network::default(), Network::default()]));
        deployment
            .expect_id()
            .return_const("TestDeployment".to_string());
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_test_vault_with_deployment(deployment);
        assert_eq!(
            DeploymentoImpl
                .get_deployment_networks(vault, "TestDeployment".to_string())
                .await
                .unwrap(),
            Some(vec![Network::default(), Network::default()])
        );
    }

    #[tokio::test]
    async fn get_deployment_network_unknown_deployment() {
        let vault = create_empty_test_vault();
        assert_eq!(
            DeploymentoImpl
                .get_deployment_network(
                    vault,
                    "UnknownDeployment".to_string(),
                    "TestNetwork".to_string()
                )
                .await,
            Err(GetDeploymentNetworkError::DeploymentNotFound(
                "UnknownDeployment".to_string()
            ))
        );
    }

    #[tokio::test]
    async fn get_deployment_network_unknown_network() {
        let mut deployment = MockedDockerDeployment::new();
        deployment.expect_is_default().return_const(true);
        deployment
            .expect_network()
            .once()
            .with(predicate::eq("UnknownNetwork".to_string()))
            .returning(|_| Ok(None));
        deployment
            .expect_id()
            .return_const("TestDeployment".to_string());
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_test_vault_with_deployment(deployment);
        assert_eq!(
            DeploymentoImpl
                .get_deployment_network(
                    vault,
                    "TestDeployment".to_string(),
                    "UnknownNetwork".to_string()
                )
                .await,
            Err(GetDeploymentNetworkError::NetworkNotFound(
                "UnknownNetwork".to_string()
            ))
        );
    }

    #[tokio::test]
    async fn get_deployment_network_ok() {
        let mut deployment = MockedDockerDeployment::new();
        deployment.expect_is_default().return_const(true);
        deployment
            .expect_network()
            .once()
            .with(predicate::eq("TestNetwork".to_string()))
            .returning(|_| Ok(Some(Network::default())));
        deployment
            .expect_id()
            .return_const("TestDeployment".to_string());
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_test_vault_with_deployment(deployment);
        assert_eq!(
            DeploymentoImpl
                .get_deployment_network(
                    vault,
                    "TestDeployment".to_string(),
                    "TestNetwork".to_string()
                )
                .await,
            Ok(Network::default())
        );
    }

    #[tokio::test]
    async fn get_deployment_network_err() {
        let mut deployment = MockedDockerDeployment::new();
        deployment.expect_is_default().return_const(true);
        deployment
            .expect_network()
            .once()
            .with(predicate::eq("UnknownNetwork".to_string()))
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        deployment
            .expect_id()
            .return_const("TestDeployment".to_string());
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_test_vault_with_deployment(deployment);
        assert!(matches!(
            DeploymentoImpl
                .get_deployment_network(
                    vault,
                    "TestDeployment".to_string(),
                    "UnknownNetwork".to_string()
                )
                .await,
            Err(GetDeploymentNetworkError::Other { .. })
        ));
    }

    #[tokio::test]
    async fn reserve_ipv4_address_err_unknown_deployment() {
        assert_eq!(
            DeploymentoImpl
                .reserve_ipv4_address(
                    create_empty_test_vault(),
                    "MockedDeployment".to_string(),
                    String::new()
                )
                .await,
            Err(ReserveIpv4AddressError::DeploymentNotFound(
                "MockedDeployment".to_string()
            ))
        )
    }

    #[tokio::test]
    async fn reserve_ipv4_address_err_network() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_network()
            .once()
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        deployment.expect_is_default().return_const(true);
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_test_vault_with_deployment(deployment);
        assert_eq!(
            DeploymentoImpl
                .reserve_ipv4_address(
                    vault,
                    "MockedDeployment".to_string(),
                    "TestNetwork".to_string()
                )
                .await,
            Err(ReserveIpv4AddressError::Other {
                network_id: "TestNetwork".to_string(),
                reason: anyhow::anyhow!("TestError").to_string()
            })
        )
    }

    #[tokio::test]
    async fn reserve_ipv4_address_ok_unknown_network() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment.expect_network().once().returning(|_| Ok(None));
        deployment.expect_is_default().return_const(true);
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_test_vault_with_deployment(deployment);
        assert_eq!(
            DeploymentoImpl
                .reserve_ipv4_address(
                    vault,
                    "MockedDeployment".to_string(),
                    "TestNetwork".to_string()
                )
                .await,
            Err(ReserveIpv4AddressError::NetworkNotFound(
                "TestNetwork".to_string()
            ))
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
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_network()
            .times(6)
            .returning(move |_| Ok(Some(bollard_network.clone())));
        deployment.expect_is_default().return_const(true);
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_test_vault_with_deployment(deployment);
        assert_eq!(
            DeploymentoImpl
                .reserve_ipv4_address(
                    vault.clone(),
                    "MockedDeployment".to_string(),
                    "TestNetwork".to_string()
                )
                .await,
            Ok(Ipv4Addr::new(90, 70, 23, 2))
        );
        assert_eq!(
            DeploymentoImpl
                .reserve_ipv4_address(
                    vault.clone(),
                    "MockedDeployment".to_string(),
                    "TestNetwork".to_string()
                )
                .await,
            Ok(Ipv4Addr::new(90, 70, 23, 3))
        );
        assert_eq!(
            DeploymentoImpl
                .reserve_ipv4_address(
                    vault.clone(),
                    "MockedDeployment".to_string(),
                    "TestNetwork".to_string()
                )
                .await,
            Ok(Ipv4Addr::new(90, 70, 23, 4))
        );
        assert_eq!(
            DeploymentoImpl
                .reserve_ipv4_address(
                    vault.clone(),
                    "MockedDeployment".to_string(),
                    "TestNetwork".to_string()
                )
                .await,
            Ok(Ipv4Addr::new(90, 70, 23, 5))
        );
        assert_eq!(
            DeploymentoImpl
                .reserve_ipv4_address(
                    vault.clone(),
                    "MockedDeployment".to_string(),
                    "TestNetwork".to_string()
                )
                .await,
            Ok(Ipv4Addr::new(90, 70, 23, 6))
        );
        assert_eq!(
            DeploymentoImpl
                .reserve_ipv4_address(
                    vault.clone(),
                    "MockedDeployment".to_string(),
                    "TestNetwork".to_string()
                )
                .await,
            Err(ReserveIpv4AddressError::NoFreeIpAddress),
        );
    }

    #[tokio::test]
    async fn create_network_ok() {
        let config = NetworkConfig {
            kind: NetworkKind::Bridge,
            name: "TestNetwork".to_string(),
            cidr_subnet: None,
            gateway: None,
            parent_adapter: None,
            options: None,
        };
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment.expect_is_default().return_const(true);
        deployment
            .expect_create_network()
            .once()
            .with(predicate::always(), predicate::eq(config.clone()))
            .returning(|_, _| Ok(Network::default()));
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_test_vault_with_deployment(deployment);
        assert_eq!(
            DeploymentoImpl
                .create_network(vault, "MockedDeployment".to_string(), config)
                .await,
            Ok(Network::default())
        );
    }

    #[tokio::test]
    async fn create_network_err_deployment_err() {
        let config = NetworkConfig {
            kind: NetworkKind::Bridge,
            name: "TestNetwork".to_string(),
            cidr_subnet: None,
            gateway: None,
            parent_adapter: None,
            options: None,
        };
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment.expect_is_default().return_const(true);
        deployment
            .expect_create_network()
            .once()
            .with(predicate::always(), predicate::eq(config.clone()))
            .returning(|_, _| {
                Err(crate::jeweler::network::CreateNetworkError::Other(
                    "TestError".to_string(),
                ))
            });
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_test_vault_with_deployment(deployment);
        assert_eq!(
            DeploymentoImpl
                .create_network(vault, "MockedDeployment".to_string(), config)
                .await,
            Err(CreateNetworkError::Deployment(
                crate::jeweler::network::CreateNetworkError::Other("TestError".to_string())
            ))
        );
    }

    #[tokio::test]
    async fn create_network_err_unknown_deployment() {
        let config = NetworkConfig {
            kind: NetworkKind::Bridge,
            name: "TestNetwork".to_string(),
            cidr_subnet: None,
            gateway: None,
            parent_adapter: None,
            options: None,
        };
        let vault = create_empty_test_vault();
        assert_eq!(
            DeploymentoImpl
                .create_network(vault, "MockedDeployment".to_string(), config)
                .await,
            Err(CreateNetworkError::DeploymentNotFound(
                "MockedDeployment".to_string()
            ))
        );
    }
}
