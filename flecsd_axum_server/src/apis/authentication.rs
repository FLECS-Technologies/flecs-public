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
pub enum AuthProvidersDefaultLocationGetResponse {
    /// Success
    Status200_Success(String),
    /// No default auth provider configured
    Status404_NoDefaultAuthProviderConfigured,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum AuthProvidersDefaultProtocolGetResponse {
    /// Success
    Status200_Success(models::AuthProtocol),
    /// No default auth provider configured
    Status404_NoDefaultAuthProviderConfigured,
}

/// Authentication
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Authentication {
    /// Get the location of the default authentication provider.
    ///
    /// AuthProvidersDefaultLocationGet - GET /v2/auth/providers/default/location
    async fn auth_providers_default_location_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<AuthProvidersDefaultLocationGetResponse, ()>;

    /// Get the protocol of the default authentication provider.
    ///
    /// AuthProvidersDefaultProtocolGet - GET /v2/auth/providers/default/protocol
    async fn auth_providers_default_protocol_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<AuthProvidersDefaultProtocolGetResponse, ()>;
}
