use super::{CreateInstanceError, InstanceCommon, InstanceId, Logs, StoredProviderReference};
use crate::forge::time::SystemTimeExt;
use crate::jeweler::deployment::DeploymentId;
use crate::jeweler::gem::deployment::Deployment;
use crate::jeweler::gem::deployment::compose::ComposeDeployment;
use crate::jeweler::gem::instance::compose::config::InstanceConfig;
use crate::jeweler::gem::instance::status::InstanceStatus;
use crate::jeweler::gem::manifest::multi::AppManifestMulti;
use crate::jeweler::gem::manifest::{AppManifest, DependencyKey, multi};
use crate::jeweler::{GetAppKey, serialize_deployment_id, serialize_manifest_key};
use crate::lore::{InstanceLore, InstanceLoreRef};
use crate::quest::{Quest, State, SyncQuest};
use crate::vault::pouch::AppKey;
use crate::{legacy, vault};
use async_trait::async_trait;
use flecsd_axum_server::models::{AppInstance, InstancesInstanceIdGet200Response};
use futures_util::future::{BoxFuture, join_all};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem::swap;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub mod config;

#[derive(Debug, Serialize)]
pub struct ComposeInstance {
    pub id: InstanceId,
    #[serde(serialize_with = "serialize_manifest_key", rename = "app_key")]
    pub manifest: Arc<multi::AppManifestMulti>,
    #[serde(serialize_with = "serialize_deployment_id", rename = "deployment_id")]
    pub deployment: Arc<dyn ComposeDeployment>,
    pub name: String,
    pub desired: InstanceStatus,
    #[serde(skip_serializing)]
    lore: InstanceLoreRef,
    config: InstanceConfig,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ComposeInstanceDeserializable {
    pub id: InstanceId,
    pub app_key: AppKey,
    pub deployment_id: DeploymentId,
    pub name: String,
    pub desired: InstanceStatus,
    pub config: Option<InstanceConfig>,
}

#[async_trait]
impl InstanceCommon for ComposeInstance {
    fn id(&self) -> InstanceId {
        self.id
    }

    fn app_key(&self) -> &AppKey {
        &self.manifest.key
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn manifest(&self) -> AppManifest {
        AppManifest::Multi(self.manifest.clone())
    }

    fn replace_manifest(&mut self, manifest: AppManifest) -> AppManifest {
        let AppManifest::Multi(mut manifest) = manifest else {
            panic!("Can not replace manifest of ComposeInstance with {manifest:?}");
        };
        swap(&mut manifest, &mut self.manifest);
        AppManifest::Multi(manifest)
    }

    async fn generate_info(&self) -> anyhow::Result<AppInstance> {
        let status = self.status().await?;
        Ok(flecsd_axum_server::models::AppInstance {
            instance_id: format!("{}", self.id),
            instance_name: self.name.clone(),
            app_key: self.app_key().clone().into(),
            status: status.into(),
            desired: self.desired.into(),
            editors: None,
        })
    }

    async fn generate_detailed_info(&self) -> anyhow::Result<InstancesInstanceIdGet200Response> {
        let status = self.status().await?;
        Ok(
            flecsd_axum_server::models::InstancesInstanceIdGet200Response {
                instance_id: format!("{}", self.id),
                instance_name: self.name.clone(),
                app_key: self.manifest.app_key().clone().into(),
                status: status.into(),
                desired: self.desired.into(),
                config_files: Vec::new().into(),
                hostname: String::new(),
                ip_address: String::new(),
                ports: Vec::new(),
                volumes: Vec::new(),
                editors: None,
            },
        )
    }

    async fn status(&self) -> anyhow::Result<InstanceStatus> {
        let status = self.deployment.instance_status(&self.manifest).await?;
        let status = Self::aggregate_status(status);
        Ok(status)
    }

    fn desired_status(&self) -> InstanceStatus {
        self.desired
    }

    fn taken_ipv4_addresses(&self) -> Vec<Ipv4Addr> {
        // TODO
        Vec::new()
    }

    async fn logs(&self) -> anyhow::Result<Logs> {
        self.deployment.instance_logs(&self.manifest).await
    }

    async fn import(
        &mut self,
        quest: SyncQuest,
        src: PathBuf,
        _dst: PathBuf,
    ) -> anyhow::Result<()> {
        let image = self
            .manifest
            .images()
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Manifest contains no image"))?;
        let src = src.join("volumes");
        let mut results = Vec::new();
        for volume_name in self.manifest.volume_names() {
            let result = self
                .import_volume_quest(&quest, volume_name, src.clone(), image.clone())
                .await;
            results.push(result);
        }
        for result in join_all(results).await {
            result?;
        }
        Ok(())
    }

    async fn halt(&self) -> anyhow::Result<()> {
        ComposeInstance::halt(self).await
    }

    fn dependencies(&self) -> &HashMap<DependencyKey, StoredProviderReference> {
        &self.config.dependencies
    }

    fn clear_dependency(&mut self, feature: &DependencyKey) -> Option<StoredProviderReference> {
        self.config.dependencies.remove(feature)
    }

    fn set_dependency(
        &mut self,
        feature: DependencyKey,
        provider: StoredProviderReference,
    ) -> Option<StoredProviderReference> {
        self.config.dependencies.insert(feature, provider)
    }
}

impl ComposeInstance {
    fn lore(&self) -> &InstanceLore {
        self.lore.as_ref().as_ref()
    }

    fn workdir(&self) -> PathBuf {
        self.lore().instance_workdir_path(&self.id.to_string())
    }

    fn aggregate_status(status_vec: Vec<InstanceStatus>) -> InstanceStatus {
        if status_vec.is_empty() {
            return InstanceStatus::Stopped;
        }
        let mut status_iter = status_vec.into_iter();
        let resulting_status = status_iter.next().unwrap_or(InstanceStatus::Unknown);
        for status in status_iter {
            if status != resulting_status {
                return InstanceStatus::Unknown;
            }
        }
        resulting_status
    }

    pub fn try_create_with_state(
        lore: InstanceLoreRef,
        instance: ComposeInstanceDeserializable,
        manifests: &vault::pouch::manifest::Gems,
        deployments: &vault::pouch::deployment::Gems,
    ) -> Result<Self, CreateInstanceError> {
        let manifest = manifests
            .get(&instance.app_key)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No manifest for instance {} of {} found",
                    instance.id,
                    instance.app_key
                )
            })?
            .clone();
        let AppManifest::Multi(manifest) = manifest else {
            return Err(anyhow::anyhow!(
                "ComposeInstances can only be created with AppManifestSingle, not with {}",
                manifest.key()
            )
            .into());
        };
        let deployment = deployments
            .get(&instance.deployment_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Deployment {} of instance {} for app {} does not exist",
                    instance.deployment_id,
                    instance.id,
                    instance.app_key
                )
            })?
            .clone();
        let Deployment::Compose(deployment) = deployment else {
            return Err(anyhow::anyhow!(
                "ComposeInstances can only be created with ComposeDeployments, not with {}",
                deployment.id()
            )
            .into());
        };
        Ok(Self {
            lore,
            manifest,
            deployment,
            desired: instance.desired,
            id: instance.id,
            name: instance.name,
            config: instance.config.unwrap_or_default(),
        })
    }

    pub async fn try_create_from_legacy(
        lore: InstanceLoreRef,
        instance: legacy::deployment::Instance,
        manifest: Arc<AppManifestMulti>,
        deployment: Arc<dyn ComposeDeployment>,
    ) -> Result<Self, CreateInstanceError> {
        let instance = legacy::deployment::migrate_compose_instance(instance, deployment.id())?;
        Self::try_create(
            Quest::new_synced("Try create instance"),
            lore,
            deployment,
            manifest,
            instance.name,
            instance.id,
            instance.desired,
        )
        .await
    }

    pub async fn try_create_new(
        quest: SyncQuest,
        lore: InstanceLoreRef,
        deployment: Arc<dyn ComposeDeployment>,
        manifest: Arc<AppManifestMulti>,
        name: String,
    ) -> Result<Self, CreateInstanceError> {
        Self::try_create(
            quest,
            lore,
            deployment,
            manifest,
            name,
            InstanceId::new_random(),
            InstanceStatus::Stopped,
        )
        .await
    }

    pub async fn try_create(
        quest: SyncQuest,
        lore: InstanceLoreRef,
        deployment: Arc<dyn ComposeDeployment>,
        manifest: Arc<AppManifestMulti>,
        name: String,
        instance_id: InstanceId,
        desired: InstanceStatus,
    ) -> Result<Self, CreateInstanceError> {
        tokio::fs::create_dir_all(
            lore.as_ref()
                .as_ref()
                .instance_workdir_path(&instance_id.to_string()),
        )
        .await?;
        let mut results = Vec::new();
        for volume_name in manifest.external_volume_names() {
            let deployment = deployment.clone();
            let result = quest
                .lock()
                .await
                .create_sub_quest(
                    format!("Create external volume {volume_name}"),
                    |quest| async move { deployment.create_volume(quest, &volume_name).await },
                )
                .await
                .2;
            results.push(result);
        }
        for result in join_all(results).await {
            result?;
        }
        Ok(Self {
            lore,
            deployment,
            name,
            manifest,
            desired,
            id: instance_id,
            config: InstanceConfig::default(),
        })
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.desired = InstanceStatus::Running;
        if self.status().await? == InstanceStatus::Running {
            return Ok(());
        }
        self.deployment
            .start_instance(&self.manifest, &self.workdir())
            .await?;
        Ok(())
    }

    pub async fn resume(&self) -> anyhow::Result<()> {
        if self.desired != InstanceStatus::Running
            || self.status().await? == InstanceStatus::Running
        {
            return Ok(());
        }
        self.deployment
            .start_instance(&self.manifest, &self.workdir())
            .await?;
        Ok(())
    }

    pub async fn halt(&self) -> anyhow::Result<()> {
        if self.status().await? == InstanceStatus::Stopped {
            return Ok(());
        }
        self.deployment.stop_instance(&self.manifest).await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> anyhow::Result<()> {
        self.desired = InstanceStatus::Stopped;
        self.halt().await
    }

    pub async fn stop_and_delete(mut self) -> Result<(), (anyhow::Error, Self)> {
        self.desired = InstanceStatus::NotCreated;
        if let Err(e) = self.halt().await {
            return Err((e, self));
        };
        if let Err(e) = tokio::fs::remove_dir_all(&self.workdir()).await {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err((e.into(), self));
            }
        };
        Ok(())
    }

    async fn import_volume_quest(
        &self,
        quest: &SyncQuest,
        volume_name: String,
        src: PathBuf,
        image: String,
    ) -> BoxFuture<'static, crate::Result<()>> {
        let deployment = self.deployment.clone();
        let container_path = PathBuf::from("/flecs_tmp_volume");
        quest
            .lock()
            .await
            .create_sub_quest(
                format!("Import volume {volume_name} from {src:?}"),
                |quest| async move {
                    if !tokio::fs::try_exists(&src).await? {
                        let mut quest = quest.lock().await;
                        quest.state = State::Skipped;
                        quest.detail = Some("Directory does not exist".to_string());
                    } else {
                        deployment
                            .import_volume(quest, &src, &container_path, &volume_name, &image)
                            .await?;
                    }
                    Ok(())
                },
            )
            .await
            .2
    }

    async fn export_volume_quest(
        &self,
        quest: &SyncQuest,
        volume_name: String,
        dst: PathBuf,
        image: String,
    ) -> BoxFuture<'static, crate::Result<()>> {
        let deployment = self.deployment.clone();
        let container_path = PathBuf::from("/flecs_tmp_volume");
        quest
            .lock()
            .await
            .create_sub_quest(
                format!("Export volume {volume_name} to {dst:?}"),
                |quest| async move {
                    if deployment
                        .inspect_volume(volume_name.clone())
                        .await?
                        .is_none()
                    {
                        let mut quest = quest.lock().await;
                        quest.state = State::Skipped;
                        quest.detail = Some("Volume does not exist".to_string());
                        Ok(())
                    } else {
                        deployment
                            .export_volume(quest, volume_name, &dst, &container_path, &image)
                            .await
                    }
                },
            )
            .await
            .2
    }

    pub async fn export(&mut self, quest: SyncQuest, path: &Path) -> anyhow::Result<()> {
        let image = self
            .manifest
            .images()
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Manifest contains no image"))?;
        let path = path.join("volumes");
        let mut results = Vec::new();
        for volume_name in self.manifest.volume_names() {
            let result = self
                .export_volume_quest(&quest, volume_name, path.clone(), image.clone())
                .await;
            results.push(result);
        }
        for result in join_all(results).await {
            result?;
        }
        Ok(())
    }

    pub async fn is_running(&self) -> anyhow::Result<bool> {
        Ok(self.status().await? == InstanceStatus::Running)
    }

    pub async fn update(
        &mut self,
        quest: SyncQuest,
        new_manifest: Arc<AppManifestMulti>,
        base_path: &Path,
    ) -> anyhow::Result<()> {
        let is_running = self.is_running().await?;
        if is_running {
            self.halt().await?;
        }
        let now = std::time::SystemTime::now();
        let backup_path = base_path.join("backup");
        let current_version = self.manifest.key.version.clone();
        let new_backup_path = backup_path
            .join(&current_version)
            .join(now.unix_millis().to_string());
        self.export(quest.clone(), &new_backup_path).await?;
        let new_version = new_manifest.key.version.clone();
        self.manifest = new_manifest;
        if current_version > new_version {
            let mut entries = tokio::fs::read_dir(backup_path.join(&new_version)).await?;
            let mut latest_backup = None;
            while let Some(entry) = entries.next_entry().await? {
                match &latest_backup {
                    None => latest_backup = Some(entry.path()),
                    Some(current) => {
                        if entry.path() > *current {
                            latest_backup = Some(entry.path());
                        }
                    }
                }
            }
            if let Some(backup) = latest_backup {
                self.import(quest, backup, base_path.to_path_buf()).await?;
            }
        }
        if is_running {
            self.start().await?;
        }
        Ok(())
    }
}
