use flecsd_axum_server::apis::experimental::{
    ProvidersAuthDefaultDeleteResponse as DeleteResponse,
    ProvidersAuthDefaultGetResponse as GetResponse, ProvidersAuthDefaultPutResponse as PutResponse,
};
use flecsd_axum_server::models::PutDefaultProviderRequest as PutRequest;

pub mod first_time_setup;

pub async fn delete() -> DeleteResponse {
    todo!()
}

pub async fn get() -> GetResponse {
    todo!()
}

pub async fn put(_request: PutRequest) -> PutResponse {
    todo!()
}
