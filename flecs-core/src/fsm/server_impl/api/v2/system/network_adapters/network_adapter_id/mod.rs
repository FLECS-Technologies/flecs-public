use crate::forge::vec::VecExtension;
use crate::sorcerer::systemus::Systemus;
use flecsd_axum_server::apis::system::SystemNetworkAdaptersNetworkAdapterIdGetResponse as GetResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::SystemNetworkAdaptersNetworkAdapterIdGetPathParams as GetPathParams;
use ipnet::IpNet;
use ipnet::{Ipv4Net, Ipv6Net};
use net_spider::net_device::NetDeviceReader;
use net_spider::network_adapter::{NetType, NetworkAdapter, NetworkAdapterReader};
use std::net::IpAddr;
use std::sync::Arc;

pub fn get(
    systemus: Arc<dyn Systemus>,
    network_adapter_reader: Arc<dyn NetworkAdapterReader>,
    net_device_reader: Arc<dyn NetDeviceReader>,
    path_params: GetPathParams,
) -> GetResponse {
    match systemus.read_network_adapter(&*network_adapter_reader, &path_params.network_adapter_id) {
        Ok(Some(network_adapter)) => GetResponse::Status200_Success(create_network_adapter_model(
            network_adapter,
            &*net_device_reader,
        )),
        Ok(None) => GetResponse::Status404_NetworkAdapterNotFound,
        Err(e) => {
            GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

fn convert_addresses(
    addresses: Vec<IpAddr>,
) -> (Vec<models::Ipv4Address>, Vec<models::Ipv6Address>) {
    let (mut ipv4, mut ipv6) = (vec![], vec![]);
    for address in addresses {
        match address {
            IpAddr::V4(addr) => ipv4.push(models::Ipv4Address::from(addr.to_string())),
            IpAddr::V6(addr) => ipv6.push(models::Ipv6Address::from(addr.to_string())),
        }
    }
    (ipv4, ipv6)
}

fn create_networks_model(
    ipv4_networks: Vec<Ipv4Net>,
    ipv6_networks: Vec<Ipv6Net>,
) -> Vec<models::Network> {
    ipv4_networks
        .into_iter()
        .map(IntoModel::into_model)
        .chain(ipv6_networks.into_iter().map(IntoModel::into_model))
        .collect()
}

pub fn create_network_adapter_model(
    value: NetworkAdapter,
    net_device_reader: &dyn NetDeviceReader,
) -> models::NetworkAdapter {
    let is_connected = value.is_connected(net_device_reader);
    let (ipv4_addresses, ipv6_addresses) = convert_addresses(value.ip_addresses);
    let networks = create_networks_model(value.ipv4_networks, value.ipv6_networks);
    models::NetworkAdapter {
        is_connected,
        name: value.name,
        ipv4_addresses: ipv4_addresses.empty_to_none(),
        ipv6_addresses: ipv6_addresses.empty_to_none(),
        networks: networks.empty_to_none(),
        gateway: value.gateway.as_ref().map(ToString::to_string),
        mac_address: value.mac,
        net_type: value.net_type.into_model(),
    }
}

trait IntoModel<M> {
    fn into_model(self) -> M;
}

impl IntoModel<models::Network> for IpNet {
    fn into_model(self) -> models::Network {
        match self {
            IpNet::V4(network) => models::Network::Ipv4Network(Box::new(network.into_model())),
            IpNet::V6(network) => models::Network::Ipv6Network(Box::new(network.into_model())),
        }
    }
}

impl IntoModel<models::Ipv4Network> for Ipv4Net {
    fn into_model(self) -> models::Ipv4Network {
        models::Ipv4Network {
            address: self.addr().to_string(),
            netmask: self.netmask().to_string(),
        }
    }
}

impl IntoModel<models::Ipv6Network> for Ipv6Net {
    fn into_model(self) -> models::Ipv6Network {
        models::Ipv6Network {
            address: self.addr().to_string(),
            prefix_len: self.prefix_len(),
        }
    }
}

impl IntoModel<models::Network> for Ipv4Net {
    fn into_model(self) -> models::Network {
        IpNet::V4(self).into_model()
    }
}

impl IntoModel<models::Network> for Ipv6Net {
    fn into_model(self) -> models::Network {
        IpNet::V6(self).into_model()
    }
}

impl IntoModel<models::NetworkType> for NetType {
    fn into_model(self) -> models::NetworkType {
        match self {
            NetType::Unknown => models::NetworkType::Unknown,
            NetType::Wired => models::NetworkType::Wired,
            NetType::Wireless => models::NetworkType::Wireless,
            NetType::Local => models::NetworkType::Local,
            NetType::Bridge => models::NetworkType::Bridge,
            NetType::Virtual => models::NetworkType::Virtual,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relic::network::tests::{
        MockNetDeviceReader, MockNetworkAdapterReader, full_network_adapter,
        minimal_network_adapter,
    };
    use crate::sorcerer::systemus::MockSystemus;
    use mockall::predicate::eq;
    use ntest::test_case;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use std::str::FromStr;

    fn convert_addresses_test(input: &[&str], expected_ipv4: &[&str], expected_ipv6: &[&str]) {
        let input: Vec<_> = input.iter().map(|s| IpAddr::from_str(s).unwrap()).collect();
        let expected_ipv4: Vec<_> = expected_ipv4
            .iter()
            .map(|s| models::Ipv4Address::from(s.to_string()))
            .collect();
        let expected_ipv6: Vec<_> = expected_ipv6
            .iter()
            .map(|s| models::Ipv6Address::from(s.to_string()))
            .collect();
        assert_eq!(convert_addresses(input), (expected_ipv4, expected_ipv6));
    }

    #[test]
    fn convert_addresses_empty() {
        convert_addresses_test(&[], &[], &[]);
    }

    #[test]
    fn convert_addresses_only_ipv4() {
        let ips = ["10.16.124.61", "252.251.6.10"];
        convert_addresses_test(&ips, &ips, &[]);
    }

    #[test]
    fn convert_addresses_only_ipv6() {
        let ips = [
            "e35d:8ba6:2009:bfe1:441c:be79:d80f:872",
            "b686:4c75:40b9:e9a2:5ac7:8016:4c70:3509",
            "2db8:8a61:27e0:1289:ef24:9112:257:e4c1",
            "d78b:41b3:cccc:573a:61f8:c5e6:a752:c743",
        ];
        convert_addresses_test(&ips, &[], &ips);
    }

    #[test]
    fn convert_addresses_mixed() {
        convert_addresses_test(
            &[
                "135.114.44.25",
                "207.146.25.33",
                "202.62.216.115",
                "131.101.44.65",
                "3c72:cd21:8bec:6f7e:63a2:9a08:7d21:82b0",
                "ba4a:a22e:c3ab:8b3e:c238:1604:9388:a623",
                "242.62.141.184",
                "36.162.232.13",
                "149.196.12.236",
                "20.199.113.70",
                "44ba:202a:782e:540f:f715:e79e:1e43:92c4",
                "34.34.191.49",
                "7460:2ac0:5def:3fa4:68be:4ae9:aaf4:c1d0",
                "209.165.216.73",
                "5734:aa6:b794:2a05:5353:7645:4327:19b7",
                "4fd5:83d3:a8ee:397f:6144:6c45:f3fb:4948",
            ],
            &[
                "135.114.44.25",
                "207.146.25.33",
                "202.62.216.115",
                "131.101.44.65",
                "242.62.141.184",
                "36.162.232.13",
                "149.196.12.236",
                "20.199.113.70",
                "34.34.191.49",
                "209.165.216.73",
            ],
            &[
                "3c72:cd21:8bec:6f7e:63a2:9a08:7d21:82b0",
                "ba4a:a22e:c3ab:8b3e:c238:1604:9388:a623",
                "44ba:202a:782e:540f:f715:e79e:1e43:92c4",
                "7460:2ac0:5def:3fa4:68be:4ae9:aaf4:c1d0",
                "5734:aa6:b794:2a05:5353:7645:4327:19b7",
                "4fd5:83d3:a8ee:397f:6144:6c45:f3fb:4948",
            ],
        );
    }

    fn create_networks_model_test(ipv4: &[&str], ipv6: &[&str], expected: Vec<models::Network>) {
        let ipv4 = ipv4
            .iter()
            .map(|ip| Ipv4Net::from_str(ip).unwrap())
            .collect();
        let ipv6 = ipv6
            .iter()
            .map(|ip| Ipv6Net::from_str(ip).unwrap())
            .collect();
        assert_eq!(create_networks_model(ipv4, ipv6), expected);
    }

    #[test]
    fn create_networks_model_empty() {
        create_networks_model_test(&[], &[], Vec::new());
    }

    #[test]
    fn create_networks_model_ipv4_only() {
        create_networks_model_test(
            &[
                "80.187.156.96/30",
                "62.128.0.0/9",
                "115.0.0.0/8",
                "203.167.192.0/18",
                "96.224.0.0/13",
            ],
            &[],
            vec![
                models::Network::Ipv4Network(Box::new(models::Ipv4Network::new(
                    "80.187.156.96".to_string(),
                    "255.255.255.252".to_string(),
                ))),
                models::Network::Ipv4Network(Box::new(models::Ipv4Network::new(
                    "62.128.0.0".to_string(),
                    "255.128.0.0".to_string(),
                ))),
                models::Network::Ipv4Network(Box::new(models::Ipv4Network::new(
                    "115.0.0.0".to_string(),
                    "255.0.0.0".to_string(),
                ))),
                models::Network::Ipv4Network(Box::new(models::Ipv4Network::new(
                    "203.167.192.0".to_string(),
                    "255.255.192.0".to_string(),
                ))),
                models::Network::Ipv4Network(Box::new(models::Ipv4Network::new(
                    "96.224.0.0".to_string(),
                    "255.248.0.0".to_string(),
                ))),
            ],
        );
    }

    #[test]
    fn create_networks_model_ipv6_only() {
        create_networks_model_test(
            &[],
            &[
                "e32:3580::/25",
                "ee13:25b8::/29",
                "fd07:325c:4000::/35",
                "908d:be78:7008:7800::/53",
                "2cf:9fa4::/30",
            ],
            vec![
                models::Network::Ipv6Network(Box::new(models::Ipv6Network::new(
                    "e32:3580::".to_string(),
                    25,
                ))),
                models::Network::Ipv6Network(Box::new(models::Ipv6Network::new(
                    "ee13:25b8::".to_string(),
                    29,
                ))),
                models::Network::Ipv6Network(Box::new(models::Ipv6Network::new(
                    "fd07:325c:4000::".to_string(),
                    35,
                ))),
                models::Network::Ipv6Network(Box::new(models::Ipv6Network::new(
                    "908d:be78:7008:7800::".to_string(),
                    53,
                ))),
                models::Network::Ipv6Network(Box::new(models::Ipv6Network::new(
                    "2cf:9fa4::".to_string(),
                    30,
                ))),
            ],
        );
    }

    #[test]
    fn create_networks_model_both() {
        create_networks_model_test(
            &[
                "221.151.155.0/24",
                "224.0.0.0/10",
                "74.217.128.0/18",
                "190.3.206.0/23",
                "239.32.0.0/14",
            ],
            &[
                "8e01:c700::/24",
                "4e3b:d054:9ba6:7cb4::/60",
                "c023:2914:8000::/34",
                "80c6::/16",
                "54e4:c9ab:7000::/38",
            ],
            vec![
                models::Network::Ipv4Network(Box::new(models::Ipv4Network::new(
                    "221.151.155.0".to_string(),
                    "255.255.255.0".to_string(),
                ))),
                models::Network::Ipv4Network(Box::new(models::Ipv4Network::new(
                    "224.0.0.0".to_string(),
                    "255.192.0.0".to_string(),
                ))),
                models::Network::Ipv4Network(Box::new(models::Ipv4Network::new(
                    "74.217.128.0".to_string(),
                    "255.255.192.0".to_string(),
                ))),
                models::Network::Ipv4Network(Box::new(models::Ipv4Network::new(
                    "190.3.206.0".to_string(),
                    "255.255.254.0".to_string(),
                ))),
                models::Network::Ipv4Network(Box::new(models::Ipv4Network::new(
                    "239.32.0.0".to_string(),
                    "255.252.0.0".to_string(),
                ))),
                models::Network::Ipv6Network(Box::new(models::Ipv6Network::new(
                    "8e01:c700::".to_string(),
                    24,
                ))),
                models::Network::Ipv6Network(Box::new(models::Ipv6Network::new(
                    "4e3b:d054:9ba6:7cb4::".to_string(),
                    60,
                ))),
                models::Network::Ipv6Network(Box::new(models::Ipv6Network::new(
                    "c023:2914:8000::".to_string(),
                    34,
                ))),
                models::Network::Ipv6Network(Box::new(models::Ipv6Network::new(
                    "80c6::".to_string(),
                    16,
                ))),
                models::Network::Ipv6Network(Box::new(models::Ipv6Network::new(
                    "54e4:c9ab:7000::".to_string(),
                    38,
                ))),
            ],
        );
    }

    #[test_case(NetType::Unknown, models::NetworkType::Unknown)]
    #[test_case(NetType::Wired, models::NetworkType::Wired)]
    #[test_case(NetType::Wireless, models::NetworkType::Wireless)]
    #[test_case(NetType::Local, models::NetworkType::Local)]
    #[test_case(NetType::Bridge, models::NetworkType::Bridge)]
    #[test_case(NetType::Virtual, models::NetworkType::Virtual)]
    fn model_from_net_type(source: NetType, expected: models::NetworkType) {
        assert_eq!(source.into_model(), expected);
    }

    #[test_case("10.20.30.0", 24, "255.255.255.0")]
    #[test_case("127.0.2.0", 24, "255.255.255.0")]
    #[test_case("100.0.0.0", 8, "255.0.0.0")]
    #[test_case("200.200.80.0", 20, "255.255.240.0")]
    fn model_from_ipv4_network(ip: &str, network_size: u8, expected_subnet_mask: &str) {
        let network = Ipv4Net::new_assert(Ipv4Addr::from_str(ip).unwrap(), network_size);
        let expected_model =
            models::Ipv4Network::new(ip.to_string(), expected_subnet_mask.to_string());
        assert_eq!(
            <Ipv4Net as IntoModel<models::Ipv4Network>>::into_model(network),
            expected_model
        );
        let expected_model = models::Network::Ipv4Network(Box::new(expected_model));
        assert_eq!(
            <Ipv4Net as IntoModel<models::Network>>::into_model(network),
            expected_model
        );
        assert_eq!(
            <IpNet as IntoModel<models::Network>>::into_model(IpNet::V4(network)),
            expected_model
        );
    }

    #[test_case("81f2:f385:4800::", 37)]
    #[test_case("86e5:6018:d00::", 44)]
    #[test_case("4761:45da:6::", 50)]
    #[test_case("b884:6129:db74:a800::", 53)]
    #[test_case("15a1:b1ac::", 33)]
    #[test_case("3cf9:2cff::", 33)]
    #[test_case("ffa4:aafb:9c26:3040::", 59)]
    #[test_case("b20d:a3e5:3857:b800::", 53)]
    #[test_case("7519:f47a:9000::", 37)]
    #[test_case("d5f0:bf0f:7ec0::", 45)]
    #[test_case("23f1:99b8:6000::", 35)]
    #[test_case("97a7:922d:5ec0::", 46)]
    #[test_case("edd3:206b:1e8f:f6c0::", 58)]
    #[test_case("e831:1727:7500::", 40)]
    #[test_case("b4c9:b860:e45e:b500::", 57)]
    #[test_case("71d4:f385:375a:e000::", 51)]
    #[test_case("383e:da05:7800::", 39)]
    #[test_case("9926:8e1a:47ee:c000::", 50)]
    #[test_case("5abd:c7f5:e300::", 43)]
    fn model_from_ipv6_network(ip: &str, prefix_len: u8) {
        let network = Ipv6Net::new_assert(Ipv6Addr::from_str(ip).unwrap(), prefix_len);
        let expected_model = models::Ipv6Network::new(ip.to_string(), prefix_len);
        assert_eq!(
            <Ipv6Net as IntoModel<models::Ipv6Network>>::into_model(network),
            expected_model
        );
        let expected_model = models::Network::Ipv6Network(Box::new(expected_model));
        assert_eq!(
            <Ipv6Net as IntoModel<models::Network>>::into_model(network),
            expected_model
        );
        assert_eq!(
            <IpNet as IntoModel<models::Network>>::into_model(IpNet::V6(network)),
            expected_model
        );
    }

    #[test]
    fn create_network_adapter_model_min_connected() {
        let adapter = minimal_network_adapter();
        let mut mock_net_device_reader = MockNetDeviceReader::new();
        mock_net_device_reader
            .expect_get_net_property()
            .once()
            .with(eq("TestAdapterMinimal"), eq("carrier"))
            .returning(|_, _| Ok("1".to_string()));
        let expected_network_adapter = models::NetworkAdapter {
            name: "TestAdapterMinimal".to_string(),
            ipv4_addresses: None,
            ipv6_addresses: None,
            networks: None,
            gateway: None,
            mac_address: None,
            net_type: models::NetworkType::Unknown,
            is_connected: true,
        };
        assert_eq!(
            create_network_adapter_model(adapter, &mock_net_device_reader),
            expected_network_adapter
        );
    }

    #[test]
    fn create_network_adapter_model_min_disconnected() {
        let adapter = minimal_network_adapter();
        let mut mock_net_device_reader = MockNetDeviceReader::new();
        mock_net_device_reader
            .expect_get_net_property()
            .once()
            .with(eq("TestAdapterMinimal"), eq("carrier"))
            .returning(|_, _| Ok("0".to_string()));
        let expected_network_adapter = models::NetworkAdapter {
            name: "TestAdapterMinimal".to_string(),
            ipv4_addresses: None,
            ipv6_addresses: None,
            networks: None,
            gateway: None,
            mac_address: None,
            net_type: models::NetworkType::Unknown,
            is_connected: false,
        };
        assert_eq!(
            create_network_adapter_model(adapter, &mock_net_device_reader),
            expected_network_adapter
        );
    }

    fn network_adapter_model_full() -> models::NetworkAdapter {
        models::NetworkAdapter {
            name: "TestAdapterFull".to_string(),
            ipv4_addresses: Some(vec![
                models::Ipv4Address::from("22.41.12.11".to_string()),
                models::Ipv4Address::from("22.41.87.55".to_string()),
            ]),
            ipv6_addresses: Some(vec![
                models::Ipv6Address::from("15a1:b1ac::12".to_string()),
                models::Ipv6Address::from("d5f0:bf0f:7ec0::1:100".to_string()),
            ]),
            networks: Some(vec![
                models::Network::Ipv4Network(Box::new(models::Ipv4Network::new(
                    "22.41.0.0".to_string(),
                    "255.255.0.0".to_string(),
                ))),
                models::Network::Ipv6Network(Box::new(models::Ipv6Network::new(
                    "15a1:b1ac::".to_string(),
                    33,
                ))),
                models::Network::Ipv6Network(Box::new(models::Ipv6Network::new(
                    "d5f0:bf0f:7ec0::".to_string(),
                    45,
                ))),
            ]),
            gateway: Some("22.41.0.1".to_string()),
            mac_address: Some("D7:60:A1:12:35:80".to_string()),
            net_type: models::NetworkType::Wireless,
            is_connected: false,
        }
    }

    #[test]
    fn create_network_adapter_model_full() {
        let adapter = full_network_adapter();
        let mut mock_net_device_reader = MockNetDeviceReader::new();
        mock_net_device_reader
            .expect_get_net_property()
            .once()
            .with(eq("TestAdapterFull"), eq("carrier"))
            .returning(|_, _| Ok("0".to_string()));
        let expected_network_adapter = network_adapter_model_full();
        assert_eq!(
            create_network_adapter_model(adapter, &mock_net_device_reader),
            expected_network_adapter
        );
    }

    #[test]
    fn get_200() {
        let mut systemus = MockSystemus::new();
        systemus
            .expect_read_network_adapter()
            .once()
            .returning(|_, _| Ok(Some(full_network_adapter())));
        let systemus = Arc::new(systemus);
        let network_adapter_reader: Arc<dyn NetworkAdapterReader> =
            Arc::new(MockNetworkAdapterReader::default());
        let mut net_device_reader = MockNetDeviceReader::default();
        net_device_reader
            .expect_get_net_property()
            .once()
            .with(eq("TestAdapterFull"), eq("carrier"))
            .returning(|_, _| Ok("0".to_string()));
        let net_device_reader: Arc<dyn NetDeviceReader> = Arc::new(net_device_reader);
        let parameters = GetPathParams {
            network_adapter_id: "TestAdapterFull".to_string(),
        };
        assert_eq!(
            get(
                systemus,
                network_adapter_reader,
                net_device_reader,
                parameters
            ),
            GetResponse::Status200_Success(network_adapter_model_full())
        )
    }

    #[test]
    fn get_404() {
        let mut systemus = MockSystemus::new();
        systemus
            .expect_read_network_adapter()
            .once()
            .returning(|_, _| Ok(None));
        let systemus = Arc::new(systemus);
        let network_adapter_reader: Arc<dyn NetworkAdapterReader> =
            Arc::new(MockNetworkAdapterReader::default());
        let net_device_reader: Arc<dyn NetDeviceReader> = Arc::new(MockNetDeviceReader::default());
        let parameters = GetPathParams {
            network_adapter_id: "TestAdapter".to_string(),
        };
        assert_eq!(
            get(
                systemus,
                network_adapter_reader,
                net_device_reader,
                parameters
            ),
            GetResponse::Status404_NetworkAdapterNotFound
        )
    }

    #[test]
    fn get_500() {
        let mut systemus = MockSystemus::new();
        systemus
            .expect_read_network_adapter()
            .once()
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        let systemus = Arc::new(systemus);
        let network_adapter_reader: Arc<dyn NetworkAdapterReader> =
            Arc::new(MockNetworkAdapterReader::default());
        let net_device_reader: Arc<dyn NetDeviceReader> = Arc::new(MockNetDeviceReader::default());
        let parameters = GetPathParams {
            network_adapter_id: "TestAdapter".to_string(),
        };
        assert!(matches!(
            get(
                systemus,
                network_adapter_reader,
                net_device_reader,
                parameters
            ),
            GetResponse::Status500_InternalServerError(_)
        ))
    }
}
