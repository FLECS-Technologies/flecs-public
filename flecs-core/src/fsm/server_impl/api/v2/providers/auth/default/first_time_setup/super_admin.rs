use flecsd_axum_server::apis::experimental::{
    ProvidersAuthDefaultFirstTimeSetupSuperAdminGetResponse as GetResponse,
    ProvidersAuthDefaultFirstTimeSetupSuperAdminPostResponse as PostResponse,
};
use flecsd_axum_server::models::SuperAdmin;

pub async fn get() -> GetResponse {
    todo!()
}

pub async fn post(_request: SuperAdmin) -> PostResponse {
    todo!()
}
