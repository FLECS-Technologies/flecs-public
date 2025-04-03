use super::Result;
use crate::jeweler::gem::instance::{InstanceId, InstanceStatus};
use crate::jeweler::gem::manifest::ConfigFile;
use crate::quest::SyncQuest;
use async_trait::async_trait;
// TODO: Use more generic struct as soon as the second type of deployment is implemented
pub use bollard::container::Config;
use std::path::{Path, PathBuf};

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

// TODO: Take Quest as parameter, create subquests
#[async_trait]
pub trait InstanceDeployment {
    async fn delete_instance(&self, id: InstanceId) -> Result<bool>;
    async fn start_instance(
        &self,
        config: Config<String>,
        id: Option<InstanceId>,
        config_files: &[ConfigFile],
    ) -> Result<InstanceId>;
    async fn stop_instance(&self, id: InstanceId, config_files: &[ConfigFile]) -> Result<()>;
    async fn instance_status(&self, id: InstanceId) -> Result<InstanceStatus>;
    async fn instance_logs(&self, quest: SyncQuest, id: InstanceId) -> Result<Logs>;
    async fn copy_from_instance(
        &self,
        quest: SyncQuest,
        id: InstanceId,
        src: &Path,
        dst: &Path,
        is_dst_file_path: bool,
    ) -> Result<()>;
    async fn copy_to_instance(
        &self,
        quest: SyncQuest,
        id: InstanceId,
        src: &Path,
        dst: &Path,
        is_dst_file_path: bool,
    ) -> Result<()>;
    // TODO: Maybe move function to enum InstanceStatus
    async fn is_instance_runnable(&self, id: InstanceId) -> Result<bool> {
        Ok(self.instance_status(id).await? == InstanceStatus::Stopped)
    }
    // TODO: Maybe move function to enum InstanceStatus
    async fn is_instance_running(&self, id: InstanceId) -> Result<bool> {
        Ok(self.instance_status(id).await? == InstanceStatus::Running)
    }
    async fn copy_configs_from_instance(
        &self,
        id: InstanceId,
        config_files: &[ConfigFile],
        dst: PathBuf,
    ) -> Result<()>;
}
