use crate::fsm::console_client::ConsoleClient;
use crate::fsm::server_impl::{additional_info_from_error, ok};
use crate::sorcerer::licenso::Licenso;
use crate::vault::Vault;
use flecsd_axum_server::apis::device::DeviceLicenseActivationPostResponse as PostResponse;
use std::sync::Arc;

pub mod status;

pub async fn post<L: Licenso>(
    vault: Arc<Vault>,
    licenso: Arc<L>,
    client_config: ConsoleClient,
) -> PostResponse {
    match licenso.activate_license(&vault, client_config).await {
        Ok(()) => PostResponse::Status200_Success(ok()),
        Err(e) => PostResponse::Status500_InternalServerError(additional_info_from_error(e)),
    }
}
