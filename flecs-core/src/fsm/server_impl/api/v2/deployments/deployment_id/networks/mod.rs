pub mod dhcp;
pub mod network_id;

use crate::fsm::server_impl::api::v2::deployments::deployment_id::networks::network_id::try_model_from_network;
use crate::jeweler::network::{Network, NetworkConfig};
use crate::sorcerer::deploymento::{CreateNetworkError, Deploymento};
use crate::vault::Vault;
use flecsd_axum_server::apis::deployments::{
    DeploymentsDeploymentIdNetworksGetResponse as GetResponse,
    DeploymentsDeploymentIdNetworksPostResponse as PostResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    DeploymentsDeploymentIdNetworksGetPathParams as GetPathParams,
    DeploymentsDeploymentIdNetworksPostPathParams as PostPathParams,
    PostDeploymentNetwork as PostRequest,
};
use ipnet::Ipv4Net;
use std::net::Ipv4Addr;
use std::str::FromStr;
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
        Err(e) => {
            GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

pub async fn post<T: Deploymento>(
    vault: Arc<Vault>,
    deploymento: Arc<T>,
    path_params: PostPathParams,
    request: PostRequest,
) -> PostResponse {
    match try_network_config_from_post_request(request) {
        Err(e) => {
            PostResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(config) => {
            match deploymento
                .create_network(vault, path_params.deployment_id, config)
                .await
            {
                Ok(_) => PostResponse::Status201_Created,
                Err(CreateNetworkError::Deployment(
                    crate::jeweler::network::CreateNetworkError::ExactNetworkExists(_),
                )) => PostResponse::Status200_AlreadyCreated,
                Err(CreateNetworkError::DeploymentNotFound(_)) => {
                    PostResponse::Status404_DeploymentNotFound
                }
                Err(
                    e @ CreateNetworkError::Deployment(
                        crate::jeweler::network::CreateNetworkError::NetworkConfigInvalid {
                            ..
                        },
                    ),
                )
                | Err(
                    e @ CreateNetworkError::Deployment(
                        crate::jeweler::network::CreateNetworkError::DifferentNetworkExists(_),
                    ),
                ) => PostResponse::Status400_MalformedRequest(models::AdditionalInfo::new(
                    e.to_string(),
                )),
                Err(e) => PostResponse::Status500_InternalServerError(models::AdditionalInfo::new(
                    e.to_string(),
                )),
            }
        }
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

fn try_network_config_from_post_request(
    request: PostRequest,
) -> Result<NetworkConfig, anyhow::Error> {
    let cidr_subnet = try_subnet_from_post_request(&request)?;
    let gateway = try_gateway_from_post_request(&request)?;
    Ok(NetworkConfig {
        name: request.network_id,
        parent_adapter: request.parent_adapter,
        cidr_subnet,
        gateway,
        kind: request.network_kind.into(),
        options: request.options,
    })
}

fn try_subnet_from_post_request(request: &PostRequest) -> Result<Option<Ipv4Net>, anyhow::Error> {
    if let Some(models::Ipam {
        ipv4: Some(models::Ipv4Ipam {
            address, netmask, ..
        }),
        ..
    }) = request.ipam.as_ref()
    {
        Ok(Some(Ipv4Net::with_netmask(
            Ipv4Addr::from_str(address)?,
            Ipv4Addr::from_str(netmask)?,
        )?))
    } else {
        Ok(None)
    }
}

fn try_gateway_from_post_request(request: &PostRequest) -> Result<Option<Ipv4Addr>, anyhow::Error> {
    if let Some(models::Ipam {
        ipv4: Some(models::Ipv4Ipam {
            gateway: Some(gateway),
            ..
        }),
        ..
    }) = request.ipam.as_ref()
    {
        Ok(Some(Ipv4Addr::from_str(gateway)?))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::network::NetworkKind;
    use crate::sorcerer::deploymento::MockDeploymento;
    use crate::vault::tests::create_empty_test_vault;
    use mockall::predicate;
    use std::collections::HashMap;

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
                ipam: Some(models::Ipam { ipv4: None }),
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
            ipam: Some(models::Ipam { ipv4: None }),
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

    #[test]
    fn try_gateway_from_post_request_ok_some() {
        let request = PostRequest {
            network_id: "Network".to_string(),
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam {
                ipv4: Some(models::Ipv4Ipam {
                    address: "None".to_string(),
                    netmask: "None".to_string(),
                    gateway: Some("10.20.100.1".to_string()),
                }),
            }),
        };
        assert_eq!(
            try_gateway_from_post_request(&request).unwrap(),
            Some(Ipv4Addr::new(10, 20, 100, 1))
        );
    }

    #[test]
    fn try_gateway_from_post_request_ok_none_ipv4_ipam() {
        let request = PostRequest {
            network_id: "Network".to_string(),
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam { ipv4: None }),
        };
        assert_eq!(try_gateway_from_post_request(&request).unwrap(), None);
    }

    #[test]
    fn try_gateway_from_post_request_ok_none_ipam() {
        let request = PostRequest {
            network_id: "Network".to_string(),
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: None,
        };
        assert_eq!(try_gateway_from_post_request(&request).unwrap(), None);
    }

    #[test]
    fn try_gateway_from_post_request_err() {
        let request = PostRequest {
            network_id: "Network".to_string(),
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam {
                ipv4: Some(models::Ipv4Ipam {
                    address: "None".to_string(),
                    netmask: "None".to_string(),
                    gateway: Some("invalid".to_string()),
                }),
            }),
        };
        assert!(try_gateway_from_post_request(&request).is_err());
    }

    #[test]
    fn try_subnet_from_post_request_ok_some() {
        let request = PostRequest {
            network_id: "Network".to_string(),
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam {
                ipv4: Some(models::Ipv4Ipam::new(
                    "10.20.100.0".to_string(),
                    "255.255.255.0".to_string(),
                )),
            }),
        };
        assert_eq!(
            try_subnet_from_post_request(&request).unwrap(),
            Some(Ipv4Net::from_str("10.20.100.0/24").unwrap())
        );
    }

    #[test]
    fn try_subnet_from_post_request_ok_none_ipv4_ipam() {
        let request = PostRequest {
            network_id: "Network".to_string(),
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam { ipv4: None }),
        };
        assert_eq!(try_subnet_from_post_request(&request).unwrap(), None);
    }

    #[test]
    fn try_subnet_from_post_request_ok_none_ipam() {
        let request = PostRequest {
            network_id: "Network".to_string(),
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: None,
        };
        assert_eq!(try_subnet_from_post_request(&request).unwrap(), None);
    }

    #[test]
    fn try_subnet_from_post_request_err_network() {
        let request = PostRequest {
            network_id: "Network".to_string(),
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam {
                ipv4: Some(models::Ipv4Ipam::new(
                    "10.20.100.0".to_string(),
                    "255.0.255.0".to_string(),
                )),
            }),
        };
        assert!(try_subnet_from_post_request(&request).is_err());
    }

    #[test]
    fn try_subnet_from_post_request_err_address() {
        let request = PostRequest {
            network_id: "Network".to_string(),
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam {
                ipv4: Some(models::Ipv4Ipam::new(
                    "invalid".to_string(),
                    "255.255.0.0".to_string(),
                )),
            }),
        };
        assert!(try_subnet_from_post_request(&request).is_err());
    }

    #[test]
    fn try_subnet_from_post_request_err_netmask() {
        let request = PostRequest {
            network_id: "Network".to_string(),
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam {
                ipv4: Some(models::Ipv4Ipam::new(
                    "10.20.100.0".to_string(),
                    "invalid".to_string(),
                )),
            }),
        };
        assert!(try_subnet_from_post_request(&request).is_err());
    }

    fn try_network_config_from_post_request_data() -> PostRequest {
        PostRequest {
            network_id: "Network".to_string(),
            network_kind: models::NetworkKind::Ipvlanl2,
            options: Some(HashMap::from([(
                "custom-option".to_string(),
                "value".to_string(),
            )])),
            parent_adapter: Some("parent".to_string()),
            ipam: Some(models::Ipam {
                ipv4: Some(models::Ipv4Ipam {
                    address: "10.20.100.0".to_string(),
                    netmask: "255.255.255.0".to_string(),
                    gateway: Some("10.20.100.1".to_string()),
                }),
            }),
        }
    }

    #[test]
    fn try_network_config_from_post_request_ok() {
        let request = try_network_config_from_post_request_data();
        assert_eq!(
            try_network_config_from_post_request(request).unwrap(),
            NetworkConfig {
                kind: NetworkKind::IpvlanL2,
                name: "Network".to_string(),
                cidr_subnet: Some(Ipv4Net::from_str("10.20.100.0/24").unwrap()),
                gateway: Some(Ipv4Addr::from_str("10.20.100.1").unwrap()),
                parent_adapter: Some("parent".to_string()),
                options: Some(HashMap::from([(
                    "custom-option".to_string(),
                    "value".to_string(),
                )])),
            }
        )
    }

    #[test]
    fn try_network_config_from_post_request_err_gateway() {
        let mut request = try_network_config_from_post_request_data();
        request
            .ipam
            .as_mut()
            .unwrap()
            .ipv4
            .as_mut()
            .unwrap()
            .gateway = Some("invalid".to_string());
        assert!(try_network_config_from_post_request(request).is_err());
    }

    #[test]
    fn try_network_config_from_post_request_err_subnet() {
        let mut request = try_network_config_from_post_request_data();
        request
            .ipam
            .as_mut()
            .unwrap()
            .ipv4
            .as_mut()
            .unwrap()
            .netmask = "invalid".to_string();
        assert!(try_network_config_from_post_request(request).is_err());
    }

    fn post_test_data<F>(
        mock_result: F,
    ) -> (
        Arc<Vault>,
        Arc<MockDeploymento>,
        PostPathParams,
        PostRequest,
    )
    where
        F: Fn() -> anyhow::Result<Network, CreateNetworkError> + Send + 'static,
    {
        let mut deploymento = MockDeploymento::new();
        let expected_config = NetworkConfig {
            kind: NetworkKind::Internal,
            name: "TestNetwork".to_string(),
            cidr_subnet: None,
            gateway: None,
            parent_adapter: None,
            options: Default::default(),
        };
        deploymento
            .expect_create_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq("MockDeployment".to_string()),
                predicate::eq(expected_config),
            )
            .returning(move |_, _, _| mock_result());
        let vault = create_empty_test_vault();
        let path_params = PostPathParams {
            deployment_id: "MockDeployment".to_string(),
        };
        let post_request = PostRequest {
            network_id: "TestNetwork".to_string(),
            network_kind: models::NetworkKind::Internal,
            options: None,
            parent_adapter: None,
            ipam: None,
        };
        (vault, Arc::new(deploymento), path_params, post_request)
    }

    #[tokio::test]
    async fn post_200() {
        let (vault, deploymento, path_params, post_request) = post_test_data(|| {
            Err(CreateNetworkError::Deployment(
                crate::jeweler::network::CreateNetworkError::ExactNetworkExists(Network::default()),
            ))
        });
        assert_eq!(
            post(vault, deploymento, path_params, post_request).await,
            PostResponse::Status200_AlreadyCreated
        );
    }

    #[tokio::test]
    async fn post_201() {
        let (vault, deploymento, path_params, post_request) =
            post_test_data(|| Ok(Network::default()));
        assert_eq!(
            post(vault, deploymento, path_params, post_request).await,
            PostResponse::Status201_Created
        );
    }

    #[tokio::test]
    async fn post_400_other_network_exists() {
        let (vault, deploymento, path_params, post_request) = post_test_data(|| {
            Err(CreateNetworkError::Deployment(
                crate::jeweler::network::CreateNetworkError::DifferentNetworkExists(
                    Network::default(),
                ),
            ))
        });
        assert!(matches!(
            post(vault, deploymento, path_params, post_request).await,
            PostResponse::Status400_MalformedRequest(_)
        ));
    }

    #[tokio::test]
    async fn post_400_network_config_invalid() {
        let (vault, deploymento, path_params, post_request) = post_test_data(|| {
            Err(CreateNetworkError::Deployment(
                crate::jeweler::network::CreateNetworkError::NetworkConfigInvalid {
                    location: "Test".to_string(),
                    reason: "TestReason".to_string(),
                },
            ))
        });
        assert!(matches!(
            post(vault, deploymento, path_params, post_request).await,
            PostResponse::Status400_MalformedRequest(_)
        ));
    }

    #[tokio::test]
    async fn post_400_request_invalid() {
        let deploymento = Arc::new(MockDeploymento::new());
        let vault = create_empty_test_vault();
        let path_params = PostPathParams {
            deployment_id: "MockDeployment".to_string(),
        };
        let post_request = PostRequest {
            network_id: "TestNetwork".to_string(),
            network_kind: models::NetworkKind::Internal,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam {
                ipv4: Some(models::Ipv4Ipam {
                    address: "".to_string(),
                    netmask: "".to_string(),
                    gateway: Some("10.20.30.400".to_string()),
                }),
            }),
        };
        assert!(matches!(
            post(vault, deploymento, path_params, post_request).await,
            PostResponse::Status400_MalformedRequest(_)
        ));
    }

    #[tokio::test]
    async fn post_404() {
        let (vault, deploymento, path_params, post_request) = post_test_data(|| {
            Err(CreateNetworkError::DeploymentNotFound(
                "MockDeployment".to_string(),
            ))
        });
        assert_eq!(
            post(vault, deploymento, path_params, post_request).await,
            PostResponse::Status404_DeploymentNotFound
        );
    }

    #[tokio::test]
    async fn post_500() {
        let (vault, deploymento, path_params, post_request) = post_test_data(|| {
            Err(CreateNetworkError::Deployment(
                crate::jeweler::network::CreateNetworkError::Other("TestError".to_string()),
            ))
        });
        assert!(matches!(
            post(vault, deploymento, path_params, post_request).await,
            PostResponse::Status500_InternalServerError(_)
        ));
    }
}
