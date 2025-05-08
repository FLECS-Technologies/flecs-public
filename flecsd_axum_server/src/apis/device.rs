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
pub enum DeviceLicenseActivationPostResponse {
    /// Success
    Status200_Success(models::AdditionalInfo),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum DeviceLicenseActivationStatusGetResponse {
    /// Success
    Status200_Success(models::DeviceLicenseActivationStatusGet200Response),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum DeviceLicenseInfoGetResponse {
    /// Success
    Status200_Success(models::DeviceLicenseInfoGet200Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum DeviceOnboardingPostResponse {
    /// Accepted
    Status202_Accepted(models::JobMeta),
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

/// Device
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Device {
    /// Execute device activation.
    ///
    /// DeviceLicenseActivationPost - POST /v2/device/license/activation
    async fn device_license_activation_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<DeviceLicenseActivationPostResponse, ()>;

    /// Check if device is activated.
    ///
    /// DeviceLicenseActivationStatusGet - GET /v2/device/license/activation/status
    async fn device_license_activation_status_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<DeviceLicenseActivationStatusGetResponse, ()>;

    /// Get information about license.
    ///
    /// DeviceLicenseInfoGet - GET /v2/device/license/info
    async fn device_license_info_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<DeviceLicenseInfoGetResponse, ()>;

    /// DeviceOnboardingPost - POST /v2/device/onboarding
    async fn device_onboarding_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        body: models::DosManifest,
    ) -> Result<DeviceOnboardingPostResponse, ()>;
}
