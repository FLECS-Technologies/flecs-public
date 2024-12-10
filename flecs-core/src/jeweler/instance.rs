use super::Result;
use crate::jeweler::gem::instance::{InstanceId, InstanceStatus};
use crate::quest::SyncQuest;
use async_trait::async_trait;
// TODO: Use more generic struct as soon as the second type of deployment is implemented
pub use bollard::container::Config;
use std::path::Path;

// TODO: Take Quest as parameter, create subquests
#[async_trait]
pub trait InstanceDeployment {
    async fn delete_instance(&self, id: InstanceId) -> Result<bool>;
    async fn start_instance(
        &self,
        config: Config<String>,
        id: Option<InstanceId>,
    ) -> Result<InstanceId>;
    async fn stop_instance(&self, id: InstanceId) -> Result<()>;
    async fn instance_status(&self, id: InstanceId) -> Result<InstanceStatus>;
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
    ) -> Result<()>;
    // TODO: Maybe move function to enum InstanceStatus
    async fn is_instance_runnable(&self, id: InstanceId) -> Result<bool> {
        Ok(self.instance_status(id).await? == InstanceStatus::Created)
    }
    // TODO: Maybe move function to enum InstanceStatus
    async fn is_instance_running(&self, id: InstanceId) -> Result<bool> {
        Ok(self.instance_status(id).await? == InstanceStatus::Running)
    }
}
