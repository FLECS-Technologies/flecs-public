pub mod transport_protocol;
use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::manifest::single::PortMapping;
use crate::sorcerer::instancius::{Instancius, QueryInstanceConfigError};
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::{
    InstancesInstanceIdConfigPortsDeleteResponse as DeleteResponse,
    InstancesInstanceIdConfigPortsGetResponse as GetResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    InstancesInstanceIdConfigPortsDeletePathParams as DeletePathParams,
    InstancesInstanceIdConfigPortsGetPathParams as GetPathParams,
};
use std::str::FromStr;
use std::sync::Arc;

pub async fn delete<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: DeletePathParams,
) -> DeleteResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .delete_instance_config_port_mappings(vault, instance_id)
        .await
    {
        Ok(_) => DeleteResponse::Status200_ExposedPortsOfInstanceWithThisInstance,
        Err(QueryInstanceConfigError::NotFound(_)) => {
            DeleteResponse::Status404_NoInstanceWithThisInstance
        }
        Err(e @ QueryInstanceConfigError::NotSupported(_)) => {
            DeleteResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .get_instance_config_port_mappings(vault, instance_id)
        .await
    {
        Err(QueryInstanceConfigError::NotFound(_)) => {
            GetResponse::Status404_NoInstanceWithThisInstance
        }
        Err(e @ QueryInstanceConfigError::NotSupported(_)) => {
            GetResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(mapping) => GetResponse::Status200_Success(models::InstancePorts {
            tcp: port_mappings_to_instance_ports(&mapping.tcp),
            udp: port_mappings_to_instance_ports(&mapping.udp),
            sctp: port_mappings_to_instance_ports(&mapping.sctp),
        }),
    }
}

fn port_mappings_to_instance_ports(
    port_mappings: &[PortMapping],
) -> Vec<models::InstancePortMapping> {
    port_mappings
        .iter()
        .map(models::InstancePortMapping::from)
        .collect()
}

impl From<&PortMapping> for models::InstancePortMapping {
    fn from(value: &PortMapping) -> Self {
        match value {
            PortMapping::Single(host, container) => {
                models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                    models::InstancePortMappingSingle {
                        host_port: *host,
                        container_port: *container,
                    },
                ))
            }
            PortMapping::Range { from, to } => {
                models::InstancePortMapping::InstancePortMappingRange(Box::new(
                    models::InstancePortMappingRange {
                        host_ports: models::PortRange {
                            start: *from.range().start(),
                            end: *from.range().end(),
                        },
                        container_ports: models::PortRange {
                            start: *to.range().start(),
                            end: *to.range().end(),
                        },
                    },
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::gem::instance::docker::config::InstancePortMapping;
    use crate::jeweler::gem::manifest::single::PortRange;
    use crate::sorcerer::instancius::MockInstancius;

    #[tokio::test]
    async fn delete_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_port_mappings()
            .withf(move |_, id| id.value == 0x12341234)
            .once()
            .returning(|_, _| {
                Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                    0x12341234,
                )))
            });
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: "12341234".to_string(),
                },
            )
            .await,
            DeleteResponse::Status404_NoInstanceWithThisInstance
        ));
    }

    #[tokio::test]
    async fn delete_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_port_mappings()
            .withf(move |_, id| id.value == 6)
            .once()
            .returning(|_, _| Ok(()));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: "00000006".to_string(),
                },
            )
            .await,
            DeleteResponse::Status200_ExposedPortsOfInstanceWithThisInstance
        ));
    }

    #[tokio::test]
    async fn get_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_port_mappings()
            .withf(move |_, id| id.value == 0x12341234)
            .once()
            .returning(|_, _| {
                Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                    0x12341234,
                )))
            });
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "12341234".to_string(),
                },
            )
            .await,
            GetResponse::Status404_NoInstanceWithThisInstance
        ));
    }

    #[tokio::test]
    async fn get_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_port_mappings()
            .withf(move |_, id| id.value == 6)
            .once()
            .returning(|_, _| {
                Ok(InstancePortMapping {
                    tcp: vec![PortMapping::Single(80, 8080)],
                    udp: vec![PortMapping::Range {
                        from: PortRange::new(50..=100),
                        to: PortRange::new(150..=200),
                    }],
                    sctp: vec![],
                })
            });
        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "00000006".to_string(),
                },
            )
            .await,
            GetResponse::Status200_Success(models::InstancePorts {
                tcp: vec![models::InstancePortMapping::InstancePortMappingSingle(
                    Box::new(models::InstancePortMappingSingle {
                        host_port: 80,
                        container_port: 8080,
                    })
                )],
                udp: vec![models::InstancePortMapping::InstancePortMappingRange(
                    Box::new(models::InstancePortMappingRange {
                        host_ports: models::PortRange {
                            start: 50,
                            end: 100,
                        },
                        container_ports: models::PortRange {
                            start: 150,
                            end: 200,
                        },
                    })
                )],
                sctp: vec![],
            })
        );
    }

    #[test]
    fn port_mappings_to_instance_ports_test() {
        let port_mappings = [
            PortMapping::Single(100, 1000),
            PortMapping::Single(6, 110),
            PortMapping::Range {
                from: PortRange::new(10..=20),
                to: PortRange::new(20..=30),
            },
        ];
        assert_eq!(port_mappings_to_instance_ports(&[]), vec![]);
        assert_eq!(
            port_mappings_to_instance_ports(&port_mappings),
            vec![
                models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                    models::InstancePortMappingSingle {
                        host_port: 100,
                        container_port: 1000,
                    }
                )),
                models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                    models::InstancePortMappingSingle {
                        host_port: 6,
                        container_port: 110,
                    }
                )),
                models::InstancePortMapping::InstancePortMappingRange(Box::new(
                    models::InstancePortMappingRange {
                        host_ports: models::PortRange { start: 10, end: 20 },
                        container_ports: models::PortRange { start: 20, end: 30 },
                    }
                ))
            ]
        );
    }

    #[test]
    fn from_port_mapping_range() {
        let port_mapping = PortMapping::Range {
            from: PortRange::new(6..=9),
            to: PortRange::new(11..=14),
        };
        assert_eq!(
            models::InstancePortMapping::from(&port_mapping),
            models::InstancePortMapping::InstancePortMappingRange(Box::new(
                models::InstancePortMappingRange {
                    host_ports: models::PortRange { start: 6, end: 9 },
                    container_ports: models::PortRange { start: 11, end: 14 },
                }
            ))
        )
    }

    #[test]
    fn from_port_mapping_single() {
        let port_mapping = PortMapping::Single(100, 1000);
        assert_eq!(
            models::InstancePortMapping::from(&port_mapping),
            models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                models::InstancePortMappingSingle {
                    host_port: 100,
                    container_port: 1000,
                }
            ))
        )
    }
}
