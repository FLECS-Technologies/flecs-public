use crate::jeweler::deployment::Deployment;
use crate::quest::SyncQuest;
use bollard::container::Config;
use bollard::models::ContainerStateStatusEnum;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone)]
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

#[derive(Debug, Deserialize)]
pub struct InstanceDeserializable {
    pub name: String,
    pub id: InstanceId,
    pub config: InstanceConfig,
    pub desired: InstanceStatus,
}

#[derive(Debug, Serialize)]
pub struct Instance {
    pub name: String,
    pub id: InstanceId,
    pub config: InstanceConfig,
    #[serde(skip)]
    deployment: Arc<dyn Deployment>,
    desired: InstanceStatus,
}

impl Instance {
    pub(super) fn new(
        id: InstanceId,
        name: String,
        config: InstanceConfig,
        deployment: Arc<dyn Deployment>,
        desired: InstanceStatus,
    ) -> Self {
        Self {
            name,
            id,
            config,
            deployment,
            desired,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_instance_id() {
        assert_eq!("0", InstanceId { value: 0 }.to_string());
        assert_eq!("1", InstanceId { value: 1 }.to_string());
        assert_eq!("f0", InstanceId { value: 240 }.to_string());
        assert_eq!("3da33", InstanceId { value: 252467 }.to_string());
    }
}
