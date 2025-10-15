pub mod path;

use crate::fsm::server_impl::api::v2::models::AdditionalInfo;
use crate::fsm::server_impl::state::{ProvidiusState, VaultState};
use crate::jeweler::gem::instance::ProviderReference;
use crate::sorcerer::providius::PutCoreAuthProviderError;
use axum::Json;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use http::StatusCode;

#[utoipa::path(
    get,
    path = "/providers/auth/core",
    tag = "Experimental",
    description = "Get information on the core auth provider",
    responses(
        (status = NO_CONTENT, description = "How the core auth provider is currently set", body = ProviderReference),
        (status = NOT_FOUND, description = "No core auth provider set"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn get(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
) -> Response {
    match providius.get_core_providers(vault).await.auth {
        Some(provider) => (StatusCode::OK, Json(provider)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
#[utoipa::path(
    put,
    path = "/providers/auth/core",
    tag = "Experimental",
    description = "Set a core auth provider",
    request_body(
        content = ProviderReference,
        description = "The provider that should be used",
    ),
    responses(
        (status = OK, description = "Provider was overwritten"),
        (status = CREATED, description = "Provider was set"),
        (status = BAD_REQUEST, description = "Bad request", body = AdditionalInfo),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn put(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    Json(provider): Json<ProviderReference>,
) -> Result<Response, PutCoreAuthProviderError> {
    match providius.put_core_auth_provider(vault, provider).await? {
        Some(_) => Ok(StatusCode::OK.into_response()),
        None => Ok(StatusCode::CREATED.into_response()),
    }
}
