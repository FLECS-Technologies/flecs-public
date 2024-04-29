use crate::ffi;
use libc::{
    freeifaddrs, getifaddrs, ifaddrs, sockaddr_in, sockaddr_in6, sockaddr_ll, AF_INET, AF_INET6,
    AF_PACKET,
};
use procfs::net::RouteEntry;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::io;
use std::mem::MaybeUninit;
use std::net::{Ipv4Addr, Ipv6Addr};

pub fn read_network_adapters() -> anyhow::Result<Vec<ffi::NetAdapter>> {
    Ok(ffi::NetInfo::try_read_from_system()?
        .into_iter()
        .map(|(name, info)| ffi::NetAdapter { name, info })
        .collect())
}

#[derive(Debug)]
pub struct NetworkAddress {
    name: String,
    address: Address,
}
#[derive(Debug)]
enum Address {
    Mac(String),
    Ipv4(ffi::IpAddr),
    Ipv6(ffi::IpAddr),
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

impl Default for ffi::NetType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl ffi::NetInfo {
    fn try_read_from_system() -> anyhow::Result<HashMap<String, Self>> {
        let mut adapters: HashMap<String, Self> = HashMap::new();
        let addresses: anyhow::Result<Vec<_>, _> = IfAddrs::new()?
            .iter()
            .map(NetworkAddress::try_from)
            .collect();
        let route_entries = procfs::net::route()?;
        for NetworkAddress { name, address } in addresses?.into_iter() {
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

impl From<&str> for ffi::NetType {
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

#[derive(Debug)]
pub enum ConvertToNetworkAdressError {
    UnsupportedSaFamily(u16),
}

impl Display for ConvertToNetworkAdressError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConvertToNetworkAdressError::UnsupportedSaFamily(val) => {
                write!(f, "SaFamily '{}' is unsupported", val)
            }
        }
    }
}

impl Error for ConvertToNetworkAdressError {}

impl TryFrom<ifaddrs> for NetworkAddress {
    type Error = ConvertToNetworkAdressError;

    fn try_from(value: ifaddrs) -> anyhow::Result<Self, Self::Error> {
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
                let addr: Ipv4Addr = s.sin_addr.s_addr.into();
                let addr = addr.to_string();
                let s = unsafe { *(value.ifa_netmask as *const sockaddr_in) };
                let subnet_mask: Ipv4Addr = s.sin_addr.s_addr.into();
                let subnet_mask = subnet_mask.to_string();
                Ok(NetworkAddress {
                    address: Address::Ipv4(ffi::IpAddr { addr, subnet_mask }),
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
                    address: Address::Ipv6(ffi::IpAddr { addr, subnet_mask }),
                    name,
                })
            }
        }
    }
}

impl IfAddrs {
    #[allow(unsafe_code, clippy::new_ret_no_self)]
    pub fn new() -> io::Result<Self> {
        let mut ifaddrs: MaybeUninit<*mut ifaddrs> = MaybeUninit::uninit();

        let ifaddrs = unsafe {
            if -1 == getifaddrs(ifaddrs.as_mut_ptr()) {
                return Err(io::Error::last_os_error());
            }
            ifaddrs.assume_init()
        };

        Ok(Self { inner: ifaddrs })
    }

    pub fn iter(&self) -> IfAddrsIterator {
        IfAddrsIterator { next: self.inner }
    }
}

impl Drop for IfAddrs {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        unsafe {
            freeifaddrs(self.inner);
        }
    }
}

pub struct IfAddrsIterator {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address() {
        let addresses = IfAddrs::new().expect("Getting IfAddrs failed");
        for address in addresses.iter() {
            let net_addr: NetworkAddress = address
                .try_into()
                .expect("Converting to NetworkAddress failed");
            println!("Parsed: {:?}", net_addr);
        }
    }

    #[test]
    fn test_adapters() {
        let infos = ffi::NetInfo::try_read_from_system().expect("Failed to read network adapters");
        for (name, info) in infos {
            println!("Parsed adapter {}: {:?}", name, info);
        }
    }
}