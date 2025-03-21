use libc::{
    freeifaddrs, getifaddrs, ifaddrs, sockaddr_in, sockaddr_in6, sockaddr_ll, AF_INET, AF_INET6,
    AF_PACKET,
};
use procfs::net::RouteEntry;
use procfs::ProcError;
use std::collections::HashMap;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::mem::MaybeUninit;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Proc(#[from] ProcError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Unsupported SA Family: {0}")]
    UnsupportedSaFamily(u16),
    #[error("Property is null: {0}")]
    PropertyNull(String),
}

type Result<T> = std::result::Result<T, crate::relic::network::Error>;

#[derive(Debug)]
pub enum NetType {
    Unknown,
    Wired,
    Wireless,
    Local,
    Bridge,
    Virtual,
}
#[derive(Debug, PartialEq)]
pub struct IpAddr {
    pub addr: String,
    pub subnet_mask: String,
}

#[derive(Debug)]
pub struct NetInfo {
    pub mac: String,
    pub net_type: NetType,
    pub ipv4addresses: Vec<IpAddr>,
    pub ipv6addresses: Vec<IpAddr>,
    pub gateway: String,
}

#[derive(Debug, PartialEq)]
pub struct NetworkAddress {
    name: String,
    address: Address,
}
#[derive(Debug, PartialEq)]
enum Address {
    Mac(String),
    Ipv4(IpAddr),
    Ipv6(IpAddr),
}

enum SaFamily {
    Unsupported(u16),
    Packet,
    Inet4,
    Inet6,
}

pub struct IfAddrs {
    inner: *mut ifaddrs,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Network {
    Ipv4(Ipv4Network),
    Ipv6(Ipv6Network),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Ipv4Network {
    address: Ipv4Addr,
    size: u8,
}

impl Ipv4Network {
    pub fn try_new(address: Ipv4Addr, size: u8) -> crate::Result<Self> {
        const ZERO: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
        anyhow::ensure!(size <= 32, "Network size has to be 32 or less, not {size}");
        let mask = Ipv4Addr::from(0xffffffff >> size);
        anyhow::ensure!((address & mask) == ZERO, "Address part of network is not 0");
        Ok(Self { address, size })
    }
}

impl Default for Ipv4Network {
    fn default() -> Self {
        Self {
            address: Ipv4Addr::new(172, 21, 0, 0),
            size: 16,
        }
    }
}

impl FromStr for Ipv4Network {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (address, size) = s
            .split_once('/')
            .ok_or_else(|| anyhow::anyhow!("No '/' found"))?;
        Ipv4Network::try_new(Ipv4Addr::from_str(address)?, u8::from_str(size)?)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Ipv6Network {
    address: Ipv6Addr,
    suffix: u8,
}

impl Display for Ipv4Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.address, self.size)
    }
}

impl Display for Ipv6Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.address, self.suffix)
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Network::Ipv4(ip) => {
                    ip.to_string()
                }
                Network::Ipv6(ip) => {
                    ip.to_string()
                }
            }
        )
    }
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

pub fn transfer_ipv4_to_network(network: Ipv4Network, address: Ipv4Addr) -> Ipv4Addr {
    // Remove network part from address
    let address = address & Ipv4Addr::from(0xffffffffu32 >> network.size);
    address | network.address
}

impl Default for NetType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl NetInfo {
    pub fn try_read_from_system() -> Result<HashMap<String, Self>> {
        let mut adapters: HashMap<String, Self> = HashMap::new();
        let addresses: Vec<_> = IfAddrs::new()?
            .into_iter()
            .filter_map(|ifaddrs| NetworkAddress::try_from(ifaddrs).ok())
            .collect();
        let route_entries: Vec<_> = procfs::net::route()?
            .into_iter()
            .filter(|route_entry| route_entry.destination.is_unspecified())
            .collect();
        for NetworkAddress { name, address } in addresses {
            let entry = adapters
                .entry(name.clone())
                .or_insert(Self::new(name.as_str()));
            match address {
                Address::Mac(mac) => {
                    entry.mac = mac;
                }
                Address::Ipv4(ipv4) => {
                    entry.ipv4addresses.push(ipv4);
                }
                Address::Ipv6(ipv6) => {
                    entry.ipv4addresses.push(ipv6);
                }
            }
        }
        for RouteEntry { iface, gateway, .. } in route_entries {
            let entry = adapters
                .entry(iface.clone())
                .or_insert(Self::new(iface.as_str()));
            entry.gateway = gateway.to_string();
        }
        Ok(adapters)
    }

    fn new(name: &str) -> Self {
        Self {
            net_type: name.into(),
            gateway: String::default(),
            ipv4addresses: Vec::default(),
            mac: String::default(),
            ipv6addresses: Vec::default(),
        }
    }
}

impl From<&str> for NetType {
    fn from(value: &str) -> Self {
        match value {
            v if v.starts_with("en") || v.starts_with("eth") => Self::Wired,
            v if v.starts_with("wl") => Self::Wireless,
            v if v.starts_with("lo") => Self::Local,
            v if v.starts_with("veth") => Self::Virtual,
            v if v.starts_with("br") || v.starts_with("docker") => Self::Bridge,
            _ => Self::Unknown,
        }
    }
}

impl TryFrom<ifaddrs> for NetworkAddress {
    type Error = Error;

    fn try_from(value: ifaddrs) -> Result<Self> {
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
                Ok(NetworkAddress {
                    address: Address::Mac(mac),
                    name,
                })
            }
            SaFamily::Inet4 => {
                let s = unsafe { *(value.ifa_addr as *const sockaddr_in) };
                let addr: Ipv4Addr = u32::from_be(s.sin_addr.s_addr).into();
                let addr = addr.to_string();
                let s = unsafe { *(value.ifa_netmask as *const sockaddr_in) };
                let subnet_mask: Ipv4Addr = u32::from_be(s.sin_addr.s_addr).into();
                let subnet_mask = subnet_mask.to_string();
                Ok(NetworkAddress {
                    address: Address::Ipv4(IpAddr { addr, subnet_mask }),
                    name,
                })
            }
            SaFamily::Inet6 => {
                let s = unsafe { *(value.ifa_addr as *const sockaddr_in6) };
                let addr: Ipv6Addr = s.sin6_addr.s6_addr.into();
                let addr = addr.to_string();
                let s = unsafe { *(value.ifa_netmask as *const sockaddr_in6) };
                let subnet_mask: Ipv6Addr = s.sin6_addr.s6_addr.into();
                let subnet_mask = subnet_mask.to_string();
                Ok(NetworkAddress {
                    address: Address::Ipv6(IpAddr { addr, subnet_mask }),
                    name,
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

pub fn ipv4_to_network(ip: Ipv4Addr, subnet_mask: Ipv4Addr) -> Ipv4Network {
    let address = ip & subnet_mask;
    let subnet_mask: u32 = subnet_mask.into();
    Ipv4Network {
        address,
        size: subnet_mask.count_ones() as u8,
    }
}

pub fn ipv6_to_network(ip: Ipv6Addr, subnet_mask: Ipv6Addr) -> Ipv6Network {
    let address = ip & subnet_mask;
    let subnet_mask: u128 = subnet_mask.into();
    Ipv6Network {
        address,
        suffix: subnet_mask.count_ones() as u8,
    }
}

pub fn ip_to_network(
    ip: std::net::IpAddr,
    subnet_mask: std::net::IpAddr,
) -> crate::Result<Network> {
    match (ip, subnet_mask) {
        (std::net::IpAddr::V4(ip), std::net::IpAddr::V4(subnet_mask)) => {
            Ok(Network::Ipv4(ipv4_to_network(ip, subnet_mask)))
        }
        (std::net::IpAddr::V6(ip), std::net::IpAddr::V6(subnet_mask)) => {
            Ok(Network::Ipv6(ipv6_to_network(ip, subnet_mask)))
        }
        _ => anyhow::bail!("Can not create network witch mixed ip versions"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libc::sockaddr;
    use std::ffi::CString;
    use std::str::FromStr;
    use std::{mem, ptr};

    #[test]
    fn test_transfer_ipv4_to_network() {
        assert_eq!(
            transfer_ipv4_to_network(
                Ipv4Network::try_new(
                    Ipv4Addr::new(0b10101010, 0b10101010, 0b10100000, 0b00000000),
                    20
                )
                .unwrap(),
                Ipv4Addr::new(0b01010101, 0b01010101, 0b01010101, 0b01010101)
            ),
            Ipv4Addr::new(0b10101010, 0b10101010, 0b10100101, 0b01010101)
        );
        assert_eq!(
            transfer_ipv4_to_network(
                Ipv4Network::try_new(Ipv4Addr::new(10, 20, 30, 0), 24).unwrap(),
                Ipv4Addr::new(55, 55, 55, 99)
            ),
            Ipv4Addr::new(10, 20, 30, 99)
        );
    }
    #[test]
    fn test_try_new_ipv4network() {
        let address = Ipv4Addr::new(20, 30, 1, 0);
        let size = 24;
        assert_eq!(
            Ipv4Network::try_new(address, size).unwrap(),
            Ipv4Network { address, size }
        );
        let address = Ipv4Addr::new(0, 0, 0, 0);
        let size = 33;
        assert!(Ipv4Network::try_new(address, size).is_err());
        let address = Ipv4Addr::new(0, 0, 1, 0);
        let size = 9;
        assert!(Ipv4Network::try_new(address, size).is_err());
    }

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
    fn try_network_address_from_ifaddr_ok_packet() {
        let mut addr: Box<sockaddr_ll> = unsafe { Box::new(mem::zeroed()) };
        addr.sll_addr = [0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef];
        let addr = Box::into_raw(addr) as *mut sockaddr;
        unsafe { (*addr).sa_family = 17 };
        let if_addrs = new_test_ifaddr(Some("eth0"), Some(addr), None);
        assert_eq!(
            NetworkAddress::try_from(if_addrs).unwrap(),
            NetworkAddress {
                name: "eth0".to_string(),
                address: Address::Mac("12:34:56:78:90:ab".to_string())
            }
        )
    }

    #[test]
    fn try_network_address_from_ifaddr_ok_inet4() {
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
            NetworkAddress::try_from(if_addrs).unwrap(),
            NetworkAddress {
                name: "eth0".to_string(),
                address: Address::Ipv4(IpAddr {
                    addr: "10.20.0.0".to_string(),
                    subnet_mask: "255.255.0.0".to_string()
                })
            }
        )
    }

    #[test]
    fn try_network_address_from_ifaddr_ok_inet6() {
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
            NetworkAddress::try_from(if_addrs).unwrap(),
            NetworkAddress {
                name: "wl2".to_string(),
                address: Address::Ipv6(IpAddr {
                    addr: Ipv6Addr::from_str("1122:3344:aabb:ccdd::")
                        .unwrap()
                        .to_string(),
                    subnet_mask: Ipv6Addr::from_str("ffff:ffff:ffff:ffff::")
                        .unwrap()
                        .to_string()
                })
            }
        )
    }

    #[test]
    fn try_network_address_from_ifaddr_err_addr_null() {
        let mut addr: Box<sockaddr_ll> = unsafe { Box::new(mem::zeroed()) };
        addr.sll_addr = [0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef];
        let addr = Box::into_raw(addr) as *mut sockaddr;
        unsafe { (*addr).sa_family = 17 };
        let if_addrs = new_test_ifaddr(Some("eth1"), None, None);
        assert!(matches!(
            NetworkAddress::try_from(if_addrs),
            Err(Error::PropertyNull(property)) if property == "ifa_addr"
        ));
    }

    #[test]
    fn try_network_address_from_ifaddr_err_unsupported_sa_family() {
        let mut addr: Box<sockaddr> = unsafe { Box::new(mem::zeroed()) };
        addr.sa_family = 100;
        let addr = Box::into_raw(addr);
        let if_addrs = new_test_ifaddr(Some("eth1"), Some(addr), None);
        assert!(matches!(
            NetworkAddress::try_from(if_addrs),
            Err(Error::UnsupportedSaFamily(100))
        ));
    }

    #[test]
    fn test_address() {
        let addresses = IfAddrs::new().expect("Getting IfAddrs failed");
        for address in addresses.into_iter() {
            let net_addr: NetworkAddress = address
                .try_into()
                .expect("Converting to NetworkAddress failed");
            println!("Parsed: {:?}", net_addr);
        }
    }

    #[test]
    fn test_adapters() {
        let infos = NetInfo::try_read_from_system().expect("Failed to read network adapters");
        for (name, info) in infos {
            println!("Parsed adapter {}: {:?}", name, info);
        }
    }

    #[test]
    fn test_ipv4_to_network() {
        assert_eq!(
            ipv4_to_network(
                Ipv4Addr::from_str("192.168.99.21").unwrap(),
                Ipv4Addr::from_str("255.255.252.0").unwrap()
            ),
            Ipv4Network {
                address: Ipv4Addr::from_str("192.168.96.0").unwrap(),
                size: 22
            }
        );
        assert_eq!(
            ipv4_to_network(
                Ipv4Addr::from_str("127.0.0.1").unwrap(),
                Ipv4Addr::from_str("255.0.0.0").unwrap()
            ),
            Ipv4Network {
                address: Ipv4Addr::from_str("127.0.0.0").unwrap(),
                size: 8
            }
        );
        assert_eq!(
            ipv4_to_network(
                Ipv4Addr::from_str("169.254.52.1").unwrap(),
                Ipv4Addr::from_str("255.255.0.0").unwrap()
            ),
            Ipv4Network {
                address: Ipv4Addr::from_str("169.254.0.0").unwrap(),
                size: 16
            }
        );
    }

    #[test]
    fn test_ipv6_to_network() {
        assert_eq!(
            ipv6_to_network(
                Ipv6Addr::from_str("2002:0000:0000:1234:abcd:ffff:c0a8:0101").unwrap(),
                Ipv6Addr::from_str("ffff:ffff:ffff:ffff:0000:0000:0000:0000").unwrap()
            ),
            Ipv6Network {
                address: Ipv6Addr::from_str("2002:0000:0000:1234:0000:0000:0000:0000").unwrap(),
                suffix: 64
            }
        );
        assert_eq!(
            ipv6_to_network(
                Ipv6Addr::from_str("abcd:4422:efef:0707:8888:1212:3234:1256").unwrap(),
                Ipv6Addr::from_str("ffff:ffff:0000:0000:0000:0000:0000:0000").unwrap()
            ),
            Ipv6Network {
                address: Ipv6Addr::from_str("abcd:4422:0000:0000:0000:0000:0000:0000").unwrap(),
                suffix: 32
            }
        );
        assert_eq!(
            ipv6_to_network(
                Ipv6Addr::from_str("aaaa:bbbb:cccc:dddd:eeee:1111:2222:3333").unwrap(),
                Ipv6Addr::from_str("ffff:ffff:ffff:ffff:ffff:fff0:0000:0000").unwrap()
            ),
            Ipv6Network {
                address: Ipv6Addr::from_str("aaaa:bbbb:cccc:dddd:eeee:1110:0000:0000").unwrap(),
                suffix: 92
            }
        );
    }

    #[test]
    fn test_ip_to_network() {
        assert!(ip_to_network(
            std::net::IpAddr::V6(
                Ipv6Addr::from_str("aaaa:bbbb:cccc:dddd:eeee:1111:2222:3333").unwrap()
            ),
            std::net::IpAddr::V4(Ipv4Addr::from_str("169.254.52.1").unwrap())
        )
        .is_err());
        assert!(ip_to_network(
            std::net::IpAddr::V4(Ipv4Addr::from_str("169.254.52.1").unwrap()),
            std::net::IpAddr::V6(
                Ipv6Addr::from_str("aaaa:bbbb:cccc:dddd:eeee:1111:2222:3333").unwrap()
            )
        )
        .is_err());
        assert_eq!(
            ip_to_network(
                std::net::IpAddr::V6(
                    Ipv6Addr::from_str("aaaa:bbbb:cccc:dddd:eeee:1111:2222:3333").unwrap()
                ),
                std::net::IpAddr::V6(
                    Ipv6Addr::from_str("ffff:ffff:ffff:ffff:ffff:fff0:0000:0000").unwrap()
                )
            )
            .unwrap(),
            Network::Ipv6(Ipv6Network {
                address: Ipv6Addr::from_str("aaaa:bbbb:cccc:dddd:eeee:1110:0000:0000").unwrap(),
                suffix: 92
            })
        );

        assert_eq!(
            ip_to_network(
                std::net::IpAddr::V4(Ipv4Addr::from_str("169.254.52.1").unwrap()),
                std::net::IpAddr::V4(Ipv4Addr::from_str("255.255.0.0").unwrap())
            )
            .unwrap(),
            Network::Ipv4(Ipv4Network {
                address: Ipv4Addr::from_str("169.254.0.0").unwrap(),
                size: 16
            })
        );
    }

    #[test]
    fn ipv4_network_from_str() {
        let ip = Ipv4Addr::new(10, 0b11100000, 0, 0);
        let suffix = 11;
        assert_eq!(
            Ipv4Network {
                address: ip,
                size: suffix,
            },
            Ipv4Network::from_str(&format!("{}/{}", ip, suffix)).unwrap()
        )
    }

    #[test]
    fn default_flecs_network() {
        assert_eq!(
            Ipv4Network::default(),
            Ipv4Network {
                address: Ipv4Addr::new(172, 21, 0, 0),
                size: 16
            }
        )
    }
}
