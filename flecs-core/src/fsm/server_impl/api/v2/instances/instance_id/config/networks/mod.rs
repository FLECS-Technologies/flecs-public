
use crate::sorcerer::instancius::Instancius;
use crate::vault::pouch::instance::InstanceId;
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::InstancesInstanceIdConfigNetworksGetResponse as GetResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::InstancesInstanceIdConfigNetworksGetPathParams as GetPathParams;
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
        Some(networks) => GetResponse::Status200_Success(
            networks
                .into_iter()
                .map(|(id, ip)| models::InstanceConfigNetwork::new(id, ip.to_string()))
                .collect(),
        ),
        None => GetResponse::Status404_InstanceIdNotFound,
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
                Some(HashMap::from([(
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
            .returning(|_, _| None);
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
}
