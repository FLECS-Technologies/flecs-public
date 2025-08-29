pub mod auth;
pub mod feature;

use flecsd_axum_server::apis::experimental::ProvidersGetResponse as GetResponse;

pub async fn get() -> GetResponse {
    todo!()
}
