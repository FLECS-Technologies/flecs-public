pub use super::Result;
use crate::quest::SyncQuest;
use async_trait::async_trait;
use flecs_app_manifest::AppManifest;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
pub(crate) type AppId = String;
#[async_trait]
pub trait AppDeployment {
    async fn install_app(
        &self,
        quest: SyncQuest,
        manifest: Arc<AppManifest>,
        username: String,
        password: String,
    ) -> Result<AppId>;
    async fn uninstall_app(&self, quest: SyncQuest, id: AppId) -> Result<()>;
    async fn is_app_installed(&self, quest: SyncQuest, id: AppId) -> Result<bool>;
}

#[derive(Debug, Default, Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
pub enum AppStatus {
    #[default]
    None,
    Installed,
    NotInstalled,
}
