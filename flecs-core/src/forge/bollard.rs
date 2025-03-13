use crate::relic::network::Network;
use crate::Result;
use std::str::FromStr;

const PARENT_IDENTIFIER: &str = "parent";

pub trait BollardNetworkExtension {
    fn subnet(&self) -> Option<Result<Network>>;
    fn parent_network(&self) -> Option<String>;
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

    fn parent_network(&self) -> Option<String> {
        self.options.as_ref()?.get(PARENT_IDENTIFIER).cloned()
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
}
