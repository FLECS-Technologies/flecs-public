use crate::jeweler::deployment::DeploymentId;
use crate::jeweler::network::{Network, NetworkId};
use crate::quest::Quest;
use crate::sorcerer::deploymento::{Deploymento, GetDeploymentNetworkResult};
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
    ) -> anyhow::Result<GetDeploymentNetworkResult> {
        let result = crate::sorcerer::spell::deployment::query_deployment(
            vault,
            deployment_id.clone(),
            |deployment| async move { deployment.network(network_id).await },
        )
        .await;
        match result {
            None => Ok(GetDeploymentNetworkResult::DeploymentNotFound),
            Some(Err(e)) => Err(e),
            Some(Ok(None)) => Ok(GetDeploymentNetworkResult::NetworkNotFound),
            Some(Ok(Some(network))) => Ok(GetDeploymentNetworkResult::Network(Box::new(network))),
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
                .await
                .unwrap(),
            GetDeploymentNetworkResult::DeploymentNotFound
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
                .await
                .unwrap(),
            GetDeploymentNetworkResult::NetworkNotFound
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
                .await
                .unwrap(),
            GetDeploymentNetworkResult::Network(Box::default())
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
        assert!(DeploymentoImpl
            .get_deployment_network(
                vault,
                "TestDeployment".to_string(),
                "UnknownNetwork".to_string()
            )
            .await
            .is_err());
    }
}
