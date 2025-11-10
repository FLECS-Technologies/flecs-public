use crate::fsm::server_impl::api::v2::models::AdditionalInfo;
use crate::fsm::server_impl::state::LoreState;
use axum::Json;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use http::StatusCode;

#[utoipa::path(
    get,
    path = "/system/sbom",
    tag = "System",
    description = "SBOM",
    responses(
        (status = OK, description = "SBOM in spdx format", body = serde_json::Value),
        (status = NOT_FOUND, description = "SBOM was not found at expected location"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn get(State(LoreState(lore)): State<LoreState>) -> Response {
    let sbom_path = &lore.system.core_sbom_spdx_path;
    if matches!(tokio::fs::try_exists(sbom_path).await, Ok(false)) {
        return StatusCode::NOT_FOUND.into_response();
    }
    match tokio::fs::read_to_string(sbom_path).await {
        Err(e) => AdditionalInfo::new(format!(
            "Could not read sbom at {}: {e}",
            sbom_path.display()
        ))
        .into_internal_server_error(),
        Ok(sbom) => match serde_json::from_str::<serde_json::Value>(&sbom) {
            Err(e) => AdditionalInfo::new(format!("Invalid json at {}: {e}", sbom_path.display()))
                .into_internal_server_error(),
            Ok(json) => (StatusCode::OK, Json(json)).into_response(),
        },
    }
}
