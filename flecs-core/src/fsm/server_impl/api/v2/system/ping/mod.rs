use crate::fsm::server_impl::ok;
use flecsd_axum_server::apis::system::SystemPingGetResponse as GetResponse;

pub fn get() -> GetResponse {
    GetResponse::Status200_Success(ok())
}
