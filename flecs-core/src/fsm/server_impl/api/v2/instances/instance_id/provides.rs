pub mod feature;

use crate::fsm::server_impl::api::v2::models::{AdditionalInfo, FeatureInfo};
use crate::fsm::server_impl::state::{ProvidiusState, VaultState};
use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::manifest::FeatureKey;
use crate::sorcerer::providius::GetProvidesError;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use std::collections::HashMap;
use utoipa::IntoParams;

#[serde_as]
#[derive(Debug, Deserialize, IntoParams)]
pub struct GetPathParams {
    #[serde_as(as = "DisplayFromStr")]
    pub instance_id: InstanceId,
}

#[utoipa::path(
    get,
    path = "/instances/{instance_id}/provides",
    tag = "Experimental",
    description = "Get all provided features of the specified instance",
    responses(
        (status = OK, description = "Information for all features and their config provided by this instance", body = HashMap<FeatureKey, FeatureInfo>),
        (status = NOT_FOUND, description = "Instance id not found"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
    params(GetPathParams)
)]
pub async fn get(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    Path(GetPathParams { instance_id }): Path<GetPathParams>,
) -> Result<Response, GetProvidesError> {
    let providers: HashMap<FeatureKey, FeatureInfo> = providius
        .get_provides(vault, instance_id)
        .await?
        .into_iter()
        .map(|(feature, provider)| {
            (
                feature,
                FeatureInfo {
                    config: provider.config,
                },
            )
        })
        .collect();
    Ok((StatusCode::OK, Json(providers)).into_response())
}
