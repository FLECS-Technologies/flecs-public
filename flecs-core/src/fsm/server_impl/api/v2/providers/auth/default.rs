use crate::sorcerer::providius::{DeleteDefaultProviderError, Providius};
use crate::vault::Vault;
use crate::vault::pouch::provider::ProviderId;
use flecsd_axum_server::apis::experimental::{
    ProvidersAuthDefaultDeleteResponse as DeleteResponse,
    ProvidersAuthDefaultGetResponse as GetResponse, ProvidersAuthDefaultPutResponse as PutResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::PutDefaultProviderRequest as PutRequest;
use std::str::FromStr;
use std::sync::Arc;

#[cfg(feature = "auth")]
pub mod first_time_setup;

pub async fn delete(vault: Arc<Vault>, providius: Arc<dyn Providius>) -> DeleteResponse {
    match providius.delete_default_provider(vault, "auth").await {
        Ok(Some(_)) => DeleteResponse::Status200_RemoveTheDefaultAuthProvider,
        Ok(None) => DeleteResponse::Status404_NoDefaultAuthProviderWasFound,
        Err(e @ DeleteDefaultProviderError::ProviderInUse(_)) => {
            DeleteResponse::Status409_TheCurrentStateDoesNotAllowTheRemovalOfTheDefaultAuthProvider(
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

pub async fn get() -> GetResponse {
    todo!()
}

pub async fn put(
    vault: Arc<Vault>,
    providius: Arc<dyn Providius>,
    request: PutRequest,
) -> PutResponse {
    let id = ProviderId::from_str(&request.provider_id).unwrap();
    match providius
        .set_default_provider(vault, "auth".to_string(), id)
        .await
    {
        Ok(Some(_)) => PutResponse::Status200_DefaultAuthProviderWasReplaced,
        Ok(None) => PutResponse::Status201_DefaultAuthProviderWasSet,
        Err(e) => PutResponse::Status400_BadRequest(models::AdditionalInfo::new(e.to_string())),
    }
}
