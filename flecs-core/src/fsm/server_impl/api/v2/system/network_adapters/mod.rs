use crate::relic::device::net::NetDeviceReader;
use crate::relic::network::NetworkAdapterReader;
use crate::sorcerer::systemus::Systemus;
use flecsd_axum_server::apis::system::SystemNetworkAdaptersGetResponse as GetResponse;
use flecsd_axum_server::models::AdditionalInfo;
use std::sync::Arc;

pub mod network_adapter_id;

pub fn get(
    systemus: Arc<dyn Systemus>,
    network_adapter_reader: Arc<dyn NetworkAdapterReader>,
    net_device_reader: Arc<dyn NetDeviceReader>,
) -> GetResponse {
    match systemus.read_network_adapters(&*network_adapter_reader) {
        Ok(network_adapters) => GetResponse::Status200_Success(
            network_adapters
                .into_values()
                .map(|adapter| {
                    network_adapter_id::create_network_adapter_model(adapter, &*net_device_reader)
                })
                .collect(),
        ),
        Err(e) => GetResponse::Status500_InternalServerError(AdditionalInfo::new(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relic::device::net::MockNetDeviceReader;
    use crate::relic::network::{
        Ipv4Network, Ipv6Network, MockNetworkAdapterReader, NetType, NetworkAdapter,
    };
    use crate::sorcerer::systemus::MockSystemus;
    use flecsd_axum_server::models;
    use mockall::predicate::eq;
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::str::FromStr;

    #[test]
    fn get_200() {
        let mut systemus = MockSystemus::new();
        systemus
            .expect_read_network_adapters()
            .once()
            .return_once(|_| {
                Ok(HashMap::from([(
                    "TestNet".to_string(),
                    NetworkAdapter {
                        name: "TestNet".to_string(),
                        mac: Some("D7:62:A1:BB:35:80".to_string()),
                        net_type: NetType::Wired,
                        ipv4_networks: vec![Ipv4Network::new_from_address_and_subnet_mask(
                            Ipv4Addr::new(192, 168, 0, 0),
                            Ipv4Addr::new(255, 255, 0, 0),
                        )
                        .unwrap()],
                        ipv6_networks: vec![Ipv6Network::new(
                            Ipv6Addr::new(0x1234, 0x5678, 0x90ab, 0xcdef, 0xaabb, 0xccdd, 0, 0),
                            96,
                        )],
                        ip_addresses: vec![
                            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20)),
                            IpAddr::V6(Ipv6Addr::new(
                                0x1234, 0x5678, 0x90ab, 0xcdef, 0xaabb, 0xccdd, 0, 0x1111,
                            )),
                        ],
                        gateway: Some(Ipv4Addr::new(192, 168, 1, 1)),
                    },
                )]))
            });
        let systemus = Arc::new(systemus);
        let network_adapter_reader: Arc<dyn NetworkAdapterReader> =
            Arc::new(MockNetworkAdapterReader::default());
        let mut net_device_reader = MockNetDeviceReader::default();
        net_device_reader
            .expect_get_net_property()
            .once()
            .with(eq("TestNet"), eq("carrier"))
            .returning(|_, _| Ok("1".to_string()));
        let net_device_reader: Arc<dyn NetDeviceReader> = Arc::new(net_device_reader);
        assert_eq!(
            get(systemus, network_adapter_reader, net_device_reader),
            GetResponse::Status200_Success(vec![models::NetworkAdapter {
                name: "TestNet".to_string(),
                ipv4_addresses: Some(vec![models::Ipv4Address::from_str("192.168.1.20").unwrap()]),
                ipv6_addresses: Some(vec![models::Ipv6Address::from_str(
                    "1234:5678:90ab:cdef:aabb:ccdd:0:1111"
                )
                .unwrap()]),
                networks: Some(vec![
                    models::Network::Ipv4Network(Box::new(models::Ipv4Network::new(
                        "192.168.0.0".to_string(),
                        "255.255.0.0".to_string()
                    ))),
                    models::Network::Ipv6Network(Box::new(models::Ipv6Network::new(
                        "1234:5678:90ab:cdef:aabb:ccdd::".to_string(),
                        96
                    )))
                ]),
                gateway: Some("192.168.1.1".to_string()),
                mac_address: Some("D7:62:A1:BB:35:80".to_string()),
                net_type: models::NetworkType::Wired,
                is_connected: true,
            }])
        );
    }

    #[test]
    fn get_500() {
        let mut systemus = MockSystemus::new();
        systemus
            .expect_read_network_adapters()
            .once()
            .return_once(|_| Err(anyhow::anyhow!("TestError")));
        let systemus = Arc::new(systemus);
        let network_adapter_reader: Arc<dyn NetworkAdapterReader> =
            Arc::new(MockNetworkAdapterReader::default());
        let net_device_reader: Arc<dyn NetDeviceReader> = Arc::new(MockNetDeviceReader::default());
        assert!(matches!(
            get(systemus, network_adapter_reader, net_device_reader),
            GetResponse::Status500_InternalServerError(_)
        ));
    }
}
