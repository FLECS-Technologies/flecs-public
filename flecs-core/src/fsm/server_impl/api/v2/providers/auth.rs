pub mod core;
pub mod default;
pub mod first_time_setup;
pub mod id;

use flecsd_axum_server::apis::experimental::ProvidersAuthGetResponse as GetResponse;

pub async fn get() -> GetResponse {
    todo!()
}
