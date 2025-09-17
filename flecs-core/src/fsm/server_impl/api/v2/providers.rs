pub mod auth;
pub mod feature;

use crate::sorcerer::providius::{ProvidersAndDefaults, Providius};
use crate::vault::Vault;
use flecsd_axum_server::apis::experimental::ProvidersGetResponse as GetResponse;
use flecsd_axum_server::models;
use std::collections::HashMap;
use std::sync::Arc;

pub async fn get(vault: Arc<Vault>, providius: Arc<dyn Providius>) -> GetResponse {
    let ProvidersAndDefaults {
        providers,
        mut defaults,
    } = providius.get_providers_and_defaults(vault).await;
    let mut providers: HashMap<String, models::GenericProviders> = providers
        .into_iter()
        .map(|(feature, providers)| {
            let provider = models::GenericProviders {
                default: defaults.remove(&feature).map(|id| id.to_string()),
                providers: providers
                    .into_iter()
                    .map(|(id, value)| {
                        (
                            id.to_string(),
                            models::GenericProvider {
                                app_key: value.app_key.into(),
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
            models::GenericProviders {
                default: Some(default.to_string()),
                providers: HashMap::default(),
            },
        )
    }));
    GetResponse::Status200_InformationForAllProviders(models::Providers::from(providers))
}
