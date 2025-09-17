use crate::sorcerer::providius::{ProvidersAndDefaults, Providius};
use crate::vault::Vault;
use flecsd_axum_server::apis::experimental::ProvidersFeatureGetResponse as GetResponse;
use flecsd_axum_server::models::ProvidersFeatureGetPathParams as GetPathParams;
use flecsd_axum_server::{models, types};
use std::sync::Arc;

pub mod default;
pub mod id;

pub async fn get(
    vault: Arc<Vault>,
    providius: Arc<dyn Providius>,
    path_params: GetPathParams,
) -> GetResponse {
    let ProvidersAndDefaults {
        mut providers,
        defaults,
    } = providius.get_providers_and_defaults(vault).await;
    let default = defaults.get(&path_params.feature).map(|id| id.to_string());
    let providers = providers
        .remove(&path_params.feature)
        .unwrap_or_default()
        .into_iter()
        .map(|(id, val)| {
            (
                id.to_string(),
                models::FeatureProvider {
                    app_key: val.app_key.into(),
                    config: Some(types::Object(val.config)),
                },
            )
        })
        .collect();

    GetResponse::Status200_InformationForAllProvidersOfTheSpecifiedFeature(
        models::FeatureProviders { default, providers },
    )
}
