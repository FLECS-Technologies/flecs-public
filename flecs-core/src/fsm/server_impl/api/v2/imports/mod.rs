use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::enchantment::quest_master::QuestMaster;
use crate::forge::axum::{MultipartExt, WriteMultipartError};
use crate::relic::device::usb::UsbDeviceReader;
use crate::sorcerer::importius::{ImportPathInfo, Importius};
use crate::vault::Vault;
use axum_extra::extract::Multipart;
use flecsd_axum_server::apis::flecsport::ImportsPostResponse as PostResponse;
use flecsd_axum_server::models;
use futures_util::TryFutureExt;
use std::path::PathBuf;
use std::sync::Arc;

pub async fn post<I: Importius, F: Floxy + 'static, U: UsbDeviceReader + 'static>(
    vault: Arc<Vault>,
    importius: Arc<I>,
    floxy: Arc<F>,
    usb_device_reader: Arc<U>,
    quest_master: QuestMaster,
    request: Multipart,
) -> PostResponse {
    let path = PathBuf::from(crate::lore::flimport::BASE_PATH);
    match request.write_file(path).await {
        Err(e @ WriteMultipartError::NoData) | Err(e @ WriteMultipartError::NoFileName) => {
            PostResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Err(e) => {
            PostResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(file_path) => {
            let path_info = ImportPathInfo {
                archive_path: file_path,
                temp_path: PathBuf::from(crate::lore::flimport::BASE_PATH),
                base_path: PathBuf::from(crate::lore::BASE_PATH),
            };
            match quest_master
                .lock()
                .await
                .schedule_quest(
                    format!("Importing {:?}", path_info.archive_path),
                    move |quest| async move {
                        importius
                            .import_archive(
                                quest,
                                vault,
                                FloxyOperation::new_arc(floxy),
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
                Err(e) => PostResponse::Status500_InternalServerError(models::AdditionalInfo::new(
                    e.to_string(),
                )),
            }
        }
    }
}
