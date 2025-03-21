pub mod ipv4;
pub use ipv4::*;
pub mod ipv6;
pub use ipv6::*;
mod network_adapter;
pub use network_adapter::NetInfo;
use procfs::ProcError;
use std::fmt::{Display, Formatter};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Network {
    Ipv4(Ipv4Network),
    Ipv6(Ipv6Network),
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

impl Default for NetType {
    fn default() -> Self {
        Self::Unknown
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

pub fn get_random_free_port() -> crate::Result<u16> {
    let bind = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))?;
    Ok(bind.local_addr()?.port())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv6Addr;
    use std::str::FromStr;

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
            Network::Ipv4(Ipv4Network::from_str("169.254.0.0/16").unwrap())
        );
    }

    #[test]
    fn random_port_test() {
        let random_port = get_random_free_port().unwrap();
        TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, random_port)).unwrap();
    }
}
