use crate::sorcerer::providius::{
    DeleteDefaultProviderError, GetProviderError, Providius, SetDefaultProviderError,
};
use crate::vault::Vault;
use crate::vault::pouch::provider::ProviderId;
use flecsd_axum_server::apis::experimental::{
    ProvidersFeatureDefaultDeleteResponse as DeleteResponse,
    ProvidersFeatureDefaultGetResponse as GetResponse,
    ProvidersFeatureDefaultPutResponse as PutResponse,
};
use flecsd_axum_server::models::{
    ProvidersFeatureDefaultDeletePathParams as DeletePathParams,
    ProvidersFeatureDefaultGetPathParams as GetPathParams,
    ProvidersFeatureDefaultPutPathParams as PutPathParams, PutDefaultProviderRequest as PutRequest,
};
use flecsd_axum_server::{models, types};
use std::str::FromStr;
use std::sync::Arc;

pub async fn delete(
    vault: Arc<Vault>,
    providius: Arc<dyn Providius>,
    path_params: DeletePathParams,
) -> DeleteResponse {
    match providius
        .delete_default_provider(vault, &path_params.feature)
        .await
    {
        Ok(Some(_)) => DeleteResponse::Status200_DefaultProviderForSpecifiedFeatureUnset,
        Ok(None) => DeleteResponse::Status404_DefaultProviderForSpecifiedFeatureWasNotFound,
        Err(e @ DeleteDefaultProviderError::ProviderInUse(_)) => {
            DeleteResponse::Status409_TheCurrentStateDoesNotAllowTheRemovalOfTheDefaultProvider(
                models::AdditionalInfo::new(e.to_string()),
            )
        }
        Err(e @ DeleteDefaultProviderError::FailedToCheckDependents(_)) => {
            DeleteResponse::Status500_InternalServerError(models::AdditionalInfo::new(
                e.to_string(),
            ))
        }
    }
}

pub async fn get(
    vault: Arc<Vault>,
    providius: Arc<dyn Providius>,
    path_params: GetPathParams,
) -> GetResponse {
    match providius
        .get_default_provider(vault.clone(), &path_params.feature)
        .await
    {
        Ok(Some(provider)) => GetResponse::Status200_DefaultProviderForSpecifiedFeatureWasFound(
            models::FeatureProvider {
                app_key: provider.app_key.into(),
                config: Some(types::Object(provider.config)),
            },
        ),
        Ok(None) | Err(GetProviderError::ProviderNotFound(_)) => {
            GetResponse::Status404_DefaultProviderForSpecifiedFeatureWasNotFound
        }
        Err(e) => {
            GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

pub async fn put(
    vault: Arc<Vault>,
    providius: Arc<dyn Providius>,
    request: PutRequest,
    path_params: PutPathParams,
) -> PutResponse {
    let id = ProviderId::from_str(&request.provider_id).unwrap();
    match providius
        .set_default_provider(vault, path_params.feature, id)
        .await
    {
        Ok(Some(_)) => PutResponse::Status200_DefaultProviderForSpecifiedFeatureWasReplaced,
        Ok(None) => PutResponse::Status201_DefaultProviderForSpecifiedFeatureWasSet,
        Err(e @ SetDefaultProviderError::ProviderDoesNotProvide { .. }) => {
            PutResponse::Status400_BadRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Err(SetDefaultProviderError::ProviderNotFound(_)) => {
            PutResponse::Status404_ProviderNotFound
        }
    }
}
