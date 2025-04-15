use crate::jeweler::GetDeploymentId;
use crate::jeweler::app::{AppDeployment, AppId, Token};
use crate::jeweler::deployment::CommonDeployment;
use crate::jeweler::gem::deployment::compose::ComposeDeployment;
use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::instance::status::InstanceStatus;
use crate::jeweler::gem::manifest::AppManifest;
use crate::jeweler::instance::{InstanceDeployment, Logs};
use crate::jeweler::network::{
    CreateNetworkError, Network, NetworkConfig, NetworkDeployment, NetworkId,
};
use crate::jeweler::volume::{Volume, VolumeDeployment, VolumeId};
use crate::quest::SyncQuest;
use crate::vault::pouch::deployment::DeploymentId;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct ComposeDeploymentImpl {
    pub id: DeploymentId,
    path: PathBuf,
    #[serde(default)]
    is_default: bool,
}

impl GetDeploymentId for ComposeDeploymentImpl {
    fn deployment_id(&self) -> &DeploymentId {
        &self.id
    }
}

impl ComposeDeploymentImpl {
    pub fn is_default(&self) -> bool {
        self.is_default
    }
}

impl Default for ComposeDeploymentImpl {
    fn default() -> Self {
        Self {
            path: PathBuf::from("/var/run/docker.sock"),
            id: "DefaultComposeDeployment".to_string(),
            is_default: true,
        }
    }
}

impl ComposeDeployment for ComposeDeploymentImpl {
    fn dummy(&self) {
        todo!()
    }
}

impl CommonDeployment for ComposeDeploymentImpl {
    fn id(&self) -> &crate::jeweler::deployment::DeploymentId {
        &self.id
    }

    fn is_default(&self) -> bool {
        self.is_default
    }
}

#[async_trait]
impl AppDeployment for ComposeDeploymentImpl {
    async fn install_app(
        &self,
        _quest: SyncQuest,
        _manifest: AppManifest,
        _token: Option<Token>,
    ) -> anyhow::Result<AppId> {
        todo!()
    }

    async fn uninstall_app(
        &self,
        _quest: SyncQuest,
        _manifest: AppManifest,
        _id: AppId,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn is_app_installed(
        &self,
        _quest: SyncQuest,
        _manifest: AppManifest,
        _id: AppId,
    ) -> anyhow::Result<bool> {
        todo!()
    }

    async fn installed_app_size(
        &self,
        _quest: SyncQuest,
        _manifest: AppManifest,
        _id: AppId,
    ) -> anyhow::Result<usize> {
        todo!()
    }

    async fn export_app(
        &self,
        _quest: SyncQuest,
        _manifest: AppManifest,
        _path: PathBuf,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn import_app(
        &self,
        _quest: SyncQuest,
        _manifest: AppManifest,
        _path: PathBuf,
    ) -> anyhow::Result<()> {
        todo!()
    }
}

#[async_trait]
impl InstanceDeployment for ComposeDeploymentImpl {
    async fn delete_instance(&self, _id: InstanceId) -> anyhow::Result<bool> {
        todo!()
    }

    async fn instance_status(&self, _id: InstanceId) -> anyhow::Result<InstanceStatus> {
        todo!()
    }

    async fn instance_logs(&self, _quest: SyncQuest, _id: InstanceId) -> anyhow::Result<Logs> {
        todo!()
    }
}

#[async_trait]
impl NetworkDeployment for ComposeDeploymentImpl {
    async fn create_network(
        &self,
        _quest: SyncQuest,
        _config: NetworkConfig,
    ) -> anyhow::Result<Network, CreateNetworkError> {
        todo!()
    }

    async fn default_network(&self) -> anyhow::Result<Network, CreateNetworkError> {
        todo!()
    }

    async fn delete_network(&self, _id: NetworkId) -> anyhow::Result<()> {
        todo!()
    }

    async fn network(&self, _id: NetworkId) -> anyhow::Result<Option<Network>> {
        todo!()
    }

    async fn networks(&self, _quest: SyncQuest) -> anyhow::Result<Vec<Network>> {
        todo!()
    }
}

#[async_trait]
impl VolumeDeployment for ComposeDeploymentImpl {
    async fn create_volume(&self, _quest: SyncQuest, _name: &str) -> anyhow::Result<VolumeId> {
        todo!()
    }

    async fn delete_volume(&self, _quest: SyncQuest, _id: VolumeId) -> anyhow::Result<()> {
        todo!()
    }

    async fn import_volume(
        &self,
        _quest: SyncQuest,
        _src: &Path,
        _container_path: &Path,
        _name: &str,
        _image: &str,
    ) -> anyhow::Result<VolumeId> {
        todo!()
    }

    async fn export_volume(
        &self,
        _quest: SyncQuest,
        _id: VolumeId,
        _export_path: &Path,
        _container_path: &Path,
        _image: &str,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn volumes(
        &self,
        _quest: SyncQuest,
        _instance_id: InstanceId,
    ) -> anyhow::Result<HashMap<VolumeId, Volume>> {
        todo!()
    }

    async fn export_volumes(
        &self,
        _quest: SyncQuest,
        _instance_id: InstanceId,
        _path: &Path,
        _image: &str,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
