use crate::enchantment::floxy::Floxy;
use crate::fsm::server_impl::{ok, ServerImpl};
use crate::relic::device::usb::{UsbDevice, UsbDeviceReader};
use async_trait::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecsd_axum_server::apis::system::{
    System, SystemDevicesGetResponse, SystemDevicesUsbGetResponse, SystemDevicesUsbPortGetResponse,
    SystemInfoGetResponse, SystemPingGetResponse, SystemVersionGetResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{AdditionalInfo, SystemDevicesUsbPortGetPathParams};
use http::Method;
use tracing::error;

#[async_trait]
impl<F: Floxy, T: UsbDeviceReader + Sync> System for ServerImpl<F, T> {
    async fn system_devices_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemDevicesGetResponse, ()> {
        match self.get_usb_devices() {
            Ok(usb_devices) => Ok(SystemDevicesGetResponse::Status200_Success(
                models::Devices {
                    usb: Some(usb_devices),
                },
            )),
            Err(e) => Ok(SystemDevicesGetResponse::Status500_InternalServerError(
                AdditionalInfo {
                    additional_info: e.to_string(),
                },
            )),
        }
    }

    async fn system_devices_usb_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemDevicesUsbGetResponse, ()> {
        match self.get_usb_devices() {
            Ok(usb_devices) => Ok(SystemDevicesUsbGetResponse::Status200_Success(usb_devices)),
            Err(e) => Ok(SystemDevicesUsbGetResponse::Status500_InternalServerError(
                AdditionalInfo {
                    additional_info: e.to_string(),
                },
            )),
        }
    }

    async fn system_devices_usb_port_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: SystemDevicesUsbPortGetPathParams,
    ) -> Result<SystemDevicesUsbPortGetResponse, ()> {
        let mut usb_devices = match self.usb_reader.read_usb_devices() {
            Ok(usb_devices) => usb_devices,
            Err(e) => {
                return Ok(
                    SystemDevicesUsbPortGetResponse::Status500_InternalServerError(
                        AdditionalInfo::new(e.to_string()),
                    ),
                )
            }
        };
        match usb_devices.remove(&path_params.port) {
            None => Ok(SystemDevicesUsbPortGetResponse::Status404_DeviceNotFound),
            Some(device) => Ok(SystemDevicesUsbPortGetResponse::Status200_Success(
                models::UsbDevice::from(device),
            )),
        }
    }

    async fn system_info_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemInfoGetResponse, ()> {
        Ok(SystemInfoGetResponse::Status200_Sucess(
            crate::relic::system::info::try_create_system_info().map_err(|e| {
                error!("Could not create SystemInfo: {e}");
            })?,
        ))
    }

    async fn system_ping_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemPingGetResponse, ()> {
        Ok(SystemPingGetResponse::Status200_Success(ok()))
    }

    async fn system_version_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemVersionGetResponse, ()> {
        Ok(SystemVersionGetResponse::Status200_Success(
            models::SystemVersionGet200Response {
                api: crate::lore::API_VERSION.to_string(),
                core: crate::lore::CORE_VERSION.to_string(),
            },
        ))
    }
}

impl<F: Floxy, T: UsbDeviceReader> ServerImpl<F, T> {
    fn get_usb_devices(&self) -> Result<Vec<models::UsbDevice>, crate::Error> {
        Ok(self
            .usb_reader
            .read_usb_devices()?
            .into_values()
            .map(models::UsbDevice::from)
            .collect())
    }
}

impl From<UsbDevice> for models::UsbDevice {
    fn from(value: UsbDevice) -> Self {
        Self {
            name: value.device,
            pid: value.pid as i32,
            port: value.port,
            vendor: value.vendor,
            vid: value.vid as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relic::device::usb::{Error, MockUsbDeviceReader};
    use crate::vault::{Vault, VaultConfig};
    use std::collections::HashMap;
    use std::io::ErrorKind;
    use std::sync::Arc;

    fn create_mock_usb_reader_error() -> MockUsbDeviceReader {
        let mut usb_reader = MockUsbDeviceReader::new();
        usb_reader.expect_read_usb_devices().times(1).returning(|| {
            Err(Error::Io(std::io::Error::new(
                ErrorKind::Other,
                "test error",
            )))
        });
        usb_reader
    }

    fn create_mock_usb_reader_values() -> MockUsbDeviceReader {
        let mut usb_reader = MockUsbDeviceReader::new();
        let device_1 = UsbDevice {
            vid: 123,
            pid: 456,
            port: "test_port_1".to_string(),
            device: "test-dev-1".to_string(),
            vendor: "test-vendor-1".to_string(),
        };
        let device_2 = UsbDevice {
            vid: 10,
            pid: 100,
            port: "test_port_2".to_string(),
            device: "test-dev-2".to_string(),
            vendor: "test-vendor-2".to_string(),
        };
        let device_3 = UsbDevice {
            vid: 6,
            pid: 9,
            port: "test_port_3".to_string(),
            device: "test-dev-3".to_string(),
            vendor: "test-vendor-3".to_string(),
        };
        usb_reader
            .expect_read_usb_devices()
            .times(1)
            .return_once(|| {
                Ok(HashMap::from([
                    (device_1.port.clone(), device_1),
                    (device_2.port.clone(), device_2),
                    (device_3.port.clone(), device_3),
                ]))
            });
        usb_reader
    }

    fn create_expected_usb_devices() -> Vec<models::UsbDevice> {
        vec![
            models::UsbDevice {
                name: "test-dev-1".to_string(),
                pid: 456,
                port: "test_port_1".to_string(),
                vendor: "test-vendor-1".to_string(),
                vid: 123,
            },
            models::UsbDevice {
                name: "test-dev-2".to_string(),
                pid: 100,
                port: "test_port_2".to_string(),
                vendor: "test-vendor-2".to_string(),
                vid: 10,
            },
            models::UsbDevice {
                name: "test-dev-3".to_string(),
                pid: 9,
                port: "test_port_3".to_string(),
                vendor: "test-vendor-3".to_string(),
                vid: 6,
            },
        ]
    }

    #[test]
    fn from_usb_device() {
        let usb_device = UsbDevice {
            device: "device".to_string(),
            vendor: "vendor".to_string(),
            port: "from_usb_device".to_string(),
            pid: 2,
            vid: 5,
        };
        assert_eq!(
            models::UsbDevice::from(usb_device),
            models::UsbDevice {
                name: "device".to_string(),
                vid: 5,
                pid: 2,
                vendor: "vendor".to_string(),
                port: "from_usb_device".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn get_usb_devices_err() {
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let usb_reader = create_mock_usb_reader_error();
        let server = ServerImpl::test_instance(vault, usb_reader);
        assert!(server.get_usb_devices().is_err());
    }

    #[tokio::test]
    async fn get_usb_devices_ok() {
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let usb_reader = create_mock_usb_reader_values();
        let server = ServerImpl::test_instance(vault, usb_reader);
        let expected_devices = create_expected_usb_devices();
        let result_devices = server.get_usb_devices().unwrap();
        assert_eq!(expected_devices.len(), result_devices.len());
        for expected_device in expected_devices.iter() {
            assert!(result_devices.contains(expected_device));
        }
    }

    #[tokio::test]
    async fn system_devices_get_200() {
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let usb_reader = create_mock_usb_reader_values();
        let server = ServerImpl::test_instance(vault, usb_reader);
        let expected_devices = create_expected_usb_devices();
        let Ok(SystemDevicesGetResponse::Status200_Success(models::Devices {
            usb: Some(result_devices),
        })) = server
            .system_devices_get(
                Method::default(),
                Host("host".to_string()),
                CookieJar::default(),
            )
            .await
        else {
            panic!()
        };
        assert_eq!(expected_devices.len(), result_devices.len());
        for expected_device in expected_devices.iter() {
            assert!(result_devices.contains(expected_device));
        }
    }

    #[tokio::test]
    async fn system_devices_get_500() {
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let usb_reader = create_mock_usb_reader_error();
        let server = ServerImpl::test_instance(vault, usb_reader);
        assert!(matches!(
            server
                .system_devices_get(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default()
                )
                .await,
            Ok(SystemDevicesGetResponse::Status500_InternalServerError(_))
        ))
    }

    #[tokio::test]
    async fn system_devices_usb_get_200() {
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let usb_reader = create_mock_usb_reader_values();
        let server = ServerImpl::test_instance(vault, usb_reader);
        let expected_devices = create_expected_usb_devices();
        let Ok(SystemDevicesUsbGetResponse::Status200_Success(result_devices)) = server
            .system_devices_usb_get(
                Method::default(),
                Host("host".to_string()),
                CookieJar::default(),
            )
            .await
        else {
            panic!()
        };
        assert_eq!(expected_devices.len(), result_devices.len());
        for expected_device in expected_devices.iter() {
            assert!(result_devices.contains(expected_device));
        }
    }

    #[tokio::test]
    async fn system_devices_usb_get_500() {
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let usb_reader = create_mock_usb_reader_error();
        let server = ServerImpl::test_instance(vault, usb_reader);
        assert!(matches!(
            server
                .system_devices_usb_get(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default()
                )
                .await,
            Ok(SystemDevicesUsbGetResponse::Status500_InternalServerError(
                _
            ))
        ))
    }

    #[tokio::test]
    async fn system_devices_usb_port_get_200() {
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let usb_reader = create_mock_usb_reader_values();
        let server = ServerImpl::test_instance(vault, usb_reader);
        assert_eq!(
            server
                .system_devices_usb_port_get(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    SystemDevicesUsbPortGetPathParams {
                        port: "test_port_2".to_string()
                    }
                )
                .await,
            Ok(SystemDevicesUsbPortGetResponse::Status200_Success(
                create_expected_usb_devices()[1].clone()
            ))
        )
    }

    #[tokio::test]
    async fn system_devices_usb_port_get_404() {
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let usb_reader = create_mock_usb_reader_values();
        let server = ServerImpl::test_instance(vault, usb_reader);
        assert_eq!(
            server
                .system_devices_usb_port_get(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    SystemDevicesUsbPortGetPathParams {
                        port: "unknown-port".to_string()
                    }
                )
                .await,
            Ok(SystemDevicesUsbPortGetResponse::Status404_DeviceNotFound)
        )
    }

    #[tokio::test]
    async fn system_devices_usb_port_get_500() {
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let usb_reader = create_mock_usb_reader_error();
        let server = ServerImpl::test_instance(vault, usb_reader);
        assert!(matches!(
            server
                .system_devices_usb_port_get(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    SystemDevicesUsbPortGetPathParams {
                        port: "port".to_string()
                    }
                )
                .await,
            Ok(SystemDevicesUsbPortGetResponse::Status500_InternalServerError(_))
        ))
    }
}
