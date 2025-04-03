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
pub enum ExportsExportIdDeleteResponse {
    /// Success
    Status200_Success,
    /// Export id invalid
    Status400_ExportIdInvalid,
    /// Export not found
    Status404_ExportNotFound,
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ExportsExportIdGetResponse {
    /// Success
    Status200_Success(ByteArray),
    /// Export id invalid
    Status400_ExportIdInvalid,
    /// Export not found
    Status404_ExportNotFound,
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ExportsGetResponse {
    /// Success
    Status200_Success(Vec<models::ExportId>),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ExportsPostResponse {
    /// Accepted
    Status202_Accepted(models::JobMeta),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ImportsPostResponse {
    /// Accepted
    Status202_Accepted(models::JobMeta),
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

/// Flecsport
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Flecsport {
    /// Delete specified export.
    ///
    /// ExportsExportIdDelete - DELETE /v2/exports/{export_id}
    async fn exports_export_id_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::ExportsExportIdDeletePathParams,
    ) -> Result<ExportsExportIdDeleteResponse, ()>;

    /// Download specified export.
    ///
    /// ExportsExportIdGet - GET /v2/exports/{export_id}
    async fn exports_export_id_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::ExportsExportIdGetPathParams,
    ) -> Result<ExportsExportIdGetResponse, ()>;

    /// Query all existing exports.
    ///
    /// ExportsGet - GET /v2/exports
    async fn exports_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<ExportsGetResponse, ()>;

    /// Create an export.
    ///
    /// ExportsPost - POST /v2/exports
    async fn exports_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        body: models::ExportRequest,
    ) -> Result<ExportsPostResponse, ()>;

    /// Upload and import an export file.
    ///
    /// ImportsPost - POST /v2/imports
    async fn imports_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        header_params: models::ImportsPostHeaderParams,
        body: Multipart,
    ) -> Result<ImportsPostResponse, ()>;
}
