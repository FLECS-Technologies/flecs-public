pub mod feature;
use flecsd_axum_server::apis::experimental::InstancesInstanceIdDependsGetResponse as GetResponse;
use flecsd_axum_server::models::InstancesInstanceIdDependsGetPathParams as GetPathParams;
pub async fn get(_path_params: GetPathParams) -> GetResponse {
    todo!()
}
