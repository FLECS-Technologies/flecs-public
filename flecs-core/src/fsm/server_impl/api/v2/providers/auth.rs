pub mod core;
pub mod default;
#[cfg(feature = "auth")]
pub mod first_time_setup;
pub mod id;

use crate::jeweler::gem::instance::ProviderReference;
use crate::sorcerer::providius::{AuthProvidersAndDefaults, Providius};
use crate::vault::Vault;
use crate::vault::pouch::provider::ProviderId;
use flecsd_axum_server::apis::experimental::ProvidersAuthGetResponse as GetResponse;
use flecsd_axum_server::{models, types};
use std::collections::HashMap;
use std::sync::Arc;

impl From<AuthProvidersAndDefaults> for models::AuthProviders {
    fn from(value: AuthProvidersAndDefaults) -> Self {
        Self {
            core: value.core.as_ref().map(ProviderReference::to_string),
            default: value.default.as_ref().map(ProviderId::to_string),
            providers: HashMap::from_iter(value.providers.into_iter().map(|(id, provider)| {
                (
                    id.to_string(),
                    models::AuthProvider {
                        config: Some(types::Object(provider.config)),
                        id: id.to_string(),
                        issuer_url: provider.issuer_url.to_string(),
                        kind: provider.kind,
                        name: provider.name,
                    },
                )
            })),
        }
    }
}

pub async fn get(vault: Arc<Vault>, providius: Arc<dyn Providius>) -> GetResponse {
    GetResponse::Status200_InformationForAllAuthProviders(
        providius.get_auth_providers_and_default(vault).await.into(),
    )
}
