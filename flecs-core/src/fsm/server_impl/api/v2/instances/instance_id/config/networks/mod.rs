pub mod network_id;

use crate::sorcerer::instancius::{
    ConnectInstanceConfigNetworkError, Instancius, QueryInstanceConfigError,
};
use crate::vault::Vault;
use crate::vault::pouch::instance::InstanceId;
use flecsd_axum_server::apis::instances::{
    InstancesInstanceIdConfigNetworksGetResponse as GetResponse,
    InstancesInstanceIdConfigNetworksPostResponse as PostResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    AdditionalInfo, InstancesInstanceIdConfigNetworksGetPathParams as GetPathParams,
    InstancesInstanceIdConfigNetworksPostPathParams as PostPathParams,
    InstancesInstanceIdConfigNetworksPostRequest as PostBody,
};
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;

pub async fn get<T: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<T>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .get_instance_config_networks(vault, instance_id)
        .await
    {
        Ok(networks) => GetResponse::Status200_Success(
            networks
                .into_iter()
                .map(|(id, ip)| models::InstanceConfigNetwork::new(id, ip.to_string()))
                .collect(),
        ),
        Err(QueryInstanceConfigError::NotFound(_)) => GetResponse::Status404_InstanceIdNotFound,
        Err(e @ QueryInstanceConfigError::NotSupported(_)) => {
            GetResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

pub async fn post<T: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<T>,
    path_params: PostPathParams,
    body: PostBody,
) -> PostResponse {
    let instance_id =
        crate::jeweler::gem::instance::InstanceId::from_str(&path_params.instance_id).unwrap();
    let ip = match body
        .ip_address_suggestion
        .map(|ip| Ipv4Addr::from_str(&ip))
        .transpose()
    {
        Ok(ip) => ip,
        Err(e) => {
            return PostResponse::Status400_MalformedRequest(AdditionalInfo::new(format!(
                "Failed to parse ip from body: {e}"
            )));
        }
    };
    let network_id = body.network_id;
    match instancius
        .connect_instance_to_network(vault, network_id.clone(), instance_id, ip)
        .await
    {
        Ok(_) => PostResponse::Status201_InstanceConnected {
            location: format!("/v2/instances/{instance_id}/config/networks/{network_id}"),
        },
        Err(ConnectInstanceConfigNetworkError::InstanceNotFound(_)) => {
            PostResponse::Status404_InstanceIdOrNetworkNotFound
        }
        Err(e @ ConnectInstanceConfigNetworkError::AddressOutOfRange { .. }) => {
            PostResponse::Status400_MalformedRequest(AdditionalInfo::new(e.to_string()))
        }
        Err(ConnectInstanceConfigNetworkError::NetworkAlreadyConnected { .. }) => {
            PostResponse::Status409_InstanceAlreadyConnectedToNetwork
        }
        Err(e) => PostResponse::Status500_InternalServerError(AdditionalInfo::new(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorcerer::instancius::MockInstancius;
    use crate::vault::tests::create_empty_test_vault;
    use flecsd_axum_server::models::InstanceConfigNetwork;
    use mockall::predicate;
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr};
    const NETWORK_ID: &str = "test-network";

    #[tokio::test]
    async fn get_200() {
        const INSTANCE_ID: InstanceId = InstanceId::new(10);
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_networks()
            .once()
            .with(predicate::always(), predicate::eq(INSTANCE_ID))
            .returning(|_, _| {
                Ok(HashMap::from([(
                    "net_1".to_string(),
                    IpAddr::V4(Ipv4Addr::new(10, 20, 30, 40)),
                )]))
            });
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: INSTANCE_ID.to_string(),
                },
            )
            .await,
            GetResponse::Status200_Success(vec![InstanceConfigNetwork {
                name: "net_1".to_string(),
                ip_address: "10.20.30.40".to_string()
            }])
        );
    }

    #[tokio::test]
    async fn get_404() {
        const INSTANCE_ID: InstanceId = InstanceId::new(10);
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_networks()
            .once()
            .with(predicate::always(), predicate::eq(INSTANCE_ID))
            .returning(|_, _| Err(QueryInstanceConfigError::NotFound(INSTANCE_ID)));
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: INSTANCE_ID.to_string(),
                },
            )
            .await,
            GetResponse::Status404_InstanceIdNotFound
        );
    }

    #[tokio::test]
    async fn post_201() {
        const INSTANCE_ID: InstanceId = InstanceId::new(10);
        let post_path_params = PostPathParams {
            instance_id: INSTANCE_ID.to_string(),
        };
        let ip_address = Ipv4Addr::new(10, 18, 102, 10);
        let body = PostBody {
            network_id: NETWORK_ID.to_string(),
            ip_address_suggestion: Some(ip_address.to_string()),
        };
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_connect_instance_to_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(NETWORK_ID.to_string()),
                predicate::eq(INSTANCE_ID),
                predicate::eq(Some(ip_address)),
            )
            .returning(move |_, _, _, _| Ok(IpAddr::V4(ip_address)));
        assert_eq!(
            post(vault, Arc::new(instancius), post_path_params, body,).await,
            PostResponse::Status201_InstanceConnected {
                location: format!("/v2/instances/{INSTANCE_ID}/config/networks/{NETWORK_ID}")
            }
        );
    }

    #[tokio::test]
    async fn post_400_invalid_ip() {
        const INSTANCE_ID: InstanceId = InstanceId::new(10);
        let post_path_params = PostPathParams {
            instance_id: INSTANCE_ID.to_string(),
        };
        let body = PostBody {
            network_id: NETWORK_ID.to_string(),
            ip_address_suggestion: Some("invalid ip".to_string()),
        };
        let vault = create_empty_test_vault();
        let instancius = MockInstancius::new();
        assert!(matches!(
            post(vault, Arc::new(instancius), post_path_params, body,).await,
            PostResponse::Status400_MalformedRequest(_)
        ));
    }

    #[tokio::test]
    async fn post_400_address_out_of_range() {
        const INSTANCE_ID: InstanceId = InstanceId::new(10);
        let post_path_params = PostPathParams {
            instance_id: INSTANCE_ID.to_string(),
        };
        let ip_address = Ipv4Addr::new(10, 18, 102, 10);
        let body = PostBody {
            network_id: NETWORK_ID.to_string(),
            ip_address_suggestion: Some(ip_address.to_string()),
        };
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_connect_instance_to_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(NETWORK_ID.to_string()),
                predicate::eq(INSTANCE_ID),
                predicate::eq(Some(Ipv4Addr::new(10, 18, 102, 10))),
            )
            .returning(move |_, _, _, _| {
                Err(ConnectInstanceConfigNetworkError::AddressOutOfRange {
                    address: IpAddr::V4(ip_address),
                    network: NETWORK_ID.to_string(),
                })
            });
        assert!(matches!(
            post(vault, Arc::new(instancius), post_path_params, body,).await,
            PostResponse::Status400_MalformedRequest(_)
        ));
    }

    #[tokio::test]
    async fn post_500() {
        const INSTANCE_ID: InstanceId = InstanceId::new(10);
        let post_path_params = PostPathParams {
            instance_id: INSTANCE_ID.to_string(),
        };
        let ip_address = Ipv4Addr::new(10, 18, 102, 10);
        let body = PostBody {
            network_id: NETWORK_ID.to_string(),
            ip_address_suggestion: Some(ip_address.to_string()),
        };
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_connect_instance_to_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(NETWORK_ID.to_string()),
                predicate::eq(INSTANCE_ID),
                predicate::eq(Some(Ipv4Addr::new(10, 18, 102, 10))),
            )
            .returning(move |_, _, _, _| {
                Err(ConnectInstanceConfigNetworkError::Other(
                    "TestError".to_string(),
                ))
            });
        assert!(matches!(
            post(vault, Arc::new(instancius), post_path_params, body,).await,
            PostResponse::Status500_InternalServerError(_)
        ));
    }

    #[tokio::test]
    async fn post_404() {
        const INSTANCE_ID: InstanceId = InstanceId::new(10);
        let post_path_params = PostPathParams {
            instance_id: INSTANCE_ID.to_string(),
        };
        let ip_address = Ipv4Addr::new(10, 18, 102, 10);
        let body = PostBody {
            network_id: NETWORK_ID.to_string(),
            ip_address_suggestion: Some(ip_address.to_string()),
        };
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_connect_instance_to_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(NETWORK_ID.to_string()),
                predicate::eq(INSTANCE_ID),
                predicate::eq(Some(Ipv4Addr::new(10, 18, 102, 10))),
            )
            .returning(move |_, _, _, _| {
                Err(ConnectInstanceConfigNetworkError::InstanceNotFound(
                    INSTANCE_ID,
                ))
            });
        assert_eq!(
            post(vault, Arc::new(instancius), post_path_params, body,).await,
            PostResponse::Status404_InstanceIdOrNetworkNotFound
        );
    }
}
