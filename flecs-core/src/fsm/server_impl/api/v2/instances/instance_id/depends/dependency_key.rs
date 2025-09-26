use crate::fsm::server_impl::api::v2::models::AdditionalInfo;
use crate::fsm::server_impl::state::{ProvidiusState, VaultState};
use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::instance::ProviderReference as PutRequest;
use crate::jeweler::gem::manifest::DependencyKey;
use crate::sorcerer::providius::{
    ClearDependencyError, Dependency, GetDependencyError, SetDependencyError,
};
use axum::Json;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use utoipa::{IntoParams, ToSchema};
pub mod feature;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub enum InstanceNotFoundOrNotDependent {
    InstanceNotFound,
    NotDependent,
}

#[serde_as]
#[derive(Debug, Deserialize, IntoParams)]
pub struct DeletePathParams {
    #[serde_as(as = "DisplayFromStr")]
    pub instance_id: InstanceId,
    pub dependency_key: DependencyKey,
}

#[utoipa::path(
    delete,
    path = "/instances/{instance_id}/depends/{dependency_key}",
    tag = "Experimental",
    summary = "Remove the provider for the specified dependency of the specified instance",
    params(DeletePathParams),
    responses(
        (status = OK, description = "Provider removed"),
        (status = BAD_REQUEST, description = "Bad request", body = AdditionalInfo),
        (status = NOT_FOUND, description = "Instance not found or instance not dependent on specified dependency", body = InstanceNotFoundOrNotDependent),
        (status = CONFLICT, description = "State of the instance prevents removal of provider, e.g. instance is running", body = AdditionalInfo),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    )
)]
pub async fn delete(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    Path(DeletePathParams {
        instance_id,
        dependency_key,
    }): Path<DeletePathParams>,
) -> Result<Response, ClearDependencyError> {
    providius
        .clear_dependency(vault, &dependency_key, instance_id)
        .await?;
    Ok(StatusCode::OK.into_response())
}

pub type GetPathParams = DeletePathParams;

#[utoipa::path(
    get,
    path = "/instances/{instance_id}/depends/{dependency_key}",
    tag = "Experimental",
    summary = "Get information on the dependency for the specified dependency key of the specified instance",
    params(GetPathParams),
    responses(
        (status = OK, description = "Provider removed", body = Dependency),
        (status = BAD_REQUEST, description = "Bad request", body = AdditionalInfo),
        (status = NOT_FOUND, description = "Instance not found or instance not dependent on specified dependency", body = InstanceNotFoundOrNotDependent),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    )
)]
pub async fn get(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    Path(GetPathParams {
        instance_id,
        dependency_key,
    }): Path<GetPathParams>,
) -> Result<Response, GetDependencyError> {
    let dependency = providius
        .get_dependency(vault, &dependency_key, instance_id)
        .await?;
    Ok((StatusCode::OK, Json(dependency)).into_response())
}

pub type PutPathParams = DeletePathParams;

#[utoipa::path(
    put,
    path = "/instances/{instance_id}/depends/{dependency_key}",
    tag = "Experimental",
    summary = "Set a provider for the specified feature of the specified instance",
    request_body(
        content = PutRequest,
        description = "The provider that should be used",
    ),
    params(PutPathParams),
    responses(
        (status = OK, description = "Provider was overwritten"),
        (status = CREATED, description = "Provider was set"),
        (status = BAD_REQUEST, description = "Bad request", body = AdditionalInfo),
        (status = NOT_FOUND, description = "Instance not found or instance not dependent on specified dependency", body = InstanceNotFoundOrNotDependent),
        (status = CONFLICT, description = "Provider conflicts with requirements of dependency", body = AdditionalInfo),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    )
)]
pub async fn put(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    Path(PutPathParams {
        instance_id,
        dependency_key,
    }): Path<PutPathParams>,
    Json(provider_reference): Json<PutRequest>,
) -> Result<Response, SetDependencyError> {
    let features = dependency_key.features();
    if features.len() != 1 {
        return Ok(AdditionalInfo::new(
            "This route accepts only single feature depends".to_string(),
        )
        .into_bad_request());
    }
    match providius
        .set_dependency(
            vault,
            dependency_key,
            features[0].clone(),
            instance_id,
            provider_reference,
        )
        .await?
    {
        Some(_) => Ok(StatusCode::OK.into_response()),
        None => Ok(StatusCode::CREATED.into_response()),
    }
}
