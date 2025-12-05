use crate::forge::bollard::BollardNetworkExtension;
use crate::jeweler::GetDeploymentId;
use crate::jeweler::app::{AppDeployment, AppId, Token};
use crate::jeweler::deployment::CommonDeployment;
use crate::jeweler::gem::deployment::compose::ComposeDeployment;
use crate::jeweler::gem::deployment::docker::DockerDeploymentImpl;
use crate::jeweler::gem::instance::Logs;
use crate::jeweler::gem::instance::status::InstanceStatus;
use crate::jeweler::gem::manifest::AppManifest;
use crate::jeweler::gem::manifest::multi::AppManifestMulti;
use crate::jeweler::network::{
    CreateNetworkError, InspectNetworkError, Network, NetworkConfig, NetworkDeployment, NetworkId,
};
use crate::jeweler::volume::{Volume, VolumeDeployment, VolumeId};
use crate::lore::{ExportLoreRef, ImportLoreRef, NetworkLoreRef};
use crate::quest::SyncQuest;
use crate::relic;
use crate::relic::docker_cli::{DockerCli, ExecuteCommandError};
use crate::relic::network::{NetworkAdapterReader, NetworkAdapterReaderImpl};
use crate::vault::pouch::deployment::DeploymentId;
use async_trait::async_trait;
use bollard::image::{ImportImageOptions, RemoveImageOptions};
use bollard::models::{ContainerInspectResponse, ContainerState};
use bollard::{API_DEFAULT_VERSION, Docker};
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct ComposeDeploymentImpl {
    pub id: DeploymentId,
    docker_socket_path: PathBuf,
    #[serde(default)]
    is_default: bool,
    #[serde(skip, default = "default_network_adapter_reader")]
    network_adapter_reader: Box<dyn NetworkAdapterReader>,
}

fn default_network_adapter_reader() -> Box<dyn NetworkAdapterReader> {
    Box::new(NetworkAdapterReaderImpl)
}

impl GetDeploymentId for ComposeDeploymentImpl {
    fn deployment_id(&self) -> &DeploymentId {
        &self.id
    }
}

impl std::fmt::Debug for ComposeDeploymentImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct ComposeDeploymentImpl<'a> {
            id: &'a DeploymentId,
            docker_socket_path: &'a PathBuf,
            is_default: &'a bool,
        }

        let Self {
            id,
            docker_socket_path,
            is_default,
            network_adapter_reader: _,
        } = self;
        std::fmt::Debug::fmt(
            &ComposeDeploymentImpl {
                id,
                docker_socket_path,
                is_default,
            },
            f,
        )
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ExecuteCompose {
    #[error("Compose is not valid: {0}")]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    CommandExecute(#[from] ExecuteCommandError),
}

impl ComposeDeploymentImpl {
    fn docker_client(&self) -> anyhow::Result<Arc<Docker>> {
        self.docker_client_with_timeout(Duration::from_secs(120))
    }

    fn docker_client_with_timeout(&self, timeout: Duration) -> anyhow::Result<Arc<Docker>> {
        Ok(Arc::new(Docker::connect_with_unix(
            &self.docker_socket_path.to_string_lossy(),
            timeout.as_secs(),
            API_DEFAULT_VERSION,
        )?))
    }

    fn docker_cli(&self) -> DockerCli {
        DockerCli::new_with_unix_socket(self.docker_socket_path.clone())
    }

    async fn docker_login(&self, token: Token) -> Result<(), ExecuteCommandError> {
        self.docker_cli().login(token).await
    }

    async fn docker_logout(&self) -> Result<(), ExecuteCommandError> {
        self.docker_cli().logout().await
    }

    async fn compose_pull(&self, manifest: &AppManifestMulti) -> Result<AppId, ExecuteCompose> {
        let compose = manifest.compose_json()?;
        let project_name = manifest.project_name();
        self.docker_cli()
            .compose_pull(&project_name, &compose)
            .await?;
        Ok(project_name)
    }

    async fn compose_up(
        &self,
        manifest: &AppManifestMulti,
        workdir: &Path,
    ) -> Result<AppId, ExecuteCompose> {
        let compose = manifest.compose_json()?;
        let project_name = manifest.project_name();
        self.docker_cli()
            .compose_up(&project_name, workdir, &compose)
            .await?;
        Ok(project_name)
    }

    async fn compose_stop(&self, manifest: &AppManifestMulti) -> Result<AppId, ExecuteCompose> {
        let compose = manifest.compose_json()?;
        let project_name = manifest.project_name();
        self.docker_cli()
            .compose_stop(&project_name, &compose)
            .await?;
        Ok(project_name)
    }

    async fn compose_remove(&self, manifest: &AppManifestMulti) -> Result<AppId, ExecuteCompose> {
        let compose = manifest.compose_json()?;
        let project_name = manifest.project_name();
        self.docker_cli()
            .compose_remove(&project_name, &compose)
            .await?;
        Ok(project_name)
    }

    async fn compose_container(
        &self,
        manifest: &AppManifestMulti,
    ) -> Result<Vec<String>, ExecuteCompose> {
        let compose = manifest.compose_json()?;
        let project_name = manifest.project_name();
        Ok(self
            .docker_cli()
            .compose_containers(&project_name, &compose)
            .await?)
    }

    async fn compose_logs(&self, manifest: &AppManifestMulti) -> Result<Logs, ExecuteCompose> {
        let compose = manifest.compose_json()?;
        let project_name = manifest.project_name();
        let logs = self
            .docker_cli()
            .compose_logs(&project_name, &compose)
            .await?;
        Ok(Logs {
            stdout: logs,
            stderr: String::new(),
        })
    }
}

impl Default for ComposeDeploymentImpl {
    fn default() -> Self {
        Self {
            docker_socket_path: PathBuf::from("/var/run/docker.sock"),
            id: "DefaultComposeDeployment".to_string(),
            is_default: true,
            network_adapter_reader: default_network_adapter_reader(),
        }
    }
}

#[async_trait]
impl ComposeDeployment for ComposeDeploymentImpl {
    async fn start_instance(
        &self,
        manifest: &AppManifestMulti,
        workdir: &Path,
    ) -> Result<(), ExecuteCompose> {
        self.compose_up(manifest, workdir).await?;
        Ok(())
    }

    async fn stop_instance(&self, manifest: &AppManifestMulti) -> Result<(), ExecuteCompose> {
        self.compose_stop(manifest).await?;
        self.compose_remove(manifest).await?;
        Ok(())
    }

    async fn instance_status(
        &self,
        manifest: &AppManifestMulti,
    ) -> anyhow::Result<Vec<InstanceStatus>> {
        let containers = self.compose_container(manifest).await?;
        let mut status_vec = Vec::with_capacity(containers.len());
        for container in containers {
            let docker_client = self.docker_client()?;
            let status = match relic::docker::container::inspect(docker_client, &container).await? {
                None => InstanceStatus::Stopped,
                Some(ContainerInspectResponse {
                    state:
                        Some(ContainerState {
                            status: Some(state),
                            ..
                        }),
                    ..
                }) => state.into(),
                _ => InstanceStatus::Unknown,
            };
            status_vec.push(status);
        }
        Ok(status_vec)
    }

    async fn instance_logs(&self, manifest: &AppManifestMulti) -> anyhow::Result<Logs> {
        Ok(self.compose_logs(manifest).await?)
    }
}

#[async_trait]
impl CommonDeployment for ComposeDeploymentImpl {
    fn id(&self) -> &crate::jeweler::deployment::DeploymentId {
        &self.id
    }

    fn is_default(&self) -> bool {
        self.is_default
    }

    async fn core_default_address(&self, lore: NetworkLoreRef) -> Option<IpAddr> {
        self.default_network(lore)
            .await
            .ok()?
            .gateways()
            .ok()?
            .first()
            .copied()
    }
}

#[async_trait]
impl AppDeployment for ComposeDeploymentImpl {
    async fn install_app(
        &self,
        _quest: SyncQuest,
        manifest: AppManifest,
        token: Option<Token>,
    ) -> anyhow::Result<()> {
        let AppManifest::Multi(manifest) = manifest else {
            panic!("Compose deployment can not be called with single app manifests");
        };
        let logout_needed = token.is_some();
        if let Some(token) = token {
            self.docker_login(token).await?;
        }
        let pull_result = self.compose_pull(&manifest).await;
        if logout_needed {
            self.docker_logout().await?;
        }
        pull_result?;
        Ok(())
    }

    async fn uninstall_app(&self, _quest: SyncQuest, manifest: AppManifest) -> anyhow::Result<()> {
        let AppManifest::Multi(manifest) = manifest else {
            panic!("Compose deployment can not be called with single app manifests");
        };
        let docker_client = self.docker_client()?;
        for image in &manifest.images() {
            // TODO: Subquests
            crate::relic::docker::image::remove(
                docker_client.clone(),
                image,
                Some(RemoveImageOptions {
                    force: true,
                    noprune: false,
                }),
                None,
            )
            .await?;
        }
        Ok(())
    }

    async fn is_app_installed(
        &self,
        _quest: SyncQuest,
        manifest: AppManifest,
    ) -> anyhow::Result<bool> {
        let AppManifest::Multi(manifest) = manifest else {
            panic!("Compose deployment can not be called with single app manifests");
        };
        let docker_client = self.docker_client()?;
        for image in &manifest.images() {
            // TODO: Subquests
            if crate::relic::docker::image::inspect(docker_client.clone(), image)
                .await?
                .is_none()
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn installed_app_size(
        &self,
        _quest: SyncQuest,
        manifest: AppManifest,
    ) -> anyhow::Result<usize> {
        let AppManifest::Multi(manifest) = manifest else {
            panic!("Compose deployment can not be called with single app manifests");
        };
        let docker_client = self.docker_client()?;
        let mut size = 0;
        for image in &manifest.images() {
            // TODO: Subquests
            match crate::relic::docker::image::inspect(docker_client.clone(), image).await? {
                None => anyhow::bail!("App not installed"),
                Some(info) => {
                    size += info
                        .size
                        .ok_or_else(|| anyhow::anyhow!("Size was not specified"))?
                        as usize;
                }
            }
        }
        Ok(size)
    }

    async fn export_app(
        &self,
        quest: SyncQuest,
        lore: ExportLoreRef,
        manifest: AppManifest,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        let AppManifest::Multi(manifest) = manifest else {
            anyhow::bail!("ComposeDeploymentImpl supports only AppManifest::Multi");
        };
        let mut results = Vec::new();
        let client = self.docker_client_with_timeout(lore.as_ref().as_ref().timeout)?;
        for service in manifest.services_with_image_info() {
            let path = path.join(format!(
                "{}_{}.{}.tar",
                manifest.key.name, manifest.key.version, service.name
            ));
            let client = client.clone();
            let result = quest
                .lock()
                .await
                .create_sub_quest(
                    format!(
                        "Export {} of service {} to {path:?}",
                        service.image, service.name
                    ),
                    |quest| async move {
                        relic::docker::image::save(quest, client, &path, &service.image_with_repo)
                            .await
                    },
                )
                .await
                .2;
            results.push(result);
        }
        for result in join_all(results).await {
            result?;
        }
        Ok(())
    }

    async fn import_app(
        &self,
        quest: SyncQuest,
        lore: ImportLoreRef,
        manifest: AppManifest,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        let AppManifest::Multi(manifest) = manifest else {
            anyhow::bail!("ComposeDeploymentImpl supports only AppManifest::Multi");
        };
        let mut results = Vec::new();
        let client = self.docker_client_with_timeout(lore.as_ref().as_ref().timeout)?;
        for service in manifest.services_with_image_info() {
            let path = {
                // Current format of path
                let image_path = path.join(format!(
                    "{}_{}.{}.tar",
                    manifest.key.name, manifest.key.version, service.name
                ));
                if let Ok(true) = tokio::fs::try_exists(&image_path).await {
                    image_path
                } else {
                    // Legacy format of path
                    path.join(format!("{}.tar", service.image_with_repo.replace('/', "_")))
                }
            };
            let client = client.clone();
            let result = quest
                .lock()
                .await
                .create_sub_quest(
                    format!(
                        "Import {} for service {} from {path:?}",
                        service.image, service.name
                    ),
                    |quest| async move {
                        relic::docker::image::load(
                            quest,
                            client,
                            &path,
                            ImportImageOptions::default(),
                            None,
                        )
                        .await
                    },
                )
                .await
                .2;
            results.push(result);
        }
        for result in join_all(results).await {
            result?;
        }
        Ok(())
    }
}

#[async_trait]
impl NetworkDeployment for ComposeDeploymentImpl {
    async fn create_network(
        &self,
        quest: SyncQuest,
        config: NetworkConfig,
    ) -> anyhow::Result<Network, CreateNetworkError> {
        let docker_client = self.docker_client()?;
        DockerDeploymentImpl::create_network_with_client(
            docker_client,
            quest,
            config,
            self.network_adapter_reader.as_ref(),
        )
        .await
    }

    async fn default_network(
        &self,
        lore: NetworkLoreRef,
    ) -> anyhow::Result<Network, InspectNetworkError> {
        let docker_client = self.docker_client()?;
        DockerDeploymentImpl::default_network_with_client(docker_client, lore).await
    }

    async fn delete_network(&self, id: NetworkId) -> anyhow::Result<()> {
        let docker_client = self.docker_client()?;
        relic::docker::network::remove(docker_client, &id).await
    }

    async fn network(&self, id: NetworkId) -> anyhow::Result<Option<Network>> {
        let docker_client = self.docker_client()?;
        relic::docker::network::inspect::<&str>(docker_client, &id, None).await
    }

    async fn networks(&self, _quest: SyncQuest) -> anyhow::Result<Vec<Network>> {
        let docker_client = self.docker_client()?;
        relic::docker::network::list::<String>(docker_client, None).await
    }
}

#[async_trait]
impl VolumeDeployment for ComposeDeploymentImpl {
    async fn create_volume(&self, quest: SyncQuest, name: &str) -> anyhow::Result<VolumeId> {
        let client = self.docker_client()?;
        DockerDeploymentImpl::create_volume_with_client(client, quest, name).await
    }

    async fn delete_volume(&self, _quest: SyncQuest, id: VolumeId) -> anyhow::Result<()> {
        let client = self.docker_client()?;
        relic::docker::volume::remove(client, None, &id).await
    }

    async fn import_volume(
        &self,
        quest: SyncQuest,
        src: &Path,
        container_path: &Path,
        name: &str,
        image: &str,
    ) -> anyhow::Result<VolumeId> {
        let client = self.docker_client()?;
        DockerDeploymentImpl::import_volume_with_client(
            client,
            quest,
            src,
            container_path,
            name,
            image,
        )
        .await
    }

    async fn export_volume(
        &self,
        quest: SyncQuest,
        id: VolumeId,
        export_path: &Path,
        container_path: &Path,
        image: &str,
    ) -> anyhow::Result<()> {
        let client = self.docker_client()?;
        DockerDeploymentImpl::export_volume_with_client(
            client,
            quest,
            id,
            export_path,
            container_path,
            image,
        )
        .await
    }

    async fn inspect_volume(&self, id: VolumeId) -> anyhow::Result<Option<Volume>> {
        let client = self.docker_client()?;
        DockerDeploymentImpl::inspect_volume_with_client(client, id).await
    }
}
