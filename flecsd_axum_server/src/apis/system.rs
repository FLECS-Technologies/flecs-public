use async_trait::async_trait;
use axum::extract::*;
use axum_extra::extract::{CookieJar, Multipart};
use bytes::Bytes;
use http::Method;
use serde::{Deserialize, Serialize};

use crate::{models, types::*};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum SystemDevicesGetResponse {
    /// Success
    Status200_Success(models::Devices),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum SystemDevicesUsbGetResponse {
    /// Success
    Status200_Success(Vec<models::UsbDevice>),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum SystemDevicesUsbPortGetResponse {
    /// Success
    Status200_Success(models::UsbDevice),
    /// Device not found
    Status404_DeviceNotFound,
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum SystemInfoGetResponse {
    /// Sucess
    Status200_Sucess(models::SystemInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum SystemPingGetResponse {
    /// Success
    Status200_Success(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum SystemVersionGetResponse {
    /// Success
    Status200_Success(models::SystemVersionGet200Response),
}

/// System
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait System {
    /// Get devices of system.
    ///
    /// SystemDevicesGet - GET /v2/system/devices
    async fn system_devices_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<SystemDevicesGetResponse, ()>;

    /// Get usb devices of system.
    ///
    /// SystemDevicesUsbGet - GET /v2/system/devices/usb
    async fn system_devices_usb_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<SystemDevicesUsbGetResponse, ()>;

    /// Get usb device of system.
    ///
    /// SystemDevicesUsbPortGet - GET /v2/system/devices/usb/{port}
    async fn system_devices_usb_port_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::SystemDevicesUsbPortGetPathParams,
    ) -> Result<SystemDevicesUsbPortGetResponse, ()>;

    /// Get architecture and operating system information.
    ///
    /// SystemInfoGet - GET /v2/system/info
    async fn system_info_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<SystemInfoGetResponse, ()>;

    /// Check daemon availability and connectivity.
    ///
    /// SystemPingGet - GET /v2/system/ping
    async fn system_ping_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<SystemPingGetResponse, ()>;

    /// Get FLECS core and API version.
    ///
    /// SystemVersionGet - GET /v2/system/version
    async fn system_version_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<SystemVersionGetResponse, ()>;
}
