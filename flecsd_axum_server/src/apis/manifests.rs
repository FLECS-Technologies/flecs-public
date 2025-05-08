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
pub enum ManifestsAppNameVersionGetResponse {
    /// Success
    Status200_Success(models::AppManifest),
    /// Manifest not found
    Status404_ManifestNotFound,
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ManifestsGetResponse {
    /// Success
    Status200_Success(Vec<models::AppManifest>),
}

/// Manifests
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Manifests {
    /// Get a specific manifest.
    ///
    /// ManifestsAppNameVersionGet - GET /v2/manifests/{app_name}/{version}
    async fn manifests_app_name_version_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::ManifestsAppNameVersionGetPathParams,
    ) -> Result<ManifestsAppNameVersionGetResponse, ()>;

    /// Get a list of all manifests.
    ///
    /// ManifestsGet - GET /v2/manifests
    async fn manifests_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<ManifestsGetResponse, ()>;
}
