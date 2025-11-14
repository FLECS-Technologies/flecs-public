mod docker_impl;
use crate::jeweler;
use crate::jeweler::deployment::CommonDeployment;
use crate::jeweler::gem::instance::status::InstanceStatus;
use crate::jeweler::gem::instance::{InstanceId, Logs};
use crate::jeweler::gem::manifest::single::{AppManifestSingle, ConfigFile};
use crate::jeweler::network::{CreateNetworkError, NetworkId};
use crate::lore::{InstanceLoreRef, NetworkLoreRef};
use crate::quest::SyncQuest;
use async_trait::async_trait;
use bollard::container::Config;
pub use docker_impl::*;
use erased_serde::serialize_trait_object;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub type AppInfo = bollard::models::ImageInspect;
#[async_trait]
pub trait DockerDeployment: CommonDeployment {
    async fn create_default_network(
        &self,
        lore: NetworkLoreRef,
    ) -> crate::Result<jeweler::network::Network, CreateNetworkError>;

    async fn app_info(
        &self,
        _quest: SyncQuest,
        manifest: Arc<AppManifestSingle>,
    ) -> anyhow::Result<Option<AppInfo>>;

    async fn copy_from_app_image(
        &self,
        quest: SyncQuest,
        image: String,
        src: &Path,
        dst: &Path,
        is_dst_file_path: bool,
    ) -> anyhow::Result<()>;

    async fn connect_network(
        &self,
        _quest: SyncQuest,
        id: NetworkId,
        address: Ipv4Addr,
        instance_id: InstanceId,
    ) -> anyhow::Result<()>;

    async fn disconnect_network(
        &self,
        _quest: SyncQuest,
        id: NetworkId,
        instance_id: InstanceId,
    ) -> anyhow::Result<()>;

    async fn copy_from_instance(
        &self,
        quest: SyncQuest,
        id: InstanceId,
        src: &Path,
        dst: &Path,
        is_dst_file_path: bool,
    ) -> anyhow::Result<()>;

    async fn copy_to_instance(
        &self,
        quest: SyncQuest,
        id: InstanceId,
        src: &Path,
        dst: &Path,
        is_dst_file_path: bool,
    ) -> anyhow::Result<()>;

    async fn copy_configs_from_instance(
        &self,
        id: InstanceId,
        config_files: &[ConfigFile],
        dst: PathBuf,
    ) -> anyhow::Result<()>;

    async fn start_instance(
        &self,
        lore: InstanceLoreRef,
        config: Config<String>,
        id: Option<InstanceId>,
        config_files: &[ConfigFile],
    ) -> anyhow::Result<InstanceId>;

    async fn stop_instance(
        &self,
        id: InstanceId,
        lore: InstanceLoreRef,
        config_files: &[ConfigFile],
    ) -> anyhow::Result<()>;

    async fn delete_instance(&self, id: InstanceId) -> anyhow::Result<bool>;

    async fn instance_status(&self, id: InstanceId) -> anyhow::Result<InstanceStatus>;

    async fn instance_logs(&self, quest: SyncQuest, id: InstanceId) -> anyhow::Result<Logs>;
}

serialize_trait_object!(DockerDeployment);

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::Result;
    use crate::jeweler::GetDeploymentId;
    use crate::jeweler::app::AppDeployment;
    use crate::jeweler::app::Token;
    use crate::jeweler::deployment::{CommonDeployment, DeploymentId};
    use crate::jeweler::gem::deployment::Deployment;
    use crate::jeweler::gem::deployment::docker::DockerDeployment;
    use crate::jeweler::gem::instance::InstanceId;
    use crate::jeweler::gem::manifest::AppManifest;
    use crate::jeweler::gem::manifest::single::ConfigFile;
    use crate::jeweler::network::{
        CreateNetworkError, Network, NetworkConfig, NetworkDeployment, NetworkId,
    };
    use crate::jeweler::volume::Volume;
    use crate::jeweler::volume::VolumeDeployment;
    use crate::jeweler::volume::VolumeId;
    use crate::lore::{ExportLoreRef, ImportLoreRef};
    use crate::quest::SyncQuest;
    use mockall::mock;
    use serde::{Serialize, Serializer};
    use std::fmt::{Debug, Formatter};
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::Arc;

    mock! {
        pub edDockerDeployment {}
        #[async_trait]
        impl AppDeployment for edDockerDeployment {
            async fn install_app(
                &self,
                quest: SyncQuest,
                manifest: AppManifest,
                token: Option<Token>
            ) -> Result<()>;
            async fn uninstall_app(
                &self,
                quest: SyncQuest,
                manifest: AppManifest,
            ) -> Result<()>;
            async fn is_app_installed(
                &self,
                quest: SyncQuest,
                manifest: AppManifest,
            ) -> Result<bool>;
            async fn installed_app_size(
                &self,
                quest: SyncQuest,
                manifest: AppManifest,
            ) -> Result<usize>;
            async fn export_app(
                &self,
                quest: SyncQuest,
                lore: ExportLoreRef,
                manifest: AppManifest,
                path: PathBuf
            ) -> Result<()>;
            async fn import_app(
                &self,
                quest: SyncQuest,
                lore: ImportLoreRef,
                manifest: AppManifest,
                path: PathBuf
            ) -> Result<()>;
        }
        #[async_trait]
        impl NetworkDeployment for edDockerDeployment {
            async fn create_network(&self, quest: SyncQuest, config: NetworkConfig) -> Result<Network, CreateNetworkError>;
            async fn default_network(&self, lore: NetworkLoreRef) -> Result<Network, CreateNetworkError>;
            async fn delete_network(&self, id: NetworkId) -> Result<()>;
            async fn network(&self, id: NetworkId) -> Result<Option<Network>>;
            async fn networks(&self, quest: SyncQuest) -> Result<Vec<Network>>;
        }
        #[async_trait]
        impl VolumeDeployment for edDockerDeployment {
            async fn create_volume(&self, quest: SyncQuest, name: &str) -> Result<VolumeId>;
            async fn delete_volume(&self, _quest: SyncQuest, id: VolumeId) -> Result<()>;
            async fn import_volume(
                &self,
                _quest: SyncQuest,
                src: &Path,
                container_path: &Path,
                name: &str,
                image: &str,
            ) -> Result<VolumeId>;
            async fn export_volume(
                &self,
                quest: SyncQuest,
                id: VolumeId,
                export_path: &Path,
                container_path: &Path,
                image: &str,
            ) -> Result<()>;
            async fn inspect_volume(&self, id: VolumeId) -> Result<Option<Volume>>;
        }
        impl GetDeploymentId for edDockerDeployment {
            fn deployment_id(&self) -> &DeploymentId;
        }
        #[async_trait]
        impl CommonDeployment for edDockerDeployment {
            fn id(&self) -> &DeploymentId;
            fn is_default(&self) -> bool;
            async fn core_default_address(&self, lore: NetworkLoreRef) -> Option<IpAddr>;
        }
        #[async_trait]
        impl DockerDeployment for edDockerDeployment {
            async fn create_default_network(
                &self,
                lore: NetworkLoreRef,
            ) -> crate::Result<jeweler::network::Network, CreateNetworkError>;
            async fn app_info(
                &self,
                _quest: SyncQuest,
                manifest: Arc<AppManifestSingle>
            ) -> anyhow::Result<Option<AppInfo>>;
            async fn copy_from_app_image(
                &self,
                quest: SyncQuest,
                image: String,
                src: &Path,
                dst: &Path,
                is_dst_file_path: bool,
            ) -> anyhow::Result<()>;
            async fn connect_network(
                &self,
                _quest: SyncQuest,
                id: NetworkId,
                address: Ipv4Addr,
                instance_id: InstanceId,
            ) -> anyhow::Result<()>;
            async fn disconnect_network(
                &self,
                _quest: SyncQuest,
                id: NetworkId,
                instance_id: InstanceId,
            ) -> anyhow::Result<()>;
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
            async fn start_instance(
                &self,
                lore: InstanceLoreRef,
                config: Config<String>,
                id: Option<InstanceId>,
                config_files: &[ConfigFile],
            ) -> Result<InstanceId>;
            async fn stop_instance(
                &self, id: InstanceId,
                lore:
                InstanceLoreRef,
                config_files: &[ConfigFile]
            ) -> Result<()>;
            async fn delete_instance(&self, id: InstanceId) -> Result<bool>;
            async fn instance_status(&self, id: InstanceId) -> Result<InstanceStatus>;
            async fn instance_logs(&self, quest: SyncQuest, id: InstanceId) -> Result<Logs>;
        }
    }

    impl Debug for MockedDockerDeployment {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "MockedDockerDeployment")
        }
    }

    impl Serialize for MockedDockerDeployment {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(self.id())
        }
    }

    const TEST_DEPLOYMENT_ID: &str = "some-deployment-id";
    const TEST_DEPLOYMENT_SOCK_PATH: &str = "/path/to/docker.sock";

    #[test]
    fn deployment_id() {
        let deployment = Deployment::Docker(Arc::new(DockerDeploymentImpl::new(
            TEST_DEPLOYMENT_ID.to_string(),
            PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
        )));
        assert_eq!(deployment.id(), TEST_DEPLOYMENT_ID);
    }
}
