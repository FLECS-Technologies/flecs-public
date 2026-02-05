mod ipv4;
pub use ipv4::*;
pub use net_spider::network_adapter::NetType;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};

pub fn get_random_free_port() -> crate::Result<u16> {
    let bind = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))?;
    Ok(bind.local_addr()?.port())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use ipnet::{Ipv4Net, Ipv6Net};
    use mockall::mock;
    use net_spider::network_adapter::NetworkAdapter;
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv6Addr};
    use std::str::FromStr;

    #[cfg(test)]
    mock! {
        pub NetworkAdapterReader {}
        impl net_spider::network_adapter::NetworkAdapterReader for NetworkAdapterReader {
            fn try_read_network_adapters(&self) -> net_spider::Result<HashMap<String, NetworkAdapter>>;
        }
    }

    #[cfg(test)]
    mock! {
        pub NetDeviceReader {}
        impl net_spider::net_device::NetDeviceReader for NetDeviceReader {
            fn get_net_property(&self, net_adapter: &str, property_name: &str) -> net_spider::Result<String>;
        }
    }

    pub fn minimal_network_adapter() -> NetworkAdapter {
        NetworkAdapter {
            name: "TestAdapterMinimal".to_string(),
            mac: None,
            net_type: Default::default(),
            ipv4_networks: vec![],
            ipv6_networks: vec![],
            ip_addresses: vec![],
            gateway: None,
        }
    }

    pub fn full_network_adapter() -> NetworkAdapter {
        NetworkAdapter {
            name: "TestAdapterFull".to_string(),
            mac: Some("D7:60:A1:12:35:80".to_string()),
            net_type: NetType::Wireless,
            ipv4_networks: vec![Ipv4Net::from_str("22.41.0.0/16").unwrap()],
            ipv6_networks: vec![
                Ipv6Net::from_str("15a1:b1ac::/33").unwrap(),
                Ipv6Net::from_str("d5f0:bf0f:7ec0::/45").unwrap(),
            ],
            ip_addresses: vec![
                IpAddr::V6(Ipv6Addr::from_str("15a1:b1ac::12").unwrap()),
                IpAddr::V6(Ipv6Addr::from_str("d5f0:bf0f:7ec0::1:100").unwrap()),
                IpAddr::V4(Ipv4Addr::new(22, 41, 12, 11)),
                IpAddr::V4(Ipv4Addr::new(22, 41, 87, 55)),
            ],
            gateway: Some(Ipv4Addr::new(22, 41, 0, 1)),
        }
    }

    pub fn test_adapters() -> HashMap<String, NetworkAdapter> {
        let min = minimal_network_adapter();
        let full = full_network_adapter();
        HashMap::from([(min.name.clone(), min), (full.name.clone(), full)])
    }

    #[test]
    fn random_port_test() {
        let random_port = get_random_free_port().unwrap();
        TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, random_port)).unwrap();
    }
}
