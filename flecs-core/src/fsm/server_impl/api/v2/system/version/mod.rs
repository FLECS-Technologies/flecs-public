use flecsd_axum_server::apis::system::SystemVersionGetResponse as GetResponse;
use flecsd_axum_server::models;

pub fn get() -> GetResponse {
    GetResponse::Status200_Success(models::SystemVersionGet200Response {
        api: crate::lore::API_VERSION.to_string(),
        core: crate::lore::CORE_VERSION.to_string(),
    })
}
