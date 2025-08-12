use flecsd_axum_server::apis::authentication::AuthProvidersDefaultProtocolGetResponse as GetResponse;
use flecsd_axum_server::models;

pub fn get() -> GetResponse {
    GetResponse::Status200_Success(models::AuthProtocol::Oidc)
}
