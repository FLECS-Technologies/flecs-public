pub mod network_id;

use crate::relic::network::NetworkAdapterReader;
use crate::sorcerer::instancius::{ConnectInstanceConfigNetworkError, Instancius};
use crate::vault::pouch::instance::InstanceId;
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::InstancesInstanceIdConfigNetworksGetResponse as GetResponse;
use flecsd_axum_server::apis::instances::InstancesInstanceIdConfigNetworksPostResponse as PostResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::InstancesInstanceIdConfigNetworksGetPathParams as GetPathParams;
use flecsd_axum_server::models::InstancesInstanceIdConfigNetworksPostPathParams as PostPathParams;
use flecsd_axum_server::models::{AdditionalInfo, InstanceConnectToNetworkAdapter as PostBody};
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
        Some(networks) => GetResponse::Status200_Success(
            networks
                .into_iter()
                .map(|(id, ip)| models::InstanceConfigNetwork::new(id, ip.to_string()))
                .collect(),
        ),
        None => GetResponse::Status404_InstanceIdNotFound,
    }
}

pub async fn post<T: Instancius, N: NetworkAdapterReader + 'static>(
    vault: Arc<Vault>,
    instancius: Arc<T>,
    net_reader: Arc<N>,
    path_params: PostPathParams,
    body: PostBody,
) -> PostResponse {
    let instance_id =
        crate::jeweler::gem::instance::InstanceId::from_str(&path_params.instance_id).unwrap();
    let ip = match body.ip_address.map(|ip| Ipv4Addr::from_str(&ip)) {
        None => None,
        Some(Ok(ip)) => Some(ip),
        Some(Err(e)) => {
            return PostResponse::Status400_MalformedRequest(AdditionalInfo::new(format!(
                "Failed to parse ip from body: {e}"
            )))
        }
    };
    let adapter = match net_reader
        .try_read_network_adapters()
        .map(|mut adapters| adapters.remove(&body.network_adapter))
    {
        Err(e) => {
            return PostResponse::Status500_InternalServerError(AdditionalInfo::new(format!(
                "Failed to read network adapters: {e}"
            )))
        }
        Ok(None) => return PostResponse::Status404_InstanceIdOrNetworkAdapterNotFound,
        Ok(Some(adapter)) => adapter,
    };
    match instancius
        .connect_instance_to_network_adapter(vault, adapter, instance_id, ip)
        .await
    {
        Ok((_, Some(_))) => PostResponse::Status200_InstanceWasAlreadyConnected,
        Ok((_, None)) => PostResponse::Status201_InstanceConnected,
        Err(ConnectInstanceConfigNetworkError::InstanceNotFound(_)) => {
            PostResponse::Status404_InstanceIdOrNetworkAdapterNotFound
        }
        Err(e @ ConnectInstanceConfigNetworkError::AddressOutOfRange { .. }) => {
            PostResponse::Status400_MalformedRequest(AdditionalInfo::new(e.to_string()))
        }
        Err(e) => PostResponse::Status500_InternalServerError(AdditionalInfo::new(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relic::network::{MockNetworkAdapterReader, NetType, NetworkAdapter};
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

    #[tokio::test]
    async fn post_200() {
        const INSTANCE_ID: InstanceId = InstanceId::new(10);
        let post_path_params = PostPathParams {
            instance_id: INSTANCE_ID.to_string(),
        };
        let ip_address = Ipv4Addr::new(10, 18, 102, 10);
        let body = PostBody {
            network_adapter: "test-adapter".to_string(),
            ip_address: Some(ip_address.to_string()),
        };
        let network_adapter = NetworkAdapter {
            name: "test-adapter".to_string(),
            mac: None,
            net_type: NetType::Wireless,
            ipv4_networks: vec![],
            ipv6_networks: vec![],
            ip_addresses: vec![],
            gateway: None,
        };
        let expected_network_adapter = network_adapter.clone();
        let mut network_adapter_reader = MockNetworkAdapterReader::new();
        network_adapter_reader
            .expect_try_read_network_adapters()
            .once()
            .returning(move || {
                Ok(HashMap::from([(
                    expected_network_adapter.name.clone(),
                    expected_network_adapter.clone(),
                )]))
            });
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_connect_instance_to_network_adapter()
            .once()
            .with(
                predicate::always(),
                predicate::eq(network_adapter),
                predicate::eq(INSTANCE_ID),
                predicate::eq(Some(Ipv4Addr::new(10, 18, 102, 10))),
            )
            .returning(move |_, _, _, _| {
                Ok((IpAddr::V4(ip_address), Some(IpAddr::V4(ip_address))))
            });
        assert_eq!(
            post(
                vault,
                Arc::new(instancius),
                Arc::new(network_adapter_reader),
                post_path_params,
                body,
            )
            .await,
            PostResponse::Status200_InstanceWasAlreadyConnected
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
            network_adapter: "test-adapter".to_string(),
            ip_address: None,
        };
        let network_adapter = NetworkAdapter {
            name: "test-adapter".to_string(),
            mac: None,
            net_type: NetType::Wireless,
            ipv4_networks: vec![],
            ipv6_networks: vec![],
            ip_addresses: vec![],
            gateway: None,
        };
        let expected_network_adapter = network_adapter.clone();
        let mut network_adapter_reader = MockNetworkAdapterReader::new();
        network_adapter_reader
            .expect_try_read_network_adapters()
            .once()
            .returning(move || {
                Ok(HashMap::from([(
                    expected_network_adapter.name.clone(),
                    expected_network_adapter.clone(),
                )]))
            });
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_connect_instance_to_network_adapter()
            .once()
            .with(
                predicate::always(),
                predicate::eq(network_adapter),
                predicate::eq(INSTANCE_ID),
                predicate::eq(None),
            )
            .returning(move |_, _, _, _| Ok((IpAddr::V4(ip_address), None)));
        assert_eq!(
            post(
                vault,
                Arc::new(instancius),
                Arc::new(network_adapter_reader),
                post_path_params,
                body,
            )
            .await,
            PostResponse::Status201_InstanceConnected
        );
    }

    #[tokio::test]
    async fn post_400_invalid_ip() {
        const INSTANCE_ID: InstanceId = InstanceId::new(10);
        let post_path_params = PostPathParams {
            instance_id: INSTANCE_ID.to_string(),
        };
        let body = PostBody {
            network_adapter: "test-adapter".to_string(),
            ip_address: Some("invalid ip".to_string()),
        };
        let network_adapter_reader = MockNetworkAdapterReader::new();
        let vault = create_empty_test_vault();
        let instancius = MockInstancius::new();
        assert!(matches!(
            post(
                vault,
                Arc::new(instancius),
                Arc::new(network_adapter_reader),
                post_path_params,
                body,
            )
            .await,
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
            network_adapter: "test-adapter".to_string(),
            ip_address: Some(ip_address.to_string()),
        };
        let network_adapter = NetworkAdapter {
            name: "test-adapter".to_string(),
            mac: None,
            net_type: NetType::Wireless,
            ipv4_networks: vec![],
            ipv6_networks: vec![],
            ip_addresses: vec![],
            gateway: None,
        };
        let expected_network_adapter = network_adapter.clone();
        let mut network_adapter_reader = MockNetworkAdapterReader::new();
        network_adapter_reader
            .expect_try_read_network_adapters()
            .once()
            .returning(move || {
                Ok(HashMap::from([(
                    expected_network_adapter.name.clone(),
                    expected_network_adapter.clone(),
                )]))
            });
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_connect_instance_to_network_adapter()
            .once()
            .with(
                predicate::always(),
                predicate::eq(network_adapter),
                predicate::eq(INSTANCE_ID),
                predicate::eq(Some(Ipv4Addr::new(10, 18, 102, 10))),
            )
            .returning(move |_, _, _, _| {
                Err(ConnectInstanceConfigNetworkError::AddressOutOfRange {
                    address: IpAddr::V4(ip_address),
                    adapter: "test-adapter".to_string(),
                })
            });
        assert!(matches!(
            post(
                vault,
                Arc::new(instancius),
                Arc::new(network_adapter_reader),
                post_path_params,
                body,
            )
            .await,
            PostResponse::Status400_MalformedRequest(_)
        ));
    }

    #[tokio::test]
    async fn post_500_read_adapters() {
        const INSTANCE_ID: InstanceId = InstanceId::new(10);
        let post_path_params = PostPathParams {
            instance_id: INSTANCE_ID.to_string(),
        };
        let ip_address = Ipv4Addr::new(10, 18, 102, 10);
        let body = PostBody {
            network_adapter: "test-adapter".to_string(),
            ip_address: Some(ip_address.to_string()),
        };
        let mut network_adapter_reader = MockNetworkAdapterReader::new();
        network_adapter_reader
            .expect_try_read_network_adapters()
            .once()
            .returning(|| {
                Err(crate::relic::network::Error::InvalidNetwork(
                    "TestError".to_string(),
                ))
            });
        let vault = create_empty_test_vault();
        let instancius = MockInstancius::new();
        assert!(matches!(
            post(
                vault,
                Arc::new(instancius),
                Arc::new(network_adapter_reader),
                post_path_params,
                body,
            )
            .await,
            PostResponse::Status500_InternalServerError(_)
        ));
    }

    #[tokio::test]
    async fn post_500_connect() {
        const INSTANCE_ID: InstanceId = InstanceId::new(10);
        let post_path_params = PostPathParams {
            instance_id: INSTANCE_ID.to_string(),
        };
        let ip_address = Ipv4Addr::new(10, 18, 102, 10);
        let body = PostBody {
            network_adapter: "test-adapter".to_string(),
            ip_address: Some(ip_address.to_string()),
        };
        let network_adapter = NetworkAdapter {
            name: "test-adapter".to_string(),
            mac: None,
            net_type: NetType::Wireless,
            ipv4_networks: vec![],
            ipv6_networks: vec![],
            ip_addresses: vec![],
            gateway: None,
        };
        let expected_network_adapter = network_adapter.clone();
        let mut network_adapter_reader = MockNetworkAdapterReader::new();
        network_adapter_reader
            .expect_try_read_network_adapters()
            .once()
            .returning(move || {
                Ok(HashMap::from([(
                    expected_network_adapter.name.clone(),
                    expected_network_adapter.clone(),
                )]))
            });
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_connect_instance_to_network_adapter()
            .once()
            .with(
                predicate::always(),
                predicate::eq(network_adapter),
                predicate::eq(INSTANCE_ID),
                predicate::eq(Some(Ipv4Addr::new(10, 18, 102, 10))),
            )
            .returning(move |_, _, _, _| {
                Err(ConnectInstanceConfigNetworkError::Other(
                    "TestError".to_string(),
                ))
            });
        assert!(matches!(
            post(
                vault,
                Arc::new(instancius),
                Arc::new(network_adapter_reader),
                post_path_params,
                body,
            )
            .await,
            PostResponse::Status500_InternalServerError(_)
        ));
    }

    #[tokio::test]
    async fn post_404_instance() {
        const INSTANCE_ID: InstanceId = InstanceId::new(10);
        let post_path_params = PostPathParams {
            instance_id: INSTANCE_ID.to_string(),
        };
        let ip_address = Ipv4Addr::new(10, 18, 102, 10);
        let body = PostBody {
            network_adapter: "test-adapter".to_string(),
            ip_address: Some(ip_address.to_string()),
        };
        let network_adapter = NetworkAdapter {
            name: "test-adapter".to_string(),
            mac: None,
            net_type: NetType::Wireless,
            ipv4_networks: vec![],
            ipv6_networks: vec![],
            ip_addresses: vec![],
            gateway: None,
        };
        let expected_network_adapter = network_adapter.clone();
        let mut network_adapter_reader = MockNetworkAdapterReader::new();
        network_adapter_reader
            .expect_try_read_network_adapters()
            .once()
            .returning(move || {
                Ok(HashMap::from([(
                    expected_network_adapter.name.clone(),
                    expected_network_adapter.clone(),
                )]))
            });
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_connect_instance_to_network_adapter()
            .once()
            .with(
                predicate::always(),
                predicate::eq(network_adapter),
                predicate::eq(INSTANCE_ID),
                predicate::eq(Some(Ipv4Addr::new(10, 18, 102, 10))),
            )
            .returning(move |_, _, _, _| {
                Err(ConnectInstanceConfigNetworkError::InstanceNotFound(
                    INSTANCE_ID,
                ))
            });
        assert_eq!(
            post(
                vault,
                Arc::new(instancius),
                Arc::new(network_adapter_reader),
                post_path_params,
                body,
            )
            .await,
            PostResponse::Status404_InstanceIdOrNetworkAdapterNotFound
        );
    }

    #[tokio::test]
    async fn post_404_adapter() {
        const INSTANCE_ID: InstanceId = InstanceId::new(10);
        let post_path_params = PostPathParams {
            instance_id: INSTANCE_ID.to_string(),
        };
        let ip_address = Ipv4Addr::new(10, 18, 102, 10);
        let body = PostBody {
            network_adapter: "test-adapter".to_string(),
            ip_address: Some(ip_address.to_string()),
        };
        let mut network_adapter_reader = MockNetworkAdapterReader::new();
        network_adapter_reader
            .expect_try_read_network_adapters()
            .once()
            .returning(|| Ok(HashMap::new()));
        let vault = create_empty_test_vault();
        let instancius = MockInstancius::new();
        assert_eq!(
            post(
                vault,
                Arc::new(instancius),
                Arc::new(network_adapter_reader),
                post_path_params,
                body,
            )
            .await,
            PostResponse::Status404_InstanceIdOrNetworkAdapterNotFound
        );
    }
}
