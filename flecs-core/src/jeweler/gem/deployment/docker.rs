use crate::jeweler::app::{AppDeployment, AppId, AppInfo, Token};
use crate::jeweler::gem::instance::{InstanceId, InstanceStatus};
use crate::jeweler::gem::manifest::AppManifest;
use crate::jeweler::instance::InstanceDeployment;
use crate::jeweler::network::{NetworkConfig, NetworkDeployment, NetworkId, NetworkKind};
use crate::jeweler::volume::{VolumeDeployment, VolumeId};
use crate::quest::{QuestId, State, SyncQuest};
use crate::vault::pouch::deployment::DeploymentId;
use crate::{jeweler, relic};
use async_trait::async_trait;
use bollard::auth::DockerCredentials;
use bollard::container::{Config, CreateContainerOptions, LogOutput, RemoveContainerOptions};
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::image::RemoveImageOptions;
use bollard::models::{
    ContainerInspectResponse, ContainerState, EndpointSettings, MountPointTypeEnum, Network, Volume,
};
use bollard::network::{
    ConnectNetworkOptions, CreateNetworkOptions, DisconnectNetworkOptions, ListNetworksOptions,
};
use bollard::volume::CreateVolumeOptions;
use bollard::{Docker, API_DEFAULT_VERSION};
use futures_util::future::{join_all, BoxFuture};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use tokio::{fs, join};
use tracing::log::warn;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename = "Docker")]
pub struct DockerDeployment {
    id: DeploymentId,
    path: PathBuf,
}

impl DockerDeployment {
    fn client(&self) -> anyhow::Result<Arc<Docker>> {
        Ok(Arc::new(Docker::connect_with_unix(
            &self.path.to_string_lossy(),
            120,
            API_DEFAULT_VERSION,
        )?))
    }

    pub fn new(id: String, path: PathBuf) -> Self {
        Self { id, path }
    }
}

#[async_trait]
impl AppDeployment for DockerDeployment {
    async fn install_app(
        &self,
        quest: SyncQuest,
        manifest: Arc<AppManifest>,
        token: Option<Token>,
    ) -> anyhow::Result<AppId> {
        let docker_client = self.client()?;
        let (.., id) = quest
            .lock()
            .await
            .create_sub_quest(
                format!("Download image {}", manifest.image()),
                |quest| async move {
                    relic::docker::image::pull(
                        quest,
                        docker_client,
                        token.map(|token| DockerCredentials {
                            username: Some(token.username),
                            password: Some(token.password),
                            ..DockerCredentials::default()
                        }),
                        manifest.image(),
                        manifest.key.version.as_str(),
                    )
                    .await
                },
            )
            .await;
        id.await
    }

    async fn uninstall_app(&self, quest: SyncQuest, id: AppId) -> anyhow::Result<()> {
        let docker_client = self.client()?;
        quest
            .lock()
            .await
            .create_sub_quest(format!("Removing image of {id}"), |_quest| async move {
                let _ = relic::docker::image::remove(
                    docker_client,
                    &id,
                    Some(RemoveImageOptions {
                        force: true,
                        ..RemoveImageOptions::default()
                    }),
                    None,
                )
                .await?;
                Ok(())
            })
            .await
            .2
            .await
    }

    async fn app_info(&self, _quest: SyncQuest, id: AppId) -> anyhow::Result<AppInfo> {
        let docker_client = self.client()?;
        relic::docker::image::inspect(docker_client, &id).await
    }

    async fn copy_from_app_image(
        &self,
        quest: SyncQuest,
        image: String,
        src: &Path,
        dst: &Path,
        is_dst_file_path: bool,
    ) -> anyhow::Result<()> {
        let client = self.client()?;
        let container = relic::docker::container::create(
            client.clone(),
            Option::<CreateContainerOptions<&str>>::None,
            Config {
                image: Some(image.clone()),
                network_disabled: Some(true),
                ..Config::default()
            },
        )
        .await?;
        let copy_result = relic::docker::container::copy_from(
            quest,
            client.clone(),
            src,
            dst,
            &container,
            is_dst_file_path,
        )
        .await;

        if let Err(e) = relic::docker::container::remove(
            client.clone(),
            Some(RemoveContainerOptions {
                force: true,
                ..RemoveContainerOptions::default()
            }),
            &container,
        )
        .await
        {
            warn!("Could not remove temporary container '{container}' of image {image} which was created to copy {src:?} to {dst:?}: {e}");
        }
        copy_result
    }
}

impl DockerDeployment {
    async fn create_volume_with_client(
        docker_client: Arc<Docker>,
        _quest: SyncQuest,
        name: &str,
    ) -> anyhow::Result<VolumeId> {
        let volume = relic::docker::volume::create(
            docker_client,
            CreateVolumeOptions {
                name,
                driver: "local",
                ..CreateVolumeOptions::default()
            },
        )
        .await?;
        Ok(volume.name)
    }

    async fn create_check_volume_existence_subquest(
        quest: &SyncQuest,
        docker_client: Arc<Docker>,
        volume_id: VolumeId,
    ) -> (QuestId, SyncQuest, BoxFuture<'static, anyhow::Result<bool>>) {
        quest
            .lock()
            .await
            .create_sub_quest(
                format!("Check if volume {volume_id} exists"),
                |_quest| async move {
                    Ok(relic::docker::volume::inspect(docker_client, &volume_id)
                        .await?
                        .is_some())
                },
            )
            .await
    }

    async fn export_volume_with_client(
        docker_client: Arc<Docker>,
        quest: SyncQuest,
        id: VolumeId,
        path: &Path,
        image: &str,
    ) -> anyhow::Result<()> {
        let volume_exists =
            Self::create_check_volume_existence_subquest(&quest, docker_client.clone(), id.clone())
                .await
                .2;

        let container = {
            let docker_client = docker_client.clone();
            let id = id.clone();
            let image = image.to_string();
            quest
                .lock()
                .await
                .create_sub_quest(
                    "Create temporary container".to_string(),
                    |_quest| async move {
                        if !volume_exists.await? {
                            anyhow::bail!("Volume {id} does not exist");
                        }
                        let container_id = relic::docker::container::create::<String, String>(
                            docker_client,
                            None,
                            Config {
                                image: Some(image),
                                volumes: Some(HashMap::from([(
                                    format!("{id}:/tmp_volumes/{id}/"),
                                    HashMap::new(),
                                )])),
                                network_disabled: Some(true),
                                ..Config::default()
                            },
                        )
                        .await?;
                        Ok(container_id)
                    },
                )
                .await
                .2
        };
        let (send_container_download, recv_container_download) = tokio::sync::oneshot::channel();
        let download = quest
            .lock()
            .await
            .create_sub_quest("Download data".to_string(), |quest| {
                let path = path.to_path_buf();
                let docker_client = docker_client.clone();
                async move {
                    let container: String = recv_container_download.await?;
                    relic::docker::container::download_gzip_streamed(
                        docker_client,
                        quest,
                        Path::new(&format!("/tmp_volumes/{id}/")),
                        &path,
                        &container,
                    )
                    .await
                }
            })
            .await
            .2;
        let (send_container_remove, recv_container_remove) = tokio::sync::oneshot::channel();

        let remove_container = quest
            .lock()
            .await
            .create_sub_quest("Remove temporary container".to_string(), |_quest| {
                let docker_client = docker_client.clone();
                async move {
                    download.await?;
                    let container: String = recv_container_remove.await?;
                    relic::docker::container::remove(
                        docker_client,
                        Some(RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        }),
                        &container,
                    )
                    .await
                }
            })
            .await
            .2;
        match container.await {
            Ok(container) => {
                let _ = send_container_download.send(container.clone());
                let _ = send_container_remove.send(container.clone());
                if let Err(e) = remove_container.await {
                    eprintln!("Could not remove temporary container {container}: {e}");
                }
                Ok(())
            }
            Err(e) => {
                drop(send_container_download);
                drop(send_container_remove);
                Err(e)
            }
        }
    }

    async fn list_volumes(
        docker_client: Arc<Docker>,
        quest: SyncQuest,
        instance_id: InstanceId,
    ) -> anyhow::Result<HashMap<VolumeId, Volume>> {
        let inspect = quest
            .lock()
            .await
            .create_sub_quest(format!("Search volumes of {instance_id}"), |_quest| {
                let docker_client = docker_client.clone();
                let instance_id = instance_id.to_docker_id();
                async move { relic::docker::container::inspect(docker_client, &instance_id).await }
            })
            .await
            .2;
        let Some(inspect) = inspect.await? else {
            anyhow::bail!("Container for instance {instance_id} not found");
        };
        match inspect.mounts {
            None => Ok(Default::default()),
            Some(mounts) => {
                let mut results = Vec::new();
                let mut ids = Vec::new();
                for volume_id in mounts.into_iter().filter_map(|mount| match mount.typ {
                    Some(MountPointTypeEnum::VOLUME) => mount.name,
                    _ => None,
                }) {
                    ids.push(volume_id.clone());
                    results.push(
                        quest
                            .lock()
                            .await
                            .create_sub_quest(format!("Inspect volume {volume_id}"), |_quest| {
                                let docker_client = docker_client.clone();
                                async move {
                                    relic::docker::volume::inspect(docker_client, &volume_id).await
                                }
                            })
                            .await
                            .2,
                    );
                }
                let mut volumes = HashMap::new();
                for (id, result) in ids.into_iter().zip(join_all(results).await) {
                    match result {
                        Ok(Some(volume)) => {
                            volumes.insert(id, volume);
                        }
                        Err(e) => {
                            eprintln!("Could not inspect volume {id}: {e}");
                        }
                        Ok(None) => {
                            eprintln!("Could not inspect volume {id}: Does not exist");
                        }
                    }
                }
                Ok(volumes)
            }
        }
    }
}

#[async_trait]
impl VolumeDeployment for DockerDeployment {
    async fn create_volume(&self, quest: SyncQuest, name: &str) -> anyhow::Result<VolumeId> {
        let docker_client = self.client()?;
        Self::create_volume_with_client(docker_client, quest, name).await
    }

    async fn delete_volume(&self, _quest: SyncQuest, id: VolumeId) -> anyhow::Result<()> {
        let docker_client = self.client()?;
        relic::docker::volume::remove(docker_client, None, &id).await
    }

    async fn import_volume(
        &self,
        quest: SyncQuest,
        path: &Path,
        name: &str,
        image: &str,
    ) -> anyhow::Result<VolumeId> {
        if !path.try_exists()? {
            anyhow::bail!("Could not import volume {name}, path does not exist: {path:?}");
        }
        if !fs::metadata(path).await?.is_file() {
            anyhow::bail!("Could not import volume {name}, path is not a regular file: {path:?}");
        }
        let name = name.to_string();
        let docker_client = self.client()?;
        let volume_exists = {
            let name = name.clone();
            let docker_client = docker_client.clone();
            quest
                .lock()
                .await
                .create_sub_quest(
                    "Check if volume already exists".to_string(),
                    |_quest| async move {
                        Ok(relic::docker::volume::inspect(docker_client, &name)
                            .await?
                            .is_some())
                    },
                )
                .await
                .2
        };
        let volume_gone = {
            let docker_client = docker_client.clone();
            let name = name.clone();
            quest
                .lock()
                .await
                .create_sub_quest("Delete existing volume".to_string(), |quest| async move {
                    if volume_exists.await? {
                        relic::docker::volume::remove(docker_client, None, &name).await?;
                    } else {
                        let mut quest = quest.lock().await;
                        quest.state = State::Skipped;
                        quest.detail = Some("Volume does not exist".to_string());
                    }
                    Ok(())
                })
                .await
                .2
        };
        let created_volume = {
            let docker_client = docker_client.clone();
            let name = name.clone();
            quest
                .lock()
                .await
                .create_sub_quest("Create volume".to_string(), |quest| async move {
                    volume_gone.await?;
                    Self::create_volume_with_client(docker_client, quest, &name).await
                })
                .await
                .2
        };

        let container = {
            let docker_client = docker_client.clone();
            let name = name.clone();
            let image = image.to_string();
            quest
                .lock()
                .await
                .create_sub_quest(
                    "Create temporary container".to_string(),
                    |_quest| async move {
                        let container_id = relic::docker::container::create::<String, String>(
                            docker_client,
                            None,
                            Config {
                                image: Some(image),
                                volumes: Some(HashMap::from([(
                                    format!("{name}:/tmp_volumes/{name}/"),
                                    HashMap::new(),
                                )])),
                                network_disabled: Some(true),
                                ..Config::default()
                            },
                        )
                        .await?;
                        Ok(container_id)
                    },
                )
                .await
                .2
        };
        let (send_container_upload, recv_container_upload) = tokio::sync::oneshot::channel();
        let upload = quest
            .lock()
            .await
            .create_sub_quest("Upload data".to_string(), |quest| {
                let path = path.to_path_buf();
                let docker_client = docker_client.clone();
                async move {
                    let container: String = recv_container_upload.await?;
                    relic::docker::container::upload_gzip_file_streamed(
                        docker_client,
                        quest,
                        &path,
                        Path::new(&format!("/tmp_volumes/{name}/")),
                        &container,
                    )
                    .await
                }
            })
            .await
            .2;
        let (send_container_remove, recv_container_remove) = tokio::sync::oneshot::channel();

        let remove_container = quest
            .lock()
            .await
            .create_sub_quest("Remove temporary container".to_string(), |_quest| {
                let docker_client = docker_client.clone();
                async move {
                    upload.await?;
                    let container: String = recv_container_remove.await?;
                    relic::docker::container::remove(
                        docker_client,
                        Some(RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        }),
                        &container,
                    )
                    .await
                }
            })
            .await
            .2;
        match join!(created_volume, container) {
            (Ok(volume_id), Ok(container)) => {
                let _ = send_container_upload.send(container.clone());
                let _ = send_container_remove.send(container.clone());
                if let Err(e) = remove_container.await {
                    eprintln!("Could not remove temporary container {container}: {e}");
                }
                Ok(volume_id)
            }
            (Err(e), Ok(container)) => {
                drop(send_container_upload);
                let _ = send_container_remove.send(container.clone());
                if let Err(e) = remove_container.await {
                    eprintln!("Could not remove temporary container {container}: {e}");
                }
                Err(e)
            }
            (_, Err(e)) => {
                drop(send_container_upload);
                drop(send_container_remove);
                Err(e)
            }
        }
    }

    async fn export_volume(
        &self,
        quest: SyncQuest,
        id: VolumeId,
        path: &Path,
        image: &str,
    ) -> anyhow::Result<()> {
        let docker_client = self.client()?;
        Self::export_volume_with_client(docker_client, quest, id, path, image).await
    }

    async fn volumes(
        &self,
        quest: SyncQuest,
        instance_id: InstanceId,
    ) -> anyhow::Result<HashMap<VolumeId, Volume>> {
        let docker_client = self.client()?;
        Self::list_volumes(docker_client, quest, instance_id).await
    }

    async fn export_volumes(
        &self,
        quest: SyncQuest,
        instance_id: InstanceId,
        path: &Path,
        image: &str,
    ) -> anyhow::Result<()> {
        let mut results = Vec::new();
        let docker_client = self.client()?;
        let volumes = quest
            .lock()
            .await
            .create_sub_quest(format!("Search all volumes of {instance_id}"), |quest| {
                let docker_client = docker_client.clone();
                async move { Self::list_volumes(docker_client, quest, instance_id).await }
            })
            .await
            .2;
        for volume_id in volumes.await?.keys() {
            results.push(
                quest
                    .lock()
                    .await
                    .create_sub_quest(format!("Exporting volume {volume_id}"), |quest| {
                        let volume_id = volume_id.clone();
                        let path = path.to_path_buf();
                        let docker_client = docker_client.clone();
                        let image = image.to_string();
                        async move {
                            Self::export_volume_with_client(
                                docker_client,
                                quest,
                                volume_id.clone(),
                                &path,
                                &image,
                            )
                            .await
                        }
                    })
                    .await
                    .2,
            );
        }
        let errors = futures::future::join_all(results)
            .await
            .into_iter()
            .filter_map(|result| match result {
                Ok(_) => None,
                Err(e) => Some(e.to_string()),
            })
            .collect::<Vec<_>>();
        if errors.is_empty() {
            Ok(())
        } else {
            anyhow::bail!(
                "At least one volume could not be exported: [{}]",
                errors.join(", ")
            )
        }
    }
}

#[async_trait]
impl NetworkDeployment for DockerDeployment {
    async fn create_network(
        &self,
        _quest: SyncQuest,
        mut config: NetworkConfig,
    ) -> anyhow::Result<NetworkId> {
        let docker_client = self.client()?;
        let mut options = HashMap::new();
        match &config.kind {
            NetworkKind::IpvlanL2 => {
                options.insert("ipvlan_mode", "l2");
            }
            NetworkKind::IpvlanL3 => {
                options.insert("ipvlan_mode", "l3");
            }
            _ => {}
        }
        let driver = match &config.kind {
            NetworkKind::Bridge | NetworkKind::MACVLAN | NetworkKind::Internal => {
                config.kind.to_string()
            }
            NetworkKind::IpvlanL2 | NetworkKind::IpvlanL3 => {
                let Some(parent_adapter) = &config.parent_adapter else {
                    anyhow::bail!("Can not create ipvlan network without parent");
                };
                match (config.cidr_subnet, config.gateway) {
                    (None, _) | (_, None) => {
                        let (parent_name, parent_adapter) =
                            relic::network::NetInfo::try_read_from_system()?
                                .remove_entry(parent_adapter)
                                .ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "parent network adapter {parent_adapter} does not exist"
                                    )
                                })?;
                        if parent_adapter.ipv4addresses.is_empty() {
                            anyhow::bail!("parent network adapter {parent_name} is not ready");
                        }
                        config.cidr_subnet = Some(relic::network::ipv4_to_network(
                            Ipv4Addr::from_str(&parent_adapter.ipv4addresses[0].addr)?,
                            Ipv4Addr::from_str(&parent_adapter.ipv4addresses[0].subnet_mask)?,
                        ));
                        config.gateway = Some(Ipv4Addr::from_str(&parent_adapter.gateway)?);
                    }
                    _ => {}
                }
                "ipvlan".to_string()
            }
            x => anyhow::bail!("Invalid network type {}", x),
        };
        if let Some(parent_adapter) = &config.parent_adapter {
            options.insert("parent", parent_adapter.as_str());
        }
        let options = CreateNetworkOptions {
            name: config.name.as_str(),
            driver: driver.as_str(),
            options,
            ..CreateNetworkOptions::default()
        };
        relic::docker::network::create(docker_client, options).await
    }

    async fn delete_network(&self, id: NetworkId) -> anyhow::Result<()> {
        let docker_client = self.client()?;
        relic::docker::network::remove(docker_client, &id).await
    }

    async fn network(&self, id: NetworkId) -> anyhow::Result<Network> {
        let docker_client = self.client()?;
        relic::docker::network::inspect::<&str>(docker_client, &id, None).await
    }

    async fn networks(&self, _quest: SyncQuest) -> anyhow::Result<Vec<Network>> {
        let docker_client = self.client()?;
        relic::docker::network::list(
            docker_client,
            Some(ListNetworksOptions {
                filters: HashMap::from([("name", vec!["flecs.*"])]),
            }),
        )
        .await
    }

    async fn connect_network(
        &self,
        _quest: SyncQuest,
        id: NetworkId,
        address: Ipv4Addr,
        container: &str,
    ) -> anyhow::Result<()> {
        let docker_client = self.client()?;
        let options = ConnectNetworkOptions {
            container,
            endpoint_config: EndpointSettings {
                ip_address: Some(address.to_string()),
                ..EndpointSettings::default()
            },
        };
        relic::docker::network::connect(docker_client, &id, options).await
    }

    async fn disconnect_network(
        &self,
        _quest: SyncQuest,
        id: NetworkId,
        container: &str,
    ) -> anyhow::Result<()> {
        let docker_client = self.client()?;
        let options = DisconnectNetworkOptions {
            container,
            force: false,
        };
        relic::docker::network::disconnect(docker_client, &id, options).await
    }
}

#[async_trait]
impl InstanceDeployment for DockerDeployment {
    async fn delete_instance(&self, id: InstanceId) -> anyhow::Result<bool> {
        let docker_client = self.client()?;
        relic::docker::container::remove(
            docker_client,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
            &id.to_docker_id(),
        )
        .await
    }

    async fn start_instance(
        &self,
        config: Config<String>,
        id: Option<InstanceId>,
    ) -> anyhow::Result<InstanceId> {
        let client = self.client()?;
        let id = id.unwrap_or_else(InstanceId::new_random);
        let options = Some(CreateContainerOptions {
            name: id.to_docker_id(),
            platform: None,
        });
        let docker_id = relic::docker::container::create(client.clone(), options, config).await?;
        println!("Created container {}/{}", id, docker_id);
        relic::docker::container::start(client, &id.to_docker_id()).await?;
        Ok(id)
    }

    async fn stop_instance(&self, id: InstanceId) -> anyhow::Result<()> {
        let client = self.client()?;
        relic::docker::container::stop(client, &id.to_docker_id(), None).await?;
        self.delete_instance(id).await?;
        Ok(())
    }

    async fn ready_instance(&self, id: InstanceId) -> anyhow::Result<()> {
        let docker_client = self.client()?;
        let exec_options = CreateExecOptions {
            attach_stderr: Some(true),
            cmd: Some(vec!["touch", "/flecs-tmp/ready"]),
            ..Default::default()
        };
        let mut errors = Vec::new();
        match relic::docker::container::exec(
            docker_client,
            &id.to_docker_id(),
            exec_options,
            Some(StartExecOptions {
                detach: false,
                ..Default::default()
            }),
        )
        .await?
        {
            StartExecResults::Attached { mut output, .. } => {
                while let Some(output) = output.next().await {
                    match output {
                        Err(e) => errors.push(format!(
                            "Error on container during readying container {}: {e}",
                            id.to_docker_id()
                        )),
                        Ok(LogOutput::StdErr { message }) => errors.push(format!(
                            "Error on container during readying container {}: {}",
                            id.to_docker_id(),
                            String::from_utf8_lossy(message.as_ref())
                        )),
                        _ => {}
                    }
                }
            }
            StartExecResults::Detached => {}
        }
        if errors.is_empty() {
            Ok(())
        } else {
            anyhow::bail!(errors.join(", "))
        }
    }

    async fn instance_status(&self, id: InstanceId) -> anyhow::Result<InstanceStatus> {
        let docker_client = self.client()?;
        match relic::docker::container::inspect(docker_client, &id.to_docker_id()).await? {
            None => Ok(InstanceStatus::Created),
            Some(ContainerInspectResponse {
                state:
                    Some(ContainerState {
                        status: Some(state),
                        ..
                    }),
                ..
            }) => Ok(state.into()),
            _ => Ok(InstanceStatus::Unknown),
        }
    }

    async fn copy_from_instance(
        &self,
        quest: SyncQuest,
        id: InstanceId,
        src: &Path,
        dst: &Path,
        is_dst_file_path: bool,
    ) -> anyhow::Result<()> {
        let docker_client = self.client()?;
        relic::docker::container::copy_from(
            quest,
            docker_client,
            src,
            dst,
            &id.to_docker_id(),
            is_dst_file_path,
        )
        .await
    }

    async fn copy_to_instance(
        &self,
        quest: SyncQuest,
        id: InstanceId,
        src: &Path,
        dst: &Path,
    ) -> anyhow::Result<()> {
        let docker_client = self.client()?;
        relic::docker::container::copy_to(docker_client, quest, src, dst, &id.to_docker_id(), true)
            .await
    }
}

#[async_trait]
impl jeweler::deployment::Deployment for DockerDeployment {
    fn id(&self) -> jeweler::deployment::DeploymentId {
        self.id.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::jeweler::gem::deployment::docker::DockerDeployment;
    use crate::vault::pouch::deployment::Deployment;
    use std::path::PathBuf;

    const TEST_DEPLOYMENT_ID: &str = "some-deployment-id";
    const TEST_DEPLOYMENT_SOCK_PATH: &str = "/path/to/docker.sock";

    #[test]
    fn deployment_id() {
        let deployment = Deployment::Docker(DockerDeployment::new(
            TEST_DEPLOYMENT_ID.to_string(),
            PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
        ));
        assert_eq!(deployment.id(), TEST_DEPLOYMENT_ID);
    }

    #[test]
    fn default_deployment() {
        let deployment = Deployment::default();
        #[allow(unreachable_patterns)]
        match deployment {
            Deployment::Docker(deployment) => {
                assert_eq!(deployment.id, "DefaultDockerDeployment");
                assert_eq!(deployment.path, PathBuf::from("/var/run/docker.sock"));
            }
            _ => panic!("Expected default deployment to be of type Docker"),
        }
    }
}
