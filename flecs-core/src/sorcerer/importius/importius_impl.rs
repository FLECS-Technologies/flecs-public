use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::forge::time::SystemTimeExt;
use crate::quest::SyncQuest;
use crate::relic::async_flecstract::extract_from_file;
use crate::sorcerer::importius::{ImportError, Importius};
use crate::sorcerer::spell::instance::start_all_instances_as_desired;
use crate::sorcerer::{Sorcerer, spell};
use crate::vault::Vault;
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::warn;

#[derive(Default)]
pub struct ImportiusImpl;

impl Sorcerer for ImportiusImpl {}

#[async_trait]
impl Importius for ImportiusImpl {
    async fn import_archive<F: Floxy + 'static>(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        archive_path: PathBuf,
        temp_path: PathBuf,
        base_path: PathBuf,
    ) -> Result<(), ImportError> {
        let now = std::time::SystemTime::now();
        let temp_path = temp_path.join(now.unix_millis().to_string());
        tokio::fs::create_dir_all(&temp_path).await?;
        let extract_closure = {
            let archive_path = archive_path.clone();
            let temp_path = temp_path.clone();
            move |_quest: SyncQuest| extract_from_file(archive_path, temp_path)
        };
        let extract_result = quest
            .lock()
            .await
            .create_sub_quest("Extract import archive", extract_closure)
            .await
            .2;
        extract_result.await.map_err(|error| ImportError::Extract {
            import: archive_path,
            error,
        })?;
        let manifest = quest
            .lock()
            .await
            .create_sub_quest("Read import manifest", |quest| {
                spell::flimport::read_import_manifest(quest, temp_path.clone())
            })
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
                    manifest.contents.instances.clone(),
                )
            })
            .await
            .2;
        stop_result.await.map_err(ImportError::InstanceStop)?;
        let import_closure = {
            let temp_path = temp_path.clone();
            let vault = vault.clone();
            move |quest: SyncQuest| {
                spell::flimport::import_directory(quest, vault, manifest, temp_path, base_path)
            }
        };
        let result = quest
            .lock()
            .await
            .create_sub_quest("Import", import_closure)
            .await
            .2;
        let result = result.await;
        if let Err(e) = tokio::fs::remove_dir_all(&temp_path).await {
            warn!("Could not remove temporary import directory {temp_path:?}: {e}")
        }
        result?;
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
}
