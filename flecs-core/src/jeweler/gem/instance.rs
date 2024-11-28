use crate::jeweler::deployment::{Deployment, DeploymentId};
use crate::jeweler::{serialize_deployment_id, serialize_manifest_key};
use crate::quest::SyncQuest;
use crate::vault::pouch::AppKey;
use bollard::container::Config;
use bollard::models::ContainerStateStatusEnum;
use flecs_app_manifest::AppManifest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InstanceId {
    pub(crate) value: u32,
}

impl InstanceId {
    pub fn to_docker_id(self) -> String {
        format!("flecs-{self}")
    }

    pub fn new(value: u32) -> Self {
        Self { value }
    }

    pub fn new_random() -> Self {
        Self {
            value: rand::random::<u32>(),
        }
    }
}

impl From<u32> for InstanceId {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl Display for InstanceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:01x}", self.value)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct InstanceConfig {
    // TODO: Add more info (e.g. from manifest)
    pub image: String,
}

impl From<InstanceConfig> for Config<String> {
    fn from(value: InstanceConfig) -> Self {
        // TODO: Add more info from instance config if available
        Config {
            image: Some(value.image),
            ..Default::default()
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum InstanceStatus {
    // TBD
    NotCreated,
    Requested,
    ResourcesReady,
    Created,
    Stopped,
    Running,
    Orphaned,
    Unknown,
}

impl From<ContainerStateStatusEnum> for InstanceStatus {
    fn from(value: ContainerStateStatusEnum) -> Self {
        // TBD
        match value {
            ContainerStateStatusEnum::EMPTY => Self::Created,
            ContainerStateStatusEnum::CREATED => Self::Created,
            ContainerStateStatusEnum::RUNNING => Self::Running,
            ContainerStateStatusEnum::PAUSED => Self::Running,
            ContainerStateStatusEnum::RESTARTING => Self::Running,
            ContainerStateStatusEnum::REMOVING => Self::Running,
            ContainerStateStatusEnum::EXITED => Self::Created,
            ContainerStateStatusEnum::DEAD => Self::Created,
        }
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
pub struct InstanceDeserializable {
    pub name: String,
    pub id: InstanceId,
    pub config: InstanceConfig,
    pub desired: InstanceStatus,
    pub app_key: AppKey,
    pub deployment_id: DeploymentId,
}

#[derive(Debug, Serialize)]
pub struct Instance {
    pub name: String,
    pub id: InstanceId,
    pub config: InstanceConfig,
    #[serde(serialize_with = "serialize_deployment_id", rename = "deployment_id")]
    deployment: Arc<dyn Deployment>,
    #[serde(serialize_with = "serialize_manifest_key", rename = "app_key")]
    manifest: Arc<AppManifest>,
    desired: InstanceStatus,
}

impl Instance {
    pub fn app_key(&self) -> AppKey {
        AppKey {
            version: self.manifest.version.clone(),
            name: self.manifest.app.to_string(),
        }
    }

    pub async fn create() -> anyhow::Result<Self> {
        // TODO: Create portmapping
        // TODO: Set env and ports
        // TODO: Create volumes
        // TODO: Create networks
        // TODO: Create conffiles
        todo!()
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.desired = InstanceStatus::Running;
        match self.deployment.instance_status(self.id).await? {
            InstanceStatus::Running => {}
            _ => {
                self.id = self
                    .deployment
                    .start_instance(self.config.clone(), Some(self.id))
                    .await?
            }
        }
        Ok(())
    }

    pub async fn stop_and_delete(mut self) -> anyhow::Result<(), (anyhow::Error, Self)> {
        match self.stop().await {
            Err(e) => Err((e, self)),
            Ok(_) => self.delete().await,
        }
    }

    pub async fn stop(&mut self) -> anyhow::Result<()> {
        // TODO: Disconnect networks
        // TODO: Save config files
        self.desired = InstanceStatus::Stopped;
        match self.deployment.instance_status(self.id).await? {
            InstanceStatus::Stopped => Ok(()),
            _ => self.deployment.stop_instance(self.id).await,
        }
    }

    pub async fn delete(self) -> anyhow::Result<(), (anyhow::Error, Self)> {
        // TODO: Delete volumes
        // TODO: Delete floxy config
        self.deployment
            .delete_instance(self.id)
            .await
            .map_err(|e| (e, self))
    }

    pub async fn ready(&mut self) -> anyhow::Result<()> {
        // TODO: Check status, error handling
        self.deployment.ready_instance(self.id).await
    }

    pub async fn export(&self, _path: &Path) -> anyhow::Result<()> {
        // TODO: Export config
        // TODO: Export config files
        // TODO: Export volumes
        Ok(())
    }

    pub async fn import(&self, _path: &Path) -> anyhow::Result<()> {
        // TODO: Import volumes
        // TODO: Import config files
        // TODO: Import config
        Ok(())
    }

    pub async fn status(&self) -> anyhow::Result<InstanceStatus> {
        self.deployment.instance_status(self.id).await
    }

    pub async fn copy_from(&self, quest: SyncQuest, src: &Path, dst: &Path) -> anyhow::Result<()> {
        self.deployment
            .copy_from_instance(quest, self.id, src, dst)
            .await
    }

    pub async fn copy_to(&self, quest: SyncQuest, src: &Path, dst: &Path) -> anyhow::Result<()> {
        self.deployment
            .copy_to_instance(quest, self.id, src, dst)
            .await
    }

    pub async fn is_runnable(&self) -> anyhow::Result<bool> {
        self.deployment.is_instance_runnable(self.id).await
    }

    pub async fn is_running(&self) -> anyhow::Result<bool> {
        self.deployment.is_instance_running(self.id).await
    }
}

pub fn try_create_instance(
    instance: InstanceDeserializable,
    manifests: &HashMap<AppKey, Arc<AppManifest>>,
    deployments: &HashMap<DeploymentId, Arc<dyn Deployment>>,
) -> anyhow::Result<Instance> {
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
    Ok(Instance {
        manifest,
        deployment,
        desired: instance.desired,
        id: instance.id,
        config: instance.config,
        name: instance.name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::sorcerer::appraiser::tests::create_test_manifest;

    #[test]
    fn display_instance_id() {
        assert_eq!("0", InstanceId { value: 0 }.to_string());
        assert_eq!("1", InstanceId { value: 1 }.to_string());
        assert_eq!("f0", InstanceId { value: 240 }.to_string());
        assert_eq!("3da33", InstanceId { value: 252467 }.to_string());
    }

    #[test]
    fn try_create_ok() {
        let manifest = create_test_manifest(None);
        let app_key = AppKey {
            name: manifest.app.to_string(),
            version: manifest.version.clone(),
        };
        let manifests = HashMap::from([(app_key.clone(), manifest)]);
        let deployment_id = "TestDeployment".to_string();
        let deployments = HashMap::from([(
            deployment_id.clone(),
            Arc::new(MockedDeployment::new()) as Arc<dyn Deployment>,
        )]);
        let instance = InstanceDeserializable {
            deployment_id,
            id: InstanceId::new(10),
            app_key,
            name: "TestInstance".to_string(),
            desired: InstanceStatus::Running,
            config: InstanceConfig::default(),
        };
        try_create_instance(instance, &manifests, &deployments).unwrap();
    }

    #[test]
    fn try_create_no_deployment() {
        let manifest = create_test_manifest(None);
        let app_key = AppKey {
            name: manifest.app.to_string(),
            version: manifest.version.clone(),
        };
        let manifests = HashMap::from([(app_key.clone(), manifest)]);
        let deployment_id = "TestDeployment".to_string();
        let deployments = HashMap::new();
        let instance = InstanceDeserializable {
            deployment_id,
            id: InstanceId::new(10),
            app_key,
            name: "TestInstance".to_string(),
            desired: InstanceStatus::Running,
            config: InstanceConfig::default(),
        };
        assert!(try_create_instance(instance, &manifests, &deployments).is_err());
    }

    #[test]
    fn try_create_no_manifest() {
        let manifest = create_test_manifest(None);
        let app_key = AppKey {
            name: manifest.app.to_string(),
            version: manifest.version.clone(),
        };
        let manifests = HashMap::new();
        let deployment_id = "TestDeployment".to_string();
        let deployments = HashMap::from([(
            deployment_id.clone(),
            Arc::new(MockedDeployment::new()) as Arc<dyn Deployment>,
        )]);
        let instance = InstanceDeserializable {
            deployment_id,
            id: InstanceId::new(10),
            app_key,
            name: "TestInstance".to_string(),
            desired: InstanceStatus::Running,
            config: InstanceConfig::default(),
        };
        assert!(try_create_instance(instance, &manifests, &deployments).is_err());
    }
}
