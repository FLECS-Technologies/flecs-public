use crate::fsm::server_impl::api::v2::instances::instance_id::depends::dependency_key::InstanceNotFoundOrNotDependent;
use crate::fsm::server_impl::api::v2::models::AdditionalInfo;
use crate::fsm::server_impl::state::{ProvidiusState, VaultState};
use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::instance::ProviderReference as PutRequest;
use crate::jeweler::gem::manifest::{DependencyKey, FeatureKey};
use crate::sorcerer::providius::SetDependencyError;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use utoipa::IntoParams;

#[serde_as]
#[derive(Debug, Deserialize, IntoParams)]
pub struct PutPathParams {
    #[serde_as(as = "DisplayFromStr")]
    pub instance_id: InstanceId,
    pub dependency_key: DependencyKey,
    pub feature: FeatureKey,
}

#[utoipa::path(
    put,
    path = "/instances/{instance_id}/depends/{dependency_key}/{feature}",
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
        feature,
    }): Path<PutPathParams>,
    Json(request): Json<PutRequest>,
) -> Result<Response, SetDependencyError> {
    match providius
        .set_dependency(vault, dependency_key, feature, instance_id, request)
        .await?
    {
        Some(_) => Ok(StatusCode::OK.into_response()),
        None => Ok(StatusCode::CREATED.into_response()),
    }
}
