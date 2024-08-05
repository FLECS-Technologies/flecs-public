use crate::ffi;
use flecs_core::relic::network;

pub fn read_network_adapters() -> Result<Vec<ffi::NetAdapter>, network::Error> {
    Ok(network::NetInfo::try_read_from_system()?
        .into_iter()
        .map(|(name, info)| ffi::NetAdapter {
            name,
            info: info.into(),
        })
        .collect())
}

impl From<network::NetInfo> for ffi::NetInfo {
    fn from(value: network::NetInfo) -> Self {
        Self {
            mac: value.mac,
            net_type: value.net_type.into(),
            ipv4addresses: value
                .ipv4addresses
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            ipv6addresses: value
                .ipv6addresses
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),

            gateway: value.gateway,
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

impl From<network::IpAddr> for ffi::IpAddr {
    fn from(value: network::IpAddr) -> Self {
        Self {
            addr: value.addr,
            subnet_mask: value.subnet_mask,
        }
    }
}
