use flecsd_axum_server::apis::experimental::ProvidersFeatureGetResponse as GetResponse;
use flecsd_axum_server::models::ProvidersFeatureGetPathParams as GetPathParams;
pub mod default;
pub mod id;

pub async fn get(_path_params: GetPathParams) -> GetResponse {
    todo!()
}
