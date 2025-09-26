use crate::fsm::server_impl::api::v2::models::AdditionalInfo;
use crate::fsm::server_impl::api::v2::models::auth::{ProviderOrSuperAdminNotFound, SuperAdmin};
use axum::Json;
use axum::response::Response;

#[utoipa::path(
    get,
    path = "/providers/auth/default/first-time-setup/super-admin",
    tag = "Experimental",
    description = "Check if the super admin of the default auth provider is set",
    responses(
        (status = NO_CONTENT, description = "Super admin of default auth provider is set"),
        (status = NOT_FOUND, description = "Super admin of default auth provider not set or no default provider set", body = ProviderOrSuperAdminNotFound),
    ),
)]
pub async fn get() -> Response {
    todo!()
}

#[utoipa::path(
    post,
    path = "/providers/auth/default/first-time-setup/super-admin",
    tag = "Experimental",
    description = "Set the super admin of the default auth provider",
    request_body(
        content = SuperAdmin,
        description = "Super admin that should be set to the default auth provider",
    ),
    responses(
        (status = OK, description = "Super admin of default auth provider set"),
        (status = BAD_REQUEST, description = "Invalid super admin", body = AdditionalInfo),
        (status = FORBIDDEN, description = "Forbidden"),
        (status = NOT_FOUND, description = "No default auth provider present"),
        (status = INTERNAL_SERVER_ERROR, description = "Failed to set super admin of default auth provider", body = AdditionalInfo),
    ),
)]
pub async fn post(Json(_request): Json<SuperAdmin>) -> Response {
    todo!()
}
