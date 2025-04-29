use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::forge::time::SystemTimeExt;
use crate::quest::SyncQuest;
use crate::relic::async_flecstract::{decompress_from_file, extract_from_file};
use crate::relic::device::usb::UsbDeviceReader;
use crate::sorcerer::exportius::manifest::Manifest;
use crate::sorcerer::importius::{ImportError, ImportPathInfo, Importius};
use crate::sorcerer::spell::instance::start_all_instances_as_desired;
use crate::sorcerer::{Sorcerer, spell};
use crate::vault::Vault;
use async_trait::async_trait;
use futures_util::future::BoxFuture;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::warn;

#[derive(Default)]
pub struct ImportiusImpl;

impl Sorcerer for ImportiusImpl {}

#[async_trait]
impl Importius for ImportiusImpl {
    async fn import_archive<F: Floxy + 'static, U: UsbDeviceReader + 'static>(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        usb_device_reader: Arc<U>,
        mut path_info: ImportPathInfo,
    ) -> Result<(), ImportError> {
        let now = std::time::SystemTime::now();
        path_info.temp_path = path_info.temp_path.join(now.unix_millis().to_string());
        tokio::fs::create_dir_all(&path_info.temp_path).await?;
        let temp_path = path_info.temp_path.clone();
        let result = import_archive(quest, vault, floxy, usb_device_reader, path_info).await;
        if let Err(e) = tokio::fs::remove_dir_all(&temp_path).await {
            warn!("Could not remove temporary import directory {temp_path:?}: {e}")
        }
        result
    }
}

async fn import_archive<F: Floxy + 'static, U: UsbDeviceReader + 'static>(
    quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
    usb_device_reader: Arc<U>,
    path_info: ImportPathInfo,
) -> Result<(), ImportError> {
    extract_quest(&quest, &path_info.archive_path, &path_info.temp_path)
        .await
        .await?;
    import(
        quest.clone(),
        vault.clone(),
        floxy.clone(),
        usb_device_reader,
        path_info.temp_path,
        path_info.base_path,
    )
    .await?;
    let result = quest
        .lock()
        .await
        .create_sub_quest("Start instances", |quest| {
            start_all_instances_as_desired(quest, vault, floxy)
        })
        .await
        .2;
    result.await.map_err(ImportError::InstanceStart)?;
    Ok(())
}

async fn extract_quest(
    quest: &SyncQuest,
    archive_path: &Path,
    temp_path: &Path,
) -> BoxFuture<'static, Result<(), ImportError>> {
    let extract_closure = {
        let archive_path = archive_path.to_path_buf();
        let temp_path = temp_path.to_path_buf();
        move |_quest: SyncQuest| async move {
            let result = if archive_path.extension() == Some("gz".as_ref()) {
                decompress_from_file(archive_path.clone(), temp_path).await
            } else {
                extract_from_file(archive_path.clone(), temp_path).await
            };
            result.map_err(|error| ImportError::Extract {
                import: archive_path,
                error,
            })
        }
    };
    quest
        .lock()
        .await
        .create_sub_quest("Extract import archive", extract_closure)
        .await
        .2
}

async fn import<F: Floxy + 'static, U: UsbDeviceReader + 'static>(
    quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
    usb_device_reader: Arc<U>,
    import_path: PathBuf,
    base_path: PathBuf,
) -> Result<(), ImportError> {
    // Export data is either in the root of the archive or there is exactly one directory containing the data
    let import_path = {
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(&import_path).await?;
        while let Some(entry) = dir.next_entry().await? {
            entries.push(entry);
        }
        // Exactly one directory
        if entries.len() == 1 && entries[0].file_type().await?.is_dir() {
            entries[0].path()
        } else {
            import_path
        }
    };
    let manifest = quest
        .lock()
        .await
        .create_sub_quest(
            format!("Read import manifest from {import_path:?}"),
            |quest| spell::flimport::read_import_manifest(quest, import_path.clone()),
        )
        .await
        .2;
    let manifest = manifest.await?;
    let stop_result = quest
        .lock()
        .await
        .create_sub_quest("Stop affected instances", |quest| {
            spell::instance::stop_existing_instances(
                quest,
                vault.clone(),
                floxy.clone(),
                manifest.instance_ids(),
            )
        })
        .await
        .2;
    stop_result.await.map_err(ImportError::InstanceStop)?;
    let import_closure = {
        let import_path = import_path.clone();
        let vault = vault.clone();
        |quest: SyncQuest| async move {
            match manifest {
                Manifest::V2(manifest) => {
                    spell::flimport::import_legacy_directory(
                        quest,
                        vault,
                        usb_device_reader,
                        manifest,
                        import_path,
                        base_path,
                    )
                    .await
                }
                Manifest::V3(manifest) => {
                    spell::flimport::import_directory(
                        quest,
                        vault,
                        manifest,
                        import_path,
                        base_path,
                    )
                    .await
                }
            }
        }
    };
    let result = quest
        .lock()
        .await
        .create_sub_quest("Import", import_closure)
        .await
        .2;
    result.await
}
