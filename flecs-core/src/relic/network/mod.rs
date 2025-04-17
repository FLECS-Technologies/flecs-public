mod ipv4;
mod ipv6;
mod network_adapter;
pub use ipv4::*;
pub use ipv6::*;
#[cfg(test)]
pub use network_adapter::MockNetworkAdapterReader;
#[cfg(test)]
pub use network_adapter::tests::*;
pub use network_adapter::{NetworkAdapter, NetworkAdapterReader, NetworkAdapterReaderImpl};
use procfs::ProcError;
use std::fmt::{Display, Formatter};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
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
    #[error("Invalid network: {0}")]
    InvalidNetwork(String),
}

type Result<T> = std::result::Result<T, crate::relic::network::Error>;

#[derive(Debug, Clone, PartialEq)]
pub enum NetType {
    Unknown,
    Wired,
    Wireless,
    Local,
    Bridge,
    Virtual,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Network {
    Ipv4(Ipv4Network),
    Ipv6(Ipv6Network),
}

impl Display for Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Ipv4(ip) => Display::fmt(ip, f),
            Network::Ipv6(ip) => Display::fmt(ip, f),
        }
    }
}

impl FromStr for Network {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match (Ipv4Network::from_str(s), Ipv6Network::from_str(s)) {
            (Ok(ipv4), _) => Ok(Self::Ipv4(ipv4)),
            (_, Ok(ipv6)) => Ok(Self::Ipv6(ipv6)),
            (Err(e1), Err(e2)) => Err(anyhow::anyhow!(
                "'{s}' is no valid ipv4 ({e1}) or ipv6 ({e2}) network"
            )),
        }
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
    use ntest::test_case;
    use std::net::Ipv6Addr;
    use std::str::FromStr;

    #[test]
    fn test_ip_to_network() {
        assert!(
            ip_to_network(
                std::net::IpAddr::V6(
                    Ipv6Addr::from_str("aaaa:bbbb:cccc:dddd:eeee:1111:2222:3333").unwrap()
                ),
                std::net::IpAddr::V4(Ipv4Addr::from_str("169.254.52.1").unwrap())
            )
            .is_err()
        );
        assert!(
            ip_to_network(
                std::net::IpAddr::V4(Ipv4Addr::from_str("169.254.52.1").unwrap()),
                std::net::IpAddr::V6(
                    Ipv6Addr::from_str("aaaa:bbbb:cccc:dddd:eeee:1111:2222:3333").unwrap()
                )
            )
            .is_err()
        );
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
            Network::Ipv6(
                Ipv6Network::from_str("aaaa:bbbb:cccc:dddd:eeee:1110:0000:0000/92").unwrap(),
            )
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

    #[test_case("eth0", NetType::Wired)]
    #[test_case("eth51", NetType::Wired)]
    #[test_case("en1", NetType::Wired)]
    #[test_case("en0", NetType::Wired)]
    #[test_case("wl412", NetType::Wireless)]
    #[test_case("wl2", NetType::Wireless)]
    #[test_case("lo", NetType::Local)]
    #[test_case("lo10", NetType::Local)]
    #[test_case("veth", NetType::Virtual)]
    #[test_case("veth0", NetType::Virtual)]
    #[test_case("veth7", NetType::Virtual)]
    #[test_case("br", NetType::Bridge)]
    #[test_case("br0", NetType::Bridge)]
    #[test_case("br24", NetType::Bridge)]
    #[test_case("docker", NetType::Bridge)]
    #[test_case("docker0", NetType::Bridge)]
    #[test_case("docker79", NetType::Bridge)]
    #[test_case("123", NetType::Unknown)]
    #[test_case("abab", NetType::Unknown)]
    #[test_case("wireless", NetType::Unknown)]
    #[test_case("lan", NetType::Unknown)]
    #[test_case("virtual", NetType::Unknown)]
    fn network_type_from_str(s: &str, net_type: NetType) {
        assert_eq!(NetType::from(s), net_type);
    }

    #[test]
    fn network_display() {
        let ipv4 = Ipv4Network::from_str("127.0.0.0/10").unwrap();
        let expected = ipv4.to_string();
        assert_eq!(Network::Ipv4(ipv4).to_string(), expected);
        let ipv6 = Ipv6Network::from_str("aaaa:bbbb:cccc:dddd::/64").unwrap();
        let expected = ipv6.to_string();
        assert_eq!(Network::Ipv6(ipv6).to_string(), expected);
    }

    #[test_case("10.20.30.0/24")]
    #[test_case("127.0.2.0/24")]
    #[test_case("100.0.0.0/8")]
    #[test_case("200.200.80.0/20")]
    #[test_case("127.0.0.0/10")]
    fn network_from_ipv4_str(ip: &str) {
        let ipv4 = Ipv4Network::from_str(ip).unwrap();
        assert_eq!(Network::from_str(ip).unwrap(), Network::Ipv4(ipv4));
    }

    #[test_case("aaaa:bbbb:cccc:dddd::/64")]
    #[test_case("81f2:f385:4800::/37")]
    #[test_case("86e5:6018:d00::/44")]
    #[test_case("4761:45da:6::/50")]
    #[test_case("b884:6129:db74:a800::/53")]
    #[test_case("15a1:b1ac::/33")]
    #[test_case("3cf9:2cff::/33")]
    #[test_case("ffa4:aafb:9c26:3040::/59")]
    #[test_case("b20d:a3e5:3857:b800::/53")]
    #[test_case("7519:f47a:9000::/37")]
    #[test_case("d5f0:bf0f:7ec0::/45")]
    #[test_case("23f1:99b8:6000::/35")]
    #[test_case("97a7:922d:5ec0::/46")]
    #[test_case("edd3:206b:1e8f:f6c0::/58")]
    #[test_case("e831:1727:7500::/40")]
    #[test_case("b4c9:b860:e45e:b500::/57")]
    #[test_case("71d4:f385:375a:e000::/51")]
    #[test_case("383e:da05:7800::/39")]
    #[test_case("9926:8e1a:47ee:c000::/50")]
    #[test_case("5abd:c7f5:e300::/43")]
    fn network_from_ipv6_str(ip: &str) {
        let ipv6 = Ipv6Network::from_str(ip).unwrap();
        assert_eq!(Network::from_str(ip).unwrap(), Network::Ipv6(ipv6));
    }

    #[test]
    fn network_from_ipv6_str() {
        assert!(Network::from_str("1235").is_err());
    }
}
