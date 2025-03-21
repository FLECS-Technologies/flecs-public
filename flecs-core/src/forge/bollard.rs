use crate::jeweler::network::NetworkKind;
use crate::relic::network::Network;
use crate::Result;
use std::net::IpAddr;
use std::str::FromStr;

const PARENT_IDENTIFIER: &str = "parent";

pub trait BollardNetworkExtension {
    fn subnet(&self) -> Option<Result<Network>>;
    fn subnets(&self) -> Result<Vec<Network>>;
    fn gateways(&self) -> Result<Vec<IpAddr>>;
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
            vec![network_1, network_2, network_3]
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
        assert_eq!(
            network.gateways().unwrap(),
            vec![gateway_1, gateway_2, gateway_3]
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
}
