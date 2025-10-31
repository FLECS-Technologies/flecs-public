use crate::fsm::server_impl::api::v2::models::AdditionalInfo;
use crate::fsm::server_impl::api::v2::providers::auth::id::path::redirect_port_response;
use crate::fsm::server_impl::state::{ProvidiusState, VaultState};
use crate::jeweler::gem::instance::ProviderReference;
use axum::extract::{Host, OriginalUri, Path, State};
use axum::response::Response;

#[utoipa::path(
    head,
    path = "/providers/auth/default/{path}",
    tag = "Experimental",
    description = "Access the default auth provider, on success the response will redirect",
    params(
        ("path" = String, Path, description = "The path to forward to the default auth provider"),
    ),
    responses(
        (status = TEMPORARY_REDIRECT, description = "Redirect to the location of the default provider",
            headers(
                ("location" = String),
            ),
        ),
        (status = NOT_FOUND, description = "No default auth provider set"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn any(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    host: Host,
    Path(path): Path<String>,
    OriginalUri(orig): OriginalUri,
) -> Response {
    redirect_port_response(
        providius
            .get_auth_provider_port(vault, ProviderReference::Default)
            .await,
        host,
        path,
        orig.query(),
    )
}
