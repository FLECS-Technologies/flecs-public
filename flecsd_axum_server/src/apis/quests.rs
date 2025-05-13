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
pub enum QuestsGetResponse {
    /// Success
    Status200_Success(Vec<models::Quest>),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum QuestsIdDeleteResponse {
    /// Success
    Status200_Success,
    /// Unfinished quests can not be deleted
    Status400_UnfinishedQuestsCanNotBeDeleted,
    /// Quest not found
    Status404_QuestNotFound,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum QuestsIdGetResponse {
    /// Success
    Status200_Success(models::Quest),
    /// Quest not found
    Status404_QuestNotFound,
}

/// Quests
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Quests {
    /// Get a list of all quests.
    ///
    /// QuestsGet - GET /v2/quests
    async fn quests_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<QuestsGetResponse, ()>;

    /// Delete a specific quest by its id.
    ///
    /// QuestsIdDelete - DELETE /v2/quests/{id}
    async fn quests_id_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::QuestsIdDeletePathParams,
    ) -> Result<QuestsIdDeleteResponse, ()>;

    /// Get a specific quest by its id.
    ///
    /// QuestsIdGet - GET /v2/quests/{id}
    async fn quests_id_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::QuestsIdGetPathParams,
    ) -> Result<QuestsIdGetResponse, ()>;
}
