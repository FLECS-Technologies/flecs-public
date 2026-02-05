use ipnet::Ipv4Net;
use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::str::FromStr;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Ipv4NetworkAccess {
    network: Ipv4Net,
    gateway: Ipv4Addr,
}

impl Ipv4NetworkAccess {
    pub fn next_free_ipv4_address(
        &self,
        mut unavailable_addresses: HashSet<Ipv4Addr>,
    ) -> Option<Ipv4Addr> {
        unavailable_addresses.insert(self.gateway);
        self.network
            .hosts()
            .find(|address| !unavailable_addresses.contains(address))
    }

    pub fn try_new(network: Ipv4Net, gateway: Ipv4Addr) -> crate::Result<Self> {
        anyhow::ensure!(
            network.contains(&gateway),
            "The gateway {gateway} has to be part of the network {network}."
        );
        Ok(Self { network, gateway })
    }

    pub fn network(&self) -> Ipv4Net {
        self.network
    }

    pub fn gateway(&self) -> Ipv4Addr {
        self.gateway
    }
}

impl TryFrom<bollard::models::Network> for Ipv4NetworkAccess {
    type Error = crate::Error;

    fn try_from(value: bollard::models::Network) -> std::result::Result<Self, Self::Error> {
        let config = value
            .ipam
            .ok_or_else(|| anyhow::anyhow!("No ipam present"))?
            .config
            .ok_or_else(|| anyhow::anyhow!("No ipam config present"))?
            .first()
            .ok_or_else(|| anyhow::anyhow!("No network in ipam config present"))?
            .clone();
        let network = Ipv4Net::from_str(
            config
                .subnet
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("No subnet in ipam network config present"))?,
        )?;
        Ok(Self {
            network,
            gateway: Ipv4Addr::from_str(
                &config
                    .gateway
                    .ok_or_else(|| anyhow::anyhow!("No gateway in ipam network config present"))?,
            )?,
        })
    }
}

pub fn transfer_ipv4_to_network(network: Ipv4Net, address: Ipv4Addr) -> Ipv4Addr {
    // Remove network part from address
    let address = address & Ipv4Addr::from(0xffffffffu32 >> network.prefix_len());
    address | network.addr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::test_case;
    use std::str::FromStr;

    #[test]
    fn test_transfer_ipv4_to_network() {
        assert_eq!(
            transfer_ipv4_to_network(
                Ipv4Net::new(
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
                Ipv4Net::new_assert(Ipv4Addr::new(10, 20, 30, 0), 24),
                Ipv4Addr::new(55, 55, 55, 99)
            ),
            Ipv4Addr::new(10, 20, 30, 99)
        );
    }

    #[test]
    fn next_free_ipv4_address_all_available() {
        let network = Ipv4NetworkAccess {
            network: Ipv4Net::new_assert(Ipv4Addr::new(123, 123, 123, 0), 24),
            gateway: Ipv4Addr::new(123, 123, 123, 254),
        };
        assert_eq!(
            network.next_free_ipv4_address(HashSet::default()),
            Some(Ipv4Addr::new(123, 123, 123, 1))
        );
    }

    #[test]
    fn next_free_ipv4_address_skip_gateway() {
        let network = Ipv4NetworkAccess {
            network: Ipv4Net::new_assert(Ipv4Addr::new(123, 123, 123, 0), 24),
            gateway: Ipv4Addr::new(123, 123, 123, 2),
        };
        assert_eq!(
            network.next_free_ipv4_address(HashSet::default()),
            Some(Ipv4Addr::new(123, 123, 123, 1))
        );
        assert_eq!(
            network.next_free_ipv4_address(HashSet::from([Ipv4Addr::new(123, 123, 123, 1)])),
            Some(Ipv4Addr::new(123, 123, 123, 3))
        );
    }

    #[test]
    fn next_free_ipv4_address_none_available() {
        let network = Ipv4NetworkAccess {
            network: Ipv4Net::new_assert(Ipv4Addr::new(123, 123, 123, 0), 24),
            gateway: Ipv4Addr::new(123, 123, 123, 1),
        };
        let unavailable_ips = (2..255).map(|b| Ipv4Addr::new(123, 123, 123, b)).collect();
        assert_eq!(network.next_free_ipv4_address(unavailable_ips), None);
    }

    #[test]
    fn next_free_ipv4_address_1_available() {
        let network = Ipv4NetworkAccess {
            network: Ipv4Net::new_assert(Ipv4Addr::new(123, 123, 123, 0), 24),
            gateway: Ipv4Addr::new(123, 123, 123, 3),
        };
        let unavailable_ips = (4..255).map(|b| Ipv4Addr::new(123, 123, 123, b)).collect();
        assert_eq!(
            network.next_free_ipv4_address(unavailable_ips),
            Some(Ipv4Addr::new(123, 123, 123, 1)),
        );
    }

    #[test]
    fn next_free_ipv4_address_254_available() {
        let network = Ipv4NetworkAccess {
            network: Ipv4Net::new_assert(Ipv4Addr::new(123, 123, 123, 0), 24),
            gateway: Ipv4Addr::new(123, 123, 123, 1),
        };
        let unavailable_ips = (2..254).map(|b| Ipv4Addr::new(123, 123, 123, b)).collect();
        assert_eq!(
            network.next_free_ipv4_address(unavailable_ips),
            Some(Ipv4Addr::new(123, 123, 123, 254)),
        );
    }

    #[test]
    fn next_free_ipv4_address_100_available() {
        let network = Ipv4NetworkAccess {
            network: Ipv4Net::new_assert(Ipv4Addr::new(123, 123, 123, 0), 24),

            gateway: Ipv4Addr::new(123, 123, 123, 1),
        };
        let unavailable_ips = (2..100).map(|b| Ipv4Addr::new(123, 123, 123, b)).collect();
        assert_eq!(
            network.next_free_ipv4_address(unavailable_ips),
            Some(Ipv4Addr::new(123, 123, 123, 100)),
        );
    }

    #[test]
    fn try_ipv4_network_access_from_bollard_network_ok() {
        let bollard_network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![bollard::models::IpamConfig {
                    subnet: Some("10.18.100.0/22".to_string()),
                    gateway: Some("10.18.100.10".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let expected_network = Ipv4NetworkAccess {
            network: Ipv4Net::new_assert(Ipv4Addr::new(10, 18, 100, 0), 22),
            gateway: Ipv4Addr::new(10, 18, 100, 10),
        };
        assert_eq!(
            Ipv4NetworkAccess::try_from(bollard_network).unwrap(),
            expected_network
        );
    }

    #[test]
    fn try_ipv4_network_access_from_bollard_network_invalid_gateway() {
        let bollard_network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![bollard::models::IpamConfig {
                    subnet: Some("10.18.100.0/22".to_string()),
                    gateway: Some("10.18.1000.10".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(Ipv4NetworkAccess::try_from(bollard_network).is_err());
    }

    #[test]
    fn try_ipv4_network_access_from_bollard_network_no_gateway() {
        let bollard_network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![bollard::models::IpamConfig {
                    subnet: Some("10.18.100.0/22".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(Ipv4NetworkAccess::try_from(bollard_network).is_err());
    }

    #[test]
    fn try_ipv4_network_access_from_bollard_network_no_subnet() {
        let bollard_network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![bollard::models::IpamConfig {
                    gateway: Some("10.18.100.10".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(Ipv4NetworkAccess::try_from(bollard_network).is_err());
    }

    #[test]
    fn try_ipv4_network_access_from_bollard_network_invalid_subnet() {
        let bollard_network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![bollard::models::IpamConfig {
                    subnet: Some("10.18.100.0|7".to_string()),
                    gateway: Some("10.18.100.10".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(Ipv4NetworkAccess::try_from(bollard_network).is_err());
    }

    #[test]
    fn try_ipv4_network_access_from_bollard_network_empty_ipam_configs() {
        let bollard_network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![]),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(Ipv4NetworkAccess::try_from(bollard_network).is_err());
    }

    #[test]
    fn try_ipv4_network_access_from_bollard_network_no_ipam_configs() {
        let bollard_network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: None,
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(Ipv4NetworkAccess::try_from(bollard_network).is_err());
    }

    #[test]
    fn try_ipv4_network_access_from_bollard_network_no_ipam() {
        let bollard_network = bollard::models::Network {
            ipam: None,
            ..Default::default()
        };
        assert!(Ipv4NetworkAccess::try_from(bollard_network).is_err());
    }

    #[test]
    fn try_new_ipv4_network_access_ok() {
        let network = Ipv4Net::new_assert(Ipv4Addr::new(10, 20, 0, 0), 16);
        let gateway = Ipv4Addr::new(10, 20, 20, 100);
        let network_access = Ipv4NetworkAccess::try_new(network, gateway).unwrap();
        assert_eq!(network_access.network, network);
        assert_eq!(network_access.network(), network);
        assert_eq!(network_access.gateway, gateway);
        assert_eq!(network_access.gateway(), gateway);
    }

    #[test]
    fn try_new_ipv4_network_access_err() {
        assert!(
            Ipv4NetworkAccess::try_new(
                Ipv4Net::new_assert(Ipv4Addr::new(10, 20, 0, 0), 16),
                Ipv4Addr::new(10, 10, 20, 100),
            )
            .is_err()
        );
    }

    #[test_case("10.20.30.0", 24, "255.255.255.0", "10.20.30.255")]
    #[test_case("127.0.2.0", 24, "255.255.255.0", "127.0.2.255")]
    #[test_case("100.0.0.0", 8, "255.0.0.0", "100.255.255.255")]
    #[test_case("200.200.80.0", 20, "255.255.240.0", "200.200.95.255")]
    #[test_case("127.0.0.0", 10, "255.192.0.0", "127.63.255.255")]
    fn ipv4_getter(ip: &str, prefix_len: u8, subnet_mask: &str, broadcast: &str) {
        let network = Ipv4Net::new_assert(Ipv4Addr::from_str(ip).unwrap(), prefix_len);
        assert_eq!(network.addr(), Ipv4Addr::from_str(ip).unwrap());
        assert_eq!(network.netmask(), Ipv4Addr::from_str(subnet_mask).unwrap());
        assert_eq!(network.broadcast(), Ipv4Addr::from_str(broadcast).unwrap());
    }
}
