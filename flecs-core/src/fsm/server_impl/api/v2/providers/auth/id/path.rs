use crate::fsm::server_impl::api::v2::models::AdditionalInfo;
use crate::fsm::server_impl::state::{ProvidiusState, VaultState};
use crate::jeweler::gem::instance::ProviderReference;
use crate::sorcerer::providius::GetAuthProviderPortError;
use crate::vault::pouch::provider::ProviderId;
use axum::extract::{Host, OriginalUri, Path, State};
use axum::response::{IntoResponse, Response};
use http::uri::Scheme;
use http::{HeaderValue, StatusCode};
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use utoipa::IntoParams;

#[serde_as]
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct PathParams {
    /// The id of the auth provider
    #[serde_as(as = "DisplayFromStr")]
    pub provider_id: ProviderId,
    /// The path to forward to the specified auth provider
    pub path: String,
}

pub fn redirect_port_response(
    port_result: Result<u16, GetAuthProviderPortError>,
    host: Host,
    path: String,
    query: Option<&str>,
    scheme: Option<&Scheme>,
) -> Response {
    match port_result {
        Ok(port) => {
            let mut response = StatusCode::TEMPORARY_REDIRECT.into_response();
            let mut location = String::new();
            if let Some(scheme) = scheme {
                location.push_str(scheme.as_str());
                location.push_str("://")
            } else {
                location.push_str("https://")
            }
            location.push_str(&host.0);
            location.push(':');
            location.push_str(&port.to_string());
            location.push('/');
            location.push_str(&path);
            if let Some(query) = query {
                location.push('?');
                location.push_str(query);
            };
            let Ok(location) = HeaderValue::from_str(&location) else {
                return AdditionalInfo::new(format!(
                    "Failed to construct location header from {location}"
                ))
                .into_internal_server_error();
            };
            response.headers_mut().insert("location", location);
            response
        }
        Err(GetAuthProviderPortError::CoreProviderNotSet) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => AdditionalInfo::new(e.to_string()).into_internal_server_error(),
    }
}

#[utoipa::path(
    head,
    path = "/providers/auth/{provider_id}/{path}",
    tag = "Experimental",
    description = "Access the auth provider with the specified id, on success the response will redirect",
    params(PathParams),
    responses(
        (status = TEMPORARY_REDIRECT, description = "Redirect to the location of the specified auth provider",
            headers(
                ("location" = String),
            ),
        ),
        (status = NOT_FOUND, description = "Auth provider not found"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn any(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    host: Host,
    Path(PathParams { provider_id, path }): Path<PathParams>,
    OriginalUri(orig): OriginalUri,
) -> Response {
    redirect_port_response(
        providius
            .get_auth_provider_port(vault, ProviderReference::Provider(provider_id))
            .await,
        host,
        path,
        orig.query(),
        orig.scheme(),
    )
}
