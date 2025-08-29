pub mod first_time_setup;

use flecsd_axum_server::apis::experimental::{
    ProvidersAuthCoreGetResponse as GetResponse, ProvidersAuthCorePutResponse as PutResponse,
};
use flecsd_axum_server::models::ProviderReference;

pub async fn get() -> GetResponse {
    todo!()
}

pub async fn put(_request: ProviderReference) -> PutResponse {
    todo!()
}
