pub mod core;
pub mod default;
pub mod first_time_setup;
pub mod id;

use crate::fsm::server_impl::api::v2::models;
use crate::fsm::server_impl::api::v2::models::AdditionalInfo;
use crate::fsm::server_impl::state::{ProvidiusState, VaultState};
use crate::sorcerer::providius::{ForwardedHeaders, ReplacementUrlParts};
use axum::Json;
use axum::extract::{Host, State};
use axum::response::{IntoResponse, Response};
use http::StatusCode;

#[utoipa::path(
    get,
    path = "/providers/auth",
    tag = "Experimental",
    description = "Get information for all auth providers",
    responses(
        (status = OK, description = "Information for all auth providers", body = models::AuthProvidersAndDefaults),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn get(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    host: Host,
    forwarded: ForwardedHeaders,
) -> Response {
    let providers: models::AuthProvidersAndDefaults = providius
        .get_auth_providers_and_default(
            vault,
            &ReplacementUrlParts::from_forwarded_and_host(forwarded, host),
        )
        .await
        .into();
    (StatusCode::OK, Json(providers)).into_response()
}
