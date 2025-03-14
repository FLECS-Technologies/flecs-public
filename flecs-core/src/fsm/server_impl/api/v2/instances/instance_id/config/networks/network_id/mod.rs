use crate::jeweler::gem::instance::InstanceId;
use crate::sorcerer::instancius::{
    DisconnectInstanceError, GetInstanceConfigNetworkResult, Instancius,
};
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::InstancesInstanceIdConfigNetworksNetworkIdDeleteResponse as DeleteResponse;
use flecsd_axum_server::apis::instances::InstancesInstanceIdConfigNetworksNetworkIdGetResponse as GetResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::InstancesInstanceIdConfigNetworksNetworkIdGetPathParams as GetPathParams;
use flecsd_axum_server::models::{
    AdditionalInfo, InstancesInstanceIdConfigNetworksNetworkIdDeletePathParams as DeletePathParams,
};
use std::str::FromStr;
use std::sync::Arc;

pub async fn get<T: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<T>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .get_instance_config_network(vault, instance_id, path_params.network_id)
        .await
    {
        GetInstanceConfigNetworkResult::InstanceNotFound => {
            GetResponse::Status404_InstanceIdOrNetworkNotFound
        }
        GetInstanceConfigNetworkResult::UnknownNetwork => {
            GetResponse::Status404_InstanceIdOrNetworkNotFound
        }
        GetInstanceConfigNetworkResult::Network { name, address } => {
            GetResponse::Status200_Success(models::InstanceConfigNetwork {
                name,
                ip_address: address.to_string(),
            })
        }
    }
}

pub async fn delete<T: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<T>,
    path_params: DeletePathParams,
) -> DeleteResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .disconnect_instance_from_network(vault, instance_id, path_params.network_id)
        .await
    {
        Ok(_) => DeleteResponse::Status200_Success,
        Err(DisconnectInstanceError::InstanceNotFound(_))
        | Err(DisconnectInstanceError::InstanceNotConnected { .. }) => {
            DeleteResponse::Status404_InstanceIdOrNetworkNotFound
        }
        Err(DisconnectInstanceError::Other(reason)) => {
            DeleteResponse::Status500_InternalServerError(AdditionalInfo::new(reason))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorcerer::instancius::MockInstancius;
    use crate::vault::tests::create_empty_test_vault;
    use flecsd_axum_server::models::InstanceConfigNetwork;
    use mockall::predicate;
    use std::net::{IpAddr, Ipv4Addr};

    const INSTANCE_ID: crate::vault::pouch::instance::InstanceId = InstanceId::new(20);
    const NETWORK_NAME: &str = "test-net";

    #[tokio::test]
    async fn get_200() {
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(INSTANCE_ID),
                predicate::eq(NETWORK_NAME.to_string()),
            )
            .returning(|_, _, _| GetInstanceConfigNetworkResult::Network {
                name: "test-net".to_string(),
                address: IpAddr::V4(Ipv4Addr::new(10, 20, 30, 40)),
            });
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: INSTANCE_ID.to_string(),
                    network_id: NETWORK_NAME.to_string(),
                },
            )
            .await,
            GetResponse::Status200_Success(InstanceConfigNetwork {
                name: NETWORK_NAME.to_string(),
                ip_address: "10.20.30.40".to_string()
            })
        );
    }

    #[tokio::test]
    async fn get_404_instance() {
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(INSTANCE_ID),
                predicate::eq(NETWORK_NAME.to_string()),
            )
            .returning(|_, _, _| GetInstanceConfigNetworkResult::InstanceNotFound);
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: INSTANCE_ID.to_string(),
                    network_id: NETWORK_NAME.to_string(),
                },
            )
            .await,
            GetResponse::Status404_InstanceIdOrNetworkNotFound
        );
    }

    #[tokio::test]
    async fn get_404_network() {
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(INSTANCE_ID),
                predicate::eq(NETWORK_NAME.to_string()),
            )
            .returning(|_, _, _| GetInstanceConfigNetworkResult::UnknownNetwork);
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: INSTANCE_ID.to_string(),
                    network_id: NETWORK_NAME.to_string(),
                },
            )
            .await,
            GetResponse::Status404_InstanceIdOrNetworkNotFound
        );
    }

    #[tokio::test]
    async fn delete_200() {
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_disconnect_instance_from_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(INSTANCE_ID),
                predicate::eq(NETWORK_NAME.to_string()),
            )
            .returning(|_, _, _| Ok(IpAddr::V4(Ipv4Addr::new(10, 20, 30, 40))));
        assert_eq!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: INSTANCE_ID.to_string(),
                    network_id: NETWORK_NAME.to_string(),
                },
            )
            .await,
            DeleteResponse::Status200_Success
        );
    }

    #[tokio::test]
    async fn delete_404_instance() {
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_disconnect_instance_from_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(INSTANCE_ID),
                predicate::eq(NETWORK_NAME.to_string()),
            )
            .returning(|_, _, _| Err(DisconnectInstanceError::InstanceNotFound(INSTANCE_ID)));
        assert_eq!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: INSTANCE_ID.to_string(),
                    network_id: NETWORK_NAME.to_string(),
                },
            )
            .await,
            DeleteResponse::Status404_InstanceIdOrNetworkNotFound
        );
    }

    #[tokio::test]
    async fn delete_404_network() {
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_disconnect_instance_from_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(INSTANCE_ID),
                predicate::eq(NETWORK_NAME.to_string()),
            )
            .returning(|_, _, _| {
                Err(DisconnectInstanceError::InstanceNotConnected {
                    network: NETWORK_NAME.to_string(),
                    instance: INSTANCE_ID,
                })
            });
        assert_eq!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: INSTANCE_ID.to_string(),
                    network_id: NETWORK_NAME.to_string(),
                },
            )
            .await,
            DeleteResponse::Status404_InstanceIdOrNetworkNotFound
        );
    }
}
