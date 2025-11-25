pub mod compose;
pub mod docker;
mod id;
pub mod status;

use crate::enchantment::floxy::Floxy;
use crate::jeweler::deployment::DeploymentId;
use crate::jeweler::gem::instance::status::InstanceStatus;
use crate::jeweler::gem::manifest::{AppManifest, DependencyKey, FeatureKey};
use crate::lore::Lore;
use crate::quest::SyncQuest;
use crate::vault::pouch;
use crate::vault::pouch::AppKey;
use crate::vault::pouch::provider::ProviderId;
use async_trait::async_trait;
pub use id::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::net::Ipv4Addr;
use std::num::ParseIntError;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, ToSchema)]
pub enum ProviderReference {
    #[default]
    Default,
    Provider(ProviderId),
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, ToSchema)]
pub struct StoredProviderReference {
    pub provider_reference: ProviderReference,
    pub provided_feature: FeatureKey,
}

impl ProviderReference {
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }
}

impl Display for ProviderReference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderReference::Default => write!(f, "Default"),
            ProviderReference::Provider(id) => std::fmt::Display::fmt(id, f),
        }
    }
}

impl FromStr for ProviderReference {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Default" => Ok(Self::Default),
            s => Ok(Self::Provider(ProviderId::from_str(s)?)),
        }
    }
}

impl StoredProviderReference {
    pub fn is_default(&self) -> bool {
        self.provider_reference.is_default()
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Instance {
    Compose(compose::ComposeInstance),
    Docker(docker::DockerInstance),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(clippy::large_enum_variant)]
#[serde(tag = "type")]
pub enum InstanceDeserializable {
    Compose(compose::ComposeInstanceDeserializable),
    Docker(docker::DockerInstanceDeserializable),
}

impl InstanceDeserializable {
    pub fn id(&self) -> InstanceId {
        match self {
            InstanceDeserializable::Compose(compose) => compose.id,
            InstanceDeserializable::Docker(docker) => docker.id,
        }
    }
    pub fn deployment_id(&self) -> &DeploymentId {
        match self {
            InstanceDeserializable::Compose(compose) => &compose.deployment_id,
            InstanceDeserializable::Docker(docker) => &docker.deployment_id,
        }
    }
    pub fn app_key(&self) -> &AppKey {
        match self {
            InstanceDeserializable::Compose(compose) => &compose.app_key,
            InstanceDeserializable::Docker(docker) => &docker.app_key,
        }
    }
}

pub struct Logs {
    pub stdout: String,
    pub stderr: String,
}

impl From<Logs> for flecsd_axum_server::models::InstancesInstanceIdLogsGet200Response {
    fn from(logs: Logs) -> Self {
        Self {
            stdout: logs.stdout,
            stderr: logs.stderr,
        }
    }
}

#[async_trait]
pub trait InstanceCommon {
    fn id(&self) -> InstanceId;
    fn app_key(&self) -> &AppKey;
    fn name(&self) -> &str;
    fn manifest(&self) -> AppManifest;
    fn replace_manifest(&mut self, manifest: AppManifest) -> AppManifest;
    async fn generate_info(&self) -> crate::Result<flecsd_axum_server::models::AppInstance>;
    async fn generate_detailed_info(
        &self,
    ) -> crate::Result<flecsd_axum_server::models::InstancesInstanceIdGet200Response>;
    async fn status(&self) -> anyhow::Result<InstanceStatus>;
    fn desired_status(&self) -> InstanceStatus;
    fn taken_ipv4_addresses(&self) -> Vec<Ipv4Addr>;
    async fn logs(&self) -> anyhow::Result<Logs>;
    async fn import(&mut self, quest: SyncQuest, src: PathBuf, dst: PathBuf) -> anyhow::Result<()>;
    async fn halt(&self) -> anyhow::Result<()>;
    fn dependencies(&self) -> &HashMap<DependencyKey, StoredProviderReference>;
    fn clear_dependency(&mut self, key: &DependencyKey) -> Option<StoredProviderReference>;
    fn set_dependency(
        &mut self,
        key: DependencyKey,
        provider: StoredProviderReference,
    ) -> Option<StoredProviderReference>;
}

impl Deref for Instance {
    type Target = dyn InstanceCommon;

    fn deref(&self) -> &Self::Target {
        match self {
            Instance::Compose(compose) => compose,
            Instance::Docker(docker) => docker,
        }
    }
}

impl DerefMut for Instance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Instance::Compose(compose) => compose,
            Instance::Docker(docker) => docker,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CreateInstanceError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error("IO Error during instance creation: {0}")]
    IO(#[from] std::io::Error),
    #[error("No manifest found {0}")]
    NoManifest(AppKey),
    #[error("No fitting deployment found")]
    NoFittingDeployment,
    #[error("App '{0}' not installed")]
    AppNotInstalled(AppKey),
}

impl Instance {
    pub fn try_create_with_state(
        lore: Arc<Lore>,
        instance: InstanceDeserializable,
        manifests: &pouch::manifest::Gems,
        deployments: &pouch::deployment::Gems,
    ) -> Result<Instance, CreateInstanceError> {
        match instance {
            InstanceDeserializable::Compose(compose) => {
                let instance = compose::ComposeInstance::try_create_with_state(
                    lore,
                    compose,
                    manifests,
                    deployments,
                )?;
                Ok(Self::Compose(instance))
            }
            InstanceDeserializable::Docker(docker) => {
                let instance = docker::DockerInstance::try_create_with_state(
                    lore,
                    docker,
                    manifests,
                    deployments,
                )?;
                Ok(Self::Docker(instance))
            }
        }
    }

    pub async fn export(
        &mut self,
        quest: SyncQuest,
        floxy: Arc<dyn Floxy>,
        path: &Path,
    ) -> anyhow::Result<()> {
        tokio::fs::create_dir_all(&path).await?;
        let instance_config = serde_json::to_vec_pretty(&self)?;
        let result = quest
            .lock()
            .await
            .create_sub_quest(
                format!("Export config of instance {}", self.id()),
                |_quest| tokio::fs::write(path.join("instance.json"), instance_config),
            )
            .await
            .2;
        result.await?;
        match self {
            Instance::Compose(instance) => instance.export(quest, path).await?,
            Instance::Docker(instance) => instance.export(quest, floxy, path).await?,
        }
        Ok(())
    }

    pub async fn update(
        &mut self,
        quest: SyncQuest,
        floxy: Arc<dyn Floxy>,
        manifest: AppManifest,
        backup_path: &Path,
    ) -> anyhow::Result<()> {
        match (manifest, self) {
            (AppManifest::Multi(manifest), Instance::Compose(instance)) => {
                instance.update(quest, manifest, backup_path).await
            }
            (AppManifest::Single(manifest), Instance::Docker(instance)) => {
                instance.update(quest, floxy, manifest, backup_path).await
            }
            _ => Err(anyhow::anyhow!("Instance and manifest do not match")),
        }
    }
}
