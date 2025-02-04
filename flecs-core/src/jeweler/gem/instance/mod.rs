mod config;
use crate::jeweler::deployment::{Deployment, DeploymentId};
use crate::jeweler::gem::manifest::{AppManifest, BindMount, ConfigFile, Mount, VolumeMount};
use crate::jeweler::instance::Logs;
use crate::jeweler::volume::VolumeId;
use crate::jeweler::{serialize_deployment_id, serialize_manifest_key};
use crate::quest::{Quest, SyncQuest};
use crate::vault::pouch::AppKey;
use bollard::container::Config;
use bollard::models::{ContainerStateStatusEnum, DeviceMapping, HostConfig, MountTypeEnum};
pub use config::*;
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::mem::swap;
use std::net::IpAddr;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use tokio::fs;
use tracing::warn;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InstanceId {
    pub(crate) value: u32,
}

impl InstanceId {
    pub fn to_docker_id(self) -> String {
        format!("flecs-{self}")
    }

    pub fn new(value: u32) -> Self {
        Self { value }
    }

    pub fn new_random() -> Self {
        Self {
            value: rand::random::<u32>(),
        }
    }
}

impl From<u32> for InstanceId {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl FromStr for InstanceId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u32::from_str_radix(s, 16).map(Self::new)
    }
}

impl Display for InstanceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08x}", self.value)
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

impl From<&Instance> for Config<String> {
    fn from(instance: &Instance) -> Self {
        let mut bind_mounts = instance.manifest.bind_mounts();
        let mut capabilities = instance.manifest.capabilities();
        if capabilities
            .remove(&flecs_app_manifest::generated::manifest_3_1_0::CapabilitiesItem::Docker)
        {
            bind_mounts.push(BindMount::default_docker_socket_bind_mount());
        }
        let mut mounts = bind_mounts_to_bollard_mounts(bind_mounts.as_slice());
        mounts.extend(instance.config.generate_volume_mounts());
        let arguments = instance.manifest.arguments();
        let cmd = if arguments.is_empty() {
            None
        } else {
            Some(arguments.clone())
        };
        let port_bindings = instance.config.generate_port_bindings();
        let exposed_ports = Some(
            port_bindings
                .keys()
                .cloned()
                .map(|key| (key, HashMap::new()))
                .collect(),
        );
        let host_config = Some(HostConfig {
            port_bindings: Some(port_bindings),
            mounts: Some(mounts),
            cap_add: Some(capabilities.iter().map(ToString::to_string).collect()),
            devices: Some(instance.generate_device_mappings()),
            ..HostConfig::default()
        });
        Config {
            image: Some(instance.manifest.image_with_tag().to_string()),
            hostname: Some(instance.hostname.clone()),
            env: Some(
                instance
                    .config
                    .environment_variables
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            ),
            labels: Some(
                instance
                    .manifest
                    .labels
                    .iter()
                    .map(|label| (label.label.clone(), label.value.clone().unwrap_or_default()))
                    .collect(),
            ),
            host_config,
            cmd,
            exposed_ports,
            networking_config: Some(instance.config.generate_network_config()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum InstanceStatus {
    // TBD
    NotCreated,
    Requested,
    ResourcesReady,
    Stopped,
    Running,
    Orphaned,
    Unknown,
}

impl From<InstanceStatus> for flecsd_axum_server::models::InstanceStatus {
    fn from(value: InstanceStatus) -> Self {
        match value {
            InstanceStatus::NotCreated => flecsd_axum_server::models::InstanceStatus::NotCreated,
            InstanceStatus::Requested => flecsd_axum_server::models::InstanceStatus::Requested,
            InstanceStatus::ResourcesReady => {
                flecsd_axum_server::models::InstanceStatus::ResourcesReady
            }
            InstanceStatus::Stopped => flecsd_axum_server::models::InstanceStatus::Stopped,
            InstanceStatus::Running => flecsd_axum_server::models::InstanceStatus::Running,
            InstanceStatus::Orphaned => flecsd_axum_server::models::InstanceStatus::Orphaned,
            InstanceStatus::Unknown => flecsd_axum_server::models::InstanceStatus::Unknown,
        }
    }
}

impl From<ContainerStateStatusEnum> for InstanceStatus {
    fn from(value: ContainerStateStatusEnum) -> Self {
        // TBD
        match value {
            ContainerStateStatusEnum::EMPTY => Self::Stopped,
            ContainerStateStatusEnum::CREATED => Self::Stopped,
            ContainerStateStatusEnum::RUNNING => Self::Running,
            ContainerStateStatusEnum::PAUSED => Self::Running,
            ContainerStateStatusEnum::RESTARTING => Self::Running,
            ContainerStateStatusEnum::REMOVING => Self::Running,
            ContainerStateStatusEnum::EXITED => Self::Stopped,
            ContainerStateStatusEnum::DEAD => Self::Stopped,
        }
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
pub struct InstanceDeserializable {
    pub hostname: String,
    pub name: String,
    pub id: InstanceId,
    pub config: InstanceConfig,
    pub desired: InstanceStatus,
    pub app_key: AppKey,
    pub deployment_id: DeploymentId,
}

#[derive(Debug, Serialize)]
pub struct Instance {
    pub name: String,
    pub hostname: String,
    pub id: InstanceId,
    pub config: InstanceConfig,
    #[serde(serialize_with = "serialize_deployment_id", rename = "deployment_id")]
    deployment: Arc<dyn Deployment>,
    #[serde(serialize_with = "serialize_manifest_key", rename = "app_key")]
    pub manifest: Arc<AppManifest>,
    desired: InstanceStatus,
}

impl Instance {
    pub fn app_key(&self) -> AppKey {
        self.manifest.key.clone()
    }

    pub fn replace_manifest(&mut self, mut manifest: Arc<AppManifest>) -> Arc<AppManifest> {
        swap(&mut manifest, &mut self.manifest);
        manifest
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

    pub async fn generate_info(&self) -> crate::Result<flecsd_axum_server::models::AppInstance> {
        let status = self.status().await?;
        Ok(flecsd_axum_server::models::AppInstance {
            instance_id: format!("{}", self.id),
            instance_name: self.name.clone(),
            app_key: self.app_key().into(),
            status: status.into(),
            desired: self.desired.into(),
            editors: None,
        })
    }

    pub async fn generate_detailed_info(
        &self,
    ) -> crate::Result<flecsd_axum_server::models::InstancesInstanceIdGet200Response> {
        let status = self.status().await?;
        Ok(
            flecsd_axum_server::models::InstancesInstanceIdGet200Response {
                instance_id: format!("{}", self.id),
                instance_name: self.name.clone(),
                app_key: self.app_key().into(),
                status: status.into(),
                desired: self.desired.into(),
                config_files: self
                    .manifest
                    .config_files
                    .iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .into(),
                hostname: self.hostname.clone(),
                // TODO: ip_address
                ip_address: "TODO".to_string(),
                ports: self.manifest.ports.iter().map(Into::into).collect(),
                volumes: self
                    .manifest
                    .mounts
                    .iter()
                    .filter_map(|mount| match mount {
                        Mount::Volume(volume) => {
                            Some(flecsd_axum_server::models::InstanceDetailVolume {
                                name: volume.name.clone(),
                                path: volume.container_path.to_string_lossy().to_string(),
                            })
                        }
                        _ => None,
                    })
                    .collect(),
                // TODO: Fill editors
                editors: None,
            },
        )
    }

    pub async fn create_config_file(
        quest: SyncQuest,
        deployment: Arc<dyn Deployment>,
        config_path: PathBuf,
        config_file: ConfigFile,
        manifest: Arc<AppManifest>,
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
        deployment: Arc<dyn Deployment>,
        config_path: PathBuf,
        config_files: Vec<ConfigFile>,
        manifest: Arc<AppManifest>,
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
        if let Some(error) = join_all(results)
            .await
            .into_iter()
            .filter_map(|result| result.err())
            .next()
        {
            tokio::fs::remove_dir_all(&config_path).await?;
            Err(error)
        } else {
            Ok(())
        }
    }

    pub async fn create_volumes(
        quest: SyncQuest,
        deployment: Arc<dyn Deployment>,
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
            quest
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
                .2
                .await?;
            anyhow::bail!("Could not create all volumes for instance {}", instance_id);
        } else {
            Ok(volumes)
        }
    }

    pub async fn create(
        quest: SyncQuest,
        deployment: Arc<dyn Deployment>,
        manifest: Arc<AppManifest>,
        name: String,
        address: IpAddr,
    ) -> anyhow::Result<Self> {
        let instance_id = InstanceId::new_random();
        let port_mapping = manifest.ports.clone();
        let environment_variables = manifest.environment_variables.clone();
        let config_path = crate::lore::instance_config_path(&instance_id.to_string());
        let default_network_id = deployment
            .default_network()
            .await?
            .id
            .ok_or_else(|| anyhow::anyhow!("Default network has no id"))?;
        quest
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
            .2
            .await?;
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
            .2
            .await;
        let volume_mounts = match volume_mounts {
            Ok(volume_mounts) => volume_mounts,
            Err(e) => {
                fs::remove_dir_all(config_path).await?;
                return Err(e);
            }
        };
        let config = InstanceConfig {
            environment_variables,
            port_mapping,
            volume_mounts,
            network_addresses: HashMap::from([(default_network_id, address)]),
            usb_devices: HashSet::new(),
        };
        Ok(Self {
            hostname: format!("flecs-{instance_id}"),
            id: instance_id,
            deployment,
            name,
            manifest,
            config,
            desired: InstanceStatus::Stopped,
        })
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.desired = InstanceStatus::Running;
        match self.deployment.instance_status(self.id).await? {
            InstanceStatus::Running => {}
            _ => {
                self.id = self
                    .deployment
                    .start_instance((&*self).into(), Some(self.id), &self.manifest.config_files)
                    .await?
            }
        }
        Ok(())
    }

    pub async fn stop_and_delete(
        mut self,
        quest: SyncQuest,
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
                |_quest| async move {
                    match self.stop().await {
                        Ok(()) => (self, None),
                        Err(e) => {
                            _quest.lock().await.fail_with_error(&e);
                            (self, Some(e))
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
                        |quest| async move { instance.delete(quest).await },
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

    pub async fn stop(&mut self) -> anyhow::Result<()> {
        // TODO: Disconnect networks
        self.desired = InstanceStatus::Stopped;
        match self.deployment.instance_status(self.id).await? {
            InstanceStatus::Running | InstanceStatus::Unknown | InstanceStatus::Orphaned => {
                self.deployment
                    .stop_instance(self.id, &self.manifest.config_files)
                    .await
            }
            _ => Ok(()),
        }
    }

    pub async fn delete(mut self, quest: SyncQuest) -> anyhow::Result<(), (anyhow::Error, Self)> {
        // TODO: Delete floxy config
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
        for (id, result) in volume_ids.into_iter().zip(join_all(delete_results).await) {
            if let Err(e) = result {
                warn!("Could not delete volume {id} of instance {}: {e}", self.id);
            }
        }
        quest
            .lock()
            .await
            .create_sub_quest(format!("Delete instance {}", self.id), |_quest| {
                let deployment = self.deployment.clone();
                async move { deployment.clone().delete_instance(self.id).await }
            })
            .await
            .2
            .await
            .map_err(|e| (e, self))?;
        Ok(())
    }

    pub async fn export(&self, _path: &Path) -> anyhow::Result<()> {
        // TODO: Export config
        // TODO: Export config files
        // TODO: Export volumes
        Ok(())
    }

    pub async fn import(&self, _path: &Path) -> anyhow::Result<()> {
        // TODO: Import volumes
        // TODO: Import config files
        // TODO: Import config
        Ok(())
    }

    pub async fn status(&self) -> anyhow::Result<InstanceStatus> {
        self.deployment.instance_status(self.id).await
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

    pub async fn is_runnable(&self) -> anyhow::Result<bool> {
        self.deployment.is_instance_runnable(self.id).await
    }

    pub async fn is_running(&self) -> anyhow::Result<bool> {
        self.deployment.is_instance_running(self.id).await
    }

    pub async fn get_logs(&self) -> anyhow::Result<Logs> {
        self.deployment
            .instance_logs(
                Quest::new_synced(format!("Get logs of instance {}", self.id)),
                self.id,
            )
            .await
    }
}

pub fn try_create_instance(
    instance: InstanceDeserializable,
    manifests: &HashMap<AppKey, Arc<AppManifest>>,
    deployments: &HashMap<DeploymentId, Arc<dyn Deployment>>,
) -> anyhow::Result<Instance> {
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
    Ok(Instance {
        manifest,
        deployment,
        desired: instance.desired,
        id: instance.id,
        config: instance.config,
        name: instance.name,
        hostname: instance.hostname,
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::jeweler::gem::manifest::tests::{create_test_manifest, create_test_manifest_full};
    use crate::jeweler::gem::manifest::{EnvironmentVariable, PortMapping, PortRange};
    use crate::quest::Quest;
    use crate::relic::device::usb::tests::prepare_usb_device_test_path;
    use crate::relic::device::usb::UsbDevice;
    use crate::tests::prepare_test_path;
    use bollard::secret::Network;
    use flecsd_axum_server::models::{
        InstanceDetailConfigFile, InstanceDetailConfigFiles, InstanceDetailPort,
        InstanceDetailVolume,
    };
    use std::fs::File;
    use std::io::Write;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::num::IntErrorKind;
    use std::path::PathBuf;

    #[test]
    fn display_instance_id() {
        assert_eq!("00000000", InstanceId { value: 0 }.to_string());
        assert_eq!("00000001", InstanceId { value: 1 }.to_string());
        assert_eq!("000000f0", InstanceId { value: 240 }.to_string());
        assert_eq!("0003da33", InstanceId { value: 252467 }.to_string());
    }

    #[test]
    fn try_create_ok() {
        let manifest = create_test_manifest(None);
        let app_key = manifest.key.clone();
        let manifests = HashMap::from([(app_key.clone(), manifest)]);
        let deployment_id = "TestDeployment".to_string();
        let deployments = HashMap::from([(
            deployment_id.clone(),
            Arc::new(MockedDeployment::new()) as Arc<dyn Deployment>,
        )]);
        let instance_id = InstanceId::new(10);
        let instance = InstanceDeserializable {
            deployment_id,
            id: instance_id,
            app_key,
            name: "TestInstance".to_string(),
            desired: InstanceStatus::Running,
            config: InstanceConfig::default(),
            hostname: format!("flecs-{instance_id}"),
        };
        try_create_instance(instance, &manifests, &deployments).unwrap();
    }

    #[test]
    fn try_create_no_deployment() {
        let manifest = create_test_manifest(None);
        let app_key = manifest.key.clone();
        let manifests = HashMap::from([(app_key.clone(), manifest)]);
        let deployment_id = "TestDeployment".to_string();
        let deployments = HashMap::new();
        let instance_id = InstanceId::new(10);
        let instance = InstanceDeserializable {
            deployment_id,
            id: instance_id,
            app_key,
            name: "TestInstance".to_string(),
            desired: InstanceStatus::Running,
            config: InstanceConfig::default(),
            hostname: format!("flecs-{instance_id}"),
        };
        assert!(try_create_instance(instance, &manifests, &deployments).is_err());
    }

    #[test]
    fn try_create_no_manifest() {
        let manifest = create_test_manifest(None);
        let app_key = manifest.key.clone();
        let manifests = HashMap::new();
        let deployment_id = "TestDeployment".to_string();
        let deployments = HashMap::from([(
            deployment_id.clone(),
            Arc::new(MockedDeployment::new()) as Arc<dyn Deployment>,
        )]);
        let instance_id = InstanceId::new(10);
        let instance = InstanceDeserializable {
            deployment_id,
            id: instance_id,
            app_key,
            name: "TestInstance".to_string(),
            desired: InstanceStatus::Running,
            config: InstanceConfig::default(),
            hostname: format!("flecs-{instance_id}"),
        };
        assert!(try_create_instance(instance, &manifests, &deployments).is_err());
    }

    pub fn test_instance(
        id: u32,
        deployment: Arc<dyn Deployment>,
        manifest: Arc<AppManifest>,
    ) -> Instance {
        Instance {
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
                environment_variables: vec![
                    EnvironmentVariable::from_str("variable-1=value1").unwrap(),
                    EnvironmentVariable::from_str("variable-2=").unwrap(),
                    EnvironmentVariable::from_str("variable-3").unwrap(),
                ],
                port_mapping: vec![
                    PortMapping::Single(1002, 2002),
                    PortMapping::Range {
                        from: PortRange::try_new(8000, 9000).unwrap(),
                        to: PortRange::try_new(9500, 10500).unwrap(),
                    },
                ],
                network_addresses: HashMap::new(),
                usb_devices: HashSet::from([
                    UsbDevice {
                        device: "TestDevice1".to_string(),
                        port: "test_instance_dev_1".to_string(),
                        vendor: "TestVendor1".to_string(),
                        pid: 1,
                        vid: 2,
                    },
                    UsbDevice {
                        device: "TestDevice2".to_string(),
                        port: "test_instance_dev_2".to_string(),
                        vendor: "TestVendor2".to_string(),
                        pid: 2,
                        vid: 2,
                    },
                ]),
            },
            deployment,
            manifest,
        }
    }

    #[tokio::test]
    async fn delete_ok() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_delete_instance()
            .times(1)
            .returning(|_| Ok(true));
        deployment
            .expect_delete_volume()
            .times(4)
            .returning(|_, _| Ok(()));
        test_instance(1, Arc::new(deployment), create_test_manifest(None))
            .delete(Quest::new_synced("TestQuest".to_string()))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn delete_volume_err() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_delete_instance()
            .times(1)
            .returning(|_| Ok(true));
        deployment
            .expect_delete_volume()
            .times(4)
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        test_instance(2, Arc::new(deployment), create_test_manifest(None))
            .delete(Quest::new_synced("TestQuest".to_string()))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn delete_err() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_delete_instance()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        deployment
            .expect_delete_volume()
            .times(4)
            .returning(|_, _| Ok(()));
        let (_error, instance) = test_instance(3, Arc::new(deployment), create_test_manifest(None))
            .delete(Quest::new_synced("TestQuest".to_string()))
            .await
            .err()
            .unwrap();
        assert_eq!(instance.desired, InstanceStatus::NotCreated);
    }

    #[tokio::test]
    async fn stop_err() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_stop_instance()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Ok(InstanceStatus::Running));
        let mut instance = test_instance(4, Arc::new(deployment), create_test_manifest(None));
        assert!(instance.stop().await.is_err());
        assert_eq!(instance.desired, InstanceStatus::Stopped);
    }

    #[tokio::test]
    async fn stop_ok() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_stop_instance()
            .times(1)
            .returning(|_, _| Ok(()));
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Ok(InstanceStatus::Running));
        let mut instance = test_instance(5, Arc::new(deployment), create_test_manifest(None));
        assert!(instance.stop().await.is_ok());
        assert_eq!(instance.desired, InstanceStatus::Stopped);
    }

    #[tokio::test]
    async fn stop_stopped_ok() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_stop_instance().times(0);
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Ok(InstanceStatus::Stopped));
        let mut instance = test_instance(6, Arc::new(deployment), create_test_manifest(None));
        assert!(instance.stop().await.is_ok());
        assert_eq!(instance.desired, InstanceStatus::Stopped);
    }

    #[tokio::test]
    async fn stop_and_delete_ok() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_stop_instance()
            .times(1)
            .returning(|_, _| Ok(()));
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
        assert!(
            test_instance(7, Arc::new(deployment), create_test_manifest(None))
                .stop_and_delete(Quest::new_synced("TestQuest".to_string()))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn stop_and_delete_delete_err() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_stop_instance()
            .times(1)
            .returning(|_, _| Ok(()));
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
        let (_error, instance) = test_instance(8, Arc::new(deployment), create_test_manifest(None))
            .stop_and_delete(Quest::new_synced("TestQuest".to_string()))
            .await
            .err()
            .unwrap();
        assert_eq!(instance.desired, InstanceStatus::NotCreated);
    }

    #[tokio::test]
    async fn stop_and_delete_stop_err() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_stop_instance()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Ok(InstanceStatus::Running));
        deployment.expect_delete_instance().times(0);
        deployment.expect_delete_volume().times(0);
        let (_error, instance) = test_instance(9, Arc::new(deployment), create_test_manifest(None))
            .stop_and_delete(Quest::new_synced("TestQuest".to_string()))
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
        let manifest = Arc::new(create_test_manifest_full(None));
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_copy_from_app_image()
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let config_file = ConfigFile {
            host_file_name: "test.config".to_string(),
            container_file_path: PathBuf::from("/tmp/flecs-test.config"),
            read_only: false,
        };
        Instance::create_config_file(
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
        let manifest = Arc::new(create_test_manifest_full(None));
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_copy_from_app_image()
            .times(1)
            .returning(|_, _, _, _, _| Err(anyhow::anyhow!("TestError")));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let config_file = ConfigFile {
            host_file_name: "test.config".to_string(),
            container_file_path: PathBuf::from("/tmp/flecs-test.config"),
            read_only: false,
        };
        assert!(Instance::create_config_file(
            Quest::new_synced("TestQuest".to_string()),
            deployment,
            path,
            config_file,
            manifest,
        )
        .await
        .is_err())
    }

    #[tokio::test]
    async fn create_config_files_ok() {
        let path = prepare_test_path(module_path!(), "create_config_files_ok").join("config");
        let manifest = Arc::new(create_test_manifest_full(None));
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_copy_from_app_image()
            .times(3)
            .returning(|_, _, _, _, _| Ok(()));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        Instance::create_config_files(
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
        let manifest = Arc::new(create_test_manifest_full(None));
        let deployment: Arc<dyn Deployment> = Arc::new(MockedDeployment::new());
        assert!(Instance::create_config_files(
            Quest::new_synced("TestQuest".to_string()),
            deployment,
            path.clone(),
            manifest.config_files.clone(),
            manifest,
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn create_config_files_err() {
        let path = prepare_test_path(module_path!(), "create_config_files_err").join("config");
        let manifest = Arc::new(create_test_manifest_full(None));
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_copy_from_app_image()
            .times(3)
            .returning(|_, _, _, _, _| Err(anyhow::anyhow!("TestError")));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        assert!(Instance::create_config_files(
            Quest::new_synced("TestQuest".to_string()),
            deployment,
            path.clone(),
            manifest.config_files.clone(),
            manifest,
        )
        .await
        .is_err());
        assert!(!path.try_exists().unwrap());
    }

    #[tokio::test]
    async fn create_ok() {
        let manifest = Arc::new(create_test_manifest_full(None));
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_copy_from_app_image()
            .times(3)
            .returning(|_, _, _, _, _| Ok(()));
        deployment.expect_default_network().times(1).returning(|| {
            Ok(Network {
                id: Some("DefaultTestNetworkId".to_string()),
                ..Network::default()
            })
        });
        deployment
            .expect_create_volume()
            .times(1)
            .returning(|_, _| Ok("TestVolumeId".to_string()));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let address = IpAddr::V4(Ipv4Addr::new(123, 123, 123, 123));
        let instance = Instance::create(
            Quest::new_synced("TestQuest".to_string()),
            deployment,
            manifest.clone(),
            "TestInstance".to_string(),
            address,
        )
        .await
        .unwrap();
        assert_eq!(&instance.config.port_mapping, &manifest.ports);
        assert_eq!(instance.desired, InstanceStatus::Stopped);
        assert_eq!(
            &instance.config.environment_variables,
            &manifest.environment_variables
        );
        assert_eq!(
            &instance.config.network_addresses,
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
        let manifest = Arc::new(create_test_manifest_full(None));
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_copy_from_app_image()
            .times(3)
            .returning(|_, _, _, _, _| Err(anyhow::anyhow!("TestError")));
        deployment.expect_default_network().times(1).returning(|| {
            Ok(Network {
                id: Some("DefaultTestNetworkId".to_string()),
                ..Network::default()
            })
        });
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        assert!(Instance::create(
            Quest::new_synced("TestQuest".to_string()),
            deployment,
            manifest.clone(),
            "TestInstance".to_string(),
            IpAddr::V4(Ipv4Addr::new(123, 123, 123, 123)),
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn create_default_network_without_id() {
        let manifest = Arc::new(create_test_manifest_full(None));
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_default_network()
            .times(1)
            .returning(|| Ok(Network::default()));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        assert!(Instance::create(
            Quest::new_synced("TestQuest".to_string()),
            deployment,
            manifest.clone(),
            "TestInstance".to_string(),
            IpAddr::V4(Ipv4Addr::new(123, 123, 123, 123)),
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn create_default_network_err() {
        let manifest = Arc::new(create_test_manifest_full(None));
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_default_network()
            .times(1)
            .returning(|| Err(anyhow::anyhow!("TestError")));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        assert!(Instance::create(
            Quest::new_synced("TestQuest".to_string()),
            deployment,
            manifest.clone(),
            "TestInstance".to_string(),
            IpAddr::V4(Ipv4Addr::new(123, 123, 123, 123)),
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn create_instance_info_ok() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Ok(InstanceStatus::Running));
        let deployment = Arc::new(deployment);
        let manifest = Arc::new(create_test_manifest_full(Some(true)));
        let instance_id = InstanceId::new(0x123);
        let instance = Instance {
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
            editors: None,
        };
        assert_eq!(instance.generate_info().await.unwrap(), expected_info);
    }

    #[tokio::test]
    async fn create_instance_info_err() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let deployment = Arc::new(deployment);
        let manifest = Arc::new(create_test_manifest_full(Some(true)));
        let instance_id = InstanceId::new(0x123);
        let instance = Instance {
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
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Ok(InstanceStatus::Running));
        let deployment = Arc::new(deployment);
        let manifest = Arc::new(create_test_manifest_full(Some(true)));
        let instance_id = InstanceId::new(0x123);
        let instance = Instance {
            name: "TestInstance".to_string(),
            hostname: format!("flecs-{instance_id}"),
            id: instance_id,
            manifest,
            deployment,
            config: Default::default(),
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
            ip_address: "TODO".to_string(),
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
            editors: None,
        };
        assert_eq!(
            instance.generate_detailed_info().await.unwrap(),
            expected_info
        );
    }

    #[tokio::test]
    async fn create_instance_info_details_err() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_instance_status()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let deployment = Arc::new(deployment);
        let manifest = Arc::new(create_test_manifest_full(Some(true)));
        let instance_id = InstanceId::new(0x123);
        let instance = Instance {
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

    #[test]
    fn config_from_instance() {
        let path = prepare_usb_device_test_path("test_instance_dev_1");
        let bus_path = PathBuf::from("/tmp/flecs-tests/dev/bus/usb/456".to_string());
        config::tests::prepare_path(&bus_path);
        std::fs::write(bus_path.join("789"), b"test-dev-1").unwrap();
        std::fs::write(path.join("devnum"), b"789").unwrap();
        std::fs::write(path.join("busnum"), b"456").unwrap();
        let path = prepare_usb_device_test_path("test_instance_dev_2");
        let bus_path = PathBuf::from("/tmp/flecs-tests/dev/bus/usb/200".to_string());
        config::tests::prepare_path(&bus_path);
        std::fs::write(bus_path.join("300"), b"test-dev-2").unwrap();
        std::fs::write(path.join("devnum"), b"300").unwrap();
        std::fs::write(path.join("busnum"), b"200").unwrap();
        let deployment = MockedDeployment::new();
        let deployment = Arc::new(deployment);
        let manifest = Arc::new(create_test_manifest_full(Some(true)));
        let mut instance = test_instance(123, deployment, manifest);
        instance.config.network_addresses.insert(
            "Ipv4Network".to_string(),
            IpAddr::V4(Ipv4Addr::new(20, 22, 24, 26)),
        );
        instance.config.network_addresses.insert(
            "Ipv6Network".to_string(),
            IpAddr::V6(Ipv6Addr::new(
                0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
            )),
        );
        let config: Config<String> = (&instance).into();
        assert_eq!(
            config.image,
            Some("flecs.azurecr.io/some.test.app:1.2.1".to_string())
        );
        let host_config = config.host_config.unwrap();
        assert_eq!(host_config.port_bindings.unwrap().len(), 1002);
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
                path_on_host: Some("/tmp/flecs-tests/dev/bus/usb/456/789".to_string()),
                path_in_container: Some("/tmp/flecs-tests/dev/bus/usb/456/789".to_string()),
                cgroup_permissions: Some("rwm".to_string()),
            },
            DeviceMapping {
                path_on_host: Some("/tmp/flecs-tests/dev/bus/usb/200/300".to_string()),
                path_in_container: Some("/tmp/flecs-tests/dev/bus/usb/200/300".to_string()),
                cgroup_permissions: Some("rwm".to_string()),
            },
        ];
        let resulting_device_mappings = host_config.devices.unwrap();
        assert_eq!(
            resulting_device_mappings.len(),
            expected_device_mappings.len()
        );
        for expected_device_mapping in expected_device_mappings {
            assert!(resulting_device_mappings.contains(&expected_device_mapping));
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
            InstanceStatus::Stopped
        );
        assert_eq!(
            InstanceStatus::from(ContainerStateStatusEnum::CREATED),
            InstanceStatus::Stopped
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
            InstanceStatus::Stopped
        );
        assert_eq!(
            InstanceStatus::from(ContainerStateStatusEnum::DEAD),
            InstanceStatus::Stopped
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
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_create_volume()
            .times(3)
            .returning(|_, name| Ok(name.to_string()));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
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
            Instance::create_volumes(
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
        let mut deployment = MockedDeployment::new();
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
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
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
        assert!(Instance::create_volumes(
            Quest::new_synced("TestQuest".to_string()),
            deployment,
            volumes,
            instance_id,
        )
        .await
        .is_err());
    }

    #[test]
    fn replace_manifest() {
        let mut instance = test_instance(
            2,
            Arc::new(MockedDeployment::new()),
            create_test_manifest(None),
        );
        let manifest = create_test_manifest(Some("#2".to_string()));
        assert_eq!(instance.manifest.revision(), None);
        let old_manifest = instance.replace_manifest(manifest);
        assert_eq!(old_manifest.revision(), None);
        assert_eq!(instance.manifest.revision(), Some(&"#2".to_string()));
    }
}
