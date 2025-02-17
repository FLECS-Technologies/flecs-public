use crate::fsm::server_impl::{ok, ServerImpl};
use crate::relic::device::usb::UsbDevice;
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
impl System for ServerImpl {
    async fn system_devices_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemDevicesGetResponse, ()> {
        match get_usb_devices() {
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
        match get_usb_devices() {
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
        let mut usb_devices = match crate::relic::device::usb::read_usb_devices() {
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

fn get_usb_devices() -> Result<Vec<models::UsbDevice>, crate::Error> {
    Ok(crate::relic::device::usb::read_usb_devices()?
        .into_values()
        .map(models::UsbDevice::from)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
