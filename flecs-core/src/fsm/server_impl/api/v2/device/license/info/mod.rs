use crate::fsm::server_impl::console_session_id_to_core_session_id;
use crate::vault::Vault;
use flecsd_axum_server::apis::device::DeviceLicenseInfoGetResponse as GetResponse;
use flecsd_axum_server::models::DeviceLicenseInfoGet200Response as GetResponse200;
use std::sync::Arc;

pub async fn get(vault: Arc<Vault>) -> GetResponse {
    let secrets = vault.get_secrets().await;
    GetResponse::Status200_Success(GetResponse200 {
        // TODO: Use correct type, as soon as serial numbers are implemented
        r#type: "Via user license".to_string(),
        session_id: Some(console_session_id_to_core_session_id(
            secrets.get_session_id(),
        )),
        license: secrets.license_key,
    })
}
