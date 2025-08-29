use flecsd_axum_server::apis::experimental::{
    InstancesInstanceIdDependsFeatureDeleteResponse as DeleteResponse,
    InstancesInstanceIdDependsFeatureGetResponse as GetResponse,
    InstancesInstanceIdDependsFeaturePutResponse as PutResponse,
};
use flecsd_axum_server::models::{
    InstancesInstanceIdDependsFeatureDeletePathParams as DeletePathParams,
    InstancesInstanceIdDependsFeatureGetPathParams as GetPathParams,
    InstancesInstanceIdDependsFeaturePutPathParams as PutPathParams, ProviderReference,
};

pub async fn delete(_path_params: DeletePathParams) -> DeleteResponse {
    todo!()
}

pub async fn get(_path_params: GetPathParams) -> GetResponse {
    todo!()
}

pub async fn put(
    _provider_reference: ProviderReference,
    _path_params: PutPathParams,
) -> PutResponse {
    todo!()
}
