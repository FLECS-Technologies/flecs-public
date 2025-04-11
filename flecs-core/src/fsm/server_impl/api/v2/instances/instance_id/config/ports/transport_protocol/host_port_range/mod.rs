use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::manifest::single::{PortMapping, PortRange};
use crate::sorcerer::instancius::{Instancius, QueryInstanceConfigError};
use crate::vault::Vault;
use anyhow::Error;
use flecsd_axum_server::apis::instances::{
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeleteResponse as DeleteResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetResponse as GetResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutResponse as PutResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeletePathParams as DeletePathParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetPathParams as GetPathParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutPathParams as PutPathParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutRequest as PutRequest,
};
use std::str::FromStr;
use std::sync::Arc;

pub async fn delete<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: DeletePathParams,
) -> DeleteResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    let host_port_range = match parse_host_port_path_parameter(&path_params.host_port_range) {
        Ok(host_port_range) => host_port_range,
        Err(e) => return DeleteResponse::Status400_MalformedRequest(e),
    };
    match instancius
        .delete_instance_config_port_mapping_range(
            vault,
            instance_id,
            host_port_range,
            path_params.transport_protocol.into(),
        )
        .await
    {
        Err(QueryInstanceConfigError::NotFound(_)) => {
            DeleteResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
                additional_info: Some(format!("Instance {instance_id} does not exist")),
            })
        }
        Err(e @ QueryInstanceConfigError::NotSupported(_)) => {
            DeleteResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(false) => DeleteResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
            additional_info: Some(format!(
                "Host port range ({}) is not mapped to {instance_id}",
                host_port_range
            )),
        }),
        Ok(true) => DeleteResponse::Status200_Success,
    }
}

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    let host_port_range = match parse_host_port_path_parameter(&path_params.host_port_range) {
        Ok(host_port_range) => host_port_range,
        Err(e) => return GetResponse::Status400_MalformedRequest(e),
    };
    match instancius
        .get_instance_config_port_mapping_range(
            vault,
            instance_id,
            host_port_range,
            path_params.transport_protocol.into(),
        )
        .await
    {
        Err(QueryInstanceConfigError::NotFound(_)) => {
            GetResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
                additional_info: Some(format!("Instance {instance_id} does not exist")),
            })
        }
        Err(e @ QueryInstanceConfigError::NotSupported(_)) => {
            GetResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(None) => GetResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
            additional_info: Some(format!(
                "Host port range ({}) is not mapped to {instance_id}",
                host_port_range
            )),
        }),
        Ok(Some(PortMapping::Single(host_port, container_port))) => {
            GetResponse::Status200_Success(models::InstancePortMapping::InstancePortMappingSingle(
                Box::new(models::InstancePortMappingSingle {
                    host_port,
                    container_port,
                }),
            ))
        }
        Ok(Some(PortMapping::Range { from, to })) => {
            GetResponse::Status200_Success(models::InstancePortMapping::InstancePortMappingRange(
                Box::new(models::InstancePortMappingRange {
                    host_ports: from.into(),
                    container_ports: to.into(),
                }),
            ))
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
    let host_port_range = match parse_host_port_path_parameter(&path_params.host_port_range) {
        Ok(host_port_range) => host_port_range,
        Err(e) => return PutResponse::Status400_MalformedRequest(e),
    };
    let container_port_range = match PortRange::try_from(request) {
        Err(e) => {
            return PutResponse::Status400_MalformedRequest(models::AdditionalInfo::new(format!(
                "Invalid container port range: {e}"
            )))
        }
        Ok(host_port_range) => host_port_range,
    };
    if container_port_range.range().len() != host_port_range.range().len() {
        return PutResponse::Status400_MalformedRequest(models::AdditionalInfo::new(format!(
            "The size of the container port range ({container_port_range}) \
                        and host port range ({host_port_range}) has to be equal",
        )));
    }
    match instancius
        .put_instance_config_port_mapping(
            vault,
            instance_id,
            PortMapping::Range {
                from: host_port_range,
                to: container_port_range,
            }
            .normalize(),
            path_params.transport_protocol.into(),
        )
        .await
    {
        Err(e) => {
            PutResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(None) => PutResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
            additional_info: Some(format!("Instance {instance_id} does not exist")),
        }),
        Ok(Some(false)) => PutResponse::Status201_TheSpecifiedPortMappingWasCreated,
        Ok(Some(true)) => PutResponse::Status200_TheSpecifiedPortMappingWasSet,
    }
}

fn parse_host_port_path_parameter(
    path_parameter: &str,
) -> Result<PortRange, models::AdditionalInfo> {
    match (
        PortRange::from_str(path_parameter),
        u16::from_str(path_parameter),
    ) {
        (Ok(host_port_range), _) => Ok(host_port_range),
        (_, Ok(host_port)) => Ok(PortRange::new(host_port..=host_port)),
        (Err(e1), Err(e2)) => Err(models::AdditionalInfo {
            additional_info: format!(
                "Could not parse path parameter for host port range ({path_parameter}), expected \
                either one non-zero unsigned 16 bit integer ({e2}) or two non-zero unsigned 16 bit \
                integers seperated by dash ({e1})"
            ),
        }),
    }
}

impl TryFrom<models::PortRange> for PortRange {
    type Error = Error;

    fn try_from(value: models::PortRange) -> Result<Self, Self::Error> {
        Self::try_new(value.start, value.end)
    }
}

impl From<PortRange> for models::PortRange {
    fn from(value: PortRange) -> Self {
        Self {
            start: *value.range().start(),
            end: *value.range().end(),
        }
    }
}

impl TryFrom<PutRequest> for PortRange {
    type Error = Error;

    fn try_from(value: PutRequest) -> Result<Self, Self::Error> {
        match value {
            PutRequest::PortRange(range) => Self::try_from(*range),
            PutRequest::I32(port) => {
                let port = u16::try_from(*port)?;
                Ok(Self::new(port..=port))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod range {
        use super::*;
        use crate::jeweler::gem::instance::docker::config::TransportProtocol;
        use crate::sorcerer::instancius::MockInstancius;

        #[tokio::test]
        async fn delete_400() {
            let vault = crate::vault::tests::create_empty_test_vault();
            let instancius = MockInstancius::new();
            assert!(matches!(
                delete(
                    vault,
                    Arc::new(instancius),
                    DeletePathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "20-1".to_string(),
                    },
                )
                .await,
                DeleteResponse::Status400_MalformedRequest(_)
            ));
        }

        #[tokio::test]
        async fn delete_404_range() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_delete_instance_config_port_mapping_range()
                .withf(move |_, id, port, protocol| {
                    id.value == 6
                        && *protocol == TransportProtocol::Udp
                        && *port == PortRange::new(20..=70)
                })
                .once()
                .returning(|_, _, _, _| Ok(false));
            let vault = crate::vault::tests::create_empty_test_vault();
            assert!(matches!(
                delete(
                    vault,
                    Arc::new(instancius),
                    DeletePathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "20-70".to_string(),
                    },
                )
                .await,
                DeleteResponse::Status404_ResourceNotFound(_)
            ));
        }

        #[tokio::test]
        async fn delete_404_instance() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_delete_instance_config_port_mapping_range()
                .withf(move |_, id, port, protocol| {
                    id.value == 0xaabbccdd
                        && *protocol == TransportProtocol::Udp
                        && *port == PortRange::new(50..=100)
                })
                .once()
                .returning(|_, _, _, _| {
                    Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                        0xaabbccdd,
                    )))
                });
            let vault = crate::vault::tests::create_empty_test_vault();
            assert!(matches!(
                delete(
                    vault,
                    Arc::new(instancius),
                    DeletePathParams {
                        instance_id: "aabbccdd".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "50-100".to_string(),
                    },
                )
                .await,
                DeleteResponse::Status404_ResourceNotFound(_)
            ));
        }

        #[tokio::test]
        async fn delete_200() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_delete_instance_config_port_mapping_range()
                .withf(move |_, id, port, protocol| {
                    id.value == 6
                        && *protocol == TransportProtocol::Udp
                        && *port == PortRange::new(50..=100)
                })
                .once()
                .returning(|_, _, _, _| Ok(true));
            let vault = crate::vault::tests::create_empty_test_vault();
            assert_eq!(
                delete(
                    vault,
                    Arc::new(instancius),
                    DeletePathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "50-100".to_string(),
                    },
                )
                .await,
                DeleteResponse::Status200_Success
            );
        }

        #[tokio::test]
        async fn get_400_range() {
            let vault = crate::vault::tests::create_empty_test_vault();
            let instancius = MockInstancius::new();
            assert!(matches!(
                get(
                    vault,
                    Arc::new(instancius),
                    GetPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "70-4".to_string(),
                    },
                )
                .await,
                GetResponse::Status400_MalformedRequest(_)
            ));
        }

        #[tokio::test]
        async fn get_404_range() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_get_instance_config_port_mapping_range()
                .withf(move |_, id, port, protocol| {
                    id.value == 6
                        && *protocol == TransportProtocol::Udp
                        && *port == PortRange::new(70..=100)
                })
                .once()
                .returning(|_, _, _, _| Ok(None));
            let vault = crate::vault::tests::create_empty_test_vault();
            assert!(matches!(
                get(
                    vault,
                    Arc::new(instancius),
                    GetPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "70-100".to_string(),
                    },
                )
                .await,
                GetResponse::Status404_ResourceNotFound(_)
            ));
        }

        #[tokio::test]
        async fn get_404_instance() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_get_instance_config_port_mapping_range()
                .withf(move |_, id, port, protocol| {
                    id.value == 0x12345678
                        && *protocol == TransportProtocol::Udp
                        && *port == PortRange::new(50..=100)
                })
                .once()
                .returning(|_, _, _, _| {
                    Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                        0x12345678,
                    )))
                });
            let vault = crate::vault::tests::create_empty_test_vault();
            assert!(matches!(
                get(
                    vault,
                    Arc::new(instancius),
                    GetPathParams {
                        instance_id: "12345678".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "50-100".to_string(),
                    },
                )
                .await,
                GetResponse::Status404_ResourceNotFound(_)
            ));
        }

        #[tokio::test]
        async fn get_200_range() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_get_instance_config_port_mapping_range()
                .withf(move |_, id, port, protocol| {
                    id.value == 6
                        && *protocol == TransportProtocol::Udp
                        && *port == PortRange::new(50..=100)
                })
                .once()
                .returning(|_, _, _, _| {
                    Ok(Some(PortMapping::Range {
                        from: PortRange::new(50..=100),
                        to: PortRange::new(150..=200),
                    }))
                });
            let vault = crate::vault::tests::create_empty_test_vault();
            assert_eq!(
                get(
                    vault,
                    Arc::new(instancius),
                    GetPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "50-100".to_string(),
                    },
                )
                .await,
                GetResponse::Status200_Success(
                    models::InstancePortMapping::InstancePortMappingRange(Box::new(
                        models::InstancePortMappingRange {
                            host_ports: models::PortRange {
                                start: 50,
                                end: 100,
                            },
                            container_ports: models::PortRange {
                                start: 150,
                                end: 200,
                            },
                        }
                    ))
                )
            );
        }

        #[tokio::test]
        async fn get_200_single() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_get_instance_config_port_mapping_range()
                .withf(move |_, id, port, protocol| {
                    id.value == 6
                        && *protocol == TransportProtocol::Tcp
                        && *port == PortRange::new(80..=80)
                })
                .once()
                .returning(|_, _, _, _| Ok(Some(PortMapping::Single(80, 8080))));
            let vault = crate::vault::tests::create_empty_test_vault();
            assert_eq!(
                get(
                    vault,
                    Arc::new(instancius),
                    GetPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "80-80".to_string(),
                    },
                )
                .await,
                GetResponse::Status200_Success(
                    models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                        models::InstancePortMappingSingle {
                            host_port: 80,
                            container_port: 8080,
                        }
                    ))
                )
            );
        }

        #[tokio::test]
        async fn put_400_host_range() {
            let vault = crate::vault::tests::create_empty_test_vault();
            let instancius = MockInstancius::new();
            assert!(matches!(
                put(
                    vault,
                    Arc::new(instancius),
                    PutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                        host_port_range: "70-50".to_string(),
                    },
                    PutRequest::PortRange(Box::new(models::PortRange {
                        start: 200,
                        end: 220,
                    })),
                )
                .await,
                PutResponse::Status400_MalformedRequest(_)
            ));
        }

        #[tokio::test]
        async fn put_400_container_range() {
            let vault = crate::vault::tests::create_empty_test_vault();
            let instancius = MockInstancius::new();
            assert!(matches!(
                put(
                    vault,
                    Arc::new(instancius),
                    PutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                        host_port_range: "70-90".to_string(),
                    },
                    PutRequest::PortRange(Box::new(models::PortRange {
                        start: 200,
                        end: 180,
                    })),
                )
                .await,
                PutResponse::Status400_MalformedRequest(_)
            ));
        }

        #[tokio::test]
        async fn put_400_range_mismatch() {
            let vault = crate::vault::tests::create_empty_test_vault();
            let instancius = MockInstancius::new();
            assert!(matches!(
                put(
                    vault,
                    Arc::new(instancius),
                    PutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                        host_port_range: "70-90".to_string(),
                    },
                    PutRequest::PortRange(Box::new(models::PortRange {
                        start: 200,
                        end: 400,
                    }))
                )
                .await,
                PutResponse::Status400_MalformedRequest(_)
            ));
        }

        #[tokio::test]
        async fn put_400_overlap() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_put_instance_config_port_mapping()
                .withf(move |_, id, mapping, protocol| {
                    id.value == 6
                        && *protocol == TransportProtocol::Tcp
                        && *mapping
                            == PortMapping::Range {
                                from: PortRange::new(70..=90),
                                to: PortRange::new(200..=220),
                            }
                })
                .once()
                .returning(|_, _, _, _| Err(anyhow::anyhow!("TestError")));
            let vault = crate::vault::tests::create_empty_test_vault();
            assert!(matches!(
                put(
                    vault,
                    Arc::new(instancius),
                    PutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "70-90".to_string(),
                    },
                    PutRequest::PortRange(Box::new(models::PortRange {
                        start: 200,
                        end: 220,
                    })),
                )
                .await,
                PutResponse::Status400_MalformedRequest(_)
            ));
        }

        #[tokio::test]
        async fn put_404() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_put_instance_config_port_mapping()
                .withf(move |_, id, mapping, protocol| {
                    id.value == 0xffeeddcc
                        && *protocol == TransportProtocol::Sctp
                        && *mapping
                            == PortMapping::Range {
                                from: PortRange::new(1000..=1100),
                                to: PortRange::new(200..=300),
                            }
                })
                .once()
                .returning(|_, _, _, _| Ok(None));
            let vault = crate::vault::tests::create_empty_test_vault();
            assert!(matches!(
                put(
                    vault,
                    Arc::new(instancius),
                    PutPathParams {
                        instance_id: "ffeeddcc".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                        host_port_range: "1000-1100".to_string(),
                    },
                    PutRequest::PortRange(Box::new(models::PortRange {
                        start: 200,
                        end: 300,
                    }))
                )
                .await,
                PutResponse::Status404_ResourceNotFound(_)
            ));
        }

        #[tokio::test]
        async fn put_201() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_put_instance_config_port_mapping()
                .withf(move |_, id, mapping, protocol| {
                    id.value == 6
                        && *protocol == TransportProtocol::Sctp
                        && *mapping
                            == PortMapping::Range {
                                from: PortRange::new(1000..=1100),
                                to: PortRange::new(200..=300),
                            }
                })
                .once()
                .returning(|_, _, _, _| Ok(Some(false)));
            let vault = crate::vault::tests::create_empty_test_vault();
            assert_eq!(
                put(
                    vault,
                    Arc::new(instancius),
                    PutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                        host_port_range: "1000-1100".to_string(),
                    },
                    PutRequest::PortRange(Box::new(models::PortRange {
                        start: 200,
                        end: 300,
                    })),
                )
                .await,
                PutResponse::Status201_TheSpecifiedPortMappingWasCreated
            );
        }

        #[tokio::test]
        async fn put_200() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_put_instance_config_port_mapping()
                .withf(move |_, id, mapping, protocol| {
                    id.value == 6
                        && *protocol == TransportProtocol::Udp
                        && *mapping
                            == PortMapping::Range {
                                from: PortRange::new(50..=100),
                                to: PortRange::new(200..=250),
                            }
                })
                .once()
                .returning(|_, _, _, _| Ok(Some(true)));
            let vault = crate::vault::tests::create_empty_test_vault();
            assert_eq!(
                put(
                    vault,
                    Arc::new(instancius),
                    PutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "50-100".to_string(),
                    },
                    PutRequest::PortRange(Box::new(models::PortRange {
                        start: 200,
                        end: 250,
                    })),
                )
                .await,
                PutResponse::Status200_TheSpecifiedPortMappingWasSet
            );
        }
    }

    mod single {
        use super::*;
        use crate::jeweler::gem::instance::docker::config::TransportProtocol;
        use crate::sorcerer::instancius::MockInstancius;

        #[tokio::test]
        async fn delete_404_instance() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_delete_instance_config_port_mapping_range()
                .withf(move |_, id, port, protocol| {
                    id.value == 0xffffffff
                        && *protocol == TransportProtocol::Tcp
                        && *port == PortRange::new(80..=80)
                })
                .once()
                .returning(|_, _, _, _| {
                    Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                        0xffffffff,
                    )))
                });
            let vault = crate::vault::tests::create_empty_test_vault();
            assert!(matches!(
                delete(
                    vault,
                    Arc::new(instancius),
                    DeletePathParams {
                        instance_id: "ffffffff".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "80".to_string(),
                    },
                )
                .await,
                DeleteResponse::Status404_ResourceNotFound(_)
            ));
        }

        #[tokio::test]
        async fn delete_404_host() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_delete_instance_config_port_mapping_range()
                .withf(move |_, id, port, protocol| {
                    id.value == 6
                        && *protocol == TransportProtocol::Tcp
                        && *port == PortRange::new(90..=90)
                })
                .once()
                .returning(|_, _, _, _| Ok(false));
            let vault = crate::vault::tests::create_empty_test_vault();
            assert!(matches!(
                delete(
                    vault,
                    Arc::new(instancius),
                    DeletePathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "90".to_string(),
                    },
                )
                .await,
                DeleteResponse::Status404_ResourceNotFound(_)
            ));
        }

        #[tokio::test]
        async fn delete_200() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_delete_instance_config_port_mapping_range()
                .withf(move |_, id, port, protocol| {
                    id.value == 6
                        && *protocol == TransportProtocol::Tcp
                        && *port == PortRange::new(80..=80)
                })
                .once()
                .returning(|_, _, _, _| Ok(true));
            let vault = crate::vault::tests::create_empty_test_vault();
            assert_eq!(
                delete(
                    vault,
                    Arc::new(instancius),
                    DeletePathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "80".to_string(),
                    },
                )
                .await,
                DeleteResponse::Status200_Success
            );
        }

        #[tokio::test]
        async fn get_404_instance() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_get_instance_config_port_mapping_range()
                .withf(move |_, id, port, protocol| {
                    id.value == 0xffffffff
                        && *protocol == TransportProtocol::Tcp
                        && *port == PortRange::new(80..=80)
                })
                .once()
                .returning(|_, _, _, _| {
                    Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                        0xffffffff,
                    )))
                });
            let vault = crate::vault::tests::create_empty_test_vault();
            assert!(matches!(
                get(
                    vault,
                    Arc::new(instancius),
                    GetPathParams {
                        instance_id: "ffffffff".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "80".to_string(),
                    },
                )
                .await,
                GetResponse::Status404_ResourceNotFound(_)
            ));
        }

        #[tokio::test]
        async fn get_404_host() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_get_instance_config_port_mapping_range()
                .withf(move |_, id, port, protocol| {
                    id.value == 6
                        && *protocol == TransportProtocol::Tcp
                        && *port == PortRange::new(90..=90)
                })
                .once()
                .returning(|_, _, _, _| Ok(None));
            let vault = crate::vault::tests::create_empty_test_vault();
            assert!(matches!(
                get(
                    vault,
                    Arc::new(instancius),
                    GetPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "90".to_string(),
                    },
                )
                .await,
                GetResponse::Status404_ResourceNotFound(_)
            ));
        }

        #[tokio::test]
        async fn get_200_single() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_get_instance_config_port_mapping_range()
                .withf(move |_, id, port, protocol| {
                    id.value == 6
                        && *protocol == TransportProtocol::Tcp
                        && *port == PortRange::new(80..=80)
                })
                .once()
                .returning(|_, _, _, _| Ok(Some(PortMapping::Single(80, 8080))));
            let vault = crate::vault::tests::create_empty_test_vault();
            assert_eq!(
                get(
                    vault,
                    Arc::new(instancius),
                    GetPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "80".to_string(),
                    },
                )
                .await,
                GetResponse::Status200_Success(
                    models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                        models::InstancePortMappingSingle {
                            host_port: 80,
                            container_port: 8080,
                        }
                    ))
                )
            );
        }

        #[tokio::test]
        async fn put_400_overlap() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_put_instance_config_port_mapping()
                .withf(move |_, id, mapping, protocol| {
                    id.value == 6
                        && *protocol == TransportProtocol::Udp
                        && *mapping == PortMapping::Single(80, 20)
                })
                .once()
                .returning(|_, _, _, _| Err(anyhow::anyhow!("TestError")));
            let vault = crate::vault::tests::create_empty_test_vault();
            assert!(matches!(
                put(
                    vault,
                    Arc::new(instancius),
                    PutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "80".to_string(),
                    },
                    PutRequest::I32(Box::new(20)),
                )
                .await,
                PutResponse::Status400_MalformedRequest(_)
            ));
        }

        #[tokio::test]
        async fn put_404() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_put_instance_config_port_mapping()
                .withf(move |_, id, mapping, protocol| {
                    id.value == 0xffffffff
                        && *protocol == TransportProtocol::Udp
                        && *mapping == PortMapping::Single(80, 20)
                })
                .once()
                .returning(|_, _, _, _| Ok(None));
            let vault = crate::vault::tests::create_empty_test_vault();
            assert!(matches!(
                put(
                    vault,
                    Arc::new(instancius),
                    PutPathParams {
                        instance_id: "ffffffff".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "80".to_string(),
                    },
                    PutRequest::I32(Box::new(20)),
                )
                .await,
                PutResponse::Status404_ResourceNotFound(_)
            ));
        }

        #[tokio::test]
        async fn put_201() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_put_instance_config_port_mapping()
                .withf(move |_, id, mapping, protocol| {
                    id.value == 6
                        && *protocol == TransportProtocol::Tcp
                        && *mapping == PortMapping::Single(70, 20)
                })
                .once()
                .returning(|_, _, _, _| Ok(Some(false)));
            let vault = crate::vault::tests::create_empty_test_vault();
            assert!(matches!(
                put(
                    vault,
                    Arc::new(instancius),
                    PutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "70".to_string(),
                    },
                    PutRequest::I32(Box::new(20)),
                )
                .await,
                PutResponse::Status201_TheSpecifiedPortMappingWasCreated
            ));
        }

        #[tokio::test]
        async fn put_200() {
            let mut instancius = MockInstancius::new();
            instancius
                .expect_put_instance_config_port_mapping()
                .withf(move |_, id, mapping, protocol| {
                    id.value == 6
                        && *protocol == TransportProtocol::Tcp
                        && *mapping == PortMapping::Single(80, 20)
                })
                .once()
                .returning(|_, _, _, _| Ok(Some(true)));
            let vault = crate::vault::tests::create_empty_test_vault();
            assert!(matches!(
                put(
                    vault,
                    Arc::new(instancius),
                    PutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "80".to_string(),
                    },
                    PutRequest::I32(Box::new(20)),
                )
                .await,
                PutResponse::Status200_TheSpecifiedPortMappingWasSet
            ));
        }
    }

    #[test]
    fn try_from_port_range_ok() {
        assert_eq!(
            PortRange::try_from(models::PortRange { start: 10, end: 20 }).unwrap(),
            PortRange::new(10..=20)
        );
    }

    #[test]
    fn try_from_port_range_err() {
        assert!(PortRange::try_from(models::PortRange { start: 10, end: 6 }).is_err());
    }

    #[test]
    fn from_port_range_test() {
        assert_eq!(
            models::PortRange::from(PortRange::new(9..=20)),
            models::PortRange { start: 9, end: 20 }
        )
    }
}
