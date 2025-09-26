use crate::fsm::server_impl::api::v2::models::{
    AdditionalInfo, FeatureInfo, InstanceNotFoundOrFeatureNotProvided,
};
use crate::fsm::server_impl::state::{ProvidiusState, VaultState};
use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::manifest::FeatureKey;
use crate::sorcerer::providius::GetFeatureProvidesError;
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
    pub instance_id: InstanceId,
    pub feature: FeatureKey,
}

#[utoipa::path(
    get,
    path = "/instances/{instance_id}/provides/{feature}",
    tag = "Experimental",
    description = "Check if the specified instance provides the specified feature",
    responses(
        (status = OK, description = "Information of the specified feature provided by the specified instance", body = FeatureInfo),
        (status = NOT_FOUND, description = "Instance not found or feature not provided by instance", body = InstanceNotFoundOrFeatureNotProvided),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
    params(GetPathParams)
)]
pub async fn get(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    Path(GetPathParams {
        instance_id,
        feature,
    }): Path<GetPathParams>,
) -> Result<Response, GetFeatureProvidesError> {
    let config = providius
        .get_feature_provides(vault, &feature, instance_id)
        .await?
        .config;
    Ok((StatusCode::OK, Json(FeatureInfo { config })).into_response())
}
