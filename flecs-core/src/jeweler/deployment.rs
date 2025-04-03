use crate::jeweler::app::AppDeployment;
use crate::jeweler::instance::InstanceDeployment;
use crate::jeweler::network::NetworkDeployment;
use crate::jeweler::volume::VolumeDeployment;
use async_trait::async_trait;
use erased_serde::serialize_trait_object;
use std::fmt::{Debug, Formatter};

pub type DeploymentId = String;

#[async_trait]
pub trait Deployment:
    Send
    + Sync
    + AppDeployment
    + InstanceDeployment
    + NetworkDeployment
    + VolumeDeployment
    + erased_serde::Serialize
{
    fn id(&self) -> DeploymentId;
    fn is_default(&self) -> bool;
}

serialize_trait_object!(Deployment);

impl Debug for dyn Deployment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Deployment: {}", self.id())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::jeweler::app::{AppId, AppInfo, Token};
    use crate::jeweler::gem::instance::{InstanceId, InstanceStatus};
    use crate::jeweler::gem::manifest::{AppManifest, ConfigFile};
    use crate::jeweler::instance::Config;
    use crate::jeweler::instance::Logs;
    use crate::jeweler::network::{CreateNetworkError, Network, NetworkConfig, NetworkId};
    use crate::jeweler::volume::{Volume, VolumeId};
    use crate::quest::SyncQuest;
    use crate::Result;
    use mockall::mock;
    use serde::{Serialize, Serializer};
    use std::collections::HashMap;
    use std::net::Ipv4Addr;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    mock! {
        pub edDeployment {}
        #[async_trait]
        impl AppDeployment for edDeployment {
            async fn install_app(&self, quest: SyncQuest, manifest: Arc<AppManifest>, token: Option<Token>) -> Result<AppId>;
            async fn uninstall_app(&self, quest: SyncQuest, id: AppId) -> Result<()>;
            async fn app_info(&self, quest: SyncQuest, id: AppId) -> Result<AppInfo>;
            async fn copy_from_app_image(
                &self,
                quest: SyncQuest,
                image: String,
                src: &Path,
                dst: &Path,
                is_dst_file_path: bool,
            ) -> Result<()>;
            async fn export_app(&self, quest: SyncQuest, id: String, path: PathBuf) -> Result<()>;
        }
        #[async_trait]
        impl InstanceDeployment for edDeployment {
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
            async fn copy_configs_from_instance(
                &self,
                id: InstanceId,
                config_files: &[ConfigFile],
                dst: PathBuf,
            ) -> Result<()>;
        }
        #[async_trait]
        impl NetworkDeployment for edDeployment {
            async fn create_network(&self, quest: SyncQuest, config: NetworkConfig) -> Result<Network, CreateNetworkError>;
            async fn default_network(&self) -> Result<Network, CreateNetworkError>;
            async fn delete_network(&self, id: NetworkId) -> Result<()>;
            async fn network(&self, id: NetworkId) -> Result<Option<Network>>;
            async fn networks(&self, quest: SyncQuest) -> Result<Vec<Network>>;
            async fn connect_network(
                &self,
                quest: SyncQuest,
                id: NetworkId,
                address: Ipv4Addr,
                instance_id: InstanceId,
            ) -> Result<()>;
            async fn disconnect_network(
                &self,
                quest: SyncQuest,
                id: NetworkId,
                instance_id: InstanceId,
            ) -> Result<()>;
        }
        #[async_trait]
        impl VolumeDeployment for edDeployment {
            async fn create_volume(&self, quest: SyncQuest, name: &str) -> Result<VolumeId>;
            async fn delete_volume(&self, _quest: SyncQuest, id: VolumeId) -> Result<()>;
            async fn import_volume(
                &self,
                _quest: SyncQuest,
                path: &Path,
                name: &str,
                image: &str,
            ) -> Result<VolumeId>;
            async fn export_volume(
                &self,
                quest: SyncQuest,
                id: VolumeId,
                path: &Path,
                image: &str,
            ) -> Result<()>;
            async fn volumes(
                &self,
                quest: SyncQuest,
                instance_id: InstanceId,
            ) -> Result<HashMap<VolumeId, Volume>>;
            async fn export_volumes(
                &self,
                quest: SyncQuest,
                instance_id: InstanceId,
                path: &Path,
                image: &str,
            ) -> Result<()>;
        }
        #[async_trait]
        impl Deployment for edDeployment {
            fn id(&self) -> DeploymentId;
            fn is_default(&self) -> bool;
        }
    }

    impl Serialize for MockedDeployment {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.id())
        }
    }
}
