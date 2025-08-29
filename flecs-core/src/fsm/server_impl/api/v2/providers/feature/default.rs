use flecsd_axum_server::apis::experimental::{
    ProvidersFeatureDefaultDeleteResponse as DeleteResponse,
    ProvidersFeatureDefaultGetResponse as GetResponse,
    ProvidersFeatureDefaultPutResponse as PutResponse,
};
use flecsd_axum_server::models::{
    ProvidersFeatureDefaultDeletePathParams as DeletePathParams,
    ProvidersFeatureDefaultGetPathParams as GetPathParams,
    ProvidersFeatureDefaultPutPathParams as PutPathParams, PutDefaultProviderRequest as PutRequest,
};

pub async fn delete(_path_params: DeletePathParams) -> DeleteResponse {
    todo!()
}

pub async fn get(_path_params: GetPathParams) -> GetResponse {
    todo!()
}

pub async fn put(_request: PutRequest, _path_params: PutPathParams) -> PutResponse {
    todo!()
}
