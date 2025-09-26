use crate::fsm::server_impl::api::v2::models::{AdditionalInfo, PutDefaultProviderRequest};
use crate::fsm::server_impl::state::{ProvidiusState, VaultState};
use crate::jeweler::gem::manifest::FeatureKey;
use crate::sorcerer::providius::{
    DeleteDefaultProviderError, GetProviderError, Provider, SetDefaultProviderError,
};
use axum::Json;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct DeletePathParams {
    pub feature: FeatureKey,
}

pub type GetPathParams = DeletePathParams;
pub type PutPathParams = DeletePathParams;

#[utoipa::path(
    delete,
    path = "/providers/{feature}/default",
    tag = "Experimental",
    description = "Remove the default provider for the specified feature",
    params(DeletePathParams),
    responses(
        (status = OK, description = "Default provider was removed"),
        (status = NOT_FOUND, description = "No default provider for the specified feature was found"),
        (status = CONFLICT, description = "The current state does not allow the removal of the default provider, e.g. a running instance is using it", body = AdditionalInfo),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn delete(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    Path(DeletePathParams { feature }): Path<DeletePathParams>,
) -> Result<Response, DeleteDefaultProviderError> {
    match providius.delete_default_provider(vault, &feature).await? {
        Some(_) => Ok(StatusCode::OK.into_response()),
        None => Ok(StatusCode::CREATED.into_response()),
    }
}

#[utoipa::path(
    get,
    path = "/providers/{feature}/default",
    tag = "Experimental",
    description = "Get the default provider for the specified feature",
    params(GetPathParams),
    responses(
        (status = OK, description = "Default provider was found", body = Provider),
        (status = NOT_FOUND, description = "No default provider for the specified feature was found"),
        (status = BAD_REQUEST, description = "Bad request", body = AdditionalInfo),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn get(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    Path(GetPathParams { feature }): Path<GetPathParams>,
) -> Result<Response, GetProviderError> {
    match providius.get_default_provider(vault, &feature).await? {
        Some(provider) => Ok((StatusCode::OK, Json(provider)).into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

#[utoipa::path(
    put,
    path = "/providers/{feature}/default",
    tag = "Experimental",
    description = "Set the default provider for the specified feature",
    request_body(
        content = PutDefaultProviderRequest,
        description = "Id of the provider that should be used as the default provider",
    ),
    params(PutPathParams),
    responses(
        (status = OK, description = "Default provider was replaced"),
        (status = CREATED, description = "Default provider was set"),
        (status = BAD_REQUEST, description = "Bad request", body = AdditionalInfo),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn put(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    Path(PutPathParams { feature }): Path<PutPathParams>,
    Json(PutDefaultProviderRequest { provider_id }): Json<PutDefaultProviderRequest>,
) -> Result<Response, SetDefaultProviderError> {
    match providius
        .set_default_provider(vault, feature, provider_id)
        .await?
    {
        Some(_) => Ok(StatusCode::OK.into_response()),
        None => Ok(StatusCode::CREATED.into_response()),
    }
}
