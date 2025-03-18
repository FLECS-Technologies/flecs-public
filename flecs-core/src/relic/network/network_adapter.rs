use crate::relic::device::net::{NetDeviceReader, NetDeviceReaderExt};
use crate::relic::network::ipv4::Ipv4Network;
use crate::relic::network::{Error, Ipv6Network, NetType};
use libc::{
    freeifaddrs, getifaddrs, ifaddrs, sockaddr_in, sockaddr_in6, sockaddr_ll, AF_INET, AF_INET6,
    AF_PACKET,
};
#[cfg(test)]
use mockall::automock;
use procfs::net::RouteEntry;
use std::collections::HashMap;
use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[cfg_attr(test, automock)]
pub trait NetworkAdapterReader: Send + Sync {
    fn try_read_network_adapters(
        &self,
    ) -> crate::relic::network::Result<HashMap<String, NetworkAdapter>>;
}

#[derive(Default)]
pub struct NetworkAdapterReaderImpl;

impl NetworkAdapterReader for NetworkAdapterReaderImpl {
    fn try_read_network_adapters(
        &self,
    ) -> crate::relic::network::Result<HashMap<String, NetworkAdapter>> {
        NetworkAdapter::try_read_from_system(IfAddrs::new()?)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkAdapter {
    pub name: String,
    pub mac: Option<String>,
    pub net_type: NetType,
    pub ipv4_networks: Vec<Ipv4Network>,
    pub ipv6_networks: Vec<Ipv6Network>,
    pub ip_addresses: Vec<IpAddr>,
    pub gateway: Option<Ipv4Addr>,
}

enum SaFamily {
    Unsupported(u16),
    Packet,
    Inet4,
    Inet6,
}

impl From<u16> for SaFamily {
    fn from(value: u16) -> Self {
        match value {
            x if x as i32 == AF_INET => SaFamily::Inet4,
            x if x as i32 == AF_INET6 => SaFamily::Inet6,
            x if x as i32 == AF_PACKET => SaFamily::Packet,
            x => SaFamily::Unsupported(x),
        }
    }
}

pub struct IfAddrs {
    inner: *mut ifaddrs,
}

#[derive(Debug, PartialEq, Clone)]
struct IfAddrsReadResult {
    name: String,
    address: IfAddrsReadResultAddress,
}

#[derive(Debug, PartialEq, Clone)]
enum IfAddrsReadResultAddress {
    Mac(String),
    Ipv4 {
        address: Ipv4Addr,
        subnet_mask: Ipv4Addr,
    },
    Ipv6 {
        address: Ipv6Addr,
        subnet_mask: Ipv6Addr,
    },
}

impl TryFrom<ifaddrs> for IfAddrsReadResult {
    type Error = Error;

    fn try_from(value: ifaddrs) -> crate::relic::network::Result<Self> {
        if value.ifa_addr.is_null() {
            return Err(Error::PropertyNull("ifa_addr".to_string()));
        }
        let sa_family: SaFamily = unsafe { *value.ifa_addr }.sa_family.into();
        let name = unsafe { CStr::from_ptr(value.ifa_name as *const _) }
            .to_string_lossy()
            .into_owned();
        match sa_family {
            SaFamily::Unsupported(val) => Err(Self::Error::UnsupportedSaFamily(val)),
            SaFamily::Packet => {
                let s = unsafe { *(value.ifa_addr as *const sockaddr_ll) };
                let mac = format!(
                    "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    s.sll_addr[0],
                    s.sll_addr[1],
                    s.sll_addr[2],
                    s.sll_addr[3],
                    s.sll_addr[4],
                    s.sll_addr[5]
                );

                Ok(IfAddrsReadResult {
                    name,
                    address: IfAddrsReadResultAddress::Mac(mac),
                })
            }
            SaFamily::Inet4 => {
                let s = unsafe { *(value.ifa_addr as *const sockaddr_in) };
                let address: Ipv4Addr = u32::from_be(s.sin_addr.s_addr).into();
                let s = unsafe { *(value.ifa_netmask as *const sockaddr_in) };
                let subnet_mask: Ipv4Addr = u32::from_be(s.sin_addr.s_addr).into();
                Ok(IfAddrsReadResult {
                    name,
                    address: IfAddrsReadResultAddress::Ipv4 {
                        address,
                        subnet_mask,
                    },
                })
            }
            SaFamily::Inet6 => {
                let s = unsafe { *(value.ifa_addr as *const sockaddr_in6) };
                let address: Ipv6Addr = s.sin6_addr.s6_addr.into();
                let s = unsafe { *(value.ifa_netmask as *const sockaddr_in6) };
                let subnet_mask: Ipv6Addr = s.sin6_addr.s6_addr.into();
                Ok(IfAddrsReadResult {
                    name,
                    address: IfAddrsReadResultAddress::Ipv6 {
                        address,
                        subnet_mask,
                    },
                })
            }
        }
    }
}

impl IfAddrs {
    #[allow(unsafe_code, clippy::new_ret_no_self)]
    pub fn new() -> std::io::Result<Self> {
        let mut ifaddrs: MaybeUninit<*mut ifaddrs> = MaybeUninit::uninit();

        let ifaddrs = unsafe {
            if -1 == getifaddrs(ifaddrs.as_mut_ptr()) {
                return Err(std::io::Error::last_os_error());
            }
            ifaddrs.assume_init()
        };

        Ok(Self { inner: ifaddrs })
    }
}

impl IntoIterator for IfAddrs {
    type Item = ifaddrs;
    type IntoIter = IfAddrsIterator;

    fn into_iter(self) -> Self::IntoIter {
        IfAddrsIterator {
            next: self.inner,
            _source: self,
        }
    }
}

impl Drop for IfAddrs {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                freeifaddrs(self.inner);
            }
        }
    }
}

pub struct IfAddrsIterator {
    _source: IfAddrs,
    next: *mut ifaddrs,
}

impl Iterator for IfAddrsIterator {
    type Item = ifaddrs;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        };

        Some(unsafe {
            let result = *self.next;
            self.next = (*self.next).ifa_next;

            result
        })
    }
}
impl NetworkAdapter {
    fn try_read_from_system(
        if_addrs: IfAddrs,
    ) -> crate::relic::network::Result<HashMap<String, Self>> {
        let mut adapters: HashMap<String, Self> = HashMap::new();
        let addresses = if_addrs
            .into_iter()
            .filter_map(|if_addrs| IfAddrsReadResult::try_from(if_addrs).ok());
        let route_entries = procfs::net::route()?
            .into_iter()
            .filter(|route_entry| route_entry.destination.is_unspecified());
        for result in addresses {
            let entry = adapters
                .entry(result.name.clone())
                .or_insert(Self::new(result.name.clone()));
            match result {
                IfAddrsReadResult {
                    address: IfAddrsReadResultAddress::Mac(mac),
                    ..
                } => entry.mac = Some(mac),
                IfAddrsReadResult {
                    address:
                        IfAddrsReadResultAddress::Ipv4 {
                            address,
                            subnet_mask,
                        },
                    ..
                } => {
                    entry.ipv4_networks.push(
                        Ipv4Network::new_from_address_and_subnet_mask(address, subnet_mask)
                            .map_err(|e| Error::InvalidNetwork(e.to_string()))?,
                    );
                    entry.ip_addresses.push(address.into());
                }
                IfAddrsReadResult {
                    address:
                        IfAddrsReadResultAddress::Ipv6 {
                            address,
                            subnet_mask,
                        },
                    ..
                } => {
                    entry
                        .ipv6_networks
                        .push(Ipv6Network::new_from_address_and_subnet_mask(
                            address,
                            subnet_mask,
                        ));
                    entry.ip_addresses.push(address.into());
                }
            }
        }
        for RouteEntry { iface, gateway, .. } in route_entries {
            let entry = adapters.entry(iface.clone()).or_insert(Self::new(iface));
            entry.gateway = Some(gateway);
        }
        Ok(adapters)
    }

    fn new(name: String) -> Self {
        Self {
            net_type: name.as_str().into(),
            ipv6_networks: Vec::new(),
            ipv4_networks: Vec::new(),
            gateway: None,
            name,
            mac: None,
            ip_addresses: Vec::new(),
        }
    }

    pub fn is_connected(&self, net_device_reader: &dyn NetDeviceReader) -> bool {
        net_device_reader.is_connected(self.name.as_str())
    }

    pub fn connected_networks_contain(&self, address: IpAddr) -> bool {
        match address {
            IpAddr::V4(address) => self
                .ipv4_networks
                .iter()
                .any(|network| network.contains(address)),
            IpAddr::V6(address) => self
                .ipv6_networks
                .iter()
                .any(|network| network.contains(address)),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use libc::{ifaddrs, sockaddr, sockaddr_in, sockaddr_in6, sockaddr_ll};
    use std::ffi::CString;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::str::FromStr;
    use std::{mem, ptr};

    fn new_test_ifaddr(
        name: Option<&str>,
        addr: Option<*mut sockaddr>,
        net_mask: Option<*mut sockaddr>,
    ) -> ifaddrs {
        let ifa_name = name.map_or(ptr::null_mut(), |name| {
            CString::new(name).unwrap().into_raw()
        });
        ifaddrs {
            ifa_next: ptr::null_mut(),
            ifa_name,
            ifa_flags: 2,
            ifa_addr: addr.unwrap_or(ptr::null_mut()),
            ifa_netmask: net_mask.unwrap_or(ptr::null_mut()),
            ifa_ifu: ptr::null_mut(),
            ifa_data: ptr::null_mut(),
        }
    }

    #[test]
    fn try_from_ifaddr_ok_packet() {
        let mut addr: Box<sockaddr_ll> = unsafe { Box::new(mem::zeroed()) };
        addr.sll_addr = [0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef];
        let addr = Box::into_raw(addr) as *mut sockaddr;
        unsafe { (*addr).sa_family = 17 };
        let if_addrs = new_test_ifaddr(Some("eth0"), Some(addr), None);
        assert_eq!(
            IfAddrsReadResult::try_from(if_addrs).unwrap(),
            IfAddrsReadResult {
                name: "eth0".to_string(),
                address: IfAddrsReadResultAddress::Mac("12:34:56:78:90:ab".to_string())
            }
        )
    }

    #[test]
    fn try_from_ifaddr_ok_inet4() {
        let mut addr: Box<sockaddr_in> = unsafe { Box::new(mem::zeroed()) };
        addr.sin_addr.s_addr = u32::to_be(Ipv4Addr::new(10, 20, 0, 0).into());
        let addr = Box::into_raw(addr) as *mut sockaddr;
        unsafe { (*addr).sa_family = 2 };
        let mut netmask: Box<sockaddr_in> = unsafe { Box::new(mem::zeroed()) };
        netmask.sin_addr.s_addr = u32::to_be(Ipv4Addr::new(255, 255, 0, 0).into());
        let netmask = Box::into_raw(netmask) as *mut sockaddr;
        unsafe { (*netmask).sa_family = 2 };
        let if_addrs = new_test_ifaddr(Some("eth0"), Some(addr), Some(netmask));
        assert_eq!(
            IfAddrsReadResult::try_from(if_addrs).unwrap(),
            IfAddrsReadResult {
                name: "eth0".to_string(),
                address: IfAddrsReadResultAddress::Ipv4 {
                    address: Ipv4Addr::new(10, 20, 0, 0),
                    subnet_mask: Ipv4Addr::new(255, 255, 0, 0)
                }
            }
        )
    }

    #[test]
    fn try_from_ifaddr_ok_inet6() {
        let mut addr: Box<sockaddr_in6> = unsafe { Box::new(mem::zeroed()) };
        addr.sin6_addr.s6_addr = [
            0x11, 0x22, 0x33, 0x44, 0xaa, 0xbb, 0xcc, 0xdd, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ];
        let addr = Box::into_raw(addr) as *mut sockaddr;
        unsafe { (*addr).sa_family = 10 };
        let mut netmask: Box<sockaddr_in6> = unsafe { Box::new(mem::zeroed()) };
        netmask.sin6_addr.s6_addr = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ];
        let netmask = Box::into_raw(netmask) as *mut sockaddr;
        unsafe { (*netmask).sa_family = 10 };
        let if_addrs = new_test_ifaddr(Some("wl2"), Some(addr), Some(netmask));
        assert_eq!(
            IfAddrsReadResult::try_from(if_addrs).unwrap(),
            IfAddrsReadResult {
                name: "wl2".to_string(),
                address: IfAddrsReadResultAddress::Ipv6 {
                    address: Ipv6Addr::from_str("1122:3344:aabb:ccdd::").unwrap(),
                    subnet_mask: Ipv6Addr::from_str("ffff:ffff:ffff:ffff::").unwrap()
                }
            }
        )
    }

    #[test]
    fn try_from_ifaddr_err_addr_null() {
        let mut addr: Box<sockaddr_ll> = unsafe { Box::new(mem::zeroed()) };
        addr.sll_addr = [0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef];
        let addr = Box::into_raw(addr) as *mut sockaddr;
        unsafe { (*addr).sa_family = 17 };
        let if_addrs = new_test_ifaddr(Some("eth1"), None, None);
        assert!(matches!(
            IfAddrsReadResult::try_from(if_addrs),
            Err(Error::PropertyNull(property)) if property == "ifa_addr"
        ));
    }

    #[test]
    fn try_from_ifaddr_err_unsupported_sa_family() {
        let mut addr: Box<sockaddr> = unsafe { Box::new(mem::zeroed()) };
        addr.sa_family = 100;
        let addr = Box::into_raw(addr);
        let if_addrs = new_test_ifaddr(Some("eth1"), Some(addr), None);
        assert!(matches!(
            IfAddrsReadResult::try_from(if_addrs),
            Err(Error::UnsupportedSaFamily(100))
        ));
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
            ipv4_networks: vec![Ipv4Network::from_str("22.41.0.0/16").unwrap()],
            ipv6_networks: vec![
                Ipv6Network::from_str("15a1:b1ac::/33").unwrap(),
                Ipv6Network::from_str("d5f0:bf0f:7ec0::/45").unwrap(),
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
    fn print_adapters() {
        let adapters = NetworkAdapterReaderImpl
            .try_read_network_adapters()
            .unwrap();
        println!("{:#?}", adapters);
        for adapter in adapters.values() {
            println!("IPv4 networks of {}", adapter.name);
            for ipv4 in adapter.ipv4_networks.iter() {
                println!(
                    "IPv4 address: {}, subnet_mask: {}",
                    ipv4.address(),
                    ipv4.subnet_mask()
                );
            }
            println!("IPv6 networks of {}", adapter.name);
            for ipv6 in adapter.ipv6_networks.iter() {
                println!(
                    "IPv6 address: {}, prefix_len: {}, subnet_mask: {}",
                    ipv6.address(),
                    ipv6.prefix_len(),
                    ipv6.subnet_mask()
                );
            }
        }
    }

    #[test]
    fn network_adapter_contains() {
        let contained_ipv4 = IpAddr::V4(Ipv4Addr::new(95, 12, 62, 78));
        let contained_ipv6 = IpAddr::V6(Ipv6Addr::new(
            0x10, 0x51, 0x737, 0x342, 0x6, 0x83, 0x78, 0x35,
        ));
        let adapter = NetworkAdapter {
            gateway: None,
            name: "TestAdapter".to_string(),
            ip_addresses: Vec::new(),
            mac: None,
            net_type: NetType::Unknown,
            ipv4_networks: vec![
                Ipv4Network::from_str("10.20.0.0/16").unwrap(),
                Ipv4Network::from_str("95.12.62.0/24").unwrap(),
                Ipv4Network::from_str("89.0.0.0/8").unwrap(),
            ],
            ipv6_networks: vec![
                Ipv6Network::from_str("abcd::/16").unwrap(),
                Ipv6Network::from_str("10:51:737::/48").unwrap(),
                Ipv6Network::from_str("1234::/16").unwrap(),
            ],
        };
        assert!(adapter.connected_networks_contain(contained_ipv4));
        assert!(adapter.connected_networks_contain(contained_ipv6));
        assert!(!adapter.connected_networks_contain(IpAddr::from_str("65.89.0.12").unwrap()));
        assert!(!adapter.connected_networks_contain(IpAddr::from_str("200:200::26").unwrap()));
    }
}
