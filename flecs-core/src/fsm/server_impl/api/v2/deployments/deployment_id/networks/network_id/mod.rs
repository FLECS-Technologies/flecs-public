use crate::forge::bollard::BollardNetworkExtension;
use crate::jeweler::network::CreateNetworkError as DeploymentCreateNetworkError;
use crate::jeweler::network::{Network, NetworkConfig, NetworkId, NetworkKind};
use crate::relic::network::Ipv4Network;
use crate::sorcerer::deploymento::{CreateNetworkError, Deploymento, GetDeploymentNetworkError};
use crate::vault::Vault;
use flecsd_axum_server::apis::deployments::{
    DeploymentsDeploymentIdNetworksNetworkIdGetResponse as GetResponse,
    DeploymentsDeploymentIdNetworksNetworkIdPutResponse as PutResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    DeploymentsDeploymentIdNetworksNetworkIdGetPathParams as GetPathParams,
    DeploymentsDeploymentIdNetworksNetworkIdPutPathParams as PutPathParams,
    PutDeploymentNetwork as PutRequest,
};
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;

pub async fn put<T: Deploymento>(
    vault: Arc<Vault>,
    deploymento: Arc<T>,
    path_params: PutPathParams,
    request: PutRequest,
) -> PutResponse {
    match try_network_config_from_put_request(request, path_params.network_id) {
        Err(e) => {
            PutResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(config) => {
            match deploymento
                .create_network(vault, path_params.deployment_id, config)
                .await
            {
                Ok(_) => PutResponse::Status201_Created,
                Err(CreateNetworkError::Deployment(
                    DeploymentCreateNetworkError::ExactNetworkExists(_),
                )) => PutResponse::Status200_AlreadyCreated,
                Err(CreateNetworkError::DeploymentNotFound(_)) => {
                    PutResponse::Status404_DeploymentNotFound
                }
                Err(
                    e @ CreateNetworkError::Deployment(
                        DeploymentCreateNetworkError::NetworkConfigInvalid { .. },
                    ),
                )
                | Err(
                    e @ CreateNetworkError::Deployment(
                        DeploymentCreateNetworkError::DifferentNetworkExists(_),
                    ),
                ) => PutResponse::Status400_MalformedRequest(models::AdditionalInfo::new(
                    e.to_string(),
                )),
                Err(e) => PutResponse::Status500_InternalServerError(models::AdditionalInfo::new(
                    e.to_string(),
                )),
            }
        }
    }
}

pub async fn get<T: Deploymento>(
    vault: Arc<Vault>,
    deploymento: Arc<T>,
    path_params: GetPathParams,
) -> GetResponse {
    match deploymento
        .get_deployment_network(
            vault,
            path_params.deployment_id.clone(),
            path_params.network_id.clone(),
        )
        .await
    {
        Ok(network) => match try_model_from_network(network) {
            Ok(network) => GetResponse::Status200_Success(network),
            Err(err) => GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(
                format!("Could not parse network: {err}"),
            )),
        },
        Err(e @ GetDeploymentNetworkError::DeploymentNotFound(_))
        | Err(e @ GetDeploymentNetworkError::NetworkNotFound(_)) => {
            GetResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
                additional_info: Some(e.to_string()),
            })
        }
        Err(e) => {
            GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

pub fn try_model_from_network(value: Network) -> crate::Result<models::DeploymentNetwork> {
    Ok(models::DeploymentNetwork {
        subnet: value.subnet().transpose()?.map(From::from),
        parent: value.parent_network(),
        name: value
            .name
            .ok_or_else(|| anyhow::anyhow!("Network name is required"))?,
        driver: value.driver,
    })
}

fn try_network_config_from_put_request(
    request: PutRequest,
    network_id: NetworkId,
) -> Result<NetworkConfig, anyhow::Error> {
    let cidr_subnet = try_subnet_from_put_request(&request)?;
    let gateway = try_gateway_from_put_request(&request)?;
    Ok(NetworkConfig {
        name: network_id,
        parent_adapter: request.parent_adapter,
        cidr_subnet,
        gateway,
        kind: request.network_kind.into(),
        options: request.options,
    })
}

fn try_subnet_from_put_request(request: &PutRequest) -> Result<Option<Ipv4Network>, anyhow::Error> {
    if let Some(models::Ipam {
        ipv4_subnet: Some(models::Ipv4Network { address, netmask }),
        ..
    }) = request.ipam.as_ref()
    {
        Ok(Some(Ipv4Network::new_from_address_and_subnet_mask(
            Ipv4Addr::from_str(address)?,
            Ipv4Addr::from_str(netmask)?,
        )?))
    } else {
        Ok(None)
    }
}

fn try_gateway_from_put_request(request: &PutRequest) -> Result<Option<Ipv4Addr>, anyhow::Error> {
    if let Some(models::Ipam {
        ipv4_gateway: Some(gateway),
        ..
    }) = request.ipam.as_ref()
    {
        Ok(Some(Ipv4Addr::from_str(gateway)?))
    } else {
        Ok(None)
    }
}

impl From<models::NetworkKind> for NetworkKind {
    fn from(value: models::NetworkKind) -> Self {
        match value {
            models::NetworkKind::Internal => NetworkKind::Internal,
            models::NetworkKind::Bridge => NetworkKind::Bridge,
            models::NetworkKind::Macvlan => NetworkKind::MACVLAN,
            models::NetworkKind::IpvlanL2 => NetworkKind::IpvlanL2,
            models::NetworkKind::IpvlanL3 => NetworkKind::IpvlanL3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorcerer::deploymento::MockDeploymento;
    use crate::vault::tests::create_empty_test_vault;
    use bollard::models::{Ipam, IpamConfig};
    use mockall::predicate;
    use std::collections::HashMap;

    #[tokio::test]
    async fn get_200() {
        let vault = create_empty_test_vault();
        let mut deploymento = MockDeploymento::new();
        deploymento
            .expect_get_deployment_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq("TestDeployment".to_string()),
                predicate::eq("TestNetwork".to_string()),
            )
            .returning(|_, _, _| {
                Ok(bollard::models::Network {
                    name: Some("TestNetwork".to_string()),
                    ..Default::default()
                })
            });
        let deploymento = Arc::new(deploymento);
        assert_eq!(
            get(
                vault,
                deploymento,
                GetPathParams {
                    network_id: "TestNetwork".to_string(),
                    deployment_id: "TestDeployment".to_string()
                }
            )
            .await,
            GetResponse::Status200_Success(models::DeploymentNetwork {
                name: "TestNetwork".to_string(),
                subnet: None,
                driver: None,
                parent: None,
            })
        );
    }

    #[tokio::test]
    async fn get_500_no_name() {
        let vault = create_empty_test_vault();
        let mut deploymento = MockDeploymento::new();
        deploymento
            .expect_get_deployment_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq("TestDeployment".to_string()),
                predicate::eq("TestNetwork".to_string()),
            )
            .returning(|_, _, _| Ok(bollard::models::Network::default()));
        let deploymento = Arc::new(deploymento);
        assert!(matches!(
            get(
                vault,
                deploymento,
                GetPathParams {
                    network_id: "TestNetwork".to_string(),
                    deployment_id: "TestDeployment".to_string()
                }
            )
            .await,
            GetResponse::Status500_InternalServerError(_)
        ));
    }

    #[tokio::test]
    async fn get_404_deployment() {
        let vault = create_empty_test_vault();
        let mut deploymento = MockDeploymento::new();
        deploymento
            .expect_get_deployment_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq("TestDeployment".to_string()),
                predicate::eq("TestNetwork".to_string()),
            )
            .returning(|_, _, _| {
                Err(GetDeploymentNetworkError::DeploymentNotFound(
                    "TestDeployment".to_string(),
                ))
            });
        let deploymento = Arc::new(deploymento);
        assert!(matches!(
            get(
                vault,
                deploymento,
                GetPathParams {
                    network_id: "TestNetwork".to_string(),
                    deployment_id: "TestDeployment".to_string()
                }
            )
            .await,
            GetResponse::Status404_ResourceNotFound(_)
        ));
    }

    #[tokio::test]
    async fn get_404_network() {
        let vault = create_empty_test_vault();
        let mut deploymento = MockDeploymento::new();
        deploymento
            .expect_get_deployment_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq("TestDeployment".to_string()),
                predicate::eq("TestNetwork".to_string()),
            )
            .returning(|_, _, _| {
                Err(GetDeploymentNetworkError::NetworkNotFound(
                    "TestNetwork".to_string(),
                ))
            });
        let deploymento = Arc::new(deploymento);
        assert!(matches!(
            get(
                vault,
                deploymento,
                GetPathParams {
                    network_id: "TestNetwork".to_string(),
                    deployment_id: "TestDeployment".to_string()
                }
            )
            .await,
            GetResponse::Status404_ResourceNotFound(_)
        ));
    }

    #[tokio::test]
    async fn get_500_deployment_error() {
        let vault = create_empty_test_vault();
        let mut deploymento = MockDeploymento::new();
        deploymento
            .expect_get_deployment_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq("TestDeployment".to_string()),
                predicate::eq("TestNetwork".to_string()),
            )
            .returning(|_, _, _| {
                Err(GetDeploymentNetworkError::Other {
                    network_id: "TestNetwork".to_string(),
                    reason: "TestError".to_string(),
                })
            });
        let deploymento = Arc::new(deploymento);
        assert!(matches!(
            get(
                vault,
                deploymento,
                GetPathParams {
                    network_id: "TestNetwork".to_string(),
                    deployment_id: "TestDeployment".to_string()
                }
            )
            .await,
            GetResponse::Status500_InternalServerError(_)
        ));
    }

    #[test]
    fn try_model_from_network_err_subnet() {
        let network = Network {
            name: Some("TestNetwork".to_string()),
            ipam: Some(Ipam {
                config: Some(vec![IpamConfig {
                    subnet: Some("invalid subnet".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(try_model_from_network(network).is_err());
    }

    #[test]
    fn try_model_from_network_err_no_name() {
        let network = Network::default();
        assert!(try_model_from_network(network).is_err());
    }

    #[test]
    fn try_model_from_network_ok_min() {
        let network = Network {
            name: Some("TestNetwork".to_string()),
            ..Default::default()
        };
        assert_eq!(
            try_model_from_network(network).unwrap(),
            models::DeploymentNetwork {
                name: "TestNetwork".to_string(),
                subnet: None,
                parent: None,
                driver: None,
            }
        );
    }

    #[test]
    fn try_model_from_network_ok_full() {
        let network = Network {
            name: Some("TestNetwork".to_string()),
            ipam: Some(Ipam {
                config: Some(vec![IpamConfig {
                    subnet: Some("127.10.0.0/16".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            driver: Some("TestDriver".to_string()),
            options: Some(HashMap::from([(
                "parent".to_string(),
                "TestParent".to_string(),
            )])),
            ..Default::default()
        };
        assert_eq!(
            try_model_from_network(network).unwrap(),
            models::DeploymentNetwork {
                name: "TestNetwork".to_string(),
                subnet: Some(models::Network::Ipv4Network(Box::new(
                    models::Ipv4Network::new("127.10.0.0".to_string(), "255.255.0.0".to_string())
                ))),
                parent: Some("TestParent".to_string()),
                driver: Some("TestDriver".to_string()),
            }
        );
    }

    fn put_test_data(
        mock_result: anyhow::Result<Network, CreateNetworkError>,
    ) -> (Arc<Vault>, Arc<MockDeploymento>, PutPathParams, PutRequest) {
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
            .return_const(mock_result);
        let vault = create_empty_test_vault();
        let path_params = PutPathParams {
            network_id: "TestNetwork".to_string(),
            deployment_id: "MockDeployment".to_string(),
        };
        let put_request = PutRequest {
            network_kind: models::NetworkKind::Internal,
            options: None,
            parent_adapter: None,
            ipam: None,
        };
        (vault, Arc::new(deploymento), path_params, put_request)
    }

    #[tokio::test]
    async fn put_200() {
        let (vault, deploymento, path_params, put_request) =
            put_test_data(Err(CreateNetworkError::Deployment(
                DeploymentCreateNetworkError::ExactNetworkExists(Network::default()),
            )));
        assert_eq!(
            put(vault, deploymento, path_params, put_request).await,
            PutResponse::Status200_AlreadyCreated
        );
    }

    #[tokio::test]
    async fn put_201() {
        let (vault, deploymento, path_params, put_request) = put_test_data(Ok(Network::default()));
        assert_eq!(
            put(vault, deploymento, path_params, put_request).await,
            PutResponse::Status201_Created
        );
    }

    #[tokio::test]
    async fn put_400_other_network_exists() {
        let (vault, deploymento, path_params, put_request) =
            put_test_data(Err(CreateNetworkError::Deployment(
                DeploymentCreateNetworkError::DifferentNetworkExists(Network::default()),
            )));
        assert!(matches!(
            put(vault, deploymento, path_params, put_request).await,
            PutResponse::Status400_MalformedRequest(_)
        ));
    }

    #[tokio::test]
    async fn put_400_network_config_invalid() {
        let (vault, deploymento, path_params, put_request) = put_test_data(Err(
            CreateNetworkError::Deployment(DeploymentCreateNetworkError::NetworkConfigInvalid {
                location: "Test".to_string(),
                reason: "TestReason".to_string(),
            }),
        ));
        assert!(matches!(
            put(vault, deploymento, path_params, put_request).await,
            PutResponse::Status400_MalformedRequest(_)
        ));
    }

    #[tokio::test]
    async fn put_400_request_invalid() {
        let deploymento = Arc::new(MockDeploymento::new());
        let vault = create_empty_test_vault();
        let path_params = PutPathParams {
            network_id: "TestNetwork".to_string(),
            deployment_id: "MockDeployment".to_string(),
        };
        let put_request = PutRequest {
            network_kind: models::NetworkKind::Internal,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam {
                ipv4_subnet: None,
                ipv4_gateway: Some("10.20.30.400".to_string()),
            }),
        };
        assert!(matches!(
            put(vault, deploymento, path_params, put_request).await,
            PutResponse::Status400_MalformedRequest(_)
        ));
    }

    #[tokio::test]
    async fn put_404() {
        let (vault, deploymento, path_params, put_request) = put_test_data(Err(
            CreateNetworkError::DeploymentNotFound("MockDeployment".to_string()),
        ));
        assert_eq!(
            put(vault, deploymento, path_params, put_request).await,
            PutResponse::Status404_DeploymentNotFound
        );
    }

    #[tokio::test]
    async fn put_500() {
        let (vault, deploymento, path_params, put_request) =
            put_test_data(Err(CreateNetworkError::Deployment(
                DeploymentCreateNetworkError::Other("TestError".to_string()),
            )));
        assert!(matches!(
            put(vault, deploymento, path_params, put_request).await,
            PutResponse::Status500_InternalServerError(_)
        ));
    }

    #[test]
    fn network_kind_from_model() {
        assert_eq!(
            NetworkKind::from(models::NetworkKind::Internal),
            NetworkKind::Internal
        );
        assert_eq!(
            NetworkKind::from(models::NetworkKind::Bridge),
            NetworkKind::Bridge
        );
        assert_eq!(
            NetworkKind::from(models::NetworkKind::Macvlan),
            NetworkKind::MACVLAN
        );
        assert_eq!(
            NetworkKind::from(models::NetworkKind::IpvlanL2),
            NetworkKind::IpvlanL2
        );
        assert_eq!(
            NetworkKind::from(models::NetworkKind::IpvlanL3),
            NetworkKind::IpvlanL3
        );
    }

    #[test]
    fn try_gateway_from_put_request_ok_some() {
        let request = PutRequest {
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam {
                ipv4_subnet: None,
                ipv4_gateway: Some("10.20.100.1".to_string()),
            }),
        };
        assert_eq!(
            try_gateway_from_put_request(&request).unwrap(),
            Some(Ipv4Addr::new(10, 20, 100, 1))
        );
    }

    #[test]
    fn try_gateway_from_put_request_ok_none_gateway() {
        let request = PutRequest {
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam {
                ipv4_subnet: None,
                ipv4_gateway: None,
            }),
        };
        assert_eq!(try_gateway_from_put_request(&request).unwrap(), None);
    }

    #[test]
    fn try_gateway_from_put_request_ok_none_ipam() {
        let request = PutRequest {
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: None,
        };
        assert_eq!(try_gateway_from_put_request(&request).unwrap(), None);
    }

    #[test]
    fn try_gateway_from_put_request_err() {
        let request = PutRequest {
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam {
                ipv4_subnet: None,
                ipv4_gateway: Some("invalid".to_string()),
            }),
        };
        assert!(try_gateway_from_put_request(&request).is_err());
    }

    #[test]
    fn try_subnet_from_put_request_ok_some() {
        let request = PutRequest {
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam {
                ipv4_subnet: Some(models::Ipv4Network::new(
                    "10.20.100.0".to_string(),
                    "255.255.255.0".to_string(),
                )),
                ipv4_gateway: None,
            }),
        };
        assert_eq!(
            try_subnet_from_put_request(&request).unwrap(),
            Some(Ipv4Network::from_str("10.20.100.0/24").unwrap())
        );
    }

    #[test]
    fn try_subnet_from_put_request_ok_none_subnet() {
        let request = PutRequest {
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam {
                ipv4_subnet: None,
                ipv4_gateway: None,
            }),
        };
        assert_eq!(try_subnet_from_put_request(&request).unwrap(), None);
    }

    #[test]
    fn try_subnet_from_put_request_ok_none_ipam() {
        let request = PutRequest {
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: None,
        };
        assert_eq!(try_subnet_from_put_request(&request).unwrap(), None);
    }

    #[test]
    fn try_subnet_from_put_request_err_network() {
        let request = PutRequest {
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam {
                ipv4_subnet: Some(models::Ipv4Network::new(
                    "10.20.100.0".to_string(),
                    "255.0.255.0".to_string(),
                )),
                ipv4_gateway: None,
            }),
        };
        assert!(try_subnet_from_put_request(&request).is_err());
    }

    #[test]
    fn try_subnet_from_put_request_err_address() {
        let request = PutRequest {
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam {
                ipv4_subnet: Some(models::Ipv4Network::new(
                    "invalid".to_string(),
                    "255.255.0.0".to_string(),
                )),
                ipv4_gateway: None,
            }),
        };
        assert!(try_subnet_from_put_request(&request).is_err());
    }

    #[test]
    fn try_subnet_from_put_request_err_netmask() {
        let request = PutRequest {
            network_kind: models::NetworkKind::Bridge,
            options: None,
            parent_adapter: None,
            ipam: Some(models::Ipam {
                ipv4_subnet: Some(models::Ipv4Network::new(
                    "10.20.100.0".to_string(),
                    "invalid".to_string(),
                )),
                ipv4_gateway: None,
            }),
        };
        assert!(try_subnet_from_put_request(&request).is_err());
    }

    fn try_network_config_from_put_request_data() -> PutRequest {
        PutRequest {
            network_kind: models::NetworkKind::IpvlanL2,
            options: Some(HashMap::from([(
                "custom-option".to_string(),
                "value".to_string(),
            )])),
            parent_adapter: Some("parent".to_string()),
            ipam: Some(models::Ipam {
                ipv4_subnet: Some(models::Ipv4Network::new(
                    "10.20.100.0".to_string(),
                    "255.255.255.0".to_string(),
                )),
                ipv4_gateway: Some("10.20.100.1".to_string()),
            }),
        }
    }

    #[test]
    fn try_network_config_from_put_request_ok() {
        let request = try_network_config_from_put_request_data();
        assert_eq!(
            try_network_config_from_put_request(request, "Network".to_string()).unwrap(),
            NetworkConfig {
                kind: NetworkKind::IpvlanL2,
                name: "Network".to_string(),
                cidr_subnet: Some(Ipv4Network::from_str("10.20.100.0/24").unwrap()),
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
    fn try_network_config_from_put_request_err_gateway() {
        let mut request = try_network_config_from_put_request_data();
        request.ipam.as_mut().unwrap().ipv4_gateway = Some("invalid".to_string());
        assert!(try_network_config_from_put_request(request, "Network".to_string()).is_err());
    }

    #[test]
    fn try_network_config_from_put_request_err_subnet() {
        let mut request = try_network_config_from_put_request_data();
        request
            .ipam
            .as_mut()
            .unwrap()
            .ipv4_subnet
            .as_mut()
            .unwrap()
            .netmask = "invalid".to_string();
        assert!(try_network_config_from_put_request(request, "Network".to_string()).is_err());
    }
}
