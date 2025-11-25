use crate::fsm::server_impl::api::v2::models::{Accepted, AdditionalInfo};
use crate::fsm::server_impl::state::{
    FloxyState, ImportiusState, LoreState, QuestMasterState, UsbDeviceReaderState, VaultState,
};
use crate::sorcerer::importius::{ImportPathInfo, Importius};
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use futures_util::TryFutureExt;

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
pub async fn post<I: Importius>(
    State(VaultState(vault)): State<VaultState>,
    State(LoreState(lore)): State<LoreState>,
    State(ImportiusState(importius)): State<ImportiusState<I>>,
    State(FloxyState(floxy)): State<FloxyState>,
    State(UsbDeviceReaderState(usb_device_reader)): State<UsbDeviceReaderState>,
    State(QuestMasterState(quest_master)): State<QuestMasterState>,
) -> Response {
    let path_info = ImportPathInfo {
        archive_path: lore.auth.initial_auth_provider_flecsport_path.clone(),
        temp_path: lore.import.base_path.clone(),
        base_path: lore.base_path.clone(),
    };
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
                    .import_archive(quest, vault, floxy, lore, usb_device_reader, path_info)
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
