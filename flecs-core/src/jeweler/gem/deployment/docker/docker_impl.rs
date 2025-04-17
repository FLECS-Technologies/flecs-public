use crate::forge::bollard::BollardNetworkExtension;
use crate::jeweler::GetDeploymentId;
use crate::jeweler::app::{AppDeployment, AppId, Token};
use crate::jeweler::deployment::CommonDeployment;
use crate::jeweler::gem::deployment::docker::{AppInfo, DockerDeployment};
use crate::jeweler::gem::instance::status::InstanceStatus;
use crate::jeweler::gem::instance::{InstanceId, Logs};
use crate::jeweler::gem::manifest::AppManifest;
use crate::jeweler::gem::manifest::single::ConfigFile;
use crate::jeweler::network::{
    CreateNetworkError, NetworkConfig, NetworkDeployment, NetworkId, NetworkKind,
};
use crate::jeweler::volume::{Volume, VolumeDeployment, VolumeId};
use crate::quest::{Quest, QuestId, State, SyncQuest};
use crate::relic::network::{Ipv4Network, NetworkAdapterReader, NetworkAdapterReaderImpl};
use crate::vault::pouch::deployment::DeploymentId;
use crate::{jeweler, lore, relic};
use async_trait::async_trait;
use bollard::auth::DockerCredentials;
use bollard::container::{Config, CreateContainerOptions, RemoveContainerOptions};
use bollard::image::{ImportImageOptions, RemoveImageOptions};
use bollard::models::{
    ContainerInspectResponse, ContainerState, EndpointIpamConfig, EndpointSettings, HostConfig,
    Ipam, IpamConfig, Mount, MountPointTypeEnum, MountTypeEnum, Network,
};
use bollard::network::{
    ConnectNetworkOptions, CreateNetworkOptions, DisconnectNetworkOptions, ListNetworksOptions,
};
use bollard::volume::CreateVolumeOptions;
use bollard::{API_DEFAULT_VERSION, Docker};
use futures_util::future::{BoxFuture, join_all};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::{fs, join};
use tracing::{debug, error, warn};

#[derive(Serialize, Deserialize)]
pub struct DockerDeploymentImpl {
    pub id: DeploymentId,
    path: PathBuf,
    #[serde(default)]
    is_default: bool,
    #[serde(skip, default = "default_network_adapter_reader")]
    network_adapter_reader: Box<dyn NetworkAdapterReader>,
}

impl GetDeploymentId for DockerDeploymentImpl {
    fn deployment_id(&self) -> &DeploymentId {
        &self.id
    }
}

impl Default for DockerDeploymentImpl {
    fn default() -> Self {
        Self::new_default(
            "DefaultDockerDeploymentImpl".to_string(),
            PathBuf::from("/var/run/docker.sock"),
        )
    }
}

impl CommonDeployment for DockerDeploymentImpl {
    fn id(&self) -> &jeweler::deployment::DeploymentId {
        &self.id
    }

    fn is_default(&self) -> bool {
        self.is_default
    }
}

impl std::fmt::Debug for DockerDeploymentImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct DockerDeploymentImpl<'a> {
            id: &'a DeploymentId,
            path: &'a PathBuf,
            is_default: &'a bool,
        }

        let Self {
            id,
            path,
            is_default,
            network_adapter_reader: _,
        } = self;
        std::fmt::Debug::fmt(
            &DockerDeploymentImpl {
                id,
                path,
                is_default,
            },
            f,
        )
    }
}

impl PartialEq for DockerDeploymentImpl {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.path == other.path
    }
}

impl Eq for DockerDeploymentImpl {}

fn default_network_adapter_reader() -> Box<dyn NetworkAdapterReader> {
    Box::new(NetworkAdapterReaderImpl)
}

impl DockerDeploymentImpl {
    fn client(&self) -> anyhow::Result<Arc<Docker>> {
        Ok(Arc::new(Docker::connect_with_unix(
            &self.path.to_string_lossy(),
            120,
            API_DEFAULT_VERSION,
        )?))
    }

    pub fn new(id: String, path: PathBuf) -> Self {
        Self {
            id,
            path,
            network_adapter_reader:
                crate::jeweler::gem::deployment::docker::docker_impl::default_network_adapter_reader(
                ),
            is_default: false,
        }
    }

    pub fn new_default(id: String, path: PathBuf) -> Self {
        Self {
            id,
            path,
            network_adapter_reader:
                crate::jeweler::gem::deployment::docker::docker_impl::default_network_adapter_reader(
                ),
            is_default: true,
        }
    }
    fn network_config_fits_network(
        config: &NetworkConfig,
        network: &Network,
    ) -> anyhow::Result<bool> {
        Ok(Self::name_fits_network(&config.name, network)
            && Self::kind_fits_network(config.kind, network)
            && Self::subnet_fits_network(config.cidr_subnet, network)?
            && Self::gateway_fits_network(config.gateway, network)?
            && Self::parent_fits_network(config.parent_adapter.as_ref(), network)
            && Self::options_fit_network(config.options.as_ref(), network))
    }

    fn name_fits_network(name: &String, network: &Network) -> bool {
        network.name.as_ref() == Some(name)
    }

    fn kind_fits_network(kind: NetworkKind, network: &Network) -> bool {
        kind == network.guess_network_kind()
    }

    fn subnet_fits_network(subnet: Option<Ipv4Network>, network: &Network) -> anyhow::Result<bool> {
        let fits = match subnet {
            Some(subnet) => network
                .subnets()?
                .contains(&relic::network::Network::Ipv4(subnet)),
            None => true,
        };
        Ok(fits)
    }

    fn gateway_fits_network(gateway: Option<Ipv4Addr>, network: &Network) -> anyhow::Result<bool> {
        let fits = match gateway {
            Some(gateway) => network.gateways()?.contains(&IpAddr::V4(gateway)),
            None => true,
        };
        Ok(fits)
    }

    fn parent_fits_network(parent: Option<&NetworkId>, network: &Network) -> bool {
        parent == network.parent_network().as_ref()
    }

    fn options_fit_network(options: Option<&HashMap<String, String>>, network: &Network) -> bool {
        match (options, network.options.as_ref()) {
            (None, _) => true,
            (Some(options), None) => options.is_empty(),
            (Some(options), Some(existing_options)) => options
                .iter()
                .all(|(key, value)| existing_options.get(key) == Some(value)),
        }
    }

    async fn copy_to_instance(
        docker_client: Arc<Docker>,
        quest: SyncQuest,
        id: InstanceId,
        src: &Path,
        dst: &Path,
        is_dst_file_path: bool,
    ) -> anyhow::Result<()> {
        relic::docker::container::copy_to(
            docker_client,
            quest,
            src,
            dst,
            &id.to_docker_id(),
            true,
            is_dst_file_path,
        )
        .await
    }

    async fn copy_from_instance(
        docker_client: Arc<Docker>,
        quest: SyncQuest,
        id: InstanceId,
        src: &Path,
        dst: &Path,
        is_dst_file_path: bool,
    ) -> anyhow::Result<()> {
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

    async fn copy_config_to_instance(
        client: Arc<Docker>,
        id: InstanceId,
        config_files: &[ConfigFile],
    ) -> crate::Result<()> {
        for config_file in config_files {
            let src = crate::lore::instance_config_path(&id.to_string())
                .join(&config_file.host_file_name);
            if let Err(e) = Self::copy_to_instance(
                client.clone(),
                Quest::new_synced(format!(
                    "Copy config {} to instance {}",
                    config_file.host_file_name, id
                )),
                id,
                &src,
                &config_file.container_file_path,
                true,
            )
            .await
            {
                anyhow::bail!(
                    "Could not copy config file {src:?} of instance {id} to {:?}: {e}",
                    config_file.container_file_path
                )
            }
        }
        Ok(())
    }

    async fn copy_config_from_instance(
        client: Arc<Docker>,
        id: InstanceId,
        config_files: &[ConfigFile],
        dst: PathBuf,
    ) -> crate::Result<()> {
        for config_file in config_files {
            let dst = dst.join(&config_file.host_file_name);
            if let Err(e) = Self::copy_from_instance(
                client.clone(),
                Quest::new_synced(format!(
                    "Copy config {:?} from instance {}",
                    config_file.container_file_path, id
                )),
                id,
                &config_file.container_file_path,
                &dst,
                true,
            )
            .await
            {
                anyhow::bail!(
                    "Could not copy config file {:?} of instance {id} to {dst:?}: {e}",
                    config_file.container_file_path
                )
            }
        }
        Ok(())
    }
    pub async fn create_volume_with_client(
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

    fn temporary_volume_container_config(
        image: String,
        volume_id: VolumeId,
        volume_dst: impl AsRef<Path>,
    ) -> Config<String> {
        let volume_dst = volume_dst.as_ref().to_string_lossy().to_string();
        Config {
            image: Some(image),
            volumes: Some(HashMap::from([(
                format!("{volume_id}:{}", volume_dst),
                HashMap::new(),
            )])),
            network_disabled: Some(true),
            host_config: Some(HostConfig {
                mounts: Some(vec![Mount {
                    typ: Some(MountTypeEnum::VOLUME),
                    source: Some(volume_id),
                    target: Some(volume_dst),
                    ..Default::default()
                }]),
                ..HostConfig::default()
            }),
            ..Config::default()
        }
    }

    pub async fn import_volume_with_client(
        docker_client: Arc<Docker>,
        quest: SyncQuest,
        src: &Path,
        container_path: &Path,
        name: &str,
        image: &str,
    ) -> anyhow::Result<VolumeId> {
        if !src.try_exists()? {
            anyhow::bail!("Could not import volume {name}, path does not exist: {src:?}");
        }
        if !fs::metadata(src).await?.is_file() {
            anyhow::bail!("Could not import volume {name}, path is not a regular file: {src:?}");
        }
        let name = name.to_string();
        let volume_exists = {
            let name = name.clone();
            let docker_client = docker_client.clone();
            quest
                .lock()
                .await
                .create_sub_quest(
                    "Check if volume already exists".to_string(),
                    |_quest| async move {
                        Ok::<bool, anyhow::Error>(
                            relic::docker::volume::inspect(docker_client, &name)
                                .await?
                                .is_some(),
                        )
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
                    Ok::<(), anyhow::Error>(())
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
            let container_path = container_path.to_path_buf();
            quest
                .lock()
                .await
                .create_sub_quest(
                    "Create temporary container".to_string(),
                    |_quest| async move {
                        let container_id = relic::docker::container::create::<String, String>(
                            docker_client,
                            None,
                            Self::temporary_volume_container_config(image, name, container_path),
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
                let path = src.to_path_buf();
                let dst = container_path
                    .parent()
                    .unwrap_or(Path::new("/"))
                    .to_path_buf();
                let docker_client = docker_client.clone();
                async move {
                    let container: String = recv_container_upload.await?;
                    relic::docker::container::copy_archive_file_to(
                        docker_client,
                        quest,
                        path,
                        dst,
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
                upload.await?;
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

    pub async fn export_volume_with_client(
        docker_client: Arc<Docker>,
        quest: SyncQuest,
        id: VolumeId,
        export_path: &Path,
        container_path: &Path,
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
            let volume_dst = container_path.to_path_buf();
            quest
                .lock()
                .await
                .create_sub_quest(
                    "Create temporary container".to_string(),
                    |_quest| async move {
                        if !volume_exists.await? {
                            anyhow::bail!("Volume {id} does not exist");
                        }
                        let config = Self::temporary_volume_container_config(image, id, volume_dst);
                        let container_id = relic::docker::container::create::<String, String>(
                            docker_client,
                            None,
                            config,
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
                let dst = export_path.to_path_buf();
                let src = container_path.to_path_buf();
                let docker_client = docker_client.clone();
                async move {
                    let container: String = recv_container_download.await?;
                    relic::docker::container::copy_archive_to_file(
                        quest,
                        docker_client,
                        &src,
                        dst,
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
                download.await?;
                let _ = send_container_remove.send(container.clone());
                if let Err(e) = remove_container.await {
                    error!("Could not remove temporary container {container}: {e}");
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

    pub async fn list_volumes(
        docker_client: Arc<Docker>,
        quest: SyncQuest,
        instance_id: InstanceId,
    ) -> anyhow::Result<HashMap<VolumeId, (Volume, PathBuf)>> {
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
                let mut id_dst_list = Vec::new();
                for (volume_id, dst) in mounts.into_iter().filter_map(|mount| match mount.typ {
                    Some(MountPointTypeEnum::VOLUME) => Some((
                        mount.name.unwrap_or_default(),
                        mount.destination.unwrap_or_default(),
                    )),
                    _ => None,
                }) {
                    id_dst_list.push((volume_id.clone(), PathBuf::from(dst)));
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
                for ((id, dst), result) in id_dst_list.into_iter().zip(join_all(results).await) {
                    match result {
                        Ok(Some(volume)) => {
                            volumes.insert(id, (volume, dst));
                        }
                        Err(e) => {
                            error!("Could not inspect volume {id}: {e}");
                        }
                        Ok(None) => {
                            error!("Could not inspect volume {id}: Does not exist");
                        }
                    }
                }
                Ok(volumes)
            }
        }
    }

    pub async fn create_network_with_client(
        docker_client: Arc<Docker>,
        _quest: SyncQuest,
        mut config: NetworkConfig,
        network_adapter_reader: &dyn NetworkAdapterReader,
    ) -> Result<Network, CreateNetworkError> {
        if let Some(existing_network) =
            relic::docker::network::inspect::<&str>(docker_client.clone(), &config.name, None)
                .await?
        {
            return if Self::network_config_fits_network(&config, &existing_network)? {
                Err(CreateNetworkError::ExactNetworkExists(existing_network))
            } else {
                Err(CreateNetworkError::DifferentNetworkExists(existing_network))
            };
        };
        let mut options = config.options.unwrap_or_default();
        match &config.kind {
            NetworkKind::IpvlanL2 => {
                options.insert("ipvlan_mode".to_string(), "l2".to_string());
            }
            NetworkKind::IpvlanL3 => {
                options.insert("ipvlan_mode".to_string(), "l3".to_string());
            }
            _ => {}
        }
        let driver = match &config.kind {
            NetworkKind::Bridge | NetworkKind::MACVLAN | NetworkKind::Internal => {
                config.kind.to_string()
            }
            NetworkKind::IpvlanL2 | NetworkKind::IpvlanL3 => {
                let Some(parent_adapter) = &config.parent_adapter else {
                    return Err(CreateNetworkError::NetworkConfigInvalid {
                        location: "parent_adapter".to_string(),
                        reason: "Can not create ipvlan network without parent".to_string(),
                    });
                };
                match (config.cidr_subnet, config.gateway) {
                    (None, _) | (_, None) => {
                        let (parent_name, parent_adapter) = network_adapter_reader
                            .try_read_network_adapters()?
                            .remove_entry(parent_adapter)
                            .ok_or_else(|| CreateNetworkError::NetworkConfigInvalid {
                                location: "parent_adapter".to_string(),
                                reason: format!(
                                    "Parent network adapter {parent_adapter} does not exist"
                                ),
                            })?;
                        if parent_adapter.ipv4_networks.is_empty() {
                            return Err(CreateNetworkError::NetworkConfigInvalid {
                                location: "parent_adapter".to_string(),
                                reason: format!(
                                    "Parent network adapter {parent_name} is not ready"
                                ),
                            });
                        }
                        config.cidr_subnet = Some(relic::network::ipv4_to_network(
                            *parent_adapter.ipv4_networks[0].address(),
                            parent_adapter.ipv4_networks[0].subnet_mask(),
                        ));
                        config.gateway = parent_adapter.gateway;
                    }
                    _ => {}
                }
                "ipvlan".to_string()
            }
            x => {
                return Err(CreateNetworkError::NetworkConfigInvalid {
                    location: "kind".to_string(),
                    reason: format!("Invalid network type {x}"),
                });
            }
        };
        if let Some(parent_adapter) = config.parent_adapter {
            options.insert("parent".to_string(), parent_adapter);
        }
        let options = CreateNetworkOptions {
            name: config.name,
            driver,
            options,
            ipam: Ipam {
                config: Some(vec![IpamConfig {
                    gateway: config.gateway.as_ref().map(ToString::to_string),
                    subnet: config.cidr_subnet.as_ref().map(ToString::to_string),
                    ..IpamConfig::default()
                }]),
                ..Ipam::default()
            },
            ..CreateNetworkOptions::default()
        };
        Ok(relic::docker::network::create(docker_client, options).await?)
    }

    pub async fn default_network_with_client(
        docker_client: Arc<Docker>,
        network_adapter_reader: &dyn NetworkAdapterReader,
    ) -> anyhow::Result<Network, CreateNetworkError> {
        let default_network_name = lore::network::default_network_name();
        let network = relic::docker::network::list(
            docker_client.clone(),
            Some(ListNetworksOptions {
                filters: HashMap::from([("name", vec![default_network_name])]),
            }),
        )
        .await?
        .into_iter()
        .find(|network| network.name.as_deref() == Some(default_network_name));
        if let Some(network) = network {
            return Ok(network);
        };
        Self::create_default_network_with_client(docker_client, network_adapter_reader).await
    }

    pub async fn create_default_network_with_client(
        docker_client: Arc<Docker>,
        network_adapter_reader: &dyn NetworkAdapterReader,
    ) -> anyhow::Result<Network, CreateNetworkError> {
        Self::create_network_with_client(
            docker_client,
            Quest::new_synced("Create default network".to_string()),
            lore::network::default_network_config(),
            network_adapter_reader,
        )
        .await
    }
}

#[async_trait]
impl DockerDeployment for DockerDeploymentImpl {
    async fn create_default_network(
        &self,
    ) -> crate::Result<jeweler::network::Network, CreateNetworkError> {
        self.create_network(
            Quest::new_synced("Create default network".to_string()),
            lore::network::default_network_config(),
        )
        .await
    }
    async fn app_info(&self, _quest: SyncQuest, id: AppId) -> anyhow::Result<Option<AppInfo>> {
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
            warn!(
                "Could not remove temporary container '{container}' \
            of image {image} which was created to copy {src:?} to {dst:?}: {e}"
            );
        }
        copy_result
    }

    async fn connect_network(
        &self,
        _quest: SyncQuest,
        id: NetworkId,
        address: Ipv4Addr,
        instance_id: InstanceId,
    ) -> anyhow::Result<()> {
        let docker_client = self.client()?;
        let options = ConnectNetworkOptions {
            container: instance_id.to_docker_id(),
            endpoint_config: EndpointSettings {
                ip_address: Some(address.to_string()),
                ipam_config: Some(EndpointIpamConfig {
                    ipv4_address: Some(address.to_string()),
                    ..EndpointIpamConfig::default()
                }),
                ..EndpointSettings::default()
            },
        };
        relic::docker::network::connect(docker_client, &id, options).await
    }

    async fn disconnect_network(
        &self,
        _quest: SyncQuest,
        id: NetworkId,
        instance_id: InstanceId,
    ) -> anyhow::Result<()> {
        let docker_client = self.client()?;
        let options = DisconnectNetworkOptions {
            container: instance_id.to_docker_id(),
            force: false,
        };
        relic::docker::network::disconnect(docker_client, &id, options).await
    }

    async fn copy_from_instance(
        &self,
        quest: SyncQuest,
        id: InstanceId,
        src: &Path,
        dst: &Path,
        is_dst_file_path: bool,
    ) -> anyhow::Result<()> {
        Self::copy_from_instance(self.client()?, quest, id, src, dst, is_dst_file_path).await
    }

    async fn copy_to_instance(
        &self,
        quest: SyncQuest,
        id: InstanceId,
        src: &Path,
        dst: &Path,
        is_dst_file_path: bool,
    ) -> anyhow::Result<()> {
        Self::copy_to_instance(self.client()?, quest, id, src, dst, is_dst_file_path).await
    }

    async fn copy_configs_from_instance(
        &self,
        id: InstanceId,
        config_files: &[ConfigFile],
        dst: PathBuf,
    ) -> anyhow::Result<()> {
        let client = self.client()?;
        Self::copy_config_from_instance(client, id, config_files, dst).await
    }

    async fn start_instance(
        &self,
        config: Config<String>,
        id: Option<InstanceId>,
        config_files: &[ConfigFile],
    ) -> anyhow::Result<InstanceId> {
        let client = self.client()?;
        let id = id.unwrap_or_else(InstanceId::new_random);
        let docker_id = id.to_docker_id();
        relic::docker::container::remove(
            client.clone(),
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
            &docker_id,
        )
        .await?;
        let options = Some(CreateContainerOptions {
            name: docker_id,
            platform: None,
        });
        let docker_id = relic::docker::container::create(client.clone(), options, config).await?;
        debug!("Created container {}/{}", id, docker_id);
        if let Err(e) = Self::copy_config_to_instance(client.clone(), id, config_files).await {
            let _ = relic::docker::container::remove(
                client.clone(),
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
                &docker_id,
            )
            .await;
            return Err(e);
        }
        relic::docker::container::start(client, &id.to_docker_id()).await?;
        Ok(id)
    }

    async fn stop_instance(
        &self,
        id: InstanceId,
        config_files: &[ConfigFile],
    ) -> anyhow::Result<()> {
        let client = self.client()?;
        relic::docker::container::stop(client.clone(), &id.to_docker_id(), None).await?;
        Self::copy_config_from_instance(
            client,
            id,
            config_files,
            crate::lore::instance_config_path(&id.to_string()),
        )
        .await?;
        self.delete_instance(id).await?;
        Ok(())
    }
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

    async fn instance_status(&self, id: InstanceId) -> anyhow::Result<InstanceStatus> {
        let docker_client = self.client()?;
        match relic::docker::container::inspect(docker_client, &id.to_docker_id()).await? {
            None => Ok(InstanceStatus::Stopped),
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

    async fn instance_logs(&self, quest: SyncQuest, id: InstanceId) -> anyhow::Result<Logs> {
        let docker_client = self.client()?;
        let (stdout, stderr) =
            relic::docker::container::logs(docker_client, quest, &id.to_docker_id()).await?;
        Ok(Logs { stderr, stdout })
    }
}

#[async_trait]
impl AppDeployment for DockerDeploymentImpl {
    async fn install_app(
        &self,
        quest: SyncQuest,
        manifest: AppManifest,
        token: Option<Token>,
    ) -> anyhow::Result<AppId> {
        let AppManifest::Single(manifest) = manifest else {
            panic!("Docker deployment can not be called with multi app manifests")
        };
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

    async fn uninstall_app(
        &self,
        quest: SyncQuest,
        _manifest: AppManifest,
        id: AppId,
    ) -> anyhow::Result<()> {
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

    async fn is_app_installed(
        &self,
        quest: SyncQuest,
        _manifest: AppManifest,
        id: AppId,
    ) -> anyhow::Result<bool> {
        Ok(self.app_info(quest, id).await.is_ok())
    }

    async fn installed_app_size(
        &self,
        quest: SyncQuest,
        _manifest: AppManifest,
        id: AppId,
    ) -> anyhow::Result<usize> {
        Ok(self
            .app_info(quest, id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("App not installed"))?
            .size
            .ok_or_else(|| anyhow::anyhow!("Size was not specified"))? as usize)
    }

    async fn export_app(
        &self,
        quest: SyncQuest,
        manifest: AppManifest,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        let AppManifest::Single(manifest) = manifest else {
            anyhow::bail!("DockerDeploymentImpl supports only AppManifest::Single");
        };
        let image = manifest.image_with_tag();
        let path = path.join(format!(
            "{}_{}.tar",
            manifest.key.name, manifest.key.version
        ));
        relic::docker::image::save(quest, self.client()?, &path, &image).await
    }

    async fn import_app(
        &self,
        quest: SyncQuest,
        _manifest: AppManifest,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        relic::docker::image::load(
            quest,
            self.client()?,
            &path,
            ImportImageOptions::default(),
            None,
        )
        .await
    }
}

#[async_trait]
impl VolumeDeployment for DockerDeploymentImpl {
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
        src: &Path,
        container_path: &Path,
        name: &str,
        image: &str,
    ) -> anyhow::Result<VolumeId> {
        Self::import_volume_with_client(self.client()?, quest, src, container_path, name, image)
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
        let docker_client = self.client()?;
        Self::export_volume_with_client(
            docker_client,
            quest,
            id,
            export_path,
            container_path,
            image,
        )
        .await
    }
}

#[async_trait]
impl NetworkDeployment for DockerDeploymentImpl {
    async fn create_network(
        &self,
        quest: SyncQuest,
        config: NetworkConfig,
    ) -> Result<Network, CreateNetworkError> {
        let docker_client = self.client()?;
        Self::create_network_with_client(
            docker_client,
            quest,
            config,
            self.network_adapter_reader.as_ref(),
        )
        .await
    }

    async fn default_network(&self) -> Result<jeweler::network::Network, CreateNetworkError> {
        let docker_client = self.client()?;
        Self::default_network_with_client(docker_client, self.network_adapter_reader.as_ref()).await
    }

    async fn delete_network(&self, id: NetworkId) -> anyhow::Result<()> {
        let docker_client = self.client()?;
        relic::docker::network::remove(docker_client, &id).await
    }

    async fn network(&self, id: NetworkId) -> anyhow::Result<Option<Network>> {
        let docker_client = self.client()?;
        relic::docker::network::inspect::<&str>(docker_client, &id, None).await
    }

    async fn networks(&self, _quest: SyncQuest) -> anyhow::Result<Vec<Network>> {
        let docker_client = self.client()?;
        relic::docker::network::list::<String>(docker_client, None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::jeweler::network::{Network, NetworkConfig, NetworkKind};
    use crate::relic::network::Ipv4Network;
    use std::collections::HashMap;
    use std::net::Ipv4Addr;
    use std::str::FromStr;

    fn fitting_network_config_data() -> (Network, NetworkConfig) {
        let config = NetworkConfig {
            kind: NetworkKind::Bridge,
            name: "TestNetwork".to_string(),
            cidr_subnet: Some(Ipv4Network::from_str("10.67.3.0/24").unwrap()),
            gateway: Some(Ipv4Addr::from_str("10.67.3.12").unwrap()),
            parent_adapter: Some("ParentTestNetwork".to_string()),
            options: Some(HashMap::from([
                ("Option1".to_string(), "value 1".to_string()),
                ("Option2".to_string(), "value 2".to_string()),
                ("Option3".to_string(), "value 3".to_string()),
            ])),
        };
        let network = Network {
            name: Some("TestNetwork".to_string()),
            driver: Some("bridge".to_string()),
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![bollard::models::IpamConfig {
                    subnet: Some("10.67.3.0/24".to_string()),
                    gateway: Some("10.67.3.12".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            options: Some(HashMap::from([
                ("parent".to_string(), "ParentTestNetwork".to_string()),
                ("Option1".to_string(), "value 1".to_string()),
                ("Option2".to_string(), "value 2".to_string()),
                ("Option3".to_string(), "value 3".to_string()),
            ])),
            ..Network::default()
        };
        (network, config)
    }

    #[test]
    fn network_config_fits_network_everything_fits() {
        let (network, config) = fitting_network_config_data();
        assert!(DockerDeploymentImpl::network_config_fits_network(&config, &network).unwrap())
    }

    #[test]
    fn network_config_fits_network_false_name() {
        let (network, mut config) = fitting_network_config_data();
        config.name = "Other".to_string();
        assert!(!DockerDeploymentImpl::network_config_fits_network(&config, &network).unwrap())
    }

    #[test]
    fn network_config_fits_network_false_kind() {
        let (network, mut config) = fitting_network_config_data();
        config.kind = NetworkKind::MACVLAN;
        assert!(!DockerDeploymentImpl::network_config_fits_network(&config, &network).unwrap())
    }

    #[test]
    fn network_config_fits_network_false_subnet() {
        let (network, mut config) = fitting_network_config_data();
        config.cidr_subnet = Some(Ipv4Network::from_str("10.20.30.0/24").unwrap());
        assert!(!DockerDeploymentImpl::network_config_fits_network(&config, &network).unwrap())
    }

    #[test]
    fn network_config_fits_network_false_gateway() {
        let (network, mut config) = fitting_network_config_data();
        config.gateway = Some(Ipv4Addr::from_str("10.20.30.40").unwrap());
        assert!(!DockerDeploymentImpl::network_config_fits_network(&config, &network).unwrap())
    }

    #[test]
    fn network_config_fits_network_false_parent() {
        let (network, mut config) = fitting_network_config_data();
        config.parent_adapter = Some("Other".to_string());
        assert!(!DockerDeploymentImpl::network_config_fits_network(&config, &network).unwrap())
    }

    #[test]
    fn network_config_fits_network_false_options() {
        let (network, mut config) = fitting_network_config_data();
        config
            .options
            .as_mut()
            .unwrap()
            .insert("Custom Option".to_string(), "unexpected".to_string());
        assert!(!DockerDeploymentImpl::network_config_fits_network(&config, &network).unwrap())
    }

    #[test]
    fn network_config_fits_network_err_subnet() {
        let (mut network, config) = fitting_network_config_data();
        network
            .ipam
            .as_mut()
            .unwrap()
            .config
            .as_mut()
            .unwrap()
            .push(bollard::models::IpamConfig {
                subnet: Some("invalid".to_string()),
                ..Default::default()
            });
        assert!(DockerDeploymentImpl::network_config_fits_network(&config, &network).is_err())
    }

    #[test]
    fn network_config_fits_network_err_gateway() {
        let (mut network, config) = fitting_network_config_data();
        network
            .ipam
            .as_mut()
            .unwrap()
            .config
            .as_mut()
            .unwrap()
            .push(bollard::models::IpamConfig {
                gateway: Some("invalid".to_string()),
                ..Default::default()
            });
        assert!(DockerDeploymentImpl::network_config_fits_network(&config, &network).is_err())
    }

    #[test]
    fn name_fits_network_true() {
        let network = Network {
            name: Some("TestNetwork".to_string()),
            ..Network::default()
        };
        assert!(DockerDeploymentImpl::name_fits_network(
            &"TestNetwork".to_string(),
            &network
        ));
    }

    #[test]
    fn name_fits_network_false_some() {
        let network = Network {
            name: Some("OtherTestNetwork".to_string()),
            ..Network::default()
        };
        assert!(!DockerDeploymentImpl::name_fits_network(
            &"TestNetwork".to_string(),
            &network
        ));
    }

    #[test]
    fn name_fits_network_false_none() {
        let network = Network {
            name: None,
            ..Network::default()
        };
        assert!(!DockerDeploymentImpl::name_fits_network(
            &"TestNetwork".to_string(),
            &network
        ));
    }

    #[test]
    fn kind_fits_network_true() {
        let network = Network {
            driver: Some("bridge".to_string()),
            ..Network::default()
        };
        assert!(DockerDeploymentImpl::kind_fits_network(
            NetworkKind::Bridge,
            &network
        ));
    }

    #[test]
    fn kind_fits_network_false() {
        let network = Network {
            driver: None,
            ..Network::default()
        };
        assert!(!DockerDeploymentImpl::kind_fits_network(
            NetworkKind::Bridge,
            &network
        ));
    }

    fn subnet_network_data() -> Network {
        Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![
                    bollard::models::IpamConfig {
                        subnet: Some("1.22.223.0/24".to_string()),
                        ..Default::default()
                    },
                    bollard::models::IpamConfig {
                        subnet: Some("44.11.0.0/16".to_string()),
                        ..Default::default()
                    },
                    bollard::models::IpamConfig {
                        subnet: Some("60.0.0.0/12".to_string()),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            }),
            ..Network::default()
        }
    }

    #[test]
    fn subnet_fits_network_true_some() {
        let network = subnet_network_data();
        assert!(
            DockerDeploymentImpl::subnet_fits_network(
                Some(Ipv4Network::from_str("44.11.0.0/16").unwrap()),
                &network
            )
            .unwrap()
        );
    }

    #[test]
    fn subnet_fits_network_true_none() {
        let network = subnet_network_data();
        assert!(DockerDeploymentImpl::subnet_fits_network(None, &network).unwrap());
    }

    #[test]
    fn subnet_fits_network_false() {
        let network = subnet_network_data();
        assert!(
            !DockerDeploymentImpl::subnet_fits_network(
                Some(Ipv4Network::from_str("44.21.0.0/16").unwrap()),
                &network
            )
            .unwrap()
        );
    }

    #[test]
    fn subnet_fits_network_err() {
        let mut network = subnet_network_data();
        network
            .ipam
            .as_mut()
            .unwrap()
            .config
            .as_mut()
            .unwrap()
            .push(bollard::models::IpamConfig {
                subnet: Some("invalid".to_string()),
                ..Default::default()
            });
        assert!(
            DockerDeploymentImpl::subnet_fits_network(
                Some(Ipv4Network::from_str("44.11.0.0/16").unwrap()),
                &network
            )
            .is_err()
        );
    }

    fn gateway_network_data() -> Network {
        Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![
                    bollard::models::IpamConfig {
                        gateway: Some("1.22.223.12".to_string()),
                        ..Default::default()
                    },
                    bollard::models::IpamConfig {
                        gateway: Some("44.11.24.12".to_string()),
                        ..Default::default()
                    },
                    bollard::models::IpamConfig {
                        gateway: Some("60.0.0.1".to_string()),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            }),
            ..Network::default()
        }
    }

    #[test]
    fn gateway_fits_network_true_some() {
        let network = gateway_network_data();
        assert!(
            DockerDeploymentImpl::gateway_fits_network(
                Some(Ipv4Addr::from_str("44.11.24.12").unwrap()),
                &network
            )
            .unwrap()
        );
    }

    #[test]
    fn gateway_fits_network_true_none() {
        let network = gateway_network_data();
        assert!(DockerDeploymentImpl::gateway_fits_network(None, &network).unwrap());
    }

    #[test]
    fn gateway_fits_network_err() {
        let mut network = gateway_network_data();
        network
            .ipam
            .as_mut()
            .unwrap()
            .config
            .as_mut()
            .unwrap()
            .push(bollard::models::IpamConfig {
                gateway: Some("invalid".to_string()),
                ..Default::default()
            });
        assert!(
            DockerDeploymentImpl::gateway_fits_network(
                Some(Ipv4Addr::from_str("44.11.24.12").unwrap()),
                &network
            )
            .is_err()
        );
    }

    fn options_network_data() -> Network {
        Network {
            options: Some(HashMap::from([
                ("parent".to_string(), "ParentTestNetwork".to_string()),
                ("Option1".to_string(), "value 1".to_string()),
                ("Option2".to_string(), "value 2".to_string()),
                ("Option3".to_string(), "value 3".to_string()),
            ])),
            ..Network::default()
        }
    }

    #[test]
    fn options_fit_network_true_all() {
        let network = options_network_data();
        let options = Some(HashMap::from([
            ("parent".to_string(), "ParentTestNetwork".to_string()),
            ("Option1".to_string(), "value 1".to_string()),
            ("Option2".to_string(), "value 2".to_string()),
            ("Option3".to_string(), "value 3".to_string()),
        ]));
        assert!(DockerDeploymentImpl::options_fit_network(
            options.as_ref(),
            &network
        ));
    }

    #[test]
    fn options_fit_network_true_some() {
        let network = options_network_data();
        let options = Some(HashMap::from([
            ("Option1".to_string(), "value 1".to_string()),
            ("Option3".to_string(), "value 3".to_string()),
        ]));
        assert!(DockerDeploymentImpl::options_fit_network(
            options.as_ref(),
            &network
        ));
    }

    #[test]
    fn options_fit_network_true_empty() {
        let network = options_network_data();
        let options = Some(HashMap::new());
        assert!(DockerDeploymentImpl::options_fit_network(
            options.as_ref(),
            &network
        ));
    }

    #[test]
    fn options_fit_network_true_none_some() {
        let network = options_network_data();
        let options = None;
        assert!(DockerDeploymentImpl::options_fit_network(
            options.as_ref(),
            &network
        ));
    }

    #[test]
    fn options_fit_network_true_none_none() {
        let network = Network::default();
        let options = None;
        assert!(DockerDeploymentImpl::options_fit_network(
            options.as_ref(),
            &network
        ));
    }

    #[test]
    fn options_fit_network_true_empty_none() {
        let network = Network::default();
        let options = Some(HashMap::new());
        assert!(DockerDeploymentImpl::options_fit_network(
            options.as_ref(),
            &network
        ));
    }

    #[test]
    fn options_fit_network_false_different_value() {
        let network = options_network_data();
        let options = Some(HashMap::from([
            ("Option1".to_string(), "different".to_string()),
            ("Option3".to_string(), "value 3".to_string()),
        ]));
        assert!(!DockerDeploymentImpl::options_fit_network(
            options.as_ref(),
            &network
        ));
    }

    #[test]
    fn options_fit_network_false_different_key() {
        let network = options_network_data();
        let options = Some(HashMap::from([(
            "DifferentOption".to_string(),
            "value 3".to_string(),
        )]));
        assert!(!DockerDeploymentImpl::options_fit_network(
            options.as_ref(),
            &network
        ));
    }

    #[test]
    fn options_fit_network_false_some_none() {
        let network = Network::default();
        let options = Some(HashMap::from([(
            "Option 1".to_string(),
            "value 1".to_string(),
        )]));
        assert!(!DockerDeploymentImpl::options_fit_network(
            options.as_ref(),
            &network
        ));
    }
}
