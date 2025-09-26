pub mod dependency_key;

use crate::fsm::server_impl::api::v2::models::AdditionalInfo;
use crate::fsm::server_impl::state::{ProvidiusState, VaultState};
use crate::jeweler::gem::manifest::DependencyKey;
use crate::sorcerer::providius::{Dependency, GetDependenciesError};
use axum::Json;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use utoipa::IntoParams;

#[serde_as]
#[derive(Debug, Deserialize, IntoParams)]
pub struct GetPathParams {
    #[serde_as(as = "DisplayFromStr")]
    pub instance_id: crate::jeweler::gem::instance::InstanceId,
}

#[utoipa::path(
    get,
    path = "/instances/{instance_id}/depends",
    tag = "Experimental",
    description = "Get information on all dependencies of the specified instance",
    responses(
        (status = OK, description = "All dependencies of the specified instance and how they are currently solved", body = HashMap<DependencyKey, Dependency>),
        (status = NOT_FOUND, description = "Instance not found"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
    params(GetPathParams)
)]
pub async fn get(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    Path(GetPathParams { instance_id }): Path<GetPathParams>,
) -> Result<Response, GetDependenciesError> {
    let dependencies = providius.get_dependencies(vault, instance_id).await?;
    Ok((StatusCode::OK, Json(dependencies)).into_response())
}
