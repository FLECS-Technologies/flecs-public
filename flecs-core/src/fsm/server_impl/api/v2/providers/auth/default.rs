use crate::fsm::server_impl::api::v2::models::{AdditionalInfo, PutDefaultProviderRequest};
use crate::fsm::server_impl::state::{ProvidiusState, VaultState};
use crate::jeweler::gem::manifest::FeatureKey;
use crate::jeweler::gem::manifest::providers::auth::AuthProvider;
use crate::sorcerer::providius::{DeleteDefaultProviderError, SetDefaultProviderError};
use axum::Json;
use axum::extract::{Host, State};
use axum::response::{IntoResponse, Response};
use http::StatusCode;

#[utoipa::path(
    delete,
    path = "/providers/auth/default",
    tag = "Experimental",
    description = "Remove the default auth provider",
    responses(
        (status = OK, description = "Default auth provider was removed"),
        (status = NOT_FOUND, description = "No default auth provider was found"),
        (status = CONFLICT, description = "The current state does not allow the removal of the default auth provider, e.g. a running instance is using it", body = AdditionalInfo),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn delete(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
) -> Result<Response, DeleteDefaultProviderError> {
    match providius
        .delete_default_provider(vault, &FeatureKey::auth())
        .await?
    {
        Some(_) => Ok(StatusCode::OK.into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

#[utoipa::path(
    get,
    path = "/providers/auth/default",
    tag = "Experimental",
    description = "Get the default auth provider",
    responses(
        (status = OK, description = "Default auth provider was found", body = AuthProvider),
        (status = NOT_FOUND, description = "No default auth provider was found"),
    ),
)]
pub async fn get(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    host: Host,
) -> Response {
    let mut providers = providius.get_auth_providers_and_default(vault, &host).await;
    match providers.default {
        Some(provider_id) => match providers.providers.remove(&provider_id) {
            Some(provider) => (StatusCode::OK, Json(provider)).into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        },
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/providers/auth/default",
    tag = "Experimental",
    description = "Set the default auth provider",
    request_body(
        content = PutDefaultProviderRequest,
        description = "Id of the provider that should be used as the default auth provider",
    ),
    responses(
        (status = OK, description = "Default auth provider was replaced"),
        (status = CREATED, description = "Default auth provider was set"),
        (status = BAD_REQUEST, description = "Bad request", body = AdditionalInfo),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn put(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    Json(PutDefaultProviderRequest { provider_id }): Json<PutDefaultProviderRequest>,
) -> Result<Response, SetDefaultProviderError> {
    match providius
        .set_default_provider(vault, FeatureKey::auth(), provider_id)
        .await?
    {
        Some(_) => Ok(StatusCode::OK.into_response()),
        None => Ok(StatusCode::CREATED.into_response()),
    }
}
