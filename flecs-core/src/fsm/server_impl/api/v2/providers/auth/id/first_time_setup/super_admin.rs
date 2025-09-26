use crate::fsm::server_impl::api::v2::models::AdditionalInfo;
use crate::fsm::server_impl::api::v2::models::auth::{ProviderOrSuperAdminNotFound, SuperAdmin};
use crate::vault::pouch::provider::ProviderId;
use axum::Json;
use axum::extract::Path;
use axum::response::Response;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use utoipa::IntoParams;

#[serde_as]
#[derive(Debug, Deserialize, IntoParams)]
pub struct PostPathParams {
    #[serde_as(as = "DisplayFromStr")]
    pub id: ProviderId,
}

pub type GetPathParams = PostPathParams;

#[utoipa::path(
    get,
    path = "/providers/auth/{id}/first-time-setup/super-admin",
    tag = "Experimental",
    description = "Check if the super admin of the specified auth provider is set",
    params(GetPathParams),
    responses(
        (status = NO_CONTENT, description = "Super admin of specified auth provider is set"),
        (status = NOT_FOUND, description = "Super admin of specified auth provider not set or specified provider not found", body = ProviderOrSuperAdminNotFound),
    ),
)]
pub async fn get(Path(GetPathParams { id: _id }): Path<GetPathParams>) -> Response {
    todo!()
}

#[utoipa::path(
    post,
    path = "/providers/auth/{id}/first-time-setup/super-admin",
    tag = "Experimental",
    description = "Set the super admin of the specified auth provider",
    request_body(
        content = SuperAdmin,
        description = "Super admin that should be set to the specified auth provider",
    ),
    params(PostPathParams),
    responses(
        (status = OK, description = "Super admin of specified auth provider set"),
        (status = BAD_REQUEST, description = "Invalid super admin", body = AdditionalInfo),
        (status = FORBIDDEN, description = "Forbidden"),
        (status = NOT_FOUND, description = "Specified auth provider not found"),
        (status = INTERNAL_SERVER_ERROR, description = "Failed to set super admin of specified auth provider", body = AdditionalInfo),
    ),
)]
pub async fn post(
    Path(PostPathParams { id: _id }): Path<PostPathParams>,
    Json(_request): Json<SuperAdmin>,
) -> Response {
    todo!()
}
