use crate::lore::Lore;
use flecsd_axum_server::apis::authentication::AuthProvidersDefaultLocationGetResponse as GetResponse;
use std::sync::Arc;

pub fn get(lore: Arc<Lore>) -> GetResponse {
    match &lore.auth.issuer_url {
        Some(issuer_url) => GetResponse::Status200_Success(issuer_url.to_string()),
        None => GetResponse::Status404_NoDefaultAuthProviderConfigured,
    }
}
