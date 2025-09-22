use flecsd_axum_server::apis::experimental::ProvidersAuthIdGetResponse as GetResponse;
use flecsd_axum_server::models::ProvidersAuthIdGetPathParams as GetPathParams;

#[cfg(feature = "auth")]
pub mod first_time_setup;

pub async fn get(_path_params: GetPathParams) -> GetResponse {
    todo!()
}
