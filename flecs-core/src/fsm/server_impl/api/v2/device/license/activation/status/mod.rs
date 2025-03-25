use crate::fsm::server_impl::additional_info_from_error;
use crate::sorcerer::licenso::Licenso;
use crate::vault::Vault;
use flecs_console_client::apis::configuration::Configuration;
use flecsd_axum_server::apis::device::DeviceLicenseActivationStatusGetResponse as GetResponse;
use flecsd_axum_server::models::DeviceLicenseActivationStatusGet200Response as GetResponse200;
use std::sync::Arc;

pub async fn get<L: Licenso>(
    vault: Arc<Vault>,
    licenso: Arc<L>,
    client_config: Arc<Configuration>,
) -> GetResponse {
    match licenso.validate_license(&vault, client_config).await {
        Ok(is_valid) => GetResponse::Status200_Success(GetResponse200 { is_valid }),
        Err(e) => GetResponse::Status500_InternalServerError(additional_info_from_error(e)),
    }
}
