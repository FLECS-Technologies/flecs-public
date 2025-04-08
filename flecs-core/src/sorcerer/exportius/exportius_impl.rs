use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::jeweler::gem::instance::InstanceId;
use crate::quest::SyncQuest;
use crate::sorcerer::exportius::{
    ExportAppError, ExportDeploymentError, ExportInstanceError, Exportius,
};
use crate::sorcerer::Sorcerer;
use crate::vault::pouch::AppKey;
use crate::vault::Vault;
use async_trait::async_trait;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Default)]
pub struct ExportiusImpl;

impl Sorcerer for ExportiusImpl {}

#[async_trait]
impl Exportius for ExportiusImpl {
    async fn delete_export(&self, export_dir: &Path, export_id: String) -> Result<bool, Error> {
        crate::sorcerer::spell::flecsport::delete_export(export_dir, export_id).await
    }

    async fn get_export(
        &self,
        export_dir: &Path,
        export_id: String,
    ) -> Result<Option<PathBuf>, Error> {
        crate::sorcerer::spell::flecsport::get_export(export_dir, export_id).await
    }

    async fn export_apps(
        quest: SyncQuest,
        vault: Arc<Vault>,
        apps: Vec<AppKey>,
        path: PathBuf,
    ) -> Result<(), ExportAppError> {
        crate::sorcerer::spell::flecsport::export_apps(quest, vault, apps, path).await
    }

    async fn export_instances<F: Floxy + 'static>(
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        instances: Vec<InstanceId>,
        path: PathBuf,
    ) -> Result<(), ExportInstanceError> {
        crate::sorcerer::spell::flecsport::export_instances(quest, vault, floxy, instances, path)
            .await
    }

    async fn export_deployments(
        quest: SyncQuest,
        vault: Arc<Vault>,
        path: PathBuf,
    ) -> Result<(), ExportDeploymentError> {
        crate::sorcerer::spell::flecsport::export_deployments(quest, vault, path).await
    }
}
