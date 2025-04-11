pub mod host_port_range;
use crate::fsm::server_impl::api::v2::instances::instance_id::config::ports::port_mappings_to_instance_ports;
use crate::jeweler::gem::instance::docker::config::TransportProtocol;
use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::manifest::single::{PortMapping, PortRange};
use crate::sorcerer::instancius::{Instancius, QueryInstanceConfigError};
use crate::vault::Vault;
use anyhow::Error;
use flecsd_axum_server::apis::instances::{
    InstancesInstanceIdConfigPortsTransportProtocolDeleteResponse as DeleteResponse,
    InstancesInstanceIdConfigPortsTransportProtocolGetResponse as GetResponse,
    InstancesInstanceIdConfigPortsTransportProtocolPutResponse as PutResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    InstancesInstanceIdConfigPortsTransportProtocolDeletePathParams as DeletePathParams,
    InstancesInstanceIdConfigPortsTransportProtocolGetPathParams as GetPathParams,
    InstancesInstanceIdConfigPortsTransportProtocolPutPathParams as PutPathParams,
};
use std::str::FromStr;
use std::sync::Arc;

type PutRequest = Vec<models::InstancePortMapping>;

pub async fn delete<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: DeletePathParams,
) -> DeleteResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .delete_instance_config_protocol_port_mappings(
            vault,
            instance_id,
            path_params.transport_protocol.into(),
        )
        .await
    {
        Ok(_) => DeleteResponse::Status200_RemovedAllPublishedPortsOfInstanceWithThisInstance,
        Err(QueryInstanceConfigError::NotFound(_)) => {
            DeleteResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo::new())
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
        .get_instance_config_protocol_port_mappings(
            vault,
            instance_id,
            path_params.transport_protocol.into(),
        )
        .await
    {
        Ok(port_mapping) => GetResponse::Status200_PublishedPortsForInstanceWithThisInstance(
            port_mappings_to_instance_ports(&port_mapping),
        ),
        Err(QueryInstanceConfigError::NotFound(_)) => {
            GetResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo::new())
        }
        Err(e @ QueryInstanceConfigError::NotSupported(_)) => {
            GetResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

pub async fn put<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: PutPathParams,
    request: PutRequest,
) -> PutResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    let port_mapping = match request
        .into_iter()
        .map(PortMapping::try_from)
        .collect::<Result<Vec<_>, _>>()
    {
        Err(e) => {
            return PutResponse::Status400_MalformedRequest(models::AdditionalInfo::new(format!(
                "Invalid port mapping: {e}"
            )))
        }
        Ok(port_mapping) => port_mapping,
    };
    if let Err(errors) = validate_port_mappings(&port_mapping) {
        return PutResponse::Status400_MalformedRequest(models::AdditionalInfo::new(format!(
            "Invalid port mapping: {}",
            errors.join("\n")
        )));
    }
    match instancius
        .put_instance_config_protocol_port_mappings(
            vault,
            instance_id,
            port_mapping,
            path_params.transport_protocol.into(),
        )
        .await
    {
        Ok(_) => PutResponse::Status200_PublishedPortsOfInstanceWithThisInstance,
        Err(QueryInstanceConfigError::NotFound(_)) => {
            PutResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo::new())
        }
        Err(e @ QueryInstanceConfigError::NotSupported(_)) => {
            PutResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

fn validate_port_mappings(port_mappings: &[PortMapping]) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for port_mapping in port_mappings {
        if let PortMapping::Range { from, to } = port_mapping {
            if from.range().len() != to.range().len() {
                errors.push(
                    format!("The size of the container port range ({to}) and host port range ({from}) has to be equal")
                )
            }
        }
    }
    for (i, one) in port_mappings.iter().enumerate() {
        for (j, two) in port_mappings.iter().enumerate() {
            if i != j && one.do_host_ports_overlap(two) {
                errors.push(format!(
                    "Host ports of mapping {one} overlaps with host ports of mapping {two}"
                ))
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

impl TryFrom<models::InstancePortMapping> for PortMapping {
    type Error = Error;

    fn try_from(value: models::InstancePortMapping) -> Result<Self, Self::Error> {
        match value {
            models::InstancePortMapping::InstancePortMappingRange(mapping) => Ok(Self::Range {
                from: PortRange::try_from(mapping.host_ports)?,
                to: PortRange::try_from(mapping.container_ports)?,
            }),
            models::InstancePortMapping::InstancePortMappingSingle(mapping) => {
                Ok(Self::Single(mapping.host_port, mapping.container_port))
            }
        }
    }
}

impl From<models::TransportProtocol> for TransportProtocol {
    fn from(value: models::TransportProtocol) -> Self {
        match value {
            models::TransportProtocol::Tcp => TransportProtocol::Tcp,
            models::TransportProtocol::Udp => TransportProtocol::Udp,
            models::TransportProtocol::Sctp => TransportProtocol::Sctp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::gem::manifest::single::{PortMapping, PortRange};
    use crate::sorcerer::instancius::MockInstancius;
    use flecsd_axum_server::models;

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_protocol_port_mappings()
            .withf(move |_, id, protocol| id.value == 6 && *protocol == TransportProtocol::Tcp)
            .once()
            .returning(|_, _, _| Ok(Vec::new()));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: "00000006".to_string(),
                    transport_protocol: models::TransportProtocol::Tcp
                },
            )
            .await,
            DeleteResponse::Status200_RemovedAllPublishedPortsOfInstanceWithThisInstance
        );
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_protocol_port_mappings()
            .withf(move |_, id, protocol| {
                id.value == 0xaaaaaaaa && *protocol == TransportProtocol::Tcp
            })
            .once()
            .returning(|_, _, _| {
                Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                    0xaaaaaaaa,
                )))
            });
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: "aaaaaaaa".to_string(),
                    transport_protocol: models::TransportProtocol::Tcp
                },
            )
            .await,
            DeleteResponse::Status404_ResourceNotFound(_)
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_protocol_port_mappings()
            .withf(move |_, id, protocol| {
                id.value == 0xabcdabcd && *protocol == TransportProtocol::Tcp
            })
            .once()
            .returning(|_, _, _| {
                Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                    0xabcdabcd,
                )))
            });
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "abcdabcd".to_string(),
                    transport_protocol: models::TransportProtocol::Tcp
                },
            )
            .await,
            GetResponse::Status404_ResourceNotFound(_)
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_protocol_port_mappings()
            .withf(move |_, id, protocol| id.value == 6 && *protocol == TransportProtocol::Tcp)
            .once()
            .returning(|_, _, _| Ok(vec![PortMapping::Single(80, 8080)]));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "00000006".to_string(),
                    transport_protocol: models::TransportProtocol::Tcp
                },
            )
            .await,
            GetResponse::Status200_PublishedPortsForInstanceWithThisInstance(vec![
                models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                    models::InstancePortMappingSingle {
                        host_port: 80,
                        container_port: 8080
                    }
                ))
            ])
        );
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_400_overlap() {
        let vault = crate::vault::tests::create_empty_test_vault();
        let instancius = MockInstancius::new();
        let port_mappings = vec![
            models::InstancePortMapping::InstancePortMappingRange(Box::new(
                models::InstancePortMappingRange {
                    host_ports: models::PortRange {
                        start: 2000,
                        end: 3000,
                    },
                    container_ports: models::PortRange {
                        start: 6000,
                        end: 7000,
                    },
                },
            )),
            models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                models::InstancePortMappingSingle {
                    host_port: 2500,
                    container_port: 10000,
                },
            )),
        ];
        assert!(matches!(
            put(
                vault,
                Arc::new(instancius),
                PutPathParams {
                    instance_id: "00000006".to_string(),
                    transport_protocol: models::TransportProtocol::Sctp,
                },
                port_mappings,
            )
            .await,
            PutResponse::Status400_MalformedRequest(_)
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_400_port_mapping() {
        let vault = crate::vault::tests::create_empty_test_vault();
        let instancius = MockInstancius::new();
        assert!(matches!(
            put(
                vault,
                Arc::new(instancius),
                PutPathParams {
                    instance_id: "00000006".to_string(),
                    transport_protocol: models::TransportProtocol::Udp,
                },
                vec![models::InstancePortMapping::InstancePortMappingRange(
                    Box::new(models::InstancePortMappingRange {
                        host_ports: models::PortRange {
                            start: 2000,
                            end: 1000,
                        },
                        container_ports: models::PortRange {
                            start: 6000,
                            end: 7000,
                        },
                    },)
                )],
            )
            .await,
            PutResponse::Status400_MalformedRequest(_)
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_protocol_port_mappings()
            .withf(move |_, id, mapping, protocol| {
                id.value == 0x77778888 && *protocol == TransportProtocol::Udp && mapping.is_empty()
            })
            .once()
            .returning(|_, _, _, _| {
                Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                    0x77778888,
                )))
            });
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            put(
                vault,
                Arc::new(instancius),
                PutPathParams {
                    instance_id: "77778888".to_string(),
                    transport_protocol: models::TransportProtocol::Udp,
                },
                vec![],
            )
            .await,
            PutResponse::Status404_ResourceNotFound(_)
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_protocol_port_mappings()
            .withf(move |_, id, mapping, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Udp
                    && *mapping
                        == vec![
                            PortMapping::Single(100, 20),
                            PortMapping::Range {
                                from: PortRange::new(2000..=3000),
                                to: PortRange::new(6000..=7000),
                            },
                            PortMapping::Single(60, 70),
                        ]
            })
            .once()
            .returning(|_, _, _, _| Ok(()));
        let vault = crate::vault::tests::create_empty_test_vault();
        let port_mappings = vec![
            models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                models::InstancePortMappingSingle {
                    host_port: 100,
                    container_port: 20,
                },
            )),
            models::InstancePortMapping::InstancePortMappingRange(Box::new(
                models::InstancePortMappingRange {
                    host_ports: models::PortRange {
                        start: 2000,
                        end: 3000,
                    },
                    container_ports: models::PortRange {
                        start: 6000,
                        end: 7000,
                    },
                },
            )),
            models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                models::InstancePortMappingSingle {
                    host_port: 60,
                    container_port: 70,
                },
            )),
        ];
        assert_eq!(
            put(
                vault,
                Arc::new(instancius),
                PutPathParams {
                    instance_id: "00000006".to_string(),
                    transport_protocol: models::TransportProtocol::Udp,
                },
                port_mappings
            )
            .await,
            PutResponse::Status200_PublishedPortsOfInstanceWithThisInstance
        );
    }

    #[test]
    fn validate_port_mappings_empty() {
        assert!(validate_port_mappings(&[]).is_ok());
    }

    #[test]
    fn validate_port_mappings_ok() {
        assert!(validate_port_mappings(&[PortMapping::Single(10, 20)]).is_ok());
        assert!(validate_port_mappings(&[PortMapping::Range {
            from: PortRange::new(10..=20),
            to: PortRange::new(70..=80)
        }])
        .is_ok());
        assert!(validate_port_mappings(&[
            PortMapping::Range {
                from: PortRange::new(600..=700),
                to: PortRange::new(800..=900)
            },
            PortMapping::Range {
                from: PortRange::new(10..=20),
                to: PortRange::new(70..=80)
            },
            PortMapping::Single(1, 20),
        ])
        .is_ok());
    }

    #[test]
    fn validate_port_mappings_err_invalid_range() {
        let errors = validate_port_mappings(&[PortMapping::Range {
            from: PortRange::new(10..=20),
            to: PortRange::new(30..=80),
        }])
        .err()
        .unwrap();
        assert_eq!(errors.len(), 1, "{errors:?}");
    }

    #[test]
    fn validate_port_mappings_err_multiple_invalid_range() {
        let errors = validate_port_mappings(&[
            PortMapping::Range {
                from: PortRange::new(10..=20),
                to: PortRange::new(30..=80),
            },
            PortMapping::Range {
                from: PortRange::new(70..=700),
                to: PortRange::new(30..=80),
            },
            PortMapping::Single(1000, 2000),
        ])
        .err()
        .unwrap();
        assert_eq!(errors.len(), 2, "{errors:?}");
    }

    #[test]
    fn validate_port_mappings_err_overlap() {
        let errors = validate_port_mappings(&[
            PortMapping::Range {
                from: PortRange::new(10..=20),
                to: PortRange::new(30..=40),
            },
            PortMapping::Single(15, 50),
        ])
        .err()
        .unwrap();
        assert_eq!(errors.len(), 2, "{errors:?}");
    }

    #[test]
    fn validate_port_mappings_err_multiple_overlap() {
        let errors = validate_port_mappings(&[
            PortMapping::Range {
                from: PortRange::new(10..=20),
                to: PortRange::new(30..=40),
            },
            PortMapping::Range {
                from: PortRange::new(12..=17),
                to: PortRange::new(60..=65),
            },
            PortMapping::Single(15, 50),
        ])
        .err()
        .unwrap();
        assert_eq!(errors.len(), 6, "{errors:?}");
    }

    #[test]
    fn validate_port_mappings_err_multiple() {
        let errors = validate_port_mappings(&[
            PortMapping::Range {
                from: PortRange::new(10..=20),
                to: PortRange::new(30..=80),
            },
            PortMapping::Range {
                from: PortRange::new(12..=17),
                to: PortRange::new(60..=90),
            },
            PortMapping::Single(15, 50),
        ])
        .err()
        .unwrap();
        assert_eq!(errors.len(), 8, "{errors:?}");
    }

    #[test]
    fn try_from_instance_port_mapping_range_ok() {
        let instance_port_mapping = models::InstancePortMapping::InstancePortMappingRange(
            Box::new(models::InstancePortMappingRange {
                host_ports: models::PortRange { start: 7, end: 10 },
                container_ports: models::PortRange { start: 17, end: 20 },
            }),
        );
        let expected_mapping = PortMapping::Range {
            from: PortRange::new(7..=10),
            to: PortRange::new(17..=20),
        };
        assert_eq!(
            PortMapping::try_from(instance_port_mapping).unwrap(),
            expected_mapping
        );
    }

    #[test]
    fn try_from_instance_port_mapping_range_host_err() {
        let instance_port_mapping = models::InstancePortMapping::InstancePortMappingRange(
            Box::new(models::InstancePortMappingRange {
                host_ports: models::PortRange { start: 70, end: 20 },
                container_ports: models::PortRange { start: 17, end: 20 },
            }),
        );
        assert!(PortMapping::try_from(instance_port_mapping).is_err(),);
    }

    #[test]
    fn try_from_instance_port_mapping_range_container_err() {
        let instance_port_mapping = models::InstancePortMapping::InstancePortMappingRange(
            Box::new(models::InstancePortMappingRange {
                host_ports: models::PortRange { start: 70, end: 80 },
                container_ports: models::PortRange { start: 60, end: 40 },
            }),
        );
        assert!(PortMapping::try_from(instance_port_mapping).is_err(),);
    }

    #[test]
    fn try_from_instance_port_mapping_single_ok() {
        let instance_port_mapping = models::InstancePortMapping::InstancePortMappingSingle(
            Box::new(models::InstancePortMappingSingle {
                host_port: 10,
                container_port: 17,
            }),
        );
        let expected_mapping = PortMapping::Single(10, 17);
        assert_eq!(
            PortMapping::try_from(instance_port_mapping).unwrap(),
            expected_mapping
        );
    }

    #[test]
    fn transport_protocol_from() {
        assert_eq!(
            TransportProtocol::from(models::TransportProtocol::Tcp),
            TransportProtocol::Tcp
        );
        assert_eq!(
            TransportProtocol::from(models::TransportProtocol::Udp),
            TransportProtocol::Udp
        );
        assert_eq!(
            TransportProtocol::from(models::TransportProtocol::Sctp),
            TransportProtocol::Sctp
        );
    }
}
