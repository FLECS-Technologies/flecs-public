use crate::ffi;
use flecs_core::Result;
use flecs_core::relic::network;
use flecs_core::relic::network::{NetworkAdapterReader, NetworkAdapterReaderImpl};

pub fn read_network_adapters() -> Result<Vec<ffi::NetAdapter>> {
    Ok(NetworkAdapterReaderImpl
        .try_read_network_adapters()?
        .into_iter()
        .map(|(name, info)| ffi::NetAdapter {
            name,
            info: info.into(),
        })
        .collect())
}

impl From<network::NetworkAdapter> for ffi::NetInfo {
    fn from(value: network::NetworkAdapter) -> Self {
        Self {
            mac: value.mac.unwrap_or_default(),
            net_type: value.net_type.into(),
            ipv4addresses: value
                .ipv4_networks
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            ipv6addresses: value
                .ipv6_networks
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            gateway: value.gateway.map(|ip| ip.to_string()).unwrap_or_default(),
        }
    }
}

impl From<network::NetType> for ffi::NetType {
    fn from(value: network::NetType) -> Self {
        match value {
            network::NetType::Unknown => ffi::NetType::Unknown,
            network::NetType::Wired => ffi::NetType::Wired,
            network::NetType::Wireless => ffi::NetType::Wireless,
            network::NetType::Local => ffi::NetType::Local,
            network::NetType::Bridge => ffi::NetType::Bridge,
            network::NetType::Virtual => ffi::NetType::Virtual,
        }
    }
}

impl From<network::Ipv4Network> for ffi::IpAddr {
    fn from(value: network::Ipv4Network) -> Self {
        Self {
            addr: value.address().to_string(),
            subnet_mask: value.subnet_mask().to_string(),
        }
    }
}

impl From<network::Ipv6Network> for ffi::IpAddr {
    fn from(value: network::Ipv6Network) -> Self {
        Self {
            addr: value.address().to_string(),
            subnet_mask: value.subnet_mask().to_string(),
        }
    }
}
