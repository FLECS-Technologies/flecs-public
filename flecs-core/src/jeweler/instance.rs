use super::Result;
use crate::jeweler::deployment::Deployment;
use crate::jeweler::volume::VolumeDeployment;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[async_trait]
pub trait InstanceDeployment: VolumeDeployment {
    async fn create_instance(&self, config: InstanceConfig) -> Result<InstanceId>;
    async fn delete_instance(&self, id: InstanceId) -> Result<()>;
    async fn start_instance(&self, id: InstanceId) -> Result<()>;
    async fn stop_instance(&self, id: InstanceId) -> Result<()>;
    async fn ready_instance(&self, id: InstanceId) -> Result<()>;
    async fn import_instance(&self, path: &Path) -> Result<InstanceId>;
    async fn export_instance(&self, id: InstanceId, path: &Path) -> Result<()> {
        let config = self.instance_config(id.clone()).await?;
        self.export_config(config, path).await?;
        self.export_volumes(id, path).await?;
        Ok(())
    }
    async fn export_config(&self, config: InstanceConfig, path: &Path) -> Result<()>;
    async fn instance_status(&self, id: InstanceId) -> Result<InstanceStatus>;
    async fn instance_config(&self, id: InstanceId) -> Result<InstanceConfig>;
    async fn instance(&self, id: InstanceId) -> Result<(InstanceConfig, InstanceStatus)> {
        Ok((
            self.instance_config(id.clone()).await?,
            self.instance_status(id).await?,
        ))
    }
    async fn instances(&self) -> Result<HashMap<InstanceId, (InstanceConfig, InstanceStatus)>>;
    async fn export_instances(&self, path: &Path) -> Result<()> {
        for id in self.instances().await?.keys() {
            self.export_instance(id.clone(), path).await?;
        }
        Ok(())
    }
    async fn copy_from_instance(&self, id: InstanceId, src: &Path, dst: &Path) -> Result<()>;
    async fn copy_to_instance(&self, id: InstanceId, src: &Path, dst: &Path) -> Result<()>;
    // TODO: Maybe move function to enum InstanceStatus
    async fn is_instance_runnable(&self, id: InstanceId) -> Result<bool> {
        Ok(self.instance_status(id).await? == InstanceStatus::Created)
    }
    // TODO: Maybe move function to enum InstanceStatus
    async fn is_instance_running(&self, id: InstanceId) -> Result<bool> {
        Ok(self.instance_status(id).await? == InstanceStatus::Running)
    }
}

pub(crate) type InstanceId = String;
#[derive(Default, Clone)]
pub struct InstanceConfig {
    // TBD
}

#[derive(Debug, PartialEq)]
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

pub struct Instance {
    id: InstanceId,
    pub config: InstanceConfig,
    deployment: Arc<dyn Deployment>,
    desired: InstanceStatus,
}

impl Instance {
    pub(super) fn new(
        id: InstanceId,
        config: InstanceConfig,
        deployment: Arc<dyn Deployment>,
        desired: InstanceStatus,
    ) -> Self {
        Self {
            id,
            config,
            deployment,
            desired,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        self.desired = InstanceStatus::Running;
        match self.deployment.instance_status(self.id.clone()).await? {
            InstanceStatus::Running => Ok(()),
            _ => self.deployment.start_instance(self.id.clone()).await,
        }
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.desired = InstanceStatus::Stopped;
        match self.deployment.instance_status(self.id.clone()).await? {
            InstanceStatus::Stopped => Ok(()),
            _ => self.deployment.stop_instance(self.id.clone()).await,
        }
    }

    pub async fn delete(self) -> Result<(), (anyhow::Error, Self)> {
        self.deployment
            .delete_instance(self.id.clone())
            .await
            .map_err(|e| (e, self))
    }

    pub async fn ready(&mut self) -> Result<()> {
        // TODO: Check status, error handling
        self.deployment.ready_instance(self.id.clone()).await
    }

    pub async fn export(&self, path: &Path) -> Result<()> {
        self.deployment.export_instance(self.id.clone(), path).await
    }

    pub async fn status(&self) -> Result<InstanceStatus> {
        self.deployment.instance_status(self.id.clone()).await
    }

    pub async fn copy_from(&self, src: &Path, dst: &Path) -> Result<()> {
        self.deployment
            .copy_from_instance(self.id.clone(), src, dst)
            .await
    }

    pub async fn copy_to(&self, src: &Path, dst: &Path) -> Result<()> {
        self.deployment
            .copy_to_instance(self.id.clone(), src, dst)
            .await
    }

    pub async fn is_runnable(&self) -> Result<bool> {
        self.deployment.is_instance_runnable(self.id.clone()).await
    }

    pub async fn is_running(&self) -> Result<bool> {
        self.deployment.is_instance_running(self.id.clone()).await
    }
}
