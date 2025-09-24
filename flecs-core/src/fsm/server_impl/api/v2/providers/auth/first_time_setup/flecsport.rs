use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::enchantment::quest_master::QuestMaster;
use crate::lore::Lore;
use crate::relic::device::usb::UsbDeviceReader;
use crate::sorcerer::importius::{ImportPathInfo, Importius};
use crate::vault::Vault;
use flecsd_axum_server::apis::experimental::ProvidersAuthFirstTimeSetupFlecsportPostResponse as PostResponse;
use flecsd_axum_server::models;
use futures_util::TryFutureExt;
use std::sync::Arc;

pub async fn post<I: Importius, F: Floxy + 'static, U: UsbDeviceReader + 'static>(
    vault: Arc<Vault>,
    lore: Arc<Lore>,
    importius: Arc<I>,
    floxy: Arc<F>,
    usb_device_reader: Arc<U>,
    quest_master: QuestMaster,
) -> PostResponse {
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
        Ok((id, _)) => PostResponse::Status202_Accepted(models::JobMeta::new(id.0 as i32)),
        Err(e) => {
            PostResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}
