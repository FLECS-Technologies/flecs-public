#[cfg(feature = "auth")]
pub mod first_time_setup;

use crate::jeweler::gem::instance::ProviderReference;
use crate::sorcerer::providius::{Providius, PutCoreAuthProviderError};
use crate::vault::Vault;
use flecsd_axum_server::apis::experimental::{
    ProvidersAuthCoreGetResponse as GetResponse, ProvidersAuthCorePutResponse as PutResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::ProviderReference as PutRequest;
use std::sync::Arc;

pub async fn get(vault: Arc<Vault>, providius: Arc<dyn Providius>) -> GetResponse {
    match providius.get_core_providers(vault).await.auth {
        Some(provider) => GetResponse::Status200_HowTheCoreAuthProviderIsCurrentlySet(
            models::ProviderReference::ProviderReferenceOneOf(Box::new(
                models::ProviderReferenceOneOf {
                    provider: provider.to_string(),
                },
            )),
        ),
        None => GetResponse::Status404_NoCoreAuthProviderSet,
    }
}

pub async fn put(
    vault: Arc<Vault>,
    providius: Arc<dyn Providius>,
    request: PutRequest,
) -> PutResponse {
    let provider = ProviderReference::try_from(request).unwrap();
    match providius.put_core_auth_provider(vault, provider).await {
        Ok(Some(_)) => PutResponse::Status200_ProviderWasOverwritten,
        Ok(None) => PutResponse::Status201_ProviderWasSet,
        Err(e @ PutCoreAuthProviderError::InstanceNotFound(_))
        | Err(e @ PutCoreAuthProviderError::DefaultProviderNotSet)
        | Err(e @ PutCoreAuthProviderError::DoesNotProvide { .. }) => {
            PutResponse::Status400_BadRequest(models::AdditionalInfo::new(e.to_string()))
        }
    }
}
