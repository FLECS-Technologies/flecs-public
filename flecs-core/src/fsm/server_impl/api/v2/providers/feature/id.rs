use crate::fsm::server_impl::api::v2::models::AdditionalInfo;
use crate::fsm::server_impl::state::{ProvidiusState, VaultState};
use crate::jeweler::gem::manifest::FeatureKey;
use crate::sorcerer::providius::{GetProviderError, Provider};
use crate::vault::pouch::provider::ProviderId;
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
    pub feature: FeatureKey,
    #[serde_as(as = "DisplayFromStr")]
    pub id: ProviderId,
}

#[utoipa::path(
    get,
    path = "/providers/{feature}",
    tag = "Experimental",
    description = "Get providers for the specified feature",
    params(GetPathParams),
    responses(
        (status = OK, description = "Default provider was found", body = Provider),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn get(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    Path(GetPathParams { feature, id }): Path<GetPathParams>,
) -> Response {
    match providius.get_provider(vault, &feature, id).await {
        Ok(provider) => (StatusCode::OK, Json(provider)).into_response(),
        Err(GetProviderError::ProviderNotFound(_)) => StatusCode::NOT_FOUND.into_response(),
        Err(e @ GetProviderError::ProviderDoesNotProvide { .. }) => {
            AdditionalInfo::new(e.to_string()).into_bad_request()
        }
    }
}
