use crate::fsm::server_impl::api::ForwardedHeaders;
use crate::fsm::server_impl::state::{ProvidiusState, VaultState};
use crate::jeweler::gem::manifest::providers::auth::AuthProvider;
use crate::sorcerer::providius::ReplacementUrlParts;
use crate::vault::pouch::provider::ProviderId;
use axum::Json;
use axum::extract::{Host, Path, State};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use utoipa::IntoParams;

#[serde_as]
#[derive(Debug, Deserialize, IntoParams)]
pub struct GetPathParams {
    #[serde_as(as = "DisplayFromStr")]
    pub id: ProviderId,
}

#[utoipa::path(
    get,
    path = "/providers/auth/{id}",
    tag = "Experimental",
    description = "Get the auth provider with the specified id",
    params(GetPathParams),
    responses(
        (status = OK, description = "Auth provider was found", body = AuthProvider),
        (status = NOT_FOUND, description = "Auth provider was not found"),
    ),
)]
pub async fn get(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    Path(GetPathParams { id }): Path<GetPathParams>,
    host: Host,
    forwarded: ForwardedHeaders,
) -> Response {
    match providius
        .get_auth_providers_and_default(
            vault,
            &ReplacementUrlParts::from_forwarded_and_host(forwarded, host),
        )
        .await
        .providers
        .remove(&id)
    {
        Some(auth_provider) => (StatusCode::OK, Json(auth_provider)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
