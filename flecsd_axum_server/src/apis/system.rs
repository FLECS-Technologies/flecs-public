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
