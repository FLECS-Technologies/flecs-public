use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::net::Ipv4Addr;
use std::ops::Range;
use std::str::FromStr;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Ipv4Network {
    address: Ipv4Addr,
    size: u8,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Ipv4NetworkAccess {
    network: Ipv4Network,
    gateway: Ipv4Addr,
}

impl Ipv4NetworkAccess {
    pub fn next_free_ipv4_address(
        &self,
        mut unavailable_addresses: HashSet<Ipv4Addr>,
    ) -> Option<Ipv4Addr> {
        unavailable_addresses.insert(self.gateway);
        self.network
            .iter()
            .find(|address| !unavailable_addresses.contains(address))
    }

    pub fn try_new(network: Ipv4Network, gateway: Ipv4Addr) -> crate::Result<Self> {
        anyhow::ensure!(
            network.iter().contains(gateway),
            "The gateway has to be part of the network."
        );
        Ok(Self { network, gateway })
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
        let network = Ipv4Network::from_str(
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

pub struct Ipv4Iterator {
    current: u32,
    max: u32,
}

impl Ipv4Iterator {
    pub fn contains(&self, address: Ipv4Addr) -> bool {
        (self.current..self.max).contains(&u32::from(address))
    }
}

impl Iterator for Ipv4Iterator {
    type Item = Ipv4Addr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.max {
            None
        } else {
            let next = Some(Ipv4Addr::from(self.current));
            self.current += 1;
            next
        }
    }
}

impl From<Range<u32>> for Ipv4Iterator {
    fn from(value: Range<u32>) -> Self {
        Self {
            current: value.start,
            max: value.end,
        }
    }
}

impl Ipv4Iterator {
    fn new(start: Ipv4Addr, end_exclusive: Ipv4Addr) -> Self {
        (start.into()..end_exclusive.into()).into()
    }
}

impl Ipv4Network {
    pub fn try_new(address: Ipv4Addr, size: u8) -> crate::Result<Self> {
        anyhow::ensure!(size <= 32, "Network size has to be 32 or less, not {size}");
        let mask = Ipv4Addr::from(0xffffffff >> size);
        anyhow::ensure!(
            (address & mask) == Ipv4Addr::UNSPECIFIED,
            "Address part of network is not 0"
        );
        Ok(Self { address, size })
    }

    pub fn iter(&self) -> Ipv4Iterator {
        let start = Ipv4Addr::from(u32::from(self.address) + 2_u32);
        Ipv4Iterator::new(start, self.broadcast())
    }

    pub fn broadcast(&self) -> Ipv4Addr {
        (u32::from(self.address) | 0xffffffff >> self.size).into()
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

impl Display for Ipv4Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.address, self.size)
    }
}

pub fn transfer_ipv4_to_network(network: Ipv4Network, address: Ipv4Addr) -> Ipv4Addr {
    // Remove network part from address
    let address = address & Ipv4Addr::from(0xffffffffu32 >> network.size);
    address | network.address
}

pub fn ipv4_to_network(ip: Ipv4Addr, subnet_mask: Ipv4Addr) -> Ipv4Network {
    let address = ip & subnet_mask;
    let subnet_mask: u32 = subnet_mask.into();
    Ipv4Network {
        address,
        size: subnet_mask.count_ones() as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

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
    fn ipv4_network_broadcast() {
        assert_eq!(
            Ipv4Network {
                address: Ipv4Addr::new(0b10100101, 0b11010100, 0b10100000, 0x00000000),
                size: 20,
            }
            .broadcast(),
            Ipv4Addr::new(0b10100101, 0b11010100, 0b10101111, 0b11111111)
        );
        assert_eq!(
            Ipv4Network {
                address: Ipv4Addr::new(0b10100101, 0b11010100, 0b10100000, 0x00000000),
                size: 30,
            }
            .broadcast(),
            Ipv4Addr::new(0b10100101, 0b11010100, 0b10100000, 0b00000011)
        );
        assert_eq!(
            Ipv4Network {
                address: Ipv4Addr::new(0xaa, 0xa0, 0x00, 0x00),
                size: 12,
            }
            .broadcast(),
            Ipv4Addr::new(0xaa, 0xaf, 0xff, 0xff)
        );
    }

    #[test]
    fn ipv4_network_iter() {
        let base = Ipv4Addr::new(10, 40, 0b10101000, 0);
        let size = 22;
        let iter = Ipv4Network {
            address: base,
            size,
        }
        .iter();
        assert_eq!(
            iter.current,
            u32::from(Ipv4Addr::new(10, 40, 0b10101000, 2))
        );
        assert_eq!(iter.max, u32::from(Ipv4Addr::new(10, 40, 0b10101011, 255)));
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

    #[test]
    fn ipv4_iterator_new() {
        let start = Ipv4Addr::new(10, 20, 30, 40);
        let end = Ipv4Addr::new(10, 20, 30, 200);
        let iterator = Ipv4Iterator::new(start, end);
        assert_eq!(iterator.current, u32::from(start));
        assert_eq!(iterator.max, u32::from(end));
    }

    #[test]
    fn ipv4_iterator_from_range() {
        let start = 200;
        let end = 2000;
        let iterator = Ipv4Iterator::from(start..end);
        assert_eq!(iterator.current, start);
        assert_eq!(iterator.max, end);
    }

    #[test]
    fn ipv4_iterator_next() {
        let start = 5;
        let end = 10;
        let mut iterator = Ipv4Iterator {
            current: start,
            max: end,
        };
        assert_eq!(iterator.next(), Some(Ipv4Addr::from(5)));
        assert_eq!(iterator.next(), Some(Ipv4Addr::from(6)));
        assert_eq!(iterator.next(), Some(Ipv4Addr::from(7)));
        assert_eq!(iterator.next(), Some(Ipv4Addr::from(8)));
        assert_eq!(iterator.next(), Some(Ipv4Addr::from(9)));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn next_free_ipv4_address_all_available() {
        let network = Ipv4NetworkAccess {
            network: Ipv4Network {
                address: Ipv4Addr::new(123, 123, 123, 0),
                size: 24,
            },
            gateway: Ipv4Addr::new(123, 123, 123, 254),
        };
        assert_eq!(
            network.next_free_ipv4_address(HashSet::default()),
            Some(Ipv4Addr::new(123, 123, 123, 2))
        );
    }

    #[test]
    fn next_free_ipv4_address_skip_gateway() {
        let network = Ipv4NetworkAccess {
            network: Ipv4Network {
                address: Ipv4Addr::new(123, 123, 123, 0),
                size: 24,
            },
            gateway: Ipv4Addr::new(123, 123, 123, 2),
        };
        assert_eq!(
            network.next_free_ipv4_address(HashSet::default()),
            Some(Ipv4Addr::new(123, 123, 123, 3))
        );
    }

    #[test]
    fn next_free_ipv4_address_none_available() {
        let network = Ipv4NetworkAccess {
            network: Ipv4Network {
                address: Ipv4Addr::new(123, 123, 123, 0),
                size: 24,
            },
            gateway: Ipv4Addr::new(123, 123, 123, 2),
        };
        let unavailable_ips = (3..255).map(|b| Ipv4Addr::new(123, 123, 123, b)).collect();
        assert_eq!(network.next_free_ipv4_address(unavailable_ips), None);
    }

    #[test]
    fn next_free_ipv4_address_2_available() {
        let network = Ipv4NetworkAccess {
            network: Ipv4Network {
                address: Ipv4Addr::new(123, 123, 123, 0),
                size: 24,
            },
            gateway: Ipv4Addr::new(123, 123, 123, 3),
        };
        let unavailable_ips = (4..255).map(|b| Ipv4Addr::new(123, 123, 123, b)).collect();
        assert_eq!(
            network.next_free_ipv4_address(unavailable_ips),
            Some(Ipv4Addr::new(123, 123, 123, 2)),
        );
    }

    #[test]
    fn next_free_ipv4_address_254_available() {
        let network = Ipv4NetworkAccess {
            network: Ipv4Network {
                address: Ipv4Addr::new(123, 123, 123, 0),
                size: 24,
            },
            gateway: Ipv4Addr::new(123, 123, 123, 2),
        };
        let unavailable_ips = (3..254).map(|b| Ipv4Addr::new(123, 123, 123, b)).collect();
        assert_eq!(
            network.next_free_ipv4_address(unavailable_ips),
            Some(Ipv4Addr::new(123, 123, 123, 254)),
        );
    }

    #[test]
    fn next_free_ipv4_address_100_available() {
        let network = Ipv4NetworkAccess {
            network: Ipv4Network {
                address: Ipv4Addr::new(123, 123, 123, 0),
                size: 24,
            },
            gateway: Ipv4Addr::new(123, 123, 123, 2),
        };
        let unavailable_ips = (3..100).map(|b| Ipv4Addr::new(123, 123, 123, b)).collect();
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
            network: Ipv4Network {
                address: Ipv4Addr::new(10, 18, 100, 0),
                size: 22,
            },
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
                    subnet: Some("10.18.100.0/7".to_string()),
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
        Ipv4NetworkAccess::try_new(
            Ipv4Network::try_new(Ipv4Addr::new(10, 20, 0, 0), 16).unwrap(),
            Ipv4Addr::new(10, 20, 20, 100),
        )
        .unwrap();
    }

    #[test]
    fn try_new_ipv4_network_access_err() {
        assert!(Ipv4NetworkAccess::try_new(
            Ipv4Network::try_new(Ipv4Addr::new(10, 20, 0, 0), 16).unwrap(),
            Ipv4Addr::new(10, 10, 20, 100),
        )
        .is_err());
    }

    #[test]
    fn ipv4_iterator_contains() {
        let mut iter = Ipv4Iterator {
            current: 10,
            max: 14,
        };
        assert!(!iter.contains(Ipv4Addr::from(8)));
        assert!(!iter.contains(Ipv4Addr::from(9)));
        assert!(iter.contains(Ipv4Addr::from(10)));
        assert!(iter.contains(Ipv4Addr::from(11)));
        assert!(iter.contains(Ipv4Addr::from(12)));
        assert!(iter.contains(Ipv4Addr::from(13)));
        assert!(!iter.contains(Ipv4Addr::from(14)));
        assert!(!iter.contains(Ipv4Addr::from(15)));
        iter.next();
        assert!(!iter.contains(Ipv4Addr::from(8)));
        assert!(!iter.contains(Ipv4Addr::from(9)));
        assert!(!iter.contains(Ipv4Addr::from(10)));
        assert!(iter.contains(Ipv4Addr::from(11)));
        assert!(iter.contains(Ipv4Addr::from(12)));
        assert!(iter.contains(Ipv4Addr::from(13)));
        assert!(!iter.contains(Ipv4Addr::from(14)));
        assert!(!iter.contains(Ipv4Addr::from(15)));
    }
}
