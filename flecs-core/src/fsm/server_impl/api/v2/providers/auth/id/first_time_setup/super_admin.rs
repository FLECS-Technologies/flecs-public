use flecsd_axum_server::apis::experimental::{
    ProvidersAuthIdFirstTimeSetupSuperAdminGetResponse as GetResponse,
    ProvidersAuthIdFirstTimeSetupSuperAdminPostResponse as PostResponse,
};
use flecsd_axum_server::models::{
    ProvidersAuthIdFirstTimeSetupSuperAdminGetPathParams as GetPathParams,
    ProvidersAuthIdFirstTimeSetupSuperAdminPostPathParams as PostPathParams, SuperAdmin,
};

pub async fn get(_path_params: GetPathParams) -> GetResponse {
    todo!()
}

pub async fn post(_request: SuperAdmin, _path_params: PostPathParams) -> PostResponse {
    todo!()
}
