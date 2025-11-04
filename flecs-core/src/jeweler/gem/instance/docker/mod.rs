pub mod config;
use super::{InstanceCommon, InstanceId, Logs};
use crate::enchantment::floxy::{AdditionalLocationInfo, Floxy, FloxyOperation};
use crate::forge::bollard::BollardNetworkExtension;
use crate::forge::ipaddr::BitComplementExt;
use crate::forge::time::SystemTimeExt;
use crate::jeweler::deployment::DeploymentId;
use crate::jeweler::gem::deployment::Deployment;
use crate::jeweler::gem::deployment::docker::DockerDeployment;
use crate::jeweler::gem::instance::docker::config::InstancePortMapping;
use crate::jeweler::gem::instance::status::InstanceStatus;
use crate::jeweler::gem::manifest::AppManifest;
use crate::jeweler::gem::manifest::single::{
    AppManifestSingle, BindMount, ConfigFile, Mount, VolumeMount,
};
use crate::jeweler::network::NetworkId;
use crate::jeweler::serialize_deployment_id;
use crate::jeweler::serialize_manifest_key;
use crate::jeweler::volume::VolumeId;
use crate::lore::SPECIAL_CORE_GATEWAY_HOST;
use crate::lore::{InstanceLore, Lore};
use crate::quest::{Quest, SyncQuest};
use crate::relic::device::usb::UsbDeviceReader;
use crate::vault::pouch::AppKey;
use crate::{legacy, lore, vault};
use async_trait::async_trait;
use bollard::container::Config;
use bollard::models::{ContainerStateStatusEnum, DeviceMapping, HostConfig, MountTypeEnum};
use config::InstanceConfig;
use flecsd_axum_server::models;
use flecsd_axum_server::models::{AppInstance, InstancesInstanceIdGet200Response};
use futures_util::future::{BoxFuture, join_all};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::mem::swap;
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tracing::{debug, error, warn};

#[derive(Debug, Serialize)]
pub struct DockerInstance {
    pub id: InstanceId,
    #[serde(serialize_with = "serialize_manifest_key", rename = "app_key")]
    pub manifest: Arc<AppManifestSingle>,
    #[serde(serialize_with = "serialize_deployment_id", rename = "deployment_id")]
    pub deployment: Arc<dyn DockerDeployment>,
    pub name: String,
    pub hostname: String,
    pub config: InstanceConfig,
    desired: InstanceStatus,
    #[serde(skip_serializing)]
    lore: Arc<Lore>,
}

#[derive(thiserror::Error, Debug)]
pub enum TransferIpError {
    #[error("Unknown network {0}")]
    UnknownNetwork(String),
    #[error("Failed to inspect network {network}: {error}")]
    InspectNetwork {
        network: NetworkId,
        error: anyhow::Error,
    },
    #[error("No fitting subnet in {network} to transfer {ip} to")]
    NoFittingNetwork { network: NetworkId, ip: IpAddr },
}

#[async_trait]
impl InstanceCommon for DockerInstance {
    fn id(&self) -> InstanceId {
        self.id
    }

    fn app_key(&self) -> &AppKey {
        &self.manifest.key
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn manifest(&self) -> AppManifest {
        AppManifest::Single(self.manifest.clone())
    }

    fn replace_manifest(&mut self, manifest: AppManifest) -> AppManifest {
        let AppManifest::Single(mut manifest) = manifest else {
            panic!("Can not replace manifest of DockerInstance with {manifest:?}");
        };
        swap(&mut manifest, &mut self.manifest);
        AppManifest::Single(manifest)
    }

    async fn generate_info(&self) -> anyhow::Result<AppInstance> {
        let status = self.status().await?;
        Ok(flecsd_axum_server::models::AppInstance {
            instance_id: format!("{}", self.id),
            instance_name: self.name.clone(),
            app_key: self.app_key().into(),
            status: status.into(),
            desired: self.desired.into(),
            editors: self.instance_editors(),
        })
    }

    async fn generate_detailed_info(&self) -> anyhow::Result<InstancesInstanceIdGet200Response> {
        let status = self.status().await?;
        let config_files = self
            .manifest
            .config_files
            .iter()
            .map(Into::into)
            .collect::<Vec<_>>()
            .into();
        let ports = self.manifest.ports.iter().map(Into::into).collect();
        let volumes = self
            .manifest
            .mounts
            .iter()
            .filter_map(|mount| match mount {
                Mount::Volume(volume) => Some(flecsd_axum_server::models::InstanceDetailVolume {
                    name: volume.name.clone(),
                    path: volume.container_path.to_string_lossy().to_string(),
                }),
                _ => None,
            })
            .collect();
        Ok(
            flecsd_axum_server::models::InstancesInstanceIdGet200Response {
                instance_id: format!("{}", self.id),
                instance_name: self.name.clone(),
                app_key: self.app_key().into(),
                status: status.into(),
                desired: self.desired.into(),
                config_files,
                hostname: self.hostname.clone(),
                ip_address: self
                    .config
                    .connected_networks
                    .values()
                    .next()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                ports,
                volumes,
                editors: self.instance_editors(),
            },
        )
    }

    async fn status(&self) -> anyhow::Result<InstanceStatus> {
        self.deployment.instance_status(self.id).await
    }

    fn desired_status(&self) -> InstanceStatus {
        self.desired
    }

    fn taken_ipv4_addresses(&self) -> Vec<Ipv4Addr> {
        self.config
            .connected_networks
            .values()
            .filter_map(|ip_addr| match ip_addr {
                IpAddr::V4(address) => Some(address),
                _ => None,
            })
            .copied()
            .collect()
    }

    async fn logs(&self) -> anyhow::Result<Logs> {
        self.deployment
            .instance_logs(
                Quest::new_synced(format!("Get logs of instance {}", self.id)),
                self.id,
            )
            .await
    }

    async fn import(&mut self, quest: SyncQuest, src: PathBuf, dst: PathBuf) -> anyhow::Result<()> {
        for (id, ip) in self.config.connected_networks.iter_mut() {
            match self.deployment.network(id.clone()).await.map_err(|error| {
                TransferIpError::InspectNetwork {
                    network: id.clone(),
                    error,
                }
            })? {
                None => return Err(TransferIpError::UnknownNetwork(id.clone()).into()),
                Some(network) => *ip = Self::transfer_ip_address(*ip, &network, id)?,
            }
        }
        let mut results = Vec::new();
        {
            let path = src.join("volumes");
            for volume in self.config.volume_mounts.values() {
                results.push(
                    self.import_volume_quest(
                        &quest,
                        path.clone(),
                        volume.container_path.clone(),
                        volume.name.clone(),
                    )
                    .await,
                )
            }
        }
        for result in join_all(results).await {
            result?;
        }
        for config_file in &self.manifest.config_files {
            let src = src.join("conf").join(&config_file.host_file_name);
            let dst = dst.join("conf");
            tokio::fs::create_dir_all(&dst).await?;
            let dst = dst.join(&config_file.host_file_name);
            tokio::fs::copy(src, dst).await?;
        }
        Ok(())
    }

    async fn halt(&self) -> anyhow::Result<()> {
        DockerInstance::halt(self).await
    }
}

fn bind_mounts_to_bollard_mounts(bind_mounts: &[BindMount]) -> Vec<bollard::models::Mount> {
    bind_mounts
        .iter()
        .map(|bind_mount| bollard::models::Mount {
            typ: Some(MountTypeEnum::BIND),
            source: Some(bind_mount.host_path.to_string_lossy().to_string()),
            target: Some(bind_mount.container_path.to_string_lossy().to_string()),
            ..Default::default()
        })
        .collect()
}

impl From<ContainerStateStatusEnum> for InstanceStatus {
    fn from(value: ContainerStateStatusEnum) -> Self {
        // TBD
        match value {
            ContainerStateStatusEnum::EMPTY => Self::Orphaned,
            ContainerStateStatusEnum::CREATED => Self::Orphaned,
            ContainerStateStatusEnum::RUNNING => Self::Running,
            ContainerStateStatusEnum::PAUSED => Self::Running,
            ContainerStateStatusEnum::RESTARTING => Self::Running,
            ContainerStateStatusEnum::REMOVING => Self::Running,
            ContainerStateStatusEnum::EXITED => Self::Orphaned,
            ContainerStateStatusEnum::DEAD => Self::Orphaned,
        }
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
pub struct DockerInstanceDeserializable {
    pub hostname: String,
    pub name: String,
    pub id: InstanceId,
    pub config: InstanceConfig,
    pub desired: InstanceStatus,
    pub app_key: AppKey,
    pub deployment_id: DeploymentId,
}

impl DockerInstance {
    fn lore(&self) -> &InstanceLore {
        self.lore.as_ref().as_ref()
    }

    fn config_path(&self) -> PathBuf {
        self.lore().instance_config_path(&self.id.to_string())
    }
    pub fn app_key(&self) -> AppKey {
        self.manifest.key.clone()
    }

    pub async fn get_default_network_address(&self) -> crate::Result<Option<IpAddr>> {
        match self
            .deployment
            .default_network(self.lore.clone())
            .await?
            .name
        {
            None => anyhow::bail!("Default network has no name"),
            Some(network_id) => Ok(self.config.connected_networks.get(&network_id).cloned()),
        }
    }

    fn instance_editors(&self) -> Option<models::InstanceEditors> {
        let editors: Vec<_> = self
            .manifest
            .editors()
            .iter()
            .map(|editor| {
                let path_prefix = self
                    .config
                    .editor_path_prefixes
                    .get(&editor.port.get())
                    .cloned();
                models::InstanceEditor {
                    name: editor.name.clone(),
                    port: editor.port.get(),
                    url: lore::FloxyLore::instance_editor_location(self.id, editor.port.get()),
                    path_prefix,
                }
            })
            .collect();
        if editors.is_empty() {
            None
        } else {
            Some(models::InstanceEditors::from(editors))
        }
    }

    pub fn generate_device_mappings(&self) -> Vec<DeviceMapping> {
        self.manifest
            .devices
            .iter()
            .map(|device| DeviceMapping {
                path_on_host: Some(device.path.to_string_lossy().to_string()),
                path_in_container: Some(device.path.to_string_lossy().to_string()),
                cgroup_permissions: Some("rwm".to_string()),
            })
            .chain(self.config.generate_usb_device_mappings())
            .collect()
    }

    pub async fn create_config_file(
        quest: SyncQuest,
        deployment: Arc<dyn DockerDeployment>,
        config_path: PathBuf,
        config_file: ConfigFile,
        manifest: Arc<AppManifestSingle>,
    ) -> crate::Result<()> {
        deployment
            .copy_from_app_image(
                quest,
                manifest.image_with_tag(),
                &config_file.container_file_path,
                &config_path.join(config_file.host_file_name),
                true,
            )
            .await
    }

    pub async fn create_config_files(
        quest: SyncQuest,
        deployment: Arc<dyn DockerDeployment>,
        config_path: PathBuf,
        config_files: Vec<ConfigFile>,
        manifest: Arc<AppManifestSingle>,
    ) -> crate::Result<()> {
        tokio::fs::create_dir_all(&config_path).await?;
        let mut results = Vec::new();
        for config_file in config_files {
            let result = quest
                .lock()
                .await
                .create_sub_quest(
                    format!("Create config file {:?}", config_file.container_file_path),
                    |quest| {
                        Self::create_config_file(
                            quest,
                            deployment.clone(),
                            config_path.clone(),
                            config_file,
                            manifest.clone(),
                        )
                    },
                )
                .await
                .2;
            results.push(result);
        }
        match join_all(results)
            .await
            .into_iter()
            .filter_map(|result| result.err())
            .next()
        {
            Some(error) => {
                tokio::fs::remove_dir_all(&config_path).await?;
                Err(error)
            }
            _ => Ok(()),
        }
    }

    pub async fn create_volumes(
        quest: SyncQuest,
        deployment: Arc<dyn DockerDeployment>,
        volume_mounts: Vec<VolumeMount>,
        instance_id: InstanceId,
    ) -> crate::Result<HashMap<VolumeId, VolumeMount>> {
        let mut results = Vec::new();
        for mut volume_mount in volume_mounts {
            volume_mount.name = format!("{}-{}", instance_id.to_docker_id(), volume_mount.name);
            let result = quest
                .lock()
                .await
                .create_sub_quest(
                    format!(
                        "Create volume {} for instance {instance_id}",
                        volume_mount.name
                    ),
                    |quest| {
                        let deployment = deployment.clone();
                        async move {
                            match deployment
                                .create_volume(quest, volume_mount.name.as_str())
                                .await
                            {
                                Ok(id) => Ok((id, volume_mount)),
                                Err(e) => Err(e),
                            }
                        }
                    },
                )
                .await
                .2;
            results.push(result)
        }
        let results = join_all(results).await;
        let result_count = results.len();
        let volumes: HashMap<VolumeId, VolumeMount> = results
            .into_iter()
            .filter_map(|result| result.ok())
            .collect();
        if volumes.len() != result_count {
            let volume_ids = volumes.keys().cloned().collect();
            let result = quest
                .lock()
                .await
                .create_sub_quest(
                    "Delete all created volumes, as an error occurred".to_string(),
                    |quest| {
                        crate::jeweler::extension::delete_volumes(
                            quest,
                            deployment.clone(),
                            volume_ids,
                        )
                    },
                )
                .await
                .2;
            result.await?;
            anyhow::bail!("Could not create all volumes for instance {}", instance_id);
        } else {
            Ok(volumes)
        }
    }

    pub async fn try_create_new(
        quest: SyncQuest,
        lore: Arc<Lore>,
        deployment: Arc<dyn DockerDeployment>,
        manifest: Arc<AppManifestSingle>,
        name: String,
        address: IpAddr,
    ) -> anyhow::Result<Self> {
        let instance_id = InstanceId::new_random();
        let tcp_port_mapping = manifest.ports.clone();
        let environment_variables = manifest.environment_variables.clone();
        let config_path = lore.instance.instance_config_path(&instance_id.to_string());
        let default_network_id = deployment
            .default_network(lore.clone())
            .await?
            .name
            .ok_or_else(|| anyhow::anyhow!("Default network has no name"))?;
        let result = quest
            .lock()
            .await
            .create_sub_quest(
                format!(
                    "Create config files for instance {instance_id} of {}",
                    manifest.key
                ),
                |quest| {
                    Self::create_config_files(
                        quest,
                        deployment.clone(),
                        config_path.clone(),
                        manifest.config_files.clone(),
                        manifest.clone(),
                    )
                },
            )
            .await
            .2;
        result.await?;
        let volume_mounts = quest
            .lock()
            .await
            .create_sub_quest(format!("Create volumes for {instance_id}"), |quest| {
                Self::create_volumes(
                    quest,
                    deployment.clone(),
                    manifest.volume_mounts(),
                    instance_id,
                )
            })
            .await
            .2;
        let volume_mounts = volume_mounts.await;
        let volume_mounts = match volume_mounts {
            Ok(volume_mounts) => volume_mounts,
            Err(e) => {
                fs::remove_dir_all(config_path).await?;
                return Err(e);
            }
        };
        let config = InstanceConfig {
            environment_variables,
            port_mapping: InstancePortMapping {
                tcp: tcp_port_mapping,
                udp: vec![],
                sctp: vec![],
            },
            volume_mounts,
            connected_networks: HashMap::from([(default_network_id, address)]),
            usb_devices: HashMap::new(),
            mapped_editor_ports: Default::default(),
            editor_path_prefixes: manifest.default_editor_path_prefixes(),
        };
        Ok(Self {
            hostname: format!("flecs-{instance_id}"),
            id: instance_id,
            deployment,
            name,
            manifest,
            config,
            desired: InstanceStatus::Stopped,
            lore,
        })
    }

    pub async fn start<F: Floxy>(&mut self, floxy: Arc<FloxyOperation<F>>) -> anyhow::Result<()> {
        self.desired = InstanceStatus::Running;
        self.resume(floxy).await
    }

    pub async fn resume<F: Floxy>(&self, floxy: Arc<FloxyOperation<F>>) -> anyhow::Result<()> {
        if self.desired != InstanceStatus::Running || self.is_running().await? {
            return Ok(());
        }
        self.load_reverse_proxy_config(floxy.clone()).await?;
        self.load_additional_locations_reverse_proxy_config(floxy)?;
        self.deployment
            .start_instance(
                self.lore.clone(),
                self.container_config().await,
                Some(self.id),
                &self.manifest.config_files,
            )
            .await?;
        Ok(())
    }

    fn get_reverse_proxy_editor_ports(&self) -> Vec<u16> {
        self.manifest
            .editors()
            .iter()
            .filter_map(|editor| {
                if editor.supports_reverse_proxy {
                    Some(editor.port.get())
                } else {
                    None
                }
            })
            .collect()
    }

    pub async fn load_reverse_proxy_config<F: Floxy>(
        &self,
        floxy: Arc<FloxyOperation<F>>,
    ) -> anyhow::Result<()> {
        let editor_ports = self.get_reverse_proxy_editor_ports();
        if !editor_ports.is_empty() {
            if let Some(instance_ip) = self.get_default_network_address().await? {
                floxy.add_instance_reverse_proxy_config(
                    &self.app_key().name,
                    self.id,
                    instance_ip,
                    &editor_ports,
                )?;
            }
        }
        Ok(())
    }

    pub fn load_additional_locations_reverse_proxy_config<F: Floxy>(
        &self,
        floxy: Arc<FloxyOperation<F>>,
    ) -> anyhow::Result<()> {
        let path_prefixes = &self.config.editor_path_prefixes;
        if !path_prefixes.is_empty() {
            let locations: Vec<_> = path_prefixes
                .iter()
                .map(|(port, prefix)| AdditionalLocationInfo {
                    location: format!("/{prefix}"),
                    port: *port,
                })
                .collect();
            floxy.add_additional_locations_proxy_config(
                &self.app_key().name,
                self.id,
                &locations,
            )?;
        } else {
            self.delete_additional_locations_reverse_proxy_config(floxy)?;
        }
        Ok(())
    }

    fn delete_additional_locations_reverse_proxy_config<F: Floxy>(
        &self,
        floxy: Arc<FloxyOperation<F>>,
    ) -> anyhow::Result<()> {
        floxy.delete_additional_locations_proxy_config(&self.app_key().name, self.id)
    }

    fn delete_reverse_proxy_config<F: Floxy>(
        &self,
        floxy: Arc<FloxyOperation<F>>,
    ) -> anyhow::Result<()> {
        floxy.delete_reverse_proxy_config(&self.app_key().name, self.id)
    }

    pub fn delete_server_proxy_configs<F: Floxy>(
        &mut self,
        floxy: Arc<FloxyOperation<F>>,
    ) -> anyhow::Result<bool> {
        let editor_ports: Vec<_> = std::mem::take(&mut self.config.mapped_editor_ports)
            .into_values()
            .collect();
        if !editor_ports.is_empty() {
            floxy.delete_server_proxy_configs(&self.app_key().name, self.id, &editor_ports)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn stop_and_delete<F: Floxy + 'static>(
        mut self,
        quest: SyncQuest,
        floxy: Arc<FloxyOperation<F>>,
    ) -> anyhow::Result<(), (anyhow::Error, Self)> {
        let result = quest
            .lock()
            .await
            .create_infallible_sub_quest(
                format!(
                    "Stop instance {} of app {} before deleting it",
                    self.id,
                    self.app_key()
                ),
                |_quest| {
                    let floxy = floxy.clone();
                    async move {
                        match self.stop(floxy).await {
                            Ok(()) => (self, None),
                            Err(e) => {
                                _quest.lock().await.fail_with_error(&e);
                                (self, Some(e))
                            }
                        }
                    }
                },
            )
            .await
            .2
            .await;
        match result {
            (instance, None) => {
                quest
                    .lock()
                    .await
                    .create_infallible_sub_quest(
                        format!(
                            "Delete instance {} of app {}",
                            instance.id,
                            instance.app_key()
                        ),
                        |quest| async move { instance.delete(quest, floxy.clone()).await },
                    )
                    .await
                    .2
                    .await
            }
            (mut instance, Some(e)) => {
                instance.desired = InstanceStatus::NotCreated;
                Err((e, instance))
            }
        }
    }

    pub async fn stop<F: Floxy>(&mut self, floxy: Arc<FloxyOperation<F>>) -> anyhow::Result<()> {
        self.desired = InstanceStatus::Stopped;
        self.halt().await?;
        if let Err(e) = self.delete_server_proxy_configs(floxy) {
            warn!("Instance {}: {e}", self.id);
        }
        Ok(())
    }

    pub async fn halt(&self) -> anyhow::Result<()> {
        // TODO: Disconnect networks
        match self.deployment.instance_status(self.id).await? {
            InstanceStatus::Running | InstanceStatus::Unknown | InstanceStatus::Orphaned => {
                self.deployment
                    .stop_instance(self.id, self.lore.clone(), &self.manifest.config_files)
                    .await
            }
            _ => Ok(()),
        }
    }

    pub async fn delete<F: Floxy>(
        mut self,
        quest: SyncQuest,
        floxy: Arc<FloxyOperation<F>>,
    ) -> anyhow::Result<(), (anyhow::Error, Self)> {
        self.desired = InstanceStatus::NotCreated;
        let mut volume_ids = Vec::new();
        let mut delete_results = Vec::new();
        for volume_id in self.config.volume_mounts.keys() {
            let result = quest
                .lock()
                .await
                .create_sub_quest(format!("Delete volume {volume_id}"), |quest| {
                    let deployment = self.deployment.clone();
                    let volume_id = volume_id.clone();
                    async move { deployment.clone().delete_volume(quest, volume_id).await }
                })
                .await
                .2;
            volume_ids.push(volume_id.clone());
            delete_results.push(result);
        }
        if let Err(e) = self.delete_reverse_proxy_config(floxy.clone()) {
            warn!("Instance {}: {e}", self.id);
        }
        if let Err(e) = self.delete_additional_locations_reverse_proxy_config(floxy) {
            warn!("Instance {}: {e}", self.id);
        }
        for (id, result) in volume_ids.into_iter().zip(join_all(delete_results).await) {
            if let Err(e) = result {
                warn!("Could not delete volume {id} of instance {}: {e}", self.id);
            }
        }
        let result = quest
            .lock()
            .await
            .create_sub_quest(format!("Delete instance {}", self.id), |_quest| {
                let deployment = self.deployment.clone();
                async move { deployment.clone().delete_instance(self.id).await }
            })
            .await
            .2;
        result.await.map_err(|e| (e, self))?;
        Ok(())
    }

    pub async fn export_config_files(
        id: InstanceId,
        config_files: Vec<ConfigFile>,
        src: PathBuf,
        dst: PathBuf,
    ) -> anyhow::Result<()> {
        debug!(
            "Export config files of stopped instance {} to {}",
            id,
            dst.display()
        );
        tokio::fs::create_dir_all(&dst).await?;
        for result in join_all(config_files.iter().map(|config_file| {
            let src = src.join(&config_file.host_file_name);
            let dst = dst.join(&config_file.host_file_name);
            tokio::fs::copy(src, dst)
        }))
        .await
        {
            result?;
        }
        Ok(())
    }

    pub async fn copy_from(
        &self,
        quest: SyncQuest,
        src: &Path,
        dst: &Path,
        is_dst_file_path: bool,
    ) -> anyhow::Result<()> {
        self.deployment
            .copy_from_instance(quest, self.id, src, dst, is_dst_file_path)
            .await
    }

    pub async fn copy_to(
        &self,
        quest: SyncQuest,
        src: &Path,
        dst: &Path,
        is_dst_file_path: bool,
    ) -> anyhow::Result<()> {
        self.deployment
            .copy_to_instance(quest, self.id, src, dst, is_dst_file_path)
            .await
    }

    pub async fn is_running(&self) -> anyhow::Result<bool> {
        Ok(self.deployment.instance_status(self.id).await? == InstanceStatus::Running)
    }

    pub fn deployment(&self) -> Arc<dyn DockerDeployment> {
        self.deployment.clone()
    }

    pub async fn disconnect_network(
        &mut self,
        network_id: NetworkId,
    ) -> crate::Result<Option<IpAddr>> {
        let disconnect_result = if self.is_running().await? {
            self.deployment
                .disconnect_network(
                    Quest::new_synced(format!("Disconnect {} from network {network_id}", self.id)),
                    network_id.clone(),
                    self.id,
                )
                .await
        } else {
            Ok(())
        };
        match (
            disconnect_result,
            self.config.connected_networks.entry(network_id),
        ) {
            (_, Entry::Vacant(_)) => Ok(None),
            (Ok(()), Entry::Occupied(entry)) => Ok(Some(entry.remove())),
            (Err(e), Entry::Occupied(_)) => Err(e),
        }
    }

    pub async fn connect_network(
        &mut self,
        network_id: NetworkId,
        address: Ipv4Addr,
    ) -> crate::Result<Option<IpAddr>> {
        let previous_address = if self.is_running().await? {
            let previous_address = match self.disconnect_network(network_id.clone()).await {
                Err(e) => {
                    error!("Failed to disconnect {} from {network_id}: {e}", self.id);
                    None
                }
                Ok(previous_address) => previous_address,
            };
            self.deployment
                .connect_network(
                    Quest::new_synced(format!("Connect {} to network {network_id}", self.id)),
                    network_id.clone(),
                    address,
                    self.id,
                )
                .await?;
            previous_address
        } else {
            self.config.connected_networks.get(&network_id).copied()
        };
        self.config
            .connected_networks
            .insert(network_id, IpAddr::V4(address));
        Ok(previous_address)
    }

    pub async fn import_volume_quest(
        &self,
        quest: &SyncQuest,
        src: PathBuf,
        container_path: PathBuf,
        volume_name: String,
    ) -> BoxFuture<'static, crate::Result<VolumeId>> {
        let deployment = self.deployment();
        let image = self.manifest.image_with_tag();
        quest
            .lock()
            .await
            .create_sub_quest(format!("Import volume {volume_name}"), |quest| async move {
                deployment
                    .import_volume(quest, &src, &container_path, &volume_name, &image)
                    .await
            })
            .await
            .2
    }

    pub fn try_create_with_state(
        lore: Arc<Lore>,
        instance: DockerInstanceDeserializable,
        manifests: &vault::pouch::manifest::Gems,
        deployments: &vault::pouch::deployment::Gems,
    ) -> anyhow::Result<DockerInstance> {
        let manifest = manifests
            .get(&instance.app_key)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No manifest for instance {} of {} found",
                    instance.id,
                    instance.app_key
                )
            })?
            .clone();
        let AppManifest::Single(manifest) = manifest else {
            anyhow::bail!("DockerInstances can only be created with AppManifestSingle");
        };
        let deployment = deployments
            .get(&instance.deployment_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Deployment {} of instance {} for app {} does not exist",
                    instance.deployment_id,
                    instance.id,
                    instance.app_key
                )
            })?
            .clone();
        let Deployment::Docker(deployment) = deployment else {
            anyhow::bail!("DockerInstances can only be created with DockerDeployments");
        };
        Ok(Self::create(lore, instance, manifest, deployment))
    }

    pub fn create(
        lore: Arc<Lore>,
        instance: DockerInstanceDeserializable,
        manifest: Arc<AppManifestSingle>,
        deployment: Arc<dyn DockerDeployment>,
    ) -> DockerInstance {
        Self {
            manifest,
            deployment,
            desired: instance.desired,
            id: instance.id,
            config: instance.config,
            name: instance.name,
            hostname: instance.hostname,
            lore,
        }
    }

    pub async fn try_create_from_legacy<U: UsbDeviceReader>(
        lore: Arc<Lore>,
        instance: legacy::deployment::Instance,
        usb_device_reader: &U,
        manifest: Arc<AppManifestSingle>,
        deployment: Arc<dyn DockerDeployment>,
    ) -> anyhow::Result<Self> {
        let instance = legacy::deployment::migrate_docker_instance(
            instance,
            usb_device_reader,
            deployment.id(),
        )?;
        let mut instance = Self::create(lore, instance, manifest.clone(), deployment.clone());
        instance.config.volume_mounts = Self::create_volumes(
            Quest::new_synced("Create volumes"),
            deployment,
            manifest.volume_mounts(),
            instance.id,
        )
        .await?;
        Ok(instance)
    }

    async fn export_volumes_quest<'a, I: Iterator<Item = &'a VolumeMount>>(
        quest: &SyncQuest,
        volume_mounts: I,
        volumes_dst: PathBuf,
        image: String,
        deployment: &Arc<dyn DockerDeployment>,
        id: InstanceId,
    ) -> Vec<BoxFuture<'static, crate::Result<()>>> {
        let mut export_volumes_results = Vec::new();
        for volume_mount in volume_mounts {
            let result = quest
                .lock()
                .await
                .create_sub_quest(
                    format!("Export volume {} of instance {}", volume_mount.name, id),
                    |quest| {
                        let volume_name = volume_mount.name.clone();
                        let dst = volumes_dst.clone();
                        let src = volume_mount.container_path.clone();
                        let image = image.clone();
                        let deployment = deployment.clone();
                        async move {
                            deployment
                                .export_volume(quest, volume_name, &dst, &src, &image)
                                .await
                        }
                    },
                )
                .await
                .2;
            export_volumes_results.push(result);
        }
        export_volumes_results
    }

    pub async fn export<F: Floxy>(
        &self,
        quest: SyncQuest,
        floxy: Arc<FloxyOperation<F>>,
        path: &Path,
    ) -> anyhow::Result<()> {
        let is_running = self.is_running().await?;
        if is_running {
            self.halt().await?;
        }
        let export_config_files_result = quest
            .lock()
            .await
            .create_sub_quest(
                format!("Export config files of instance {}", self.id),
                |_quest| {
                    Self::export_config_files(
                        self.id,
                        self.manifest.config_files.clone(),
                        self.lore().instance_config_path(&self.id.to_string()),
                        path.join("conf"),
                    )
                },
            )
            .await
            .2;
        let export_config_files_result = export_config_files_result.await;

        let volumes_dst = path.join("volumes");
        let export_volumes_results = Self::export_volumes_quest(
            &quest,
            self.config.volume_mounts.values(),
            volumes_dst,
            self.manifest.image_with_tag(),
            &self.deployment,
            self.id,
        )
        .await;
        let export_volumes_result = join_all(export_volumes_results).await;
        if is_running {
            if let Err(e) = self.resume(floxy).await {
                error!(
                    "Failed to restart instance {} after exporting config files and volumes: {e}",
                    self.id
                );
            }
        }
        export_config_files_result?;
        for result in export_volumes_result {
            result?;
        }
        Ok(())
    }

    pub async fn update<F: Floxy>(
        &mut self,
        quest: SyncQuest,
        floxy: Arc<FloxyOperation<F>>,
        new_manifest: Arc<AppManifestSingle>,
        base_path: &Path,
    ) -> anyhow::Result<()> {
        let is_running = self.is_running().await?;
        if is_running {
            self.halt().await?;
        }
        let now = std::time::SystemTime::now();
        let backup_path = base_path.join("backup");
        let new_backup_path = backup_path
            .join(self.app_key().version)
            .join(now.unix_millis().to_string());
        let export_config_files_result = quest
            .lock()
            .await
            .create_sub_quest(
                format!("Export config files of instance {}", self.id),
                |_quest| {
                    Self::export_config_files(
                        self.id,
                        self.manifest.config_files.clone(),
                        self.config_path(),
                        new_backup_path.join("conf"),
                    )
                },
            )
            .await
            .2;
        let export_config_files_result = export_config_files_result.await;
        let volumes_dst = new_backup_path.join("volumes");
        let export_volumes_results = Self::export_volumes_quest(
            &quest,
            self.config.volume_mounts.values(),
            volumes_dst,
            self.manifest.image_with_tag(),
            &self.deployment,
            self.id,
        )
        .await;
        let export_volumes_result = join_all(export_volumes_results).await;
        export_config_files_result?;
        for result in export_volumes_result {
            result?;
        }
        let current_version = self.app_key().version;
        let new_version = new_manifest.key.version.clone();
        self.manifest = new_manifest;
        if current_version > new_version {
            let mut entries = tokio::fs::read_dir(backup_path.join(&new_version)).await?;
            let mut latest_backup = None;
            while let Some(entry) = entries.next_entry().await? {
                match &latest_backup {
                    None => latest_backup = Some(entry.path()),
                    Some(current) => {
                        if entry.path() > *current {
                            latest_backup = Some(entry.path());
                        }
                    }
                }
            }
            if let Some(backup) = latest_backup {
                self.import(quest, backup, base_path.to_path_buf()).await?;
            }
        }
        // TODO: Prepare/use new objects from manifest (config files, ports, envs)
        if is_running {
            self.start(floxy).await?;
        }
        Ok(())
    }

    fn transfer_ip_address(
        current: IpAddr,
        network: &bollard::models::Network,
        id: &NetworkId,
    ) -> Result<IpAddr, TransferIpError> {
        match current {
            IpAddr::V4(ipv4) => match network.subnet_ipv4() {
                Err(error) => Err(TransferIpError::InspectNetwork {
                    network: id.clone(),
                    error,
                }),
                Ok(None) => Err(TransferIpError::NoFittingNetwork {
                    network: id.clone(),
                    ip: current,
                }),
                Ok(Some(network)) => {
                    let subnet_mask = network.subnet_mask();
                    // Set network part to 0
                    let ip = ipv4 & subnet_mask.complement();
                    // Use network part from new network
                    Ok(IpAddr::from(network.address() | ip))
                }
            },
            IpAddr::V6(ipv6) => match network.subnet_ipv6() {
                Err(error) => Err(TransferIpError::InspectNetwork {
                    network: id.clone(),
                    error,
                }),
                Ok(None) => Err(TransferIpError::NoFittingNetwork {
                    network: id.clone(),
                    ip: current,
                }),
                Ok(Some(network)) => {
                    let subnet_mask = network.subnet_mask();
                    // Set network part to 0
                    let ip = ipv6 & subnet_mask.complement();
                    // Use network part from new network
                    Ok(IpAddr::from(network.address() | ip))
                }
            },
        }
    }

    async fn container_config(&self) -> Config<String> {
        let mut bind_mounts = self.manifest.bind_mounts();
        let mut capabilities = self.manifest.capabilities();
        if capabilities
            .remove(&flecs_app_manifest::generated::manifest_3_1_0::CapabilitiesItem::Docker)
        {
            bind_mounts.push(BindMount::default_docker_socket_bind_mount());
        }
        let mut mounts = bind_mounts_to_bollard_mounts(bind_mounts.as_slice());
        mounts.extend(self.config.generate_volume_mounts());
        let arguments = self.manifest.arguments();
        let cmd = if arguments.is_empty() {
            None
        } else {
            Some(arguments.clone())
        };
        let port_bindings = self.config.generate_port_bindings();
        let exposed_ports = Some(
            port_bindings
                .keys()
                .cloned()
                .map(|key| (key, HashMap::new()))
                .collect(),
        );
        let extra_host = match self
            .deployment
            .core_default_address(self.lore.clone())
            .await
        {
            Some(ip) => format!("{SPECIAL_CORE_GATEWAY_HOST}:{ip}"),
            None => format!("{SPECIAL_CORE_GATEWAY_HOST}:host-gateway"),
        };
        let host_config = Some(HostConfig {
            port_bindings: Some(port_bindings),
            mounts: Some(mounts),
            cap_add: Some(capabilities.iter().map(ToString::to_string).collect()),
            devices: Some(self.generate_device_mappings()),
            extra_hosts: Some(vec![extra_host]),
            ..HostConfig::default()
        });
        let mut network_config = self.config.generate_network_config();
        if let Some(hostname) = self.manifest.hostname() {
            for endpoint in network_config.endpoints_config.values_mut() {
                let alias = hostname.clone();
                match &mut endpoint.aliases {
                    Some(aliases) => {
                        aliases.push(alias);
                    }
                    None => endpoint.aliases = Some(vec![alias]),
                }
            }
        }
        Config {
            image: Some(self.manifest.image_with_tag().to_string()),
            hostname: Some(self.hostname.clone()),
            env: Some(
                self.config
                    .environment_variables
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            ),
            labels: Some(
                self.manifest
                    .labels
                    .iter()
                    .map(|label| (label.label.clone(), label.value.clone().unwrap_or_default()))
                    .collect(),
            ),
            host_config,
            cmd,
            exposed_ports,
            networking_config: Some(network_config),
            ..Default::default()
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::enchantment::floxy::MockFloxy;
    use crate::jeweler::gem::deployment::docker::tests::MockedDockerDeployment;
    use crate::jeweler::gem::instance::Instance;
    use crate::jeweler::gem::instance::docker::config::UsbPathConfig;
    use crate::jeweler::gem::manifest::single::tests::{
        create_test_manifest, create_test_manifest_full, create_test_manifest_numbered,
    };
    use crate::jeweler::gem::manifest::single::{EnvironmentVariable, PortMapping, PortRange};
    use crate::quest::Quest;
    use crate::relic::device::usb::tests::prepare_usb_device_test_path;
    use crate::relic::var::test::MockVarReader;
    use crate::tests::prepare_test_path;
    use crate::vault::pouch::instance::tests::{NETWORK_INSTANCE, get_test_instance};
    use bollard::secret::Network;
    use flecsd_axum_server::models::{
        InstanceDetailConfigFile, InstanceDetailConfigFiles, InstanceDetailPort,
        InstanceDetailVolume,
    };
    use mockall::predicate;
    use ntest::test_case;
    use std::fs::File;
    use std::io::Write;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::num::IntErrorKind;
    use std::path::PathBuf;
    use std::str::FromStr;
    use testdir::testdir;

    #[test]
    fn try_create_ok() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let manifest = create_test_manifest(None);
        let app_key = manifest.key().clone();
        let manifests = HashMap::from([(app_key.clone(), manifest)]);
        let deployment_id = "TestDeployment".to_string();
        let deployment = Deployment::Docker(Arc::new(MockedDockerDeployment::new()));
        let deployments = HashMap::from([(deployment_id.clone(), deployment)]);
        let instance_id = InstanceId::new(10);
        let instance = DockerInstanceDeserializable {
            deployment_id,
            id: instance_id,
            app_key,
            name: "TestInstance".to_string(),
            desired: InstanceStatus::Running,
            config: InstanceConfig::default(),
            hostname: format!("flecs-{instance_id}"),
        };
        DockerInstance::try_create_with_state(lore, instance, &manifests, &deployments).unwrap();
    }

    #[test]
    fn try_create_no_deployment() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let manifest = create_test_manifest(None);
        let app_key = manifest.key().clone();
        let manifests = HashMap::from([(app_key.clone(), manifest)]);
        let deployment_id = "TestDeployment".to_string();
        let deployments = HashMap::new();
        let instance_id = InstanceId::new(10);
        let instance = DockerInstanceDeserializable {
            deployment_id,
            id: instance_id,
            app_key,
            name: "TestInstance".to_string(),
            desired: InstanceStatus::Running,
            config: InstanceConfig::default(),
            hostname: format!("flecs-{instance_id}"),
        };
        assert!(
            DockerInstance::try_create_with_state(lore, instance, &manifests, &deployments)
                .is_err()
        );
    }

    #[test]
    fn try_create_no_manifest() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let manifest = create_test_manifest(None);
        let app_key = manifest.key().clone();
        let manifests = HashMap::new();
        let deployment_id = "TestDeployment".to_string();
        let deployment = Deployment::Docker(Arc::new(MockedDockerDeployment::new()));
        let deployments = HashMap::from([(deployment_id.clone(), deployment)]);
        let instance_id = InstanceId::new(10);
        let instance = DockerInstanceDeserializable {
            deployment_id,
            id: instance_id,
            app_key,
            name: "TestInstance".to_string(),
            desired: InstanceStatus::Running,
            config: InstanceConfig::default(),
            hostname: format!("flecs-{instance_id}"),
        };
        assert!(
            DockerInstance::try_create_with_state(lore, instance, &manifests, &deployments)
                .is_err()
        );
    }

    pub fn test_instance(
        id: u32,
        lore: Arc<Lore>,
        deployment: Arc<dyn DockerDeployment>,
        manifest: Arc<AppManifestSingle>,
    ) -> DockerInstance {
        DockerInstance {
            lore,
            id: InstanceId::new(id),
            desired: InstanceStatus::Stopped,
            name: "TestInstance".to_string(),
            hostname: format!("flecs-{id:08x}"),
            config: InstanceConfig {
                volume_mounts: HashMap::from([
                    (
                        format!("Instance#{id}Volume#1"),
                        VolumeMount {
                            name: format!("{id}-Volume#1"),
                            container_path: PathBuf::from("/volume1"),
                        },
                    ),
                    (
                        format!("Instance#{id}Volume#2"),
                        VolumeMount {
                            name: format!("{id}-Volume#2"),
                            container_path: PathBuf::from("/volume2"),
                        },
                    ),
                    (
                        format!("Instance#{id}Volume#3"),
                        VolumeMount {
                            name: format!("{id}-Volume#3"),
                            container_path: PathBuf::from("/volume3"),
                        },
                    ),
                    (
                        format!("Instance#{id}Volume#4"),
                        VolumeMount {
                            name: format!("{id}-Volume#4"),
                            container_path: PathBuf::from("/volume4"),
                        },
                    ),
                ]),
                editor_path_prefixes: Default::default(),
                environment_variables: vec![
                    EnvironmentVariable::from_str("variable-1=value1").unwrap(),
                    EnvironmentVariable::from_str("variable-2=").unwrap(),
                    EnvironmentVariable::from_str("variable-3").unwrap(),
                ],
                port_mapping: InstancePortMapping {
                    tcp: vec![
                        PortMapping::Single(1002, 2002),
                        PortMapping::Range {
                            from: PortRange::try_new(8000, 9000).unwrap(),
                            to: PortRange::try_new(9500, 10500).unwrap(),
                        },
                    ],
                    udp: vec![
                        PortMapping::Single(1004, 2004),
                        PortMapping::Range {
                            from: PortRange::try_new(9000, 10000).unwrap(),
                            to: PortRange::try_new(10500, 11500).unwrap(),
                        },
                    ],
                    sctp: vec![
                        PortMapping::Single(1003, 2003),
                        PortMapping::Range {
                            from: PortRange::try_new(10000, 11000).unwrap(),
                            to: PortRange::try_new(11500, 12500).unwrap(),
                        },
                    ],
                },
                connected_networks: HashMap::new(),
                usb_devices: HashMap::from([
                    (
                        "test_instance_dev_1".to_string(),
                        UsbPathConfig {
                            port: "test_instance_dev_1".to_string(),
                            bus_num: 456,
                            dev_num: 789,
                        },
                    ),
                    (
                        "test_instance_dev_2".to_string(),
                        UsbPathConfig {
                            port: "test_instance_dev_2".to_string(),
                            bus_num: 200,
                            dev_num: 300,
                        },
                    ),
                ]),
                mapped_editor_ports: Default::default(),
            },
            deployment,
            manifest,
        }
    }

    #[tokio::test]
    async fn delete_ok() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_reverse_proxy_config()
            .once()
            .returning(|_, _| Ok(false));
        floxy
            .expect_delete_additional_locations_proxy_config()
            .returning(|_, _| Ok(false));
        let floxy = FloxyOperation::new_arc(Arc::new(floxy));
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_delete_instance()
            .times(1)
            .returning(|_| Ok(true));
        deployment
            .expect_delete_volume()
            .times(4)
            .returning(|_, _| Ok(()));
        test_instance(
            1,
            lore,
            Arc::new(deployment),
            create_test_manifest_full(None),
        )
        .delete(Quest::new_synced("TestQuest".to_string()), floxy)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn delete_volume_err() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_reverse_proxy_config()
            .returning(|_, _| Ok(false));
        floxy
            .expect_delete_additional_locations_proxy_config()
            .returning(|_, _| Ok(false));
        let floxy = FloxyOperation::new_arc(Arc::new(floxy));
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_delete_instance()
            .times(1)
            .returning(|_| Ok(true));
        deployment
            .expect_delete_volume()
            .times(4)
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        let AppManifest::Single(manifest) = create_test_manifest(None) else {
            panic!()
        };
        test_instance(2, lore, Arc::new(deployment), manifest)
            .delete(Quest::new_synced("TestQuest".to_string()), floxy)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn delete_err() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_reverse_proxy_config()
            .returning(|_, _| Ok(false));
        floxy
            .expect_delete_additional_locations_proxy_config()
            .returning(|_, _| Ok(false));
        let floxy = FloxyOperation::new_arc(Arc::new(floxy));
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_delete_instance()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        deployment
            .expect_delete_volume()
            .times(4)
            .returning(|_, _| Ok(()));
        let AppManifest::Single(manifest) = create_test_manifest(None) else {
            panic!()
        };
        let (_error, instance) = test_instance(3, lore, Arc::new(deployment), manifest)
            .delete(Quest::new_synced("TestQuest".to_string()), floxy)
            .await
            .err()
            .unwrap();
        assert_eq!(instance.desired, InstanceStatus::NotCreated);
    }

    #[tokio::test]
    async fn halt_err() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_reverse_proxy_config()
            .returning(|_, _| Ok(false));
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_stop_instance()
            .times(1)
            .returning(|_, _, _| Err(anyhow::anyhow!("TestError")));
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Ok(InstanceStatus::Running));
        let AppManifest::Single(manifest) = create_test_manifest(None) else {
            panic!()
        };
        let mut instance = test_instance(4, lore, Arc::new(deployment), manifest);
        instance.desired = InstanceStatus::Running;
        assert!(instance.halt().await.is_err());
        assert_eq!(instance.desired, InstanceStatus::Running);
    }

    #[tokio::test]
    async fn halt_ok() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_stop_instance()
            .times(1)
            .returning(|_, _, _| Ok(()));
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Ok(InstanceStatus::Running));
        let mut instance = test_instance(
            5,
            lore,
            Arc::new(deployment),
            create_test_manifest_full(None),
        );
        instance.config.mapped_editor_ports = HashMap::from([(10, 100)]);
        instance.desired = InstanceStatus::Running;
        assert!(instance.halt().await.is_ok());
        assert_eq!(instance.desired, InstanceStatus::Running);
    }

    #[tokio::test]
    async fn halt_stopped_ok() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut deployment = MockedDockerDeployment::new();
        deployment.expect_stop_instance().times(0);
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Ok(InstanceStatus::Stopped));
        let AppManifest::Single(manifest) = create_test_manifest(None) else {
            panic!()
        };
        let mut instance = test_instance(6, lore, Arc::new(deployment), manifest);
        instance.desired = InstanceStatus::Running;
        assert!(instance.halt().await.is_ok());
        assert_eq!(instance.desired, InstanceStatus::Running);
    }

    #[tokio::test]
    async fn stop_sets_desired() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut deployment = MockedDockerDeployment::new();
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        deployment.expect_stop_instance().times(0);
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Ok(InstanceStatus::Stopped));
        let AppManifest::Single(manifest) = create_test_manifest(None) else {
            panic!()
        };
        let mut instance = test_instance(6, lore, Arc::new(deployment), manifest);
        instance.desired = InstanceStatus::Running;
        assert!(instance.stop(floxy).await.is_ok());
        assert_eq!(instance.desired, InstanceStatus::Stopped);
    }

    #[tokio::test]
    async fn stop_and_delete_ok() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_reverse_proxy_config()
            .returning(|_, _| Ok(false));
        floxy
            .expect_delete_additional_locations_proxy_config()
            .returning(|_, _| Ok(false));
        let floxy = FloxyOperation::new_arc(Arc::new(floxy));
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_stop_instance()
            .times(1)
            .returning(|_, _, _| Ok(()));
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Ok(InstanceStatus::Running));
        deployment
            .expect_delete_instance()
            .times(1)
            .returning(|_| Ok(true));
        deployment
            .expect_delete_volume()
            .times(4)
            .returning(|_, _| Ok(()));
        let AppManifest::Single(manifest) = create_test_manifest(None) else {
            panic!()
        };
        assert!(
            test_instance(7, lore, Arc::new(deployment), manifest)
                .stop_and_delete(Quest::new_synced("TestQuest".to_string()), floxy)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn stop_and_delete_delete_err() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_reverse_proxy_config()
            .returning(|_, _| Ok(false));
        floxy
            .expect_delete_additional_locations_proxy_config()
            .returning(|_, _| Ok(false));
        let floxy = FloxyOperation::new_arc(Arc::new(floxy));
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_stop_instance()
            .times(1)
            .returning(|_, _, _| Ok(()));
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Ok(InstanceStatus::Running));
        deployment
            .expect_delete_instance()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        deployment
            .expect_delete_volume()
            .times(4)
            .returning(|_, _| Ok(()));
        let AppManifest::Single(manifest) = create_test_manifest(None) else {
            panic!()
        };
        let mut instance = test_instance(8, lore, Arc::new(deployment), manifest);
        instance.desired = InstanceStatus::Running;
        let (_error, instance) = instance
            .stop_and_delete(Quest::new_synced("TestQuest".to_string()), floxy)
            .await
            .err()
            .unwrap();
        assert_eq!(instance.desired, InstanceStatus::NotCreated);
    }

    #[tokio::test]
    async fn stop_and_delete_stop_err() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_reverse_proxy_config()
            .returning(|_, _| Ok(false));
        let floxy = FloxyOperation::new_arc(Arc::new(floxy));
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_stop_instance()
            .times(1)
            .returning(|_, _, _| Err(anyhow::anyhow!("TestError")));
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Ok(InstanceStatus::Running));
        deployment.expect_delete_instance().times(0);
        deployment.expect_delete_volume().times(0);
        let AppManifest::Single(manifest) = create_test_manifest(None) else {
            panic!()
        };
        let mut instance = test_instance(9, lore, Arc::new(deployment), manifest);
        instance.desired = InstanceStatus::Running;
        let (_error, instance) = instance
            .stop_and_delete(Quest::new_synced("TestQuest".to_string()), floxy)
            .await
            .err()
            .unwrap();
        assert_eq!(instance.desired, InstanceStatus::NotCreated);
    }

    #[test]
    fn instance_id_from_str() {
        assert_eq!(Ok(InstanceId::new(0)), InstanceId::from_str("0"));
        assert_eq!(
            Ok(InstanceId::new(0x01a55555)),
            InstanceId::from_str("01a55555")
        );
        assert_eq!(
            &IntErrorKind::InvalidDigit,
            InstanceId::from_str("invalid").err().unwrap().kind()
        );
        assert_eq!(
            &IntErrorKind::PosOverflow,
            InstanceId::from_str("1a2b3c4d5e6f").err().unwrap().kind()
        );
        assert_eq!(
            &IntErrorKind::Empty,
            InstanceId::from_str("").err().unwrap().kind()
        );
    }

    #[tokio::test]
    async fn create_config_file_ok() {
        let path = prepare_test_path(module_path!(), "create_config_file_ok").join("config");
        let manifest = create_test_manifest_full(None);
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_copy_from_app_image()
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));
        let deployment: Arc<dyn DockerDeployment> = Arc::new(deployment);
        let config_file = ConfigFile {
            host_file_name: "test.config".to_string(),
            container_file_path: PathBuf::from("/tmp/flecs-test.config"),
            read_only: false,
        };
        DockerInstance::create_config_file(
            Quest::new_synced("TestQuest".to_string()),
            deployment,
            path,
            config_file,
            manifest,
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn create_config_file_err() {
        let path = prepare_test_path(module_path!(), "create_config_file_err").join("config");
        let manifest = create_test_manifest_full(None);
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_copy_from_app_image()
            .times(1)
            .returning(|_, _, _, _, _| Err(anyhow::anyhow!("TestError")));
        let deployment: Arc<dyn DockerDeployment> = Arc::new(deployment);
        let config_file = ConfigFile {
            host_file_name: "test.config".to_string(),
            container_file_path: PathBuf::from("/tmp/flecs-test.config"),
            read_only: false,
        };
        assert!(
            DockerInstance::create_config_file(
                Quest::new_synced("TestQuest".to_string()),
                deployment,
                path,
                config_file,
                manifest,
            )
            .await
            .is_err()
        )
    }

    #[tokio::test]
    async fn create_config_files_ok() {
        let path = prepare_test_path(module_path!(), "create_config_files_ok").join("config");
        let manifest = create_test_manifest_full(None);
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_copy_from_app_image()
            .times(3)
            .returning(|_, _, _, _, _| Ok(()));
        let deployment: Arc<dyn DockerDeployment> = Arc::new(deployment);
        DockerInstance::create_config_files(
            Quest::new_synced("TestQuest".to_string()),
            deployment,
            path.clone(),
            manifest.config_files.clone(),
            manifest,
        )
        .await
        .unwrap();
        assert!(path.try_exists().unwrap());
    }

    #[tokio::test]
    async fn create_config_files_can_not_create_path() {
        let path = prepare_test_path(module_path!(), "create_config_files_can_not_create_path")
            .join("config");
        {
            let mut file = File::create(&path).unwrap();
            file.write_all(b"TestData").unwrap();
        }
        assert!(path.try_exists().unwrap());
        let manifest = create_test_manifest_full(None);
        let deployment: Arc<dyn DockerDeployment> = Arc::new(MockedDockerDeployment::new());
        assert!(
            DockerInstance::create_config_files(
                Quest::new_synced("TestQuest".to_string()),
                deployment,
                path.clone(),
                manifest.config_files.clone(),
                manifest,
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn create_config_files_err() {
        let path = prepare_test_path(module_path!(), "create_config_files_err").join("config");
        let manifest = create_test_manifest_full(None);
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_copy_from_app_image()
            .times(3)
            .returning(|_, _, _, _, _| Err(anyhow::anyhow!("TestError")));
        let deployment: Arc<dyn DockerDeployment> = Arc::new(deployment);
        assert!(
            DockerInstance::create_config_files(
                Quest::new_synced("TestQuest".to_string()),
                deployment,
                path.clone(),
                manifest.config_files.clone(),
                manifest,
            )
            .await
            .is_err()
        );
        assert!(!path.try_exists().unwrap());
    }

    #[tokio::test]
    async fn create_ok() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let manifest = create_test_manifest_full(None);
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_copy_from_app_image()
            .times(3)
            .returning(|_, _, _, _, _| Ok(()));
        deployment.expect_default_network().times(1).returning(|_| {
            Ok(Network {
                name: Some("DefaultTestNetworkId".to_string()),
                ..Network::default()
            })
        });
        deployment
            .expect_create_volume()
            .times(1)
            .returning(|_, _| Ok("TestVolumeId".to_string()));
        let deployment: Arc<dyn DockerDeployment> = Arc::new(deployment);
        let address = IpAddr::V4(Ipv4Addr::new(123, 123, 123, 123));
        let instance = DockerInstance::try_create_new(
            Quest::new_synced("TestQuest".to_string()),
            lore,
            deployment,
            manifest.clone(),
            "TestInstance".to_string(),
            address,
        )
        .await
        .unwrap();
        assert_eq!(&instance.config.port_mapping.tcp, &manifest.ports);
        assert_eq!(instance.desired, InstanceStatus::Stopped);
        assert_eq!(
            &instance.config.environment_variables,
            &manifest.environment_variables
        );
        assert_eq!(
            &instance.config.connected_networks,
            &HashMap::from([("DefaultTestNetworkId".to_string(), address)])
        );
        assert_eq!(
            &instance.config.volume_mounts,
            &HashMap::from([(
                "TestVolumeId".to_string(),
                VolumeMount {
                    name: format!("flecs-{}-my-app-etc", instance.id),
                    container_path: PathBuf::from("/etc/my-app")
                }
            )])
        )
    }

    #[tokio::test]
    async fn create_create_config_fails() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let manifest = create_test_manifest_full(None);
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_copy_from_app_image()
            .times(3)
            .returning(|_, _, _, _, _| Err(anyhow::anyhow!("TestError")));
        deployment.expect_default_network().times(1).returning(|_| {
            Ok(Network {
                name: Some("DefaultTestNetworkId".to_string()),
                ..Network::default()
            })
        });
        let deployment: Arc<dyn DockerDeployment> = Arc::new(deployment);
        assert!(
            DockerInstance::try_create_new(
                Quest::new_synced("TestQuest".to_string()),
                lore,
                deployment,
                manifest.clone(),
                "TestInstance".to_string(),
                IpAddr::V4(Ipv4Addr::new(123, 123, 123, 123)),
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn create_default_network_without_id() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let manifest = create_test_manifest_full(None);
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_default_network()
            .times(1)
            .returning(|_| Ok(Network::default()));
        let deployment: Arc<dyn DockerDeployment> = Arc::new(deployment);
        assert!(
            DockerInstance::try_create_new(
                Quest::new_synced("TestQuest".to_string()),
                lore,
                deployment,
                manifest.clone(),
                "TestInstance".to_string(),
                IpAddr::V4(Ipv4Addr::new(123, 123, 123, 123)),
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn create_default_network_err() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let manifest = create_test_manifest_full(None);
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_default_network()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("TestError").into()));
        let deployment: Arc<dyn DockerDeployment> = Arc::new(deployment);
        assert!(
            DockerInstance::try_create_new(
                Quest::new_synced("TestQuest".to_string()),
                lore,
                deployment,
                manifest.clone(),
                "TestInstance".to_string(),
                IpAddr::V4(Ipv4Addr::new(123, 123, 123, 123)),
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn create_instance_info_ok() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Ok(InstanceStatus::Running));
        let deployment = Arc::new(deployment);
        let manifest = create_test_manifest_full(Some(true));
        let instance_id = InstanceId::new(0x123);
        let instance = DockerInstance {
            lore,
            name: "TestInstance".to_string(),
            hostname: format!("flecs-{instance_id}"),
            id: instance_id,
            manifest,
            deployment,
            config: Default::default(),
            desired: InstanceStatus::Running,
        };
        let expected_info = flecsd_axum_server::models::AppInstance {
            instance_id: "00000123".to_string(),
            instance_name: "TestInstance".to_string(),
            app_key: flecsd_axum_server::models::AppKey {
                name: "some.test.app".to_string(),
                version: "1.2.1".to_string(),
            },
            status: flecsd_axum_server::models::InstanceStatus::Running,
            desired: flecsd_axum_server::models::InstanceStatus::Running,
            editors: Some(models::InstanceEditors::from(vec![
                models::InstanceEditor {
                    name: "Editor#1".to_string(),
                    port: 123,
                    path_prefix: None,
                    url: "/v2/instances/00000123/editor/123".to_string(),
                },
                models::InstanceEditor {
                    name: "Editor#2".to_string(),
                    port: 789,
                    path_prefix: None,
                    url: "/v2/instances/00000123/editor/789".to_string(),
                },
            ])),
        };
        assert_eq!(instance.generate_info().await.unwrap(), expected_info);
    }

    #[tokio::test]
    async fn create_instance_info_err() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let deployment = Arc::new(deployment);
        let manifest = create_test_manifest_full(Some(true));
        let instance_id = InstanceId::new(0x123);
        let instance = DockerInstance {
            lore,
            name: "TestInstance".to_string(),
            hostname: format!("flecs-{instance_id}"),
            id: instance_id,
            manifest,
            deployment,
            config: Default::default(),
            desired: InstanceStatus::Running,
        };
        assert!(instance.generate_info().await.is_err());
    }

    #[tokio::test]
    async fn create_instance_info_details_ok() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Ok(InstanceStatus::Running));
        let deployment = Arc::new(deployment);
        let manifest = create_test_manifest_full(Some(true));
        let instance_id = InstanceId::new(0x123);
        let instance = DockerInstance {
            lore,
            name: "TestInstance".to_string(),
            hostname: format!("flecs-{instance_id}"),
            id: instance_id,
            manifest,
            deployment,
            config: InstanceConfig {
                connected_networks: HashMap::from([(
                    "TestNetwork".to_string(),
                    IpAddr::V4(Ipv4Addr::new(123, 123, 123, 123)),
                )]),
                ..Default::default()
            },
            desired: InstanceStatus::Running,
        };
        let expected_info = flecsd_axum_server::models::InstancesInstanceIdGet200Response {
            instance_id: "00000123".to_string(),
            instance_name: "TestInstance".to_string(),
            app_key: flecsd_axum_server::models::AppKey {
                name: "some.test.app".to_string(),
                version: "1.2.1".to_string(),
            },
            status: flecsd_axum_server::models::InstanceStatus::Running,
            desired: flecsd_axum_server::models::InstanceStatus::Running,
            config_files: InstanceDetailConfigFiles::from(vec![
                InstanceDetailConfigFile {
                    host: "default.conf".to_string(),
                    container: "/etc/my-app/default.conf".to_string(),
                },
                InstanceDetailConfigFile {
                    host: "default.conf".to_string(),
                    container: "/etc/my-app/default.conf".to_string(),
                },
                InstanceDetailConfigFile {
                    host: "default.conf".to_string(),
                    container: "/etc/my-app/default.conf".to_string(),
                },
            ]),
            hostname: "flecs-00000123".to_string(),
            ip_address: "123.123.123.123".to_string(),
            ports: vec![
                InstanceDetailPort {
                    host: "8001".to_string(),
                    container: "8001".to_string(),
                },
                InstanceDetailPort {
                    host: "5000".to_string(),
                    container: "5000".to_string(),
                },
                InstanceDetailPort {
                    host: "5001-5008".to_string(),
                    container: "6001-6008".to_string(),
                },
                InstanceDetailPort {
                    host: "6001-6008".to_string(),
                    container: "6001-6008".to_string(),
                },
            ],
            volumes: vec![InstanceDetailVolume {
                name: "my-app-etc".to_string(),
                path: "/etc/my-app".to_string(),
            }],
            editors: Some(models::InstanceEditors::from(vec![
                models::InstanceEditor {
                    name: "Editor#1".to_string(),
                    port: 123,
                    path_prefix: None,
                    url: "/v2/instances/00000123/editor/123".to_string(),
                },
                models::InstanceEditor {
                    name: "Editor#2".to_string(),
                    port: 789,
                    path_prefix: None,
                    url: "/v2/instances/00000123/editor/789".to_string(),
                },
            ])),
        };
        assert_eq!(
            instance.generate_detailed_info().await.unwrap(),
            expected_info
        );
    }

    #[tokio::test]
    async fn create_instance_info_details_err() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let deployment = Arc::new(deployment);
        let manifest = create_test_manifest_full(Some(true));
        let instance_id = InstanceId::new(0x123);
        let instance = DockerInstance {
            lore,
            name: "TestInstance".to_string(),
            hostname: format!("flecs-{instance_id}"),
            id: instance_id,
            manifest,
            deployment,
            config: Default::default(),
            desired: InstanceStatus::Running,
        };
        assert!(instance.generate_detailed_info().await.is_err());
    }

    #[test]
    fn to_docker_id() {
        assert_eq!(
            InstanceId::new(0x2468).to_docker_id(),
            "flecs-00002468".to_string()
        );
    }

    #[tokio::test]
    async fn config_from_instance() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        prepare_usb_device_test_path("test_instance_dev_1");
        prepare_usb_device_test_path("test_instance_dev_2");
        let mut deployment = MockedDockerDeployment::new();
        deployment.expect_core_default_address().returning(|_| None);
        let deployment = Arc::new(deployment);
        let manifest = create_test_manifest_full(Some(true));
        let mut instance = test_instance(123, lore, deployment, manifest);
        instance.config.connected_networks.insert(
            "Ipv4Network".to_string(),
            IpAddr::V4(Ipv4Addr::new(20, 22, 24, 26)),
        );
        instance.config.connected_networks.insert(
            "Ipv6Network".to_string(),
            IpAddr::V6(Ipv6Addr::new(
                0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
            )),
        );
        let config = instance.container_config().await;
        assert_eq!(
            config.image,
            Some("flecs.azurecr.io/some.test.app:1.2.1".to_string())
        );
        let host_config = config.host_config.unwrap();
        assert_eq!(host_config.port_bindings.unwrap().len(), 3006);
        let mounts = host_config.mounts.unwrap();
        assert_eq!(mounts.len(), 6);
        assert!(mounts.contains(&bollard::models::Mount {
            typ: Some(MountTypeEnum::BIND),
            source: Some("/etc/my-app".to_string()),
            target: Some("/etc/my-app".to_string()),
            ..Default::default()
        }));
        assert!(mounts.contains(&bollard::models::Mount {
            typ: Some(MountTypeEnum::BIND),
            source: Some("/run/docker.sock".to_string()),
            target: Some("/run/docker.sock".to_string()),
            ..Default::default()
        }));
        for i in 1..=4 {
            assert!(mounts.contains(&bollard::models::Mount {
                typ: Some(MountTypeEnum::VOLUME),
                source: Some(format!("Instance#123Volume#{i}")),
                target: Some(format!("/volume{i}")),
                ..Default::default()
            }));
        }
        let caps = host_config.cap_add.unwrap();
        assert_eq!(caps.len(), 2);
        assert!(caps.contains(&"SYS_NICE".to_string()));
        assert!(caps.contains(&"NET_ADMIN".to_string()));
        assert_eq!(
            config.cmd,
            Some(vec![
                "--launch-arg1".to_string(),
                "--launch-arg2=value".to_string(),
            ])
        );
        assert_eq!(config.hostname, Some("flecs-0000007b".to_string()));
        assert_eq!(
            config.env,
            Some(vec![
                "variable-1=value1".to_string(),
                "variable-2=".to_string(),
                "variable-3".to_string()
            ])
        );
        assert_eq!(
            config.labels,
            Some(HashMap::from([
                ("my.label-one".to_string(), "value-1".to_string()),
                ("my.label-two".to_string(), String::new()),
                ("my.label-three".to_string(), String::new()),
            ]))
        );
        let expected_device_mappings = vec![
            DeviceMapping {
                path_on_host: Some("/dev/dev1".to_string()),
                path_in_container: Some("/dev/dev1".to_string()),
                cgroup_permissions: Some("rwm".to_string()),
            },
            DeviceMapping {
                path_on_host: Some("/dev/usb/dev1".to_string()),
                path_in_container: Some("/dev/usb/dev1".to_string()),
                cgroup_permissions: Some("rwm".to_string()),
            },
            DeviceMapping {
                path_on_host: Some("/dev/bus/usb/456/789".to_string()),
                path_in_container: Some("/dev/bus/usb/456/789".to_string()),
                cgroup_permissions: Some("rwm".to_string()),
            },
            DeviceMapping {
                path_on_host: Some("/dev/bus/usb/200/300".to_string()),
                path_in_container: Some("/dev/bus/usb/200/300".to_string()),
                cgroup_permissions: Some("rwm".to_string()),
            },
        ];
        let resulting_device_mappings = host_config.devices.unwrap();
        assert_eq!(
            resulting_device_mappings.len(),
            expected_device_mappings.len()
        );
        for expected_device_mapping in expected_device_mappings {
            assert!(
                resulting_device_mappings.contains(&expected_device_mapping),
                "{resulting_device_mappings:#?} does not contain {expected_device_mapping:#?}"
            );
        }
        assert_eq!(
            config.networking_config,
            Some(bollard::container::NetworkingConfig {
                endpoints_config: HashMap::from([
                    (
                        "Ipv6Network".to_string(),
                        bollard::models::EndpointSettings {
                            ip_address: Some("11:22:33:44:55:66:77:88".to_string()),
                            ipam_config: Some(bollard::models::EndpointIpamConfig {
                                ipv6_address: Some("11:22:33:44:55:66:77:88".to_string()),
                                ..Default::default()
                            }),
                            aliases: Some(vec!["TestHostName".to_string()]),
                            ..Default::default()
                        }
                    ),
                    (
                        "Ipv4Network".to_string(),
                        bollard::models::EndpointSettings {
                            ip_address: Some("20.22.24.26".to_string()),
                            ipam_config: Some(bollard::models::EndpointIpamConfig {
                                ipv4_address: Some("20.22.24.26".to_string()),
                                ..Default::default()
                            }),
                            aliases: Some(vec!["TestHostName".to_string()]),
                            ..Default::default()
                        }
                    ),
                ]),
            })
        )
    }

    #[test]
    fn instance_status_from_container_status() {
        assert_eq!(
            InstanceStatus::from(ContainerStateStatusEnum::EMPTY),
            InstanceStatus::Orphaned
        );
        assert_eq!(
            InstanceStatus::from(ContainerStateStatusEnum::CREATED),
            InstanceStatus::Orphaned
        );
        assert_eq!(
            InstanceStatus::from(ContainerStateStatusEnum::RUNNING),
            InstanceStatus::Running
        );
        assert_eq!(
            InstanceStatus::from(ContainerStateStatusEnum::PAUSED),
            InstanceStatus::Running
        );
        assert_eq!(
            InstanceStatus::from(ContainerStateStatusEnum::RESTARTING),
            InstanceStatus::Running
        );
        assert_eq!(
            InstanceStatus::from(ContainerStateStatusEnum::REMOVING),
            InstanceStatus::Running
        );
        assert_eq!(
            InstanceStatus::from(ContainerStateStatusEnum::EXITED),
            InstanceStatus::Orphaned
        );
        assert_eq!(
            InstanceStatus::from(ContainerStateStatusEnum::DEAD),
            InstanceStatus::Orphaned
        );
    }

    #[test]
    fn server_status_from_instance_status() {
        assert_eq!(
            flecsd_axum_server::models::InstanceStatus::NotCreated,
            InstanceStatus::NotCreated.into()
        );
        assert_eq!(
            flecsd_axum_server::models::InstanceStatus::Requested,
            InstanceStatus::Requested.into()
        );
        assert_eq!(
            flecsd_axum_server::models::InstanceStatus::ResourcesReady,
            InstanceStatus::ResourcesReady.into()
        );
        assert_eq!(
            flecsd_axum_server::models::InstanceStatus::Stopped,
            InstanceStatus::Stopped.into()
        );
        assert_eq!(
            flecsd_axum_server::models::InstanceStatus::Running,
            InstanceStatus::Running.into()
        );
        assert_eq!(
            flecsd_axum_server::models::InstanceStatus::Orphaned,
            InstanceStatus::Orphaned.into()
        );
        assert_eq!(
            flecsd_axum_server::models::InstanceStatus::Unknown,
            InstanceStatus::Unknown.into()
        );
    }

    #[tokio::test]
    async fn create_volumes_ok() {
        let instance_id = InstanceId::new(0x1234);
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_create_volume()
            .times(3)
            .returning(|_, name| Ok(name.to_string()));
        let deployment: Arc<dyn DockerDeployment> = Arc::new(deployment);
        let volumes = vec![
            VolumeMount {
                name: "Volume1".to_string(),
                container_path: PathBuf::from("/Volume1"),
            },
            VolumeMount {
                name: "Volume2".to_string(),
                container_path: PathBuf::from("/Volume2"),
            },
            VolumeMount {
                name: "Volume3".to_string(),
                container_path: PathBuf::from("/Volume3"),
            },
        ];
        let expected_volumes = HashMap::from([
            (
                format!("flecs-{instance_id}-Volume1"),
                VolumeMount {
                    name: format!("flecs-{instance_id}-Volume1"),
                    container_path: PathBuf::from("/Volume1"),
                },
            ),
            (
                format!("flecs-{instance_id}-Volume2"),
                VolumeMount {
                    name: format!("flecs-{instance_id}-Volume2"),
                    container_path: PathBuf::from("/Volume2"),
                },
            ),
            (
                format!("flecs-{instance_id}-Volume3"),
                VolumeMount {
                    name: format!("flecs-{instance_id}-Volume3"),
                    container_path: PathBuf::from("/Volume3"),
                },
            ),
        ]);
        assert_eq!(
            DockerInstance::create_volumes(
                Quest::new_synced("TestQuest".to_string()),
                deployment,
                volumes,
                instance_id,
            )
            .await
            .unwrap(),
            expected_volumes
        );
    }

    #[tokio::test]
    async fn create_volumes_err() {
        let mut deployment = MockedDockerDeployment::new();
        let instance_id = InstanceId::new(0x1234);
        deployment
            .expect_create_volume()
            .times(1)
            .withf(move |_, name| name == format!("flecs-{instance_id}-Volume1"))
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        deployment
            .expect_create_volume()
            .times(2)
            .returning(|_, name| Ok(name.to_string()));
        deployment
            .expect_delete_volume()
            .times(1)
            .withf(move |_, id| id == &format!("flecs-{instance_id}-Volume2"))
            .returning(|_, _| Ok(()));
        deployment
            .expect_delete_volume()
            .times(1)
            .withf(move |_, id| id == &format!("flecs-{instance_id}-Volume3"))
            .returning(|_, _| Ok(()));
        let deployment: Arc<dyn DockerDeployment> = Arc::new(deployment);
        let volumes = vec![
            VolumeMount {
                name: "Volume1".to_string(),
                container_path: PathBuf::from("/Volume1"),
            },
            VolumeMount {
                name: "Volume2".to_string(),
                container_path: PathBuf::from("/Volume2"),
            },
            VolumeMount {
                name: "Volume3".to_string(),
                container_path: PathBuf::from("/Volume3"),
            },
        ];
        assert!(
            DockerInstance::create_volumes(
                Quest::new_synced("TestQuest".to_string()),
                deployment,
                volumes,
                instance_id,
            )
            .await
            .is_err()
        );
    }

    #[test]
    fn replace_manifest() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let AppManifest::Single(manifest) = create_test_manifest(None) else {
            panic!()
        };
        let mut instance =
            test_instance(2, lore, Arc::new(MockedDockerDeployment::new()), manifest);
        let manifest = create_test_manifest(Some("#2".to_string()));
        assert_eq!(instance.manifest.revision(), None);
        let old_manifest = instance.replace_manifest(manifest);
        assert_eq!(old_manifest.revision(), None);
        assert_eq!(instance.manifest.revision(), Some(&"#2".to_string()));
    }

    #[tokio::test]
    async fn instance_start_ok_already_running() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_instance_status()
            .once()
            .withf(|id| id.value == 2)
            .returning(|_| Ok(InstanceStatus::Running));
        let AppManifest::Single(manifest) = create_test_manifest(None) else {
            panic!()
        };
        let mut instance = test_instance(2, lore, Arc::new(deployment), manifest);
        instance.desired = InstanceStatus::Stopped;
        instance.start(floxy).await.unwrap();
        assert_eq!(instance.desired, InstanceStatus::Running);
    }

    #[tokio::test]
    async fn instance_start_ok() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_server_proxy_configs()
            .returning(|_, _, _| Ok(false));
        floxy
            .expect_delete_additional_locations_proxy_config()
            .returning(|_, _| Ok(false));
        let floxy = FloxyOperation::new_arc(Arc::new(floxy));
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_instance_status()
            .once()
            .withf(|id| id.value == 2)
            .returning(|_| Ok(InstanceStatus::Stopped));
        deployment
            .expect_start_instance()
            .once()
            .withf(|_, _, id, _| id == &Some(InstanceId::new(2)))
            .returning(|_, _, _, _| Ok(InstanceId::new(2)));
        deployment.expect_core_default_address().returning(|_| None);
        let AppManifest::Single(manifest) = create_test_manifest(None) else {
            panic!()
        };
        let mut instance = test_instance(2, lore, Arc::new(deployment), manifest);
        instance.desired = InstanceStatus::Stopped;
        instance.start(floxy).await.unwrap();
        assert_eq!(instance.desired, InstanceStatus::Running);
    }

    #[tokio::test]
    async fn instance_load_reverse_proxy_config_ok() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut deployment = MockedDockerDeployment::new();
        deployment.expect_default_network().returning(|_| {
            Ok(Network {
                name: Some("flecs".to_string()),
                ..Default::default()
            })
        });
        let mut floxy = MockFloxy::new();
        floxy
            .expect_add_instance_reverse_proxy_config()
            .once()
            .withf(|app, id, ip, ports| {
                app == "some.test.app"
                    && id == &InstanceId::new(2)
                    && ip == &IpAddr::V4(Ipv4Addr::new(125, 20, 20, 20))
                    && ports == [789]
            })
            .returning(|_, _, _, _| Ok(false));
        let floxy = FloxyOperation::new_arc(Arc::new(floxy));
        let mut instance = test_instance(
            2,
            lore,
            Arc::new(deployment),
            create_test_manifest_full(None),
        );
        instance.config.connected_networks.insert(
            "flecs".to_string(),
            IpAddr::V4(Ipv4Addr::new(125, 20, 20, 20)),
        );
        instance.load_reverse_proxy_config(floxy).await.unwrap();
    }

    #[tokio::test]
    async fn instance_load_reverse_proxy_config_err() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_default_network()
            .returning(|_| Err(anyhow::anyhow!("TestError").into()));
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        let instance = test_instance(
            2,
            lore,
            Arc::new(deployment),
            create_test_manifest_full(None),
        );
        assert!(instance.load_reverse_proxy_config(floxy).await.is_err());
    }

    #[tokio::test]
    async fn instance_load_reverse_proxy_config_ok_no_editors() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let AppManifest::Single(manifest) = create_test_manifest_numbered(1, 2, None) else {
            panic!()
        };
        let instance = test_instance(2, lore, Arc::new(MockedDockerDeployment::new()), manifest);
        instance
            .load_reverse_proxy_config(FloxyOperation::new_arc(Arc::new(MockFloxy::new())))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn instance_load_reverse_proxy_config_ok_no_address() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut deployment = MockedDockerDeployment::new();
        deployment.expect_default_network().returning(|_| {
            Ok(Network {
                name: Some("flecs".to_string()),
                ..Default::default()
            })
        });
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        let instance = test_instance(
            2,
            lore,
            Arc::new(deployment),
            create_test_manifest_full(None),
        );
        instance.load_reverse_proxy_config(floxy).await.unwrap();
    }

    #[tokio::test]
    async fn instance_delete_reverse_proxy_config_ok() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_reverse_proxy_config()
            .once()
            .withf(|app, id| app == "some.test.app" && id == &InstanceId::new(2))
            .returning(|_, _| Ok(false));
        let floxy = FloxyOperation::new_arc(Arc::new(floxy));
        let instance = test_instance(
            2,
            lore,
            Arc::new(MockedDockerDeployment::new()),
            create_test_manifest_full(None),
        );
        instance.delete_reverse_proxy_config(floxy).unwrap();
    }

    #[tokio::test]
    async fn instance_delete_server_proxy_configs_ok() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_server_proxy_configs()
            .once()
            .withf(|app, id, ports| {
                app == "some.test.app"
                    && id == &InstanceId::new(2)
                    && ports.contains(&1000)
                    && ports.contains(&20)
            })
            .returning(|_, _, _| Ok(false));
        floxy
            .expect_delete_reverse_proxy_config()
            .returning(|_, _| Ok(false));
        floxy
            .expect_delete_additional_locations_proxy_config()
            .returning(|_, _| Ok(false));
        let floxy = FloxyOperation::new_arc(Arc::new(floxy));
        let mut instance = test_instance(
            2,
            lore,
            Arc::new(MockedDockerDeployment::new()),
            create_test_manifest_full(None),
        );
        instance.config.mapped_editor_ports = HashMap::from([(10, 20), (100, 1000)]);
        assert!(instance.delete_server_proxy_configs(floxy).unwrap());
    }

    #[tokio::test]
    async fn instance_delete_server_proxy_configs_ok_no_mapped_ports() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_reverse_proxy_config()
            .returning(|_, _| Ok(false));
        let floxy = FloxyOperation::new_arc(Arc::new(floxy));
        let mut instance = test_instance(
            2,
            lore,
            Arc::new(MockedDockerDeployment::new()),
            create_test_manifest_full(None),
        );
        instance.config.mapped_editor_ports.clear();
        assert!(!instance.delete_server_proxy_configs(floxy).unwrap());
    }

    fn disconnect_test_instance(
        disconnect_mock_result: Option<crate::Result<()>>,
        status_mock_result: Option<crate::Result<InstanceStatus>>,
        connected_networks: HashMap<String, IpAddr>,
    ) -> DockerInstance {
        const INSTANCE_ID: InstanceId = InstanceId::new(10);
        let AppManifest::Single(manifest) =
            crate::vault::pouch::manifest::tests::min_app_1_1_0_manifest()
        else {
            panic!()
        };
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut deployment = MockedDockerDeployment::new();
        if let Some(mock_result) = disconnect_mock_result {
            deployment
                .expect_disconnect_network()
                .once()
                .with(
                    predicate::always(),
                    predicate::eq("TestNetwork".to_string()),
                    predicate::eq(INSTANCE_ID),
                )
                .return_once(|_, _, _| mock_result);
        }
        if let Some(mock_result) = status_mock_result {
            deployment
                .expect_instance_status()
                .with(predicate::eq(INSTANCE_ID))
                .return_once(|_| mock_result);
        }
        DockerInstance {
            lore,
            name: "TestInstance".to_string(),
            hostname: INSTANCE_ID.to_docker_id(),
            id: INSTANCE_ID,
            config: InstanceConfig {
                connected_networks,
                ..Default::default()
            },
            deployment: Arc::new(deployment),
            manifest,
            desired: InstanceStatus::Running,
        }
    }

    #[tokio::test]
    async fn instance_disconnect_network_unknown_ok() {
        let mut instance = disconnect_test_instance(
            Some(Ok(())),
            Some(Ok(InstanceStatus::Running)),
            HashMap::new(),
        );
        assert_eq!(
            instance
                .disconnect_network("TestNetwork".to_string())
                .await
                .unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn instance_disconnect_network_unknown_err() {
        let mut instance = disconnect_test_instance(
            Some(Err(anyhow::anyhow!("TestError"))),
            Some(Ok(InstanceStatus::Running)),
            HashMap::new(),
        );
        assert_eq!(
            instance
                .disconnect_network("TestNetwork".to_string())
                .await
                .unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn instance_disconnect_network_ok() {
        let ip_address = IpAddr::V4(Ipv4Addr::new(10, 20, 30, 40));
        let mut instance = disconnect_test_instance(
            Some(Ok(())),
            Some(Ok(InstanceStatus::Running)),
            HashMap::from([("TestNetwork".to_string(), ip_address)]),
        );
        assert_eq!(
            instance
                .disconnect_network("TestNetwork".to_string())
                .await
                .unwrap(),
            Some(ip_address)
        );
    }

    #[tokio::test]
    async fn instance_disconnect_network_err_disconnect() {
        let ip_address = IpAddr::V4(Ipv4Addr::new(10, 20, 30, 40));
        let mut instance = disconnect_test_instance(
            Some(Err(anyhow::anyhow!("TestError"))),
            Some(Ok(InstanceStatus::Running)),
            HashMap::from([("TestNetwork".to_string(), ip_address)]),
        );
        assert!(
            instance
                .disconnect_network("TestNetwork".to_string())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn instance_disconnect_network_err_status() {
        let ip_address = IpAddr::V4(Ipv4Addr::new(10, 20, 30, 40));
        let mut instance = disconnect_test_instance(
            None,
            Some(Err(anyhow::anyhow!("TestError"))),
            HashMap::from([("TestNetwork".to_string(), ip_address)]),
        );
        assert!(
            instance
                .disconnect_network("TestNetwork".to_string())
                .await
                .is_err()
        );
    }

    #[test]
    fn get_instance_deployment() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("GetMockedDeployment".to_string());
        let instance = test_instance(
            2,
            lore,
            Arc::new(deployment),
            create_test_manifest_full(None),
        );
        assert_eq!(instance.deployment().id(), "GetMockedDeployment");
    }

    #[tokio::test]
    async fn instance_connect_network_running_ok() {
        let ip_address = Ipv4Addr::new(10, 20, 30, 40);
        const NETWORK_NAME: &str = "TestNet";
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_instance_status()
            .with(predicate::eq(NETWORK_INSTANCE))
            .returning(|_| Ok(InstanceStatus::Running));
        deployment
            .expect_disconnect_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(NETWORK_NAME.to_string()),
                predicate::eq(NETWORK_INSTANCE),
            )
            .returning(|_, _, _| Ok(()));
        deployment
            .expect_connect_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(NETWORK_NAME.to_string()),
                predicate::eq(ip_address),
                predicate::eq(NETWORK_INSTANCE),
            )
            .returning(|_, _, _, _| Ok(()));
        let Instance::Docker(mut instance) = get_test_instance(NETWORK_INSTANCE) else {
            panic!()
        };
        instance.deployment = Arc::new(deployment);
        assert!(matches!(
            instance
                .connect_network(NETWORK_NAME.to_string(), ip_address)
                .await,
            Ok(None)
        ));
        assert_eq!(
            instance.config.connected_networks.get(NETWORK_NAME),
            Some(&IpAddr::V4(ip_address))
        );
    }

    #[tokio::test]
    async fn instance_connect_network_stopped_ok() {
        let ip_address = Ipv4Addr::new(10, 20, 30, 40);
        const NETWORK_NAME: &str = "TestNet";
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_instance_status()
            .with(predicate::eq(NETWORK_INSTANCE))
            .returning(|_| Ok(InstanceStatus::Stopped));
        let Instance::Docker(mut instance) = get_test_instance(NETWORK_INSTANCE) else {
            panic!()
        };
        instance.deployment = Arc::new(deployment);
        assert!(matches!(
            instance
                .connect_network(NETWORK_NAME.to_string(), ip_address)
                .await,
            Ok(None)
        ));
        assert_eq!(
            instance.config.connected_networks.get(NETWORK_NAME),
            Some(&IpAddr::V4(ip_address))
        );
    }

    #[tokio::test]
    async fn instance_connect_network_err_connect() {
        let ip_address = Ipv4Addr::new(10, 20, 30, 40);
        const NETWORK_NAME: &str = "TestNet";
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_instance_status()
            .with(predicate::eq(NETWORK_INSTANCE))
            .returning(|_| Ok(InstanceStatus::Running));
        deployment
            .expect_disconnect_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(NETWORK_NAME.to_string()),
                predicate::eq(NETWORK_INSTANCE),
            )
            .returning(|_, _, _| Ok(()));
        deployment
            .expect_connect_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(NETWORK_NAME.to_string()),
                predicate::eq(ip_address),
                predicate::eq(NETWORK_INSTANCE),
            )
            .returning(|_, _, _, _| Err(anyhow::anyhow!("TestError")));
        let Instance::Docker(mut instance) = get_test_instance(NETWORK_INSTANCE) else {
            panic!()
        };
        instance.deployment = Arc::new(deployment);
        assert!(
            instance
                .connect_network(NETWORK_NAME.to_string(), ip_address)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn instance_connect_network_err_status() {
        let ip_address = Ipv4Addr::new(10, 20, 30, 40);
        const NETWORK_NAME: &str = "TestNet";
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_instance_status()
            .with(predicate::eq(NETWORK_INSTANCE))
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let Instance::Docker(mut instance) = get_test_instance(NETWORK_INSTANCE) else {
            panic!()
        };
        instance.deployment = Arc::new(deployment);
        assert!(
            instance
                .connect_network(NETWORK_NAME.to_string(), ip_address)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn instance_connect_network_reconnect() {
        let ip_address = Ipv4Addr::new(10, 20, 30, 40);
        let old_ip_address = IpAddr::V4(Ipv4Addr::new(120, 20, 40, 50));
        const NETWORK_NAME: &str = "flecs";
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_instance_status()
            .with(predicate::eq(NETWORK_INSTANCE))
            .returning(|_| Ok(InstanceStatus::Running));
        deployment
            .expect_disconnect_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(NETWORK_NAME.to_string()),
                predicate::eq(NETWORK_INSTANCE),
            )
            .returning(|_, _, _| Ok(()));
        deployment
            .expect_connect_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(NETWORK_NAME.to_string()),
                predicate::eq(ip_address),
                predicate::eq(NETWORK_INSTANCE),
            )
            .returning(|_, _, _, _| Ok(()));
        let Instance::Docker(mut instance) = get_test_instance(NETWORK_INSTANCE) else {
            panic!()
        };
        instance.deployment = Arc::new(deployment);
        assert_eq!(
            instance
                .connect_network(NETWORK_NAME.to_string(), ip_address)
                .await
                .unwrap(),
            Some(old_ip_address)
        );
        assert_eq!(
            instance.config.connected_networks.get(NETWORK_NAME),
            Some(&IpAddr::V4(ip_address))
        );
    }

    #[tokio::test]
    async fn instance_connect_network_reconnect_failed_disconnect() {
        let ip_address = Ipv4Addr::new(10, 20, 30, 40);
        const NETWORK_NAME: &str = "flecs";
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_instance_status()
            .with(predicate::eq(NETWORK_INSTANCE))
            .returning(|_| Ok(InstanceStatus::Running));
        deployment
            .expect_disconnect_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(NETWORK_NAME.to_string()),
                predicate::eq(NETWORK_INSTANCE),
            )
            .returning(|_, _, _| Err(anyhow::anyhow!("TestError")));
        deployment
            .expect_connect_network()
            .once()
            .with(
                predicate::always(),
                predicate::eq(NETWORK_NAME.to_string()),
                predicate::eq(ip_address),
                predicate::eq(NETWORK_INSTANCE),
            )
            .returning(|_, _, _, _| Ok(()));
        let Instance::Docker(mut instance) = get_test_instance(NETWORK_INSTANCE) else {
            panic!()
        };
        instance.deployment = Arc::new(deployment);
        assert_eq!(
            instance
                .connect_network(NETWORK_NAME.to_string(), ip_address)
                .await
                .unwrap(),
            None
        );
        assert_eq!(
            instance.config.connected_networks.get(NETWORK_NAME),
            Some(&IpAddr::V4(ip_address))
        );
    }

    #[test_case("192.168.54.23", "192.168.34.0/24", "192.168.34.23")]
    #[test_case("10.3.72.198", "10.12.0.0/16", "10.12.72.198")]
    #[test_case("172.16.88.7", "172.20.45.0/24", "172.20.45.7")]
    #[test_case("203.45.112.9", "8.8.0.0/16", "8.8.112.9")]
    #[test_case("8.46.219.33", "100.64.12.0/22", "100.64.15.33")]
    #[test_case("99.120.55.101", "203.0.113.0/24", "203.0.113.101")]
    #[test_case("45.67.89.10", "198.51.100.0/25", "198.51.100.10")]
    #[test_case("132.4.78.199", "192.0.2.128/26", "192.0.2.135")]
    #[test_case("11.254.30.66", "15.35.0.0/20", "15.35.14.66")]
    #[test_case("63.91.182.240", "185.76.144.0/21", "185.76.150.240")]
    #[test_case("2001:db8:abcd:1234::1", "2001:db8:abcd::/48", "2001:db8:abcd:1234::1")]
    #[test_case(
        "fd12:3456:789a:ffff::dead:beef",
        "fd12:3456::/32",
        "fd12:3456:789a:ffff::dead:beef"
    )]
    #[test_case("fe80::abcd:1234:5678:9abc", "fe80::/128", "fe80::")]
    #[test_case(
        "2606:4700:abcd:ef12::1",
        "2606:4700:abcd:1200::/56",
        "2606:4700:abcd:1212::1"
    )]
    #[test_case(
        "2001:4860:dead:beef::42",
        "2001:4860:feed::/40",
        "2001:4860:feed:beef::42"
    )]
    #[test_case(
        "2a00:1450:4001:82a::f1",
        "2a00:1450:4001:800::/61",
        "2a00:1450:4001:802::f1"
    )]
    fn transfer_ip_ok(current: &str, network: &str, expected: &str) {
        let id = network.to_string();
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![bollard::models::IpamConfig {
                    subnet: Some(network.to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let current = IpAddr::from_str(current).unwrap();
        let expected = IpAddr::from_str(expected).unwrap();
        assert_eq!(
            DockerInstance::transfer_ip_address(current, &network, &id).unwrap(),
            expected
        )
    }

    #[test]
    fn transfer_ipv4_err_ipam() {
        let id = "TestNetwork".to_string();
        let ip = IpAddr::V4(Ipv4Addr::new(10, 20, 30, 40));
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![bollard::models::IpamConfig {
                    subnet: Some("invalid".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let result = DockerInstance::transfer_ip_address(ip, &network, &id);
        assert!(
            matches!(result, Err(TransferIpError::InspectNetwork { .. })),
            "{result:?}"
        );
    }

    #[test]
    fn transfer_ipv4_err_no_fitting_network() {
        let id = "TestNetwork".to_string();
        let ip = IpAddr::V4(Ipv4Addr::new(10, 20, 30, 40));
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![bollard::models::IpamConfig {
                    subnet: Some("2a00:1450:4001:800::/61".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let result = DockerInstance::transfer_ip_address(ip, &network, &id);
        assert!(
            matches!(result, Err(TransferIpError::NoFittingNetwork { .. })),
            "{result:?}"
        );
    }

    #[test]
    fn transfer_ipv6_err_ipam() {
        let id = "TestNetwork".to_string();
        let ip = IpAddr::V6(Ipv6Addr::from_str("2a00:1450:4001:800::2").unwrap());
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![bollard::models::IpamConfig {
                    subnet: Some("invalid".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let result = DockerInstance::transfer_ip_address(ip, &network, &id);
        assert!(
            matches!(result, Err(TransferIpError::InspectNetwork { .. })),
            "{result:?}"
        );
    }

    #[test]
    fn transfer_ipv6_err_no_fitting_network() {
        let id = "TestNetwork".to_string();
        let ip = IpAddr::V6(Ipv6Addr::from_str("2a00:1450:4001:800::2").unwrap());
        let network = bollard::models::Network {
            ipam: Some(bollard::models::Ipam {
                config: Some(vec![bollard::models::IpamConfig {
                    subnet: Some("10.20.30.0/24".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let result = DockerInstance::transfer_ip_address(ip, &network, &id);
        assert!(
            matches!(result, Err(TransferIpError::NoFittingNetwork { .. })),
            "{result:?}"
        );
    }
}
