pub mod port;
use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::instance::docker::config::UsbPathConfig;
use crate::relic::device::usb::{UsbDevice, UsbDeviceReader};
use crate::sorcerer::instancius::{Instancius, QueryInstanceConfigError};
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::{
    InstancesInstanceIdConfigDevicesUsbDeleteResponse as DeleteResponse,
    InstancesInstanceIdConfigDevicesUsbGetResponse as GetResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    InstancesInstanceIdConfigDevicesUsbDeletePathParams as DeletePathParams,
    InstancesInstanceIdConfigDevicesUsbGetPathParams as GetPathParams,
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
        .delete_instance_usb_devices(vault, instance_id)
        .await
    {
        Ok(_) => DeleteResponse::Status200_Success,
        Err(QueryInstanceConfigError::NotFound(_)) => {
            DeleteResponse::Status404_NoInstanceWithThisInstance
        }
        Err(e @ QueryInstanceConfigError::NotSupported(_)) => {
            DeleteResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

pub async fn get<I: Instancius, U: UsbDeviceReader + 'static>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    usb_device_reader: Arc<U>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .get_instance_usb_devices(vault, instance_id, usb_device_reader)
        .await
    {
        Err(e) => {
            GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(None) => GetResponse::Status404_NoInstanceWithThisInstance,
        Ok(Some(usb_devices)) => GetResponse::Status200_Success(
            usb_devices
                .into_iter()
                .map(instance_config_usb_device_from)
                .collect(),
        ),
    }
}

fn instance_config_usb_device_from(
    (config, device): (UsbPathConfig, Option<UsbDevice>),
) -> models::InstanceConfigUsbDevice {
    match device {
        Some(device) => models::InstanceConfigUsbDevice {
            port: config.port,
            device_connected: true,
            pid: Some(device.pid as i32),
            name: Some(device.device),
            vendor: Some(device.vendor),
            vid: Some(device.vid as i32),
        },
        None => models::InstanceConfigUsbDevice {
            port: config.port,
            device_connected: false,
            name: None,
            vid: None,
            pid: None,
            vendor: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relic::device::usb::MockUsbDeviceReader;
    use crate::sorcerer::instancius::MockInstancius;
    use std::collections::HashMap;

    #[tokio::test]
    async fn delete_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_usb_devices()
            .withf(move |_, id| id.value == 2)
            .once()
            .returning(|_, _| Ok(HashMap::new()));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: "00000002".to_string(),
                }
            )
            .await,
            DeleteResponse::Status200_Success
        )
    }

    #[tokio::test]
    async fn delete_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_usb_devices()
            .withf(move |_, id| id.value == 0xaabbccdd)
            .once()
            .returning(|_, _| {
                Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                    0xaabbccdd,
                )))
            });
        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: "aabbccdd".to_string(),
                }
            )
            .await,
            DeleteResponse::Status404_NoInstanceWithThisInstance
        )
    }

    #[tokio::test]
    async fn get_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_devices::<MockUsbDeviceReader>()
            .withf(move |_, id, _| id.value == 6)
            .once()
            .returning(|_, _, _| Ok(Some(vec![])));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(get(
                    vault, Arc::new(instancius),
            Arc::new(MockUsbDeviceReader::new()),
                    GetPathParams {
                        instance_id: "00000006".to_string(),
                    }
                )
                .await,
            GetResponse::Status200_Success(vec) if vec.is_empty()
        ))
    }

    #[tokio::test]
    async fn get_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_devices::<MockUsbDeviceReader>()
            .withf(move |_, id, _| id.value == 0x1234abcd)
            .once()
            .returning(|_, _, _| Ok(None));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                Arc::new(MockUsbDeviceReader::new()),
                GetPathParams {
                    instance_id: "1234abcd".to_string(),
                }
            )
            .await,
            GetResponse::Status404_NoInstanceWithThisInstance
        )
    }

    #[tokio::test]
    async fn get_500() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_devices::<MockUsbDeviceReader>()
            .withf(move |_, id, _| id.value == 6)
            .once()
            .returning(|_, _, _| Err(anyhow::anyhow!("TestError")));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                Arc::new(MockUsbDeviceReader::new()),
                GetPathParams {
                    instance_id: "0000006".to_string(),
                }
            )
            .await,
            GetResponse::Status500_InternalServerError(_)
        ))
    }

    #[test]
    fn instance_config_usb_device_from_some() {
        let usb_path_config = UsbPathConfig {
            dev_num: 20,
            port: "usb12".to_string(),
            bus_num: 10,
        };
        let usb_device = UsbDevice {
            device: "test-dev".to_string(),
            vid: 12,
            pid: 24,
            port: "usb12".to_string(),
            vendor: "Vendor".to_string(),
        };
        assert_eq!(
            instance_config_usb_device_from((usb_path_config, Some(usb_device))),
            models::InstanceConfigUsbDevice {
                port: "usb12".to_string(),
                device_connected: true,
                pid: Some(24),
                vendor: Some("Vendor".to_string()),
                vid: Some(12),
                name: Some("test-dev".to_string()),
            }
        )
    }

    #[test]
    fn instance_config_usb_device_from_none() {
        let usb_path_config = UsbPathConfig {
            dev_num: 20,
            port: "usb12".to_string(),
            bus_num: 10,
        };
        assert_eq!(
            instance_config_usb_device_from((usb_path_config, None)),
            models::InstanceConfigUsbDevice {
                port: "usb12".to_string(),
                device_connected: false,
                pid: None,
                vendor: None,
                vid: None,
                name: None,
            }
        )
    }
}
