use flecsd_axum_server::apis::experimental::ProvidersAuthIdGetResponse as GetResponse;
use flecsd_axum_server::models::ProvidersAuthIdGetPathParams as GetPathParams;
pub mod first_time_setup;

pub async fn get(_path_params: GetPathParams) -> GetResponse {
    todo!()
}
