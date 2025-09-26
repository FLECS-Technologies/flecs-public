#[cfg(feature = "auth")]
pub mod auth;
pub mod feature;

use crate::fsm::server_impl::api::v2::models::{AdditionalInfo, GenericProvider, GenericProviders};
use crate::fsm::server_impl::state::{ProvidiusState, VaultState};
use crate::jeweler::gem::manifest::FeatureKey;
use crate::sorcerer::providius::ProvidersAndDefaults;
use axum::Json;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use std::collections::HashMap;

#[utoipa::path(
    get,
    path = "/providers",
    tag = "Experimental",
    description = "Get information for all providers",
    responses(
        (status = OK, description = "Information for all providers", body = HashMap<FeatureKey, GenericProviders>),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn get(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
) -> Response {
    let ProvidersAndDefaults {
        providers,
        mut defaults,
    } = providius.get_providers_and_defaults(vault).await;
    let mut providers: HashMap<FeatureKey, GenericProviders> = providers
        .into_iter()
        .map(|(feature, providers)| {
            let provider = GenericProviders {
                default: defaults.remove(&feature),
                providers: providers
                    .into_iter()
                    .map(|(id, value)| {
                        (
                            id,
                            GenericProvider {
                                app_key: value.app_key,
                            },
                        )
                    })
                    .collect(),
            };
            (feature, provider)
        })
        .collect();
    providers.extend(defaults.into_iter().map(|(feature, default)| {
        (
            feature,
            GenericProviders {
                default: Some(default),
                providers: HashMap::default(),
            },
        )
    }));
    (StatusCode::OK, Json(providers)).into_response()
}
