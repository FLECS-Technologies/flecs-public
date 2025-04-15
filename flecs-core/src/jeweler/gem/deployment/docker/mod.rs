mod docker_impl;
use crate::jeweler::app::{AppId, AppInfo};
use crate::jeweler::deployment::CommonDeployment;
use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::manifest::single::ConfigFile;
use crate::jeweler::network::{CreateNetworkError, NetworkConfig, NetworkId, NetworkKind};
use crate::quest::SyncQuest;
use crate::{jeweler, relic};
use async_trait::async_trait;
use bollard::container::Config;
pub use docker_impl::*;
use erased_serde::serialize_trait_object;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

#[async_trait]
pub trait DockerDeployment: CommonDeployment {
    fn default_network_name(&self) -> &'static str {
        "flecs"
    }

    fn default_cidr_subnet(&self) -> relic::network::Ipv4Network {
        Default::default()
    }

    fn default_gateway(&self) -> Ipv4Addr {
        Ipv4Addr::new(172, 21, 0, 1)
    }

    fn default_network_config(&self) -> NetworkConfig {
        NetworkConfig {
            kind: NetworkKind::Bridge,
            name: self.default_network_name().to_string(),
            cidr_subnet: Some(self.default_cidr_subnet()),
            gateway: Some(self.default_gateway()),
            parent_adapter: None,
            options: Default::default(),
        }
    }

    async fn create_default_network(
        &self,
    ) -> crate::Result<jeweler::network::Network, CreateNetworkError>;

    async fn app_info(&self, _quest: SyncQuest, id: AppId) -> anyhow::Result<AppInfo>;

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
        config: Config<String>,
        id: Option<InstanceId>,
        config_files: &[ConfigFile],
    ) -> anyhow::Result<InstanceId>;

    async fn stop_instance(
        &self,
        id: InstanceId,
        config_files: &[ConfigFile],
    ) -> anyhow::Result<()>;
}

serialize_trait_object!(DockerDeployment);

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::Result;
    use crate::jeweler::GetDeploymentId;
    use crate::jeweler::app::AppDeployment;
    use crate::jeweler::app::{AppId, Token};
    use crate::jeweler::deployment::{CommonDeployment, DeploymentId};
    use crate::jeweler::gem::deployment::Deployment;
    use crate::jeweler::gem::deployment::docker::DockerDeployment;
    use crate::jeweler::gem::instance::InstanceId;
    use crate::jeweler::gem::instance::status::InstanceStatus;
    use crate::jeweler::gem::manifest::AppManifest;
    use crate::jeweler::gem::manifest::single::ConfigFile;
    use crate::jeweler::instance::{InstanceDeployment, Logs};
    use crate::jeweler::network::{
        CreateNetworkError, Network, NetworkConfig, NetworkDeployment, NetworkId, NetworkKind,
    };
    use crate::jeweler::volume::VolumeDeployment;
    use crate::jeweler::volume::{Volume, VolumeId};
    use crate::quest::SyncQuest;
    use crate::relic::network::Ipv4Network;
    use mockall::mock;
    use serde::{Serialize, Serializer};
    use std::collections::HashMap;
    use std::fmt::{Debug, Formatter};
    use std::net::Ipv4Addr;
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
            ) -> Result<AppId>;
            async fn uninstall_app(
                &self,
                quest: SyncQuest,
                manifest: AppManifest,
                id: AppId
            ) -> Result<()>;
            async fn is_app_installed(
                &self,
                quest: SyncQuest,
                manifest: AppManifest,
                id: AppId,
            ) -> Result<bool>;
            async fn installed_app_size(
                &self,
                quest: SyncQuest,
                manifest: AppManifest,
                id: AppId,
            ) -> Result<usize>;
            async fn export_app(
                &self,
                quest: SyncQuest,
                manifest: AppManifest,
                path: PathBuf
            ) -> Result<()>;
            async fn import_app(
                &self,
                quest: SyncQuest,
                manifest: AppManifest,
                path: PathBuf
            ) -> Result<()>;
        }
        #[async_trait]
        impl InstanceDeployment for edDockerDeployment {
            async fn delete_instance(&self, id: InstanceId) -> Result<bool>;
            async fn instance_status(&self, id: InstanceId) -> Result<InstanceStatus>;
            async fn instance_logs(&self, quest: SyncQuest, id: InstanceId) -> Result<Logs>;
        }
        #[async_trait]
        impl NetworkDeployment for edDockerDeployment {
            async fn create_network(&self, quest: SyncQuest, config: NetworkConfig) -> Result<Network, CreateNetworkError>;
            async fn default_network(&self) -> Result<Network, CreateNetworkError>;
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
        impl GetDeploymentId for edDockerDeployment {
            fn deployment_id(&self) -> &DeploymentId;
        }
        #[async_trait]
        impl CommonDeployment for edDockerDeployment {
            fn id(&self) -> &DeploymentId;
            fn is_default(&self) -> bool;
        }
        #[async_trait]
        impl DockerDeployment for edDockerDeployment {
            fn default_network_name(&self) -> &'static str;
            fn default_cidr_subnet(&self) -> relic::network::Ipv4Network;
            fn default_gateway(&self) -> Ipv4Addr;
            fn default_network_config(&self) -> NetworkConfig;
            async fn create_default_network(
                &self,
            ) -> crate::Result<jeweler::network::Network, CreateNetworkError>;
            async fn app_info(&self, _quest: SyncQuest, id: AppId) -> anyhow::Result<AppInfo>;
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
                config: Config<String>,
                id: Option<InstanceId>,
                config_files: &[ConfigFile],
            ) -> Result<InstanceId>;
            async fn stop_instance(&self, id: InstanceId, config_files: &[ConfigFile]) -> Result<()>;
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

    #[test]
    fn default_network_config() {
        let deployment = DockerDeploymentImpl::default();
        let config = deployment.default_network_config();
        assert_eq!(config.name, deployment.default_network_name());
        assert_eq!(config.cidr_subnet, Some(deployment.default_cidr_subnet()));
        assert_eq!(config.gateway, Some(deployment.default_gateway()));
        assert_eq!(config.kind, NetworkKind::Bridge);
        assert_eq!(config.parent_adapter, None);
    }

    #[test]
    fn default_network_name() {
        let deployment = DockerDeploymentImpl::default();
        assert_eq!(deployment.default_network_name(), "flecs");
    }

    #[test]
    fn default_network_gateway() {
        let deployment = DockerDeploymentImpl::default();
        assert_eq!(deployment.default_gateway(), Ipv4Addr::new(172, 21, 0, 1));
    }

    #[test]
    fn default_network_cidr_subnet() {
        let deployment = DockerDeploymentImpl::default();
        assert_eq!(
            deployment.default_cidr_subnet(),
            Ipv4Network::try_new(Ipv4Addr::new(172, 21, 0, 0), 16).unwrap()
        );
    }
}
