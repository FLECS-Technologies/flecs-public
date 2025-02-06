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
pub enum FlunderBrowseGetResponse {
    /// Success
    Status200_Success(models::FlunderBrowseGet200Response),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

/// Flunder
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Flunder {
    /// Retrieve stored flunder topics alongside their values.
    ///
    /// FlunderBrowseGet - GET /v2/flunder/browse
    async fn flunder_browse_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        query_params: models::FlunderBrowseGetQueryParams,
    ) -> Result<FlunderBrowseGetResponse, ()>;
}
