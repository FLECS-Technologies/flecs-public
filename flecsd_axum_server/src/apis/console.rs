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
pub enum ConsoleAuthenticationDeleteResponse {
    /// No content
    Status204_NoContent,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ConsoleAuthenticationPutResponse {
    /// No content
    Status204_NoContent,
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
}

/// Console
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Console {
    /// Remove the authentication information.
    ///
    /// ConsoleAuthenticationDelete - DELETE /v2/console/authentication
    async fn console_authentication_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<ConsoleAuthenticationDeleteResponse, ()>;

    /// Set the authentication information.
    ///
    /// ConsoleAuthenticationPut - PUT /v2/console/authentication
    async fn console_authentication_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        body: models::AuthResponseData,
    ) -> Result<ConsoleAuthenticationPutResponse, ()>;
}
