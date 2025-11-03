use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::fsm::server_impl::api::v2::models::{Accepted, AdditionalInfo};
use crate::fsm::server_impl::state::{
    EnforcerState, FloxyState, ImportiusState, LoreState, ProvidiusState, QuestMasterState,
    UsbDeviceReaderState, VaultState,
};
use crate::sorcerer::importius::{ImportPathInfo, Importius};
use crate::wall;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use futures_util::TryFutureExt;
use tracing::warn;

#[allow(clippy::too_many_arguments)]
#[utoipa::path(
    post,
    path = "/providers/auth/first-time-setup/flecsport",
    tag = "Experimental",
    description = "Trigger the first time setup of auth providers via flecsport",
    responses(
        (status = ACCEPTED, description = "First time setup of auth providers via flecsport triggered", body = Accepted),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn post<I: Importius, F: Floxy + 'static>(
    State(VaultState(vault)): State<VaultState>,
    State(LoreState(lore)): State<LoreState>,
    State(ImportiusState(importius)): State<ImportiusState<I>>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    State(FloxyState(floxy)): State<FloxyState<F>>,
    State(UsbDeviceReaderState(usb_device_reader)): State<UsbDeviceReaderState>,
    State(QuestMasterState(quest_master)): State<QuestMasterState>,
    State(EnforcerState(enforcer)): State<EnforcerState>,
    axum::Extension(roles): axum::Extension<wall::watch::RolesExtension>,
    host: axum::extract::Host,
) -> Response {
    let path_info = ImportPathInfo {
        archive_path: lore.auth.initial_auth_provider_flecsport_path.clone(),
        temp_path: lore.import.base_path.clone(),
        base_path: lore.base_path.clone(),
    };
    let roles_allow_request = match enforcer
        .verify_roles(
            "/v2/providers/auth/core/first-time-setup/flecsport",
            &roles.0,
            &http::Method::POST,
        )
        .await
    {
        Ok(allow) => allow,
        Err(e) => {
            warn!("Error verifying roles: {e}");
            return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    // The initial auth provider flecsport is allowed to be imported if currently no auth provider
    // exists even if the roles do not allow it
    if !roles_allow_request
        && !providius
            .get_auth_providers_and_default(vault.clone(), &host)
            .await
            .providers
            .is_empty()
    {
        #[cfg(feature = "dev-auth")]
        warn!(
            "Authorization failed, but feature dev-auth is enabled and the request will be processed "
        );
        #[cfg(not(feature = "dev-auth"))]
        return http::StatusCode::FORBIDDEN.into_response();
    }
    match quest_master
        .lock()
        .await
        .schedule_quest(
            format!(
                "Importing initial auth provider from {:?}",
                path_info.archive_path
            ),
            move |quest| async move {
                importius
                    .import_archive(
                        quest,
                        vault,
                        FloxyOperation::new_arc(floxy),
                        lore,
                        usb_device_reader,
                        path_info,
                    )
                    .map_err(|e| anyhow::anyhow!(e))
                    .await
            },
        )
        .await
    {
        Ok((id, _)) => Accepted::new(id).into_response(),
        Err(e) => AdditionalInfo::new(e.to_string()).into_internal_server_error(),
    }
}
