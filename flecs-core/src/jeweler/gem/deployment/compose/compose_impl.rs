use crate::jeweler::GetDeploymentId;
use crate::jeweler::app::{AppDeployment, AppId, Token};
use crate::jeweler::deployment::CommonDeployment;
use crate::jeweler::gem::deployment::compose::ComposeDeployment;
use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::instance::Logs;
use crate::jeweler::gem::instance::status::InstanceStatus;
use crate::jeweler::gem::manifest::AppManifest;
use crate::jeweler::gem::manifest::multi::AppManifestMulti;
use crate::jeweler::network::{
    CreateNetworkError, Network, NetworkConfig, NetworkDeployment, NetworkId,
};
use crate::jeweler::volume::{Volume, VolumeDeployment, VolumeId};
use crate::quest::SyncQuest;
use crate::relic;
use crate::relic::docker_cli::{DockerCli, ExecuteCommandError};
use crate::vault::pouch::deployment::DeploymentId;
use async_trait::async_trait;
use bollard::image::RemoveImageOptions;
use bollard::models::{ContainerInspectResponse, ContainerState};
use bollard::{API_DEFAULT_VERSION, Docker};
use docker_compose_types::Service;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct ComposeDeploymentImpl {
    pub id: DeploymentId,
    docker_socket_path: PathBuf,
    #[serde(default)]
    is_default: bool,
}

impl GetDeploymentId for ComposeDeploymentImpl {
    fn deployment_id(&self) -> &DeploymentId {
        &self.id
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
        Ok(Arc::new(Docker::connect_with_unix(
            &self.docker_socket_path.to_string_lossy(),
            120,
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
        manifest: AppManifest,
        token: Option<Token>,
    ) -> anyhow::Result<AppId> {
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
        Ok(pull_result?)
    }

    async fn uninstall_app(
        &self,
        _quest: SyncQuest,
        manifest: AppManifest,
        _id: AppId,
    ) -> anyhow::Result<()> {
        let AppManifest::Multi(manifest) = manifest else {
            panic!("Compose deployment can not be called with single app manifests");
        };
        let docker_client = self.docker_client()?;
        for (_, service) in &manifest.compose.services.0 {
            // TODO: Subquests
            if let Some(Service {
                image: Some(image), ..
            }) = service
            {
                // TODO: Check if Docker::delete_service can be used
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
        }
        Ok(())
    }

    async fn is_app_installed(
        &self,
        _quest: SyncQuest,
        manifest: AppManifest,
        _id: AppId,
    ) -> anyhow::Result<bool> {
        let AppManifest::Multi(manifest) = manifest else {
            panic!("Compose deployment can not be called with single app manifests");
        };
        let docker_client = self.docker_client()?;
        for (_, service) in &manifest.compose.services.0 {
            // TODO: Subquests
            if let Some(Service {
                image: Some(image), ..
            }) = service
            {
                if crate::relic::docker::image::inspect(docker_client.clone(), image)
                    .await?
                    .is_none()
                {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    async fn installed_app_size(
        &self,
        _quest: SyncQuest,
        manifest: AppManifest,
        _id: AppId,
    ) -> anyhow::Result<usize> {
        let AppManifest::Multi(manifest) = manifest else {
            panic!("Compose deployment can not be called with single app manifests");
        };
        let docker_client = self.docker_client()?;
        let mut size = 0;
        for (_, service) in &manifest.compose.services.0 {
            // TODO: Subquests
            if let Some(Service {
                image: Some(image), ..
            }) = service
            {
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
        }
        Ok(size)
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
