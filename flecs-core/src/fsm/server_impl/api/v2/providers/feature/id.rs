use crate::sorcerer::providius::{GetProviderError, Providius};
use crate::vault::Vault;
use crate::vault::pouch::provider::ProviderId;
use flecsd_axum_server::apis::experimental::ProvidersFeatureIdGetResponse as GetResponse;
use flecsd_axum_server::models::ProvidersFeatureIdGetPathParams as GetPathParams;
use flecsd_axum_server::{models, types};
use std::str::FromStr;
use std::sync::Arc;

pub async fn get(
    vault: Arc<Vault>,
    providius: Arc<dyn Providius>,
    path_params: GetPathParams,
) -> GetResponse {
    let id = ProviderId::from_str(&path_params.id).unwrap();
    match providius
        .get_provider(vault, &path_params.feature, id)
        .await
    {
        Ok(provider) => GetResponse::Status200_ProviderWasFound(models::FeatureProvider {
            config: Some(types::Object(provider.config)),
            app_key: provider.app_key.into(),
        }),
        Err(GetProviderError::ProviderNotFound(_)) => GetResponse::Status404_ProviderWasNotFound,
        Err(e @ GetProviderError::ProviderDoesNotProvide { .. }) => {
            GetResponse::Status400_BadRequest(models::AdditionalInfo::new(e.to_string()))
        }
    }
}
