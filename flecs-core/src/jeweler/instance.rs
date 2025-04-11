use super::Result;
use crate::jeweler::gem::instance::status::InstanceStatus;
use crate::jeweler::gem::instance::InstanceId;
use crate::quest::SyncQuest;
use async_trait::async_trait;

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
    async fn instance_status(&self, id: InstanceId) -> Result<InstanceStatus>;
    async fn instance_logs(&self, quest: SyncQuest, id: InstanceId) -> Result<Logs>;

    async fn is_instance_running(&self, id: InstanceId) -> Result<bool> {
        Ok(self.instance_status(id).await? == InstanceStatus::Running)
    }
}
