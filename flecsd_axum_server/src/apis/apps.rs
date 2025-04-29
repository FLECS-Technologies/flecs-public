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
pub enum AppsAppDeleteResponse {
    /// Accepted
    Status202_Accepted(models::JobMeta),
    /// No such app or app/version combination
    Status404_NoSuchAppOrApp,
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum AppsAppGetResponse {
    /// Success
    Status200_Success(Vec<models::InstalledApp>),
    /// No such app or app/version combination
    Status404_NoSuchAppOrApp,
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum AppsGetResponse {
    /// Success
    Status200_Success(Vec<models::InstalledApp>),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum AppsInstallPostResponse {
    /// Accepted
    Status202_Accepted(models::JobMeta),
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum AppsSideloadPostResponse {
    /// Accepted
    Status202_Accepted(models::JobMeta),
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
}

/// Apps
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Apps {
    /// Uninstall one or all versions an App.
    ///
    /// AppsAppDelete - DELETE /v2/apps/{app}
    async fn apps_app_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::AppsAppDeletePathParams,
        query_params: models::AppsAppDeleteQueryParams,
    ) -> Result<AppsAppDeleteResponse, ()>;

    /// Query all versions or specific versions of an App.
    ///
    /// AppsAppGet - GET /v2/apps/{app}
    async fn apps_app_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::AppsAppGetPathParams,
        query_params: models::AppsAppGetQueryParams,
    ) -> Result<AppsAppGetResponse, ()>;

    /// Query installed Apps.
    ///
    /// AppsGet - GET /v2/apps
    async fn apps_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<AppsGetResponse, ()>;

    /// Install an App from the FLECS marketplace.
    ///
    /// AppsInstallPost - POST /v2/apps/install
    async fn apps_install_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        body: models::AppsInstallPostRequest,
    ) -> Result<AppsInstallPostResponse, ()>;

    /// Sideload an App from its manifest.
    ///
    /// AppsSideloadPost - POST /v2/apps/sideload
    async fn apps_sideload_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        body: models::AppsSideloadPostRequest,
    ) -> Result<AppsSideloadPostResponse, ()>;
}
