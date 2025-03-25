use crate::sorcerer::authmancer::Authmancer;
use crate::vault::Vault;
use flecsd_axum_server::apis::console::{
    ConsoleAuthenticationDeleteResponse as DeleteResponse,
    ConsoleAuthenticationPutResponse as PutResponse,
};
use flecsd_axum_server::models::AuthResponseData as PutRequest;
use std::sync::Arc;

pub async fn put<A: Authmancer>(
    vault: Arc<Vault>,
    authmancer: Arc<A>,
    request: PutRequest,
) -> PutResponse {
    authmancer.store_authentication(request, &vault).await;
    PutResponse::Status204_NoContent
}

pub async fn delete<A: Authmancer>(vault: Arc<Vault>, authmancer: Arc<A>) -> DeleteResponse {
    authmancer.delete_authentication(&vault).await;
    DeleteResponse::Status204_NoContent
}
