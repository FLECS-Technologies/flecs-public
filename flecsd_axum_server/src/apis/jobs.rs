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
pub enum JobsGetResponse {
    /// Success
    Status200_Success(Vec<models::Job>),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum JobsJobIdDeleteResponse {
    /// Success
    Status200_Success,
    /// Not found
    Status404_NotFound,
    /// Job not finished
    Status400_JobNotFinished(String),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum JobsJobIdGetResponse {
    /// Success
    Status200_Success(models::Job),
    /// Not found
    Status404_NotFound,
}

/// Jobs
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Jobs {
    /// Retrieve a list of all pending/queued/running/failed/cancelled jobs.
    ///
    /// JobsGet - GET /v2/jobs
    async fn jobs_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<JobsGetResponse, ()>;

    /// Cancel job or remove failed/successful/cancelled job from journal.
    ///
    /// JobsJobIdDelete - DELETE /v2/jobs/{job_id}
    async fn jobs_job_id_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::JobsJobIdDeletePathParams,
    ) -> Result<JobsJobIdDeleteResponse, ()>;

    /// Retrieve information for specific job_id.
    ///
    /// JobsJobIdGet - GET /v2/jobs/{job_id}
    async fn jobs_job_id_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::JobsJobIdGetPathParams,
    ) -> Result<JobsJobIdGetResponse, ()>;
}
