use crate::fsm::server_impl::api::v2::models::{AdditionalInfo, FeatureProviders};
use crate::fsm::server_impl::state::{ProvidiusState, VaultState};
use crate::jeweler::gem::manifest::FeatureKey;
use crate::sorcerer::providius::{Provider, ProvidersAndDefaults};
use axum::Json;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::Deserialize;
use utoipa::IntoParams;

pub mod default;
pub mod id;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetPathParams {
    pub feature: FeatureKey,
}

#[utoipa::path(
    get,
    path = "/providers/{feature}",
    tag = "Experimental",
    description = "Get providers for the specified feature",
    params(GetPathParams),
    responses(
        (status = OK, description = "Default provider was found", body = FeatureProviders),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn get(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    Path(GetPathParams { feature }): Path<GetPathParams>,
) -> Response {
    let ProvidersAndDefaults {
        mut providers,
        mut defaults,
    } = providius.get_providers_and_defaults(vault).await;
    let provider = FeatureProviders {
        default: defaults.remove(&feature),
        providers: providers
            .remove(&feature)
            .unwrap_or_default()
            .into_iter()
            .map(|(id, value)| {
                (
                    id,
                    Provider {
                        app_key: value.app_key,
                        config: Default::default(),
                    },
                )
            })
            .collect(),
    };

    (StatusCode::OK, Json(provider)).into_response()
}
