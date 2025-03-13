pub mod network_id;

use crate::fsm::server_impl::api::v2::deployments::deployment_id::networks::network_id::try_model_from_network;
use crate::jeweler::network::Network;
use crate::sorcerer::deploymento::Deploymento;
use crate::vault::Vault;
use flecsd_axum_server::apis::deployments::DeploymentsDeploymentIdNetworksGetResponse as GetResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    AdditionalInfo, DeploymentsDeploymentIdNetworksGetPathParams as GetPathParams,
};
use std::sync::Arc;
use tracing::error;

pub async fn get<T: Deploymento>(
    vault: Arc<Vault>,
    deploymento: Arc<T>,
    path_params: GetPathParams,
) -> GetResponse {
    match deploymento
        .get_deployment_networks(vault, path_params.deployment_id)
        .await
    {
        Ok(Some(networks)) => GetResponse::Status200_Success(create_networks_model(networks)),
        Ok(None) => GetResponse::Status404_NoDeploymentWithThisDeployment,
        Err(e) => GetResponse::Status500_InternalServerError(AdditionalInfo::new(e.to_string())),
    }
}

fn create_networks_model(networks: Vec<Network>) -> Vec<models::DeploymentNetwork> {
    let mut networks_model = Vec::new();
    for network in networks {
        match try_model_from_network(network) {
            Ok(network) => networks_model.push(network),
            Err(e) => error!("Failed to parse bollard network: {e}"),
        }
    }
    networks_model
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorcerer::deploymento::MockDeploymento;
    use crate::vault::tests::create_empty_test_vault;
    use mockall::predicate;

    #[tokio::test]
    async fn get_200() {
        let vault = create_empty_test_vault();
        let mut deploymento = MockDeploymento::new();
        deploymento
            .expect_get_deployment_networks()
            .once()
            .with(
                predicate::always(),
                predicate::eq("TestDeployment".to_string()),
            )
            .returning(|_, _| {
                Ok(Some(vec![bollard::models::Network {
                    name: Some("TestNetwork".to_string()),
                    ..Default::default()
                }]))
            });
        let deploymento = Arc::new(deploymento);
        assert_eq!(
            get(
                vault,
                deploymento,
                GetPathParams {
                    deployment_id: "TestDeployment".to_string()
                }
            )
            .await,
            GetResponse::Status200_Success(vec![models::DeploymentNetwork {
                name: "TestNetwork".to_string(),
                driver: None,
                subnet: None,
                parent: None,
            }])
        );
    }

    #[tokio::test]
    async fn get_404() {
        let vault = create_empty_test_vault();
        let mut deploymento = MockDeploymento::new();
        deploymento
            .expect_get_deployment_networks()
            .once()
            .with(
                predicate::always(),
                predicate::eq("TestDeployment".to_string()),
            )
            .returning(|_, _| Ok(None));
        let deploymento = Arc::new(deploymento);
        assert!(matches!(
            get(
                vault,
                deploymento,
                GetPathParams {
                    deployment_id: "TestDeployment".to_string()
                }
            )
            .await,
            GetResponse::Status404_NoDeploymentWithThisDeployment
        ));
    }

    #[tokio::test]
    async fn get_500_deployment_error() {
        let vault = create_empty_test_vault();
        let mut deploymento = MockDeploymento::new();
        deploymento
            .expect_get_deployment_networks()
            .once()
            .with(
                predicate::always(),
                predicate::eq("TestDeployment".to_string()),
            )
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        let deploymento = Arc::new(deploymento);
        assert!(matches!(
            get(
                vault,
                deploymento,
                GetPathParams {
                    deployment_id: "TestDeployment".to_string()
                }
            )
            .await,
            GetResponse::Status500_InternalServerError(_)
        ));
    }

    fn test_network(num: u8) -> bollard::models::Network {
        bollard::models::Network {
            name: Some(format!("TestNetwork{num}")),
            ..Default::default()
        }
    }

    fn expected_network(num: u8) -> models::DeploymentNetwork {
        models::DeploymentNetwork {
            name: format!("TestNetwork{num}"),
            subnet: None,
            driver: None,
            parent: None,
        }
    }

    #[test]
    fn create_networks_model_empty() {
        assert_eq!(create_networks_model(vec![]), vec![]);
    }

    #[test]
    fn create_networks_model_all_ok() {
        assert_eq!(
            create_networks_model(vec![test_network(10), test_network(20), test_network(200)]),
            vec![
                expected_network(10),
                expected_network(20),
                expected_network(200)
            ]
        );
    }

    #[test]
    fn create_networks_model_partly_ok() {
        let broken_network = Network::default();
        assert_eq!(
            create_networks_model(vec![
                broken_network.clone(),
                test_network(10),
                broken_network.clone(),
                test_network(20),
                broken_network.clone(),
                broken_network.clone(),
                test_network(200)
            ]),
            vec![
                expected_network(10),
                expected_network(20),
                expected_network(200)
            ]
        );
    }
}
