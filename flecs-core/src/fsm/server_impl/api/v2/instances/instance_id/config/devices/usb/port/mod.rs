use crate::fsm::server_impl::api::v2::instances::instance_id::config::devices::usb::instance_config_usb_device_from;
use crate::jeweler::gem::instance::InstanceId;
use crate::relic::device::usb::UsbDeviceReader;
use crate::sorcerer::instancius::{
    GetInstanceUsbDeviceResult, Instancius, PutInstanceUsbDeviceResult, QueryInstanceConfigError,
};
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::{
    InstancesInstanceIdConfigDevicesUsbPortDeleteResponse as DeleteResponse,
    InstancesInstanceIdConfigDevicesUsbPortGetResponse as GetResponse,
    InstancesInstanceIdConfigDevicesUsbPortPutResponse as PutResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    InstancesInstanceIdConfigDevicesUsbPortDeletePathParams as DeletePathParams,
    InstancesInstanceIdConfigDevicesUsbPortGetPathParams as GetPathParams,
    InstancesInstanceIdConfigDevicesUsbPortPutPathParams as PutPathParams,
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
        .delete_instance_usb_device(vault, instance_id, path_params.port.clone())
        .await
    {
        Ok(Some(_)) => DeleteResponse::Status200_Success,
        Err(QueryInstanceConfigError::NotFound(_)) => {
            DeleteResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
                additional_info: Some(format!("No instance with id {instance_id}")),
            })
        }
        Err(e @ QueryInstanceConfigError::NotSupported(_)) => {
            DeleteResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(None) => DeleteResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
            additional_info: Some(format!(
                "Usb port '{}' not mapped to instance {instance_id}",
                path_params.port
            )),
        }),
    }
}

pub async fn get<I: Instancius, U: UsbDeviceReader + 'static>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    usb_device_reader: Arc<U>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius.get_instance_usb_device(
        vault,
        instance_id,
        path_params.port.clone(),
        usb_device_reader,
    )
        .await
    {
        Ok(GetInstanceUsbDeviceResult::DeviceActive(config, device)) =>
            GetResponse::Status200_Success(
                instance_config_usb_device_from((config, Some(device))),
            ),
        Ok(GetInstanceUsbDeviceResult::DeviceInactive(config)) =>
            GetResponse::Status200_Success(
                instance_config_usb_device_from((config, None)),
            ),
        Ok(GetInstanceUsbDeviceResult::NotSupported) =>
            GetResponse::Status400_MalformedRequest(models::AdditionalInfo {
                additional_info: format!("Instance {instance_id} does not support usb devices"),
            }),
        Ok(GetInstanceUsbDeviceResult::InstanceNotFound) =>
            GetResponse::Status404_ResourceNotFound(
                models::OptionalAdditionalInfo {
                    additional_info: Some(format!("No instance with id {instance_id}")),
                },
            ),
        Ok(GetInstanceUsbDeviceResult::DeviceNotMapped) =>
            GetResponse::Status404_ResourceNotFound(
                models::OptionalAdditionalInfo {
                    additional_info: Some(format!(
                        "Usb port '{}' not mapped to instance {instance_id}",
                        path_params.port
                    )),
                },
            ),
        Ok(GetInstanceUsbDeviceResult::UnknownDevice) =>
            GetResponse::Status404_ResourceNotFound(
                models::OptionalAdditionalInfo {
                    additional_info: Some(format!(
                        "Usb port '{}' not mapped to instance {instance_id} and not corresponding to any known device",
                        path_params.port
                    )),
                },
            ),
        Err(e) =>
            GetResponse::Status500_InternalServerError(
                models::AdditionalInfo::new(e.to_string()),
            ),
    }
}

pub async fn put<I: Instancius, U: UsbDeviceReader + 'static>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    usb_device_reader: Arc<U>,
    path_params: PutPathParams,
) -> PutResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .put_instance_usb_device(
            vault,
            instance_id,
            path_params.port.clone(),
            usb_device_reader,
        )
        .await
    {
        Ok(PutInstanceUsbDeviceResult::InstanceNotFound) => {
            PutResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
                additional_info: Some(format!("No instance with id {instance_id}")),
            })
        }
        Ok(PutInstanceUsbDeviceResult::NotSupported) => {
            PutResponse::Status400_MalformedRequest(models::AdditionalInfo {
                additional_info: format!("Instance {instance_id} does not support usb devices"),
            })
        }
        Ok(PutInstanceUsbDeviceResult::DeviceNotFound) => {
            PutResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
                additional_info: Some(format!("No usb device with port {}", path_params.port)),
            })
        }
        Ok(PutInstanceUsbDeviceResult::DeviceMappingCreated) => {
            PutResponse::Status201_UsbDeviceWasPassedThrough
        }
        Ok(PutInstanceUsbDeviceResult::DeviceMappingUpdated(_)) => {
            PutResponse::Status200_AlreadyPassedThrough
        }
        Err(e) => {
            PutResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::gem::instance::docker::config::UsbPathConfig;
    use crate::relic::device::usb::{MockUsbDeviceReader, UsbDevice};
    use crate::sorcerer::instancius::MockInstancius;
    use flecsd_axum_server::models;

    #[tokio::test]
    async fn delete_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_usb_device()
            .withf(move |_, id, port| id.value == 6 && port == "test_port")
            .once()
            .returning(|_, _, _| {
                Ok(Some(UsbPathConfig {
                    port: "test_port".to_string(),
                    bus_num: 100,
                    dev_num: 200,
                }))
            });
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: "00000006".to_string(),
                    port: "test_port".to_string(),
                }
            )
            .await,
            DeleteResponse::Status200_Success
        ))
    }

    #[tokio::test]
    async fn delete_404_instance() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_usb_device()
            .withf(move |_, id, port| id.value == 0xabcddcba && port == "test_port")
            .once()
            .returning(|_, _, _| {
                Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                    0xabcddcba,
                )))
            });
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: "abcddcba".to_string(),
                    port: "test_port".to_string(),
                }
            )
            .await,
            DeleteResponse::Status404_ResourceNotFound(_)
        ))
    }

    #[tokio::test]
    async fn delete_404_port() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_usb_device()
            .withf(move |_, id, port| id.value == 6 && port == "unknown port")
            .once()
            .returning(|_, _, _| Ok(None));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: "00000006".to_string(),
                    port: "unknown port".to_string(),
                }
            )
            .await,
            DeleteResponse::Status404_ResourceNotFound(_)
        ))
    }

    #[tokio::test]
    async fn get_200_inactive() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 6 && port == "test_port")
            .once()
            .returning(|_, _, _, _| {
                Ok(GetInstanceUsbDeviceResult::DeviceInactive(UsbPathConfig {
                    port: "test_port".to_string(),
                    bus_num: 10,
                    dev_num: 20,
                }))
            });
        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                Arc::new(MockUsbDeviceReader::new()),
                GetPathParams {
                    instance_id: "00000006".to_string(),
                    port: "test_port".to_string(),
                }
            )
            .await,
            GetResponse::Status200_Success(models::InstanceConfigUsbDevice {
                port: "test_port".to_string(),
                name: None,
                pid: None,
                vendor: None,
                vid: None,
                device_connected: false,
            })
        )
    }

    #[tokio::test]
    async fn get_200_active() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 6 && port == "test_port")
            .once()
            .returning(|_, _, _, _| {
                Ok(GetInstanceUsbDeviceResult::DeviceActive(
                    UsbPathConfig {
                        port: "test_port".to_string(),
                        bus_num: 10,
                        dev_num: 20,
                    },
                    UsbDevice {
                        vid: 10,
                        pid: 20,
                        device: "test-dev".to_string(),
                        port: "test_port".to_string(),
                        vendor: "test-vendor".to_string(),
                    },
                ))
            });
        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                Arc::new(MockUsbDeviceReader::new()),
                GetPathParams {
                    instance_id: "00000006".to_string(),
                    port: "test_port".to_string(),
                }
            )
            .await,
            GetResponse::Status200_Success(models::InstanceConfigUsbDevice {
                port: "test_port".to_string(),
                name: Some("test-dev".to_string()),
                pid: Some(20),
                vendor: Some("test-vendor".to_string()),
                vid: Some(10),
                device_connected: true,
            })
        )
    }

    #[tokio::test]
    async fn get_404_instance() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 0xaaabbbcc && port == "test_port")
            .once()
            .returning(|_, _, _, _| Ok(GetInstanceUsbDeviceResult::InstanceNotFound));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                Arc::new(MockUsbDeviceReader::new()),
                GetPathParams {
                    instance_id: "aaabbbcc".to_string(),
                    port: "test_port".to_string(),
                }
            )
            .await,
            GetResponse::Status404_ResourceNotFound(_)
        ))
    }

    #[tokio::test]
    async fn get_404_port() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 2 && port == "test_port")
            .once()
            .returning(|_, _, _, _| Ok(GetInstanceUsbDeviceResult::DeviceNotMapped));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                Arc::new(MockUsbDeviceReader::new()),
                GetPathParams {
                    instance_id: "00000002".to_string(),
                    port: "test_port".to_string(),
                }
            )
            .await,
            GetResponse::Status404_ResourceNotFound(_)
        ))
    }

    #[tokio::test]
    async fn get_404_unknown() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 2 && port == "test_port")
            .once()
            .returning(|_, _, _, _| Ok(GetInstanceUsbDeviceResult::UnknownDevice));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                Arc::new(MockUsbDeviceReader::new()),
                GetPathParams {
                    instance_id: "00000002".to_string(),
                    port: "test_port".to_string(),
                }
            )
            .await,
            GetResponse::Status404_ResourceNotFound(_)
        ))
    }

    #[tokio::test]
    async fn get_500() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 6 && port == "test_port")
            .once()
            .returning(|_, _, _, _| Err(anyhow::anyhow!("TestError")));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                Arc::new(MockUsbDeviceReader::new()),
                GetPathParams {
                    instance_id: "00000006".to_string(),
                    port: "test_port".to_string(),
                }
            )
            .await,
            GetResponse::Status500_InternalServerError(_)
        ))
    }

    #[tokio::test]
    async fn put_404_instance() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 0xaaabbbcc && port == "test_port")
            .once()
            .returning(|_, _, _, _| Ok(PutInstanceUsbDeviceResult::InstanceNotFound));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            put(
                vault,
                Arc::new(instancius),
                Arc::new(MockUsbDeviceReader::new()),
                PutPathParams {
                    instance_id: "aaabbbcc".to_string(),
                    port: "test_port".to_string(),
                }
            )
            .await,
            PutResponse::Status404_ResourceNotFound(_)
        ))
    }

    #[tokio::test]
    async fn put_404_device() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 3 && port == "test_port")
            .once()
            .returning(|_, _, _, _| Ok(PutInstanceUsbDeviceResult::DeviceNotFound));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            put(
                vault,
                Arc::new(instancius),
                Arc::new(MockUsbDeviceReader::new()),
                PutPathParams {
                    instance_id: "00000003".to_string(),
                    port: "test_port".to_string(),
                }
            )
            .await,
            PutResponse::Status404_ResourceNotFound(_)
        ))
    }

    #[tokio::test]
    async fn put_201() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 3 && port == "test_port")
            .once()
            .returning(|_, _, _, _| Ok(PutInstanceUsbDeviceResult::DeviceMappingCreated));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            put(
                vault,
                Arc::new(instancius),
                Arc::new(MockUsbDeviceReader::new()),
                PutPathParams {
                    instance_id: "00000003".to_string(),
                    port: "test_port".to_string(),
                }
            )
            .await,
            PutResponse::Status201_UsbDeviceWasPassedThrough
        )
    }

    #[tokio::test]
    async fn put_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 6 && port == "test_port")
            .once()
            .returning(|_, _, _, _| {
                Ok(PutInstanceUsbDeviceResult::DeviceMappingUpdated(
                    UsbPathConfig {
                        port: "test_port".to_string(),
                        bus_num: 121,
                        dev_num: 919,
                    },
                ))
            });
        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            put(
                vault,
                Arc::new(instancius),
                Arc::new(MockUsbDeviceReader::new()),
                PutPathParams {
                    instance_id: "00000006".to_string(),
                    port: "test_port".to_string(),
                }
            )
            .await,
            PutResponse::Status200_AlreadyPassedThrough
        )
    }

    #[tokio::test]
    async fn put_500() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 6 && port == "test_port")
            .once()
            .returning(|_, _, _, _| Err(anyhow::anyhow!("TestError")));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            put(
                vault,
                Arc::new(instancius),
                Arc::new(MockUsbDeviceReader::new()),
                PutPathParams {
                    instance_id: "00000006".to_string(),
                    port: "test_port".to_string(),
                }
            )
            .await,
            PutResponse::Status500_InternalServerError(_)
        ))
    }
}
