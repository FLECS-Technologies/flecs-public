use crate::jeweler::gem::deployment::Deployment;
use crate::vault::Vault;
use crate::vault::pouch::Pouch;
use crate::vault::pouch::deployment::DeploymentId;
use std::future::Future;
use std::sync::Arc;

const DEFAULT_DEPLOYMENT_ID: &str = "default";

async fn get_deployment(vault: Arc<Vault>, deployment_id: DeploymentId) -> Option<Deployment> {
    let grab = vault.reservation().reserve_deployment_pouch().grab().await;
    let deployments = grab
        .deployment_pouch
        .as_ref()
        .expect("Reservations should never fail");
    if deployment_id == DEFAULT_DEPLOYMENT_ID {
        deployments.default_docker_deployment()
    } else {
        deployments.gems().get(&deployment_id).cloned()
    }
}

pub async fn query_deployment<F, Fut, T>(
    vault: Arc<Vault>,
    deployment_id: DeploymentId,
    with: F,
) -> Option<T>
where
    F: FnOnce(Deployment) -> Fut,
    Fut: Future<Output = T> + Send + 'static,
{
    let deployment = get_deployment(vault, deployment_id).await?;
    Some(with(deployment).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::gem::deployment::docker::tests::MockedDockerDeployment;
    use crate::vault::tests::create_test_vault_with_deployment;

    #[tokio::test]
    async fn get_deployment_default() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("DefaultDeploymentId".to_string());
        deployment.expect_is_default().return_const(true);
        let vault = create_test_vault_with_deployment(Deployment::Docker(Arc::new(deployment)));
        assert_eq!(
            get_deployment(vault, DEFAULT_DEPLOYMENT_ID.to_string())
                .await
                .unwrap()
                .id(),
            "DefaultDeploymentId"
        );
    }

    #[tokio::test]
    async fn get_deployment_none() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("SomeDeployment".to_string());
        deployment.expect_is_default().return_const(true);
        let vault = create_test_vault_with_deployment(Deployment::Docker(Arc::new(deployment)));
        assert!(
            get_deployment(vault, "OtherDeployment".to_string())
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn get_deployment_some() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("SomeDeployment".to_string());
        deployment.expect_is_default().return_const(true);
        let vault = create_test_vault_with_deployment(Deployment::Docker(Arc::new(deployment)));
        assert_eq!(
            get_deployment(vault, "SomeDeployment".to_string())
                .await
                .unwrap()
                .id(),
            "SomeDeployment"
        );
    }

    #[tokio::test]
    async fn query_deployment_some() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("SomeDeployment".to_string());
        deployment.expect_is_default().return_const(true);
        let vault = create_test_vault_with_deployment(Deployment::Docker(Arc::new(deployment)));
        assert_eq!(
            query_deployment(
                vault,
                "SomeDeployment".to_string(),
                |deployment| async move { deployment.id().clone() }
            )
            .await
            .unwrap(),
            "SomeDeployment"
        );
    }

    #[tokio::test]
    async fn query_deployment_none() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("SomeDeployment".to_string());
        deployment.expect_is_default().return_const(true);
        let vault = create_test_vault_with_deployment(Deployment::Docker(Arc::new(deployment)));
        assert!(
            query_deployment(
                vault,
                "OtherDeployment".to_string(),
                |deployment| async move { deployment.id().clone() }
            )
            .await
            .is_none()
        );
    }
}
