use crate::jeweler::network::NetworkKind;
use crate::relic::network::{Ipv4Network, Ipv6Network, Network};
use crate::Result;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

const PARENT_IDENTIFIER: &str = "parent";

pub trait BollardNetworkExtension {
    fn subnet(&self) -> Option<Result<Network>>;
    fn subnet_ipv4(&self) -> Result<Option<Ipv4Network>>;
    fn subnet_ipv6(&self) -> Result<Option<Ipv6Network>>;
    fn subnets(&self) -> Result<Vec<Network>>;
    fn subnets_ipv4(&self) -> Result<Vec<Ipv4Network>>;
    fn subnets_ipv6(&self) -> Result<Vec<Ipv6Network>>;
    fn gateways(&self) -> Result<Vec<IpAddr>>;
    fn gateways_ipv4(&self) -> Result<Vec<Ipv4Addr>>;
    fn gateway_ipv4(&self) -> Result<Option<Ipv4Addr>>;
    fn parent_network(&self) -> Option<String>;
    fn guess_network_kind(&self) -> NetworkKind;
}

impl BollardNetworkExtension for bollard::models::Network {
    fn subnet(&self) -> Option<Result<Network>> {
        let subnet = self
            .ipam
            .as_ref()?
            .config
            .as_ref()?
            .first()?
            .subnet
            .clone()?;
        match Network::from_str(&subnet) {
            Ok(network) => Some(Ok(network)),
            Err(e) => Some(Err(e)),
        }
    }

    fn subnet_ipv4(&self) -> Result<Option<Ipv4Network>> {
        Ok(self.subnets_ipv4()?.first().copied())
    }

    fn subnet_ipv6(&self) -> Result<Option<Ipv6Network>> {
        Ok(self.subnets_ipv6()?.first().copied())
    }

    fn subnets(&self) -> Result<Vec<Network>> {
        if let Some(bollard::models::Ipam {
            config: Some(configs),
            ..
        }) = self.ipam.as_ref()
        {
            configs
                .iter()
                .filter_map(|ipam| ipam.subnet.as_deref())
                .map(Network::from_str)
                .collect()
        } else {
            Ok(Vec::new())
        }
    }

    fn subnets_ipv4(&self) -> Result<Vec<Ipv4Network>> {
        Ok(self
            .subnets()?
            .into_iter()
            .filter_map(|subnet| match subnet {
                Network::Ipv4(subnet) => Some(subnet),
                _ => None,
            })
            .collect())
    }

    fn subnets_ipv6(&self) -> Result<Vec<Ipv6Network>> {
        Ok(self
            .subnets()?
            .into_iter()
            .filter_map(|subnet| match subnet {
                Network::Ipv6(subnet) => Some(subnet),
                _ => None,
            })
            .collect())
    }

    fn gateways(&self) -> Result<Vec<IpAddr>> {
        if let Some(bollard::models::Ipam {
            config: Some(configs),
            ..
        }) = self.ipam.as_ref()
        {
            configs
                .iter()
                .filter_map(|ipam| ipam.gateway.as_deref())
                .map(|gateway| IpAddr::from_str(gateway).map_err(|e| anyhow::anyhow!(e)))
                .collect()
        } else {
            Ok(Vec::new())
        }
    }

    fn gateways_ipv4(&self) -> Result<Vec<Ipv4Addr>> {
        Ok(self
            .gateways()?
            .into_iter()
            .filter_map(|gateway| match gateway {
                IpAddr::V4(gateway) => Some(gateway),
                _ => None,
            })
            .collect())
    }

    fn gateway_ipv4(&self) -> Result<Option<Ipv4Addr>> {
        Ok(self.gateways_ipv4()?.first().copied())
    }

    fn parent_network(&self) -> Option<String> {
        self.options.as_ref()?.get(PARENT_IDENTIFIER).cloned()
    }

    fn guess_network_kind(&self) -> NetworkKind {
        if let Some(options) = self.options.as_ref() {
            match options.get("ipvlan_mode").map(|s| s.as_str()) {
                Some("l2") => return NetworkKind::IpvlanL2,
                Some("l3") => return NetworkKind::IpvlanL3,
                _ => {}
            }
        };
        match self.driver.as_deref() {
            Some(driver) => NetworkKind::from(driver),
            None => NetworkKind::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn get_parent_network_no_options() {
        let network = bollard::models::Network {
            options: None,
            ..Default::default()
        };
        assert_eq!(network.parent_network(), None);
    }

    #[test]
    fn get_parent_network_no_parent() {
        let network = bollard::models::Network {
            options: Some(HashMap::from([
                ("option1".to_string(), "value".to_string()),
                ("option2".to_string(), "value".to_string()),
            ])),
            ..Default::default()
        };
        assert_eq!(network.parent_network(), None);
    }

    #[test]
    fn get_parent_network() {
        let network = bollard::models::Network {
            options: Some(HashMap::from([
                ("option1".to_string(), "value".to_string()),
                ("option2".to_string(), "value".to_string()),
                (PARENT_IDENTIFIER.to_string(), "some_parent".to_string()),
            ])),
            ..Default::default()
        };
        assert_eq!(network.parent_network(), Some("some_parent".to_string()));
    }

    #[test]
    fn get_subnet_no_ipam() {
        let network = bollard::models::Network {
            ipam: None,
            ..Default::default()
        };
        assert!(network.subnet().is_none());
    }

    #[test]
    fn get_subnet_no_config() {
        let network = bollard::models::Network {
            ipam: Some(Default::default()),
            ..Default::default()
        };
        assert!(network.subnet().is_none());
    }

    #[test]
    fn get_subnet_empty_config() {
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![]),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(network.subnet().is_none());
    }

    #[test]
    fn get_subnet_no_subnet() {
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![bollard::models::IpamConfig::default()]),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(network.subnet().is_none());
    }

    #[test]
    fn get_subnet_invalid_subnet() {
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![bollard::models::IpamConfig {
                    subnet: Some("12334567".to_string()),
                    ..bollard::models::IpamConfig::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(network.subnet().unwrap().is_err());
    }

    #[test]
    fn get_subnet_valid_subnet() {
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![bollard::models::IpamConfig {
                    subnet: Some("10.27.12.0/24".to_string()),
                    ..bollard::models::IpamConfig::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(
            network.subnet().unwrap().unwrap(),
            Network::from_str("10.27.12.0/24").unwrap()
        );
    }

    #[test]
    fn get_subnets_valid() {
        let network_1 = Network::from_str("10.51.56.128/25").unwrap();
        let network_2 = Network::from_str("60.0.0.0/8").unwrap();
        let network_3 = Network::from_str("1234:1234::/32").unwrap();
        let network_4 = Network::from_str("23.80.0.0/16").unwrap();
        let ipam_configs = vec![
            bollard::models::IpamConfig {
                subnet: Some(network_1.to_string()),
                ..Default::default()
            },
            Default::default(),
            bollard::models::IpamConfig {
                subnet: Some(network_2.to_string()),
                ..Default::default()
            },
            Default::default(),
            bollard::models::IpamConfig {
                subnet: Some(network_3.to_string()),
                ..Default::default()
            },
            bollard::models::IpamConfig {
                subnet: Some(network_4.to_string()),
                ..Default::default()
            },
            Default::default(),
            Default::default(),
        ];
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(ipam_configs),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(
            network.subnets().unwrap(),
            vec![network_1, network_2, network_3, network_4]
        );
    }

    #[test]
    fn get_subnets_invalid() {
        let network_1 = Network::from_str("10.51.56.128/25").unwrap();
        let network_2 = Network::from_str("60.0.0.0/8").unwrap();
        let network_3 = Network::from_str("23.80.0.0/16").unwrap();
        let ipam_configs = vec![
            bollard::models::IpamConfig {
                subnet: Some(network_1.to_string()),
                ..Default::default()
            },
            Default::default(),
            bollard::models::IpamConfig {
                subnet: Some(network_2.to_string()),
                ..Default::default()
            },
            Default::default(),
            bollard::models::IpamConfig {
                subnet: Some(network_3.to_string()),
                ..Default::default()
            },
            bollard::models::IpamConfig {
                subnet: Some("invalid".to_string()),
                ..Default::default()
            },
            Default::default(),
            Default::default(),
        ];
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(ipam_configs),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(network.subnets().is_err());
    }

    #[test]
    fn get_subnets_empty() {
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(Vec::new()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(network.subnets().unwrap().is_empty());
    }

    #[test]
    fn get_subnets_no_config() {
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: None,
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(network.subnets().unwrap().is_empty());
    }

    #[test]
    fn get_subnets_no_ipam() {
        let network = bollard::models::Network {
            ipam: None,
            ..Default::default()
        };
        assert!(network.subnets().unwrap().is_empty());
    }

    #[test]
    fn get_gateways_valid() {
        let gateway_1 = IpAddr::from_str("10.51.56.1").unwrap();
        let gateway_2 = IpAddr::from_str("60.0.0.22").unwrap();
        let gateway_3 = IpAddr::from_str("1234:1234::1").unwrap();
        let gateway_4 = IpAddr::from_str("23.80.0.16").unwrap();
        let ipam_configs = vec![
            bollard::models::IpamConfig {
                gateway: Some(gateway_1.to_string()),
                ..Default::default()
            },
            Default::default(),
            bollard::models::IpamConfig {
                gateway: Some(gateway_2.to_string()),
                ..Default::default()
            },
            Default::default(),
            bollard::models::IpamConfig {
                gateway: Some(gateway_3.to_string()),
                ..Default::default()
            },
            bollard::models::IpamConfig {
                gateway: Some(gateway_4.to_string()),
                ..Default::default()
            },
            Default::default(),
            Default::default(),
        ];
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(ipam_configs),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(
            network.gateways().unwrap(),
            vec![gateway_1, gateway_2, gateway_3, gateway_4]
        );
    }

    #[test]
    fn get_gateways_invalid() {
        let gateway_1 = IpAddr::from_str("10.51.56.1").unwrap();
        let gateway_2 = IpAddr::from_str("60.0.0.22").unwrap();
        let gateway_3 = IpAddr::from_str("23.80.0.16").unwrap();
        let ipam_configs = vec![
            bollard::models::IpamConfig {
                gateway: Some(gateway_1.to_string()),
                ..Default::default()
            },
            Default::default(),
            bollard::models::IpamConfig {
                gateway: Some(gateway_2.to_string()),
                ..Default::default()
            },
            Default::default(),
            bollard::models::IpamConfig {
                gateway: Some("invalid".to_string()),
                ..Default::default()
            },
            bollard::models::IpamConfig {
                gateway: Some(gateway_3.to_string()),
                ..Default::default()
            },
            Default::default(),
            Default::default(),
        ];
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(ipam_configs),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(network.gateways().is_err());
    }

    #[test]
    fn get_gateways_empty() {
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(Vec::new()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(network.gateways().unwrap().is_empty());
    }

    #[test]
    fn get_gateways_no_config() {
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: None,
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(network.gateways().unwrap().is_empty());
    }

    #[test]
    fn get_gateways_no_ipam() {
        let network = bollard::models::Network {
            ipam: None,
            ..Default::default()
        };
        assert!(network.gateways().unwrap().is_empty());
    }

    #[test]
    fn guess_network_kind_ipvlan_l2() {
        let network = bollard::models::Network {
            options: Some(HashMap::from([(
                "ipvlan_mode".to_string(),
                "l2".to_string(),
            )])),
            ..Default::default()
        };
        assert_eq!(network.guess_network_kind(), NetworkKind::IpvlanL2);
    }

    #[test]
    fn guess_network_kind_ipvlan_l3() {
        let network = bollard::models::Network {
            options: Some(HashMap::from([(
                "ipvlan_mode".to_string(),
                "l3".to_string(),
            )])),
            ..Default::default()
        };
        assert_eq!(network.guess_network_kind(), NetworkKind::IpvlanL3);
    }

    #[test]
    fn guess_network_kind_ipvlan_l4() {
        let network = bollard::models::Network {
            options: Some(HashMap::from([(
                "ipvlan_mode".to_string(),
                "l4".to_string(),
            )])),
            ..Default::default()
        };
        assert_eq!(network.guess_network_kind(), NetworkKind::Unknown);
    }

    #[test]
    fn guess_network_kind_from_driver() {
        let network = bollard::models::Network {
            driver: Some("bridge".to_string()),
            ..Default::default()
        };
        assert_eq!(network.guess_network_kind(), NetworkKind::Bridge);
    }

    #[test]
    fn guess_network_kind_unknown() {
        assert_eq!(
            bollard::models::Network::default().guess_network_kind(),
            NetworkKind::Unknown
        );
    }

    #[test]
    fn get_subnets_ipv4_valid() {
        let network_1 = Ipv4Network::from_str("10.51.56.128/25").unwrap();
        let network_2 = Ipv4Network::from_str("60.0.0.0/8").unwrap();
        let network_3 = Network::from_str("1234:1234::/32").unwrap();
        let network_4 = Ipv4Network::from_str("23.80.0.0/16").unwrap();
        let ipam_configs = vec![
            bollard::models::IpamConfig {
                subnet: Some(network_1.to_string()),
                ..Default::default()
            },
            Default::default(),
            bollard::models::IpamConfig {
                subnet: Some(network_2.to_string()),
                ..Default::default()
            },
            Default::default(),
            bollard::models::IpamConfig {
                subnet: Some(network_3.to_string()),
                ..Default::default()
            },
            bollard::models::IpamConfig {
                subnet: Some(network_4.to_string()),
                ..Default::default()
            },
            Default::default(),
            Default::default(),
        ];
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(ipam_configs),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(
            network.subnets_ipv4().unwrap(),
            vec![network_1, network_2, network_4]
        );
    }

    #[test]
    fn get_subnets_ipv4_err() {
        let ipam_configs = vec![bollard::models::IpamConfig {
            subnet: Some("invalid".to_string()),
            ..Default::default()
        }];
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(ipam_configs),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(network.subnets_ipv4().is_err());
    }

    #[test]
    fn get_subnet_ipv4_some() {
        let expected_network = Ipv4Network::from_str("23.80.0.0/16").unwrap();
        let ipam_configs = vec![bollard::models::IpamConfig {
            subnet: Some(expected_network.to_string()),
            ..Default::default()
        }];
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(ipam_configs),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(network.subnet_ipv4().unwrap(), Some(expected_network));
    }

    #[test]
    fn get_subnet_ipv4_none() {
        let network = Network::from_str("aabb::/16").unwrap();
        let ipam_configs = vec![bollard::models::IpamConfig {
            subnet: Some(network.to_string()),
            ..Default::default()
        }];
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(ipam_configs),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(network.subnet_ipv4().unwrap(), None);
    }

    #[test]
    fn get_subnet_ipv4_invalid() {
        let ipam_configs = vec![bollard::models::IpamConfig {
            subnet: Some("invalid".to_string()),
            ..Default::default()
        }];
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(ipam_configs),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(network.subnet_ipv4().is_err());
    }

    #[test]
    fn get_gateways_ipv4_valid() {
        let gateway_1 = Ipv4Addr::from_str("10.51.56.1").unwrap();
        let gateway_2 = Ipv4Addr::from_str("60.0.20.100").unwrap();
        let gateway_3 = IpAddr::from_str("1234:1234:abcd::1000").unwrap();
        let gateway_4 = Ipv4Addr::from_str("23.80.0.2").unwrap();
        let ipam_configs = vec![
            bollard::models::IpamConfig {
                gateway: Some(gateway_1.to_string()),
                ..Default::default()
            },
            Default::default(),
            bollard::models::IpamConfig {
                gateway: Some(gateway_2.to_string()),
                ..Default::default()
            },
            Default::default(),
            bollard::models::IpamConfig {
                gateway: Some(gateway_3.to_string()),
                ..Default::default()
            },
            bollard::models::IpamConfig {
                gateway: Some(gateway_4.to_string()),
                ..Default::default()
            },
            Default::default(),
            Default::default(),
        ];
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(ipam_configs),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(
            network.gateways_ipv4().unwrap(),
            vec![gateway_1, gateway_2, gateway_4]
        );
    }

    #[test]
    fn get_gateways_ipv4_err() {
        let ipam_configs = vec![bollard::models::IpamConfig {
            gateway: Some("invalid".to_string()),
            ..Default::default()
        }];
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(ipam_configs),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(network.gateways_ipv4().is_err());
    }

    #[test]
    fn get_gateway_ipv4_some() {
        let expected_gateway = Ipv4Addr::from_str("23.80.0.2").unwrap();
        let ipam_configs = vec![bollard::models::IpamConfig {
            gateway: Some(expected_gateway.to_string()),
            ..Default::default()
        }];
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(ipam_configs),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(network.gateway_ipv4().unwrap(), Some(expected_gateway));
    }

    #[test]
    fn get_gateway_ipv4_none() {
        let gateway = IpAddr::from_str("aabb::1").unwrap();
        let ipam_configs = vec![bollard::models::IpamConfig {
            gateway: Some(gateway.to_string()),
            ..Default::default()
        }];
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(ipam_configs),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(network.gateway_ipv4().unwrap(), None);
    }

    #[test]
    fn get_gateway_ipv4_invalid() {
        let ipam_configs = vec![bollard::models::IpamConfig {
            gateway: Some("invalid".to_string()),
            ..Default::default()
        }];
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(ipam_configs),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(network.gateway_ipv4().is_err());
    }
}
