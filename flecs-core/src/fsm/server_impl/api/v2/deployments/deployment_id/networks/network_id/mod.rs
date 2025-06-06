use crate::forge::bollard::BollardNetworkExtension;
use crate::jeweler::network::{Network, NetworkKind};
use crate::sorcerer::deploymento::{Deploymento, GetDeploymentNetworkError};
use crate::vault::Vault;
use flecsd_axum_server::apis::deployments::DeploymentsDeploymentIdNetworksNetworkIdGetResponse as GetResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::DeploymentsDeploymentIdNetworksNetworkIdGetPathParams as GetPathParams;
use std::sync::Arc;

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
        ipam: Some(models::Ipam {
            ipv4: try_ipv4_ipam_model_from_network(&value)?,
        }),
        parent: value.parent_network(),
        name: value
            .name
            .ok_or_else(|| anyhow::anyhow!("Network name is required"))?,
        driver: value.driver,
    })
}

fn try_ipv4_ipam_model_from_network(value: &Network) -> crate::Result<Option<models::Ipv4Ipam>> {
    match value.subnets_and_gateways_ipv4()?.into_iter().next() {
        None => Ok(None),
        Some((network, gateway)) => Ok(Some(models::Ipv4Ipam {
            address: network.address().to_string(),
            netmask: network.subnet_mask().to_string(),
            gateway: gateway.map(|gateway| gateway.to_string()),
        })),
    }
}

impl From<models::NetworkKind> for NetworkKind {
    fn from(value: models::NetworkKind) -> Self {
        match value {
            models::NetworkKind::Internal => NetworkKind::Internal,
            models::NetworkKind::Bridge => NetworkKind::Bridge,
            models::NetworkKind::Macvlan => NetworkKind::MACVLAN,
            models::NetworkKind::Ipvlanl2 => NetworkKind::IpvlanL2,
            models::NetworkKind::Ipvlanl3 => NetworkKind::IpvlanL3,
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
                ipam: Some(models::Ipam { ipv4: None }),
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
                ipam: Some(models::Ipam { ipv4: None }),
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
                ipam: Some(models::Ipam {
                    ipv4: Some(models::Ipv4Ipam::new(
                        "127.10.0.0".to_string(),
                        "255.255.0.0".to_string(),
                    ))
                }),
                parent: Some("TestParent".to_string()),
                driver: Some("TestDriver".to_string()),
            }
        );
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
            NetworkKind::from(models::NetworkKind::Ipvlanl2),
            NetworkKind::IpvlanL2
        );
        assert_eq!(
            NetworkKind::from(models::NetworkKind::Ipvlanl3),
            NetworkKind::IpvlanL3
        );
    }
}
