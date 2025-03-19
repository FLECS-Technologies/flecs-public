use crate::jeweler::deployment::DeploymentId;
use crate::jeweler::network::{Network, NetworkId};
use crate::quest::Quest;
use crate::sorcerer::deploymento::{Deploymento, GetDeploymentNetworkError};
use crate::sorcerer::Sorcerer;
use crate::vault::Vault;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Default)]
pub struct DeploymentoImpl;

#[async_trait]
impl Deploymento for DeploymentoImpl {
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
}

impl Sorcerer for DeploymentoImpl {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::deployment::tests::MockedDeployment;
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
        let mut deployment = MockedDeployment::new();
        deployment.expect_is_default().return_const(true);
        deployment
            .expect_networks()
            .once()
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        deployment.expect_id().return_const("TestDeployment");
        let vault = create_test_vault_with_deployment(Arc::new(deployment));
        assert!(DeploymentoImpl
            .get_deployment_networks(vault, "TestDeployment".to_string())
            .await
            .is_err());
    }

    #[tokio::test]
    async fn get_deployment_networks_ok() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_is_default().return_const(true);
        deployment
            .expect_networks()
            .once()
            .returning(|_| Ok(vec![Network::default(), Network::default()]));
        deployment.expect_id().return_const("TestDeployment");
        let vault = create_test_vault_with_deployment(Arc::new(deployment));
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
        let mut deployment = MockedDeployment::new();
        deployment.expect_is_default().return_const(true);
        deployment
            .expect_network()
            .once()
            .with(predicate::eq("UnknownNetwork".to_string()))
            .returning(|_| Ok(None));
        deployment.expect_id().return_const("TestDeployment");
        let vault = create_test_vault_with_deployment(Arc::new(deployment));
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
        let mut deployment = MockedDeployment::new();
        deployment.expect_is_default().return_const(true);
        deployment
            .expect_network()
            .once()
            .with(predicate::eq("TestNetwork".to_string()))
            .returning(|_| Ok(Some(Network::default())));
        deployment.expect_id().return_const("TestDeployment");
        let vault = create_test_vault_with_deployment(Arc::new(deployment));
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
        let mut deployment = MockedDeployment::new();
        deployment.expect_is_default().return_const(true);
        deployment
            .expect_network()
            .once()
            .with(predicate::eq("UnknownNetwork".to_string()))
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        deployment.expect_id().return_const("TestDeployment");
        let vault = create_test_vault_with_deployment(Arc::new(deployment));
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
}
