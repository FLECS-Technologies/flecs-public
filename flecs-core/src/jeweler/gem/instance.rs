use crate::jeweler::deployment::{Deployment, DeploymentId};
use crate::jeweler::gem::manifest::{
    AppManifest, ConfigFile, EnvironmentVariable, PortMapping, VolumeMount,
};
use crate::jeweler::volume::VolumeId;
use crate::jeweler::{serialize_deployment_id, serialize_manifest_key};
use crate::quest::SyncQuest;
use crate::vault::pouch::AppKey;
use bollard::container::Config;
use bollard::models::ContainerStateStatusEnum;
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use tracing::log::warn;

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

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct InstanceConfig {
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub volume_mounts: HashMap<VolumeId, VolumeMount>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub environment_variables: Vec<EnvironmentVariable>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub port_mapping: Vec<PortMapping>,
}

impl From<&Instance> for Config<String> {
    fn from(value: &Instance) -> Self {
        // TODO: Add more info from instance config if available
        Config {
            image: Some(value.manifest.image_with_tag().to_string()),
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
    Created,
    Stopped,
    Running,
    Orphaned,
    Unknown,
}

impl From<ContainerStateStatusEnum> for InstanceStatus {
    fn from(value: ContainerStateStatusEnum) -> Self {
        // TBD
        match value {
            ContainerStateStatusEnum::EMPTY => Self::Created,
            ContainerStateStatusEnum::CREATED => Self::Created,
            ContainerStateStatusEnum::RUNNING => Self::Running,
            ContainerStateStatusEnum::PAUSED => Self::Running,
            ContainerStateStatusEnum::RESTARTING => Self::Running,
            ContainerStateStatusEnum::REMOVING => Self::Running,
            ContainerStateStatusEnum::EXITED => Self::Created,
            ContainerStateStatusEnum::DEAD => Self::Created,
        }
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
pub struct InstanceDeserializable {
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
    pub id: InstanceId,
    pub config: InstanceConfig,
    #[serde(serialize_with = "serialize_deployment_id", rename = "deployment_id")]
    deployment: Arc<dyn Deployment>,
    #[serde(serialize_with = "serialize_manifest_key", rename = "app_key")]
    manifest: Arc<AppManifest>,
    desired: InstanceStatus,
}

impl Instance {
    pub fn app_key(&self) -> AppKey {
        self.manifest.key.clone()
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

    pub async fn create(
        quest: SyncQuest,
        deployment: Arc<dyn Deployment>,
        manifest: Arc<AppManifest>,
        name: String,
    ) -> anyhow::Result<Self> {
        let instance_id = InstanceId::new_random();
        let port_mapping = manifest.ports.clone();
        let environment_variables = manifest.environment_variables.clone();
        // TODO: Create volume mounts
        let volume_mounts: HashMap<VolumeId, VolumeMount> = HashMap::new();
        let config = InstanceConfig {
            environment_variables,
            port_mapping,
            volume_mounts,
        };
        // TODO: Create networks
        let config_path = crate::lore::base_path()
            .join("instances")
            .join(instance_id.to_string())
            .join("conf");
        let create_configs_result = quest
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
                        config_path,
                        manifest.config_files.clone(),
                        manifest.clone(),
                    )
                },
            )
            .await
            .2;
        // TODO: Delete all volume_mounts if error occurs
        create_configs_result.await?;
        Ok(Self {
            id: instance_id,
            deployment,
            name,
            manifest,
            config,
            desired: InstanceStatus::Created,
        })
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.desired = InstanceStatus::Running;
        match self.deployment.instance_status(self.id).await? {
            InstanceStatus::Running => {}
            _ => {
                self.id = self
                    .deployment
                    .start_instance((&*self).into(), Some(self.id))
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
        // TODO: Save config files
        self.desired = InstanceStatus::Stopped;
        match self.deployment.instance_status(self.id).await? {
            InstanceStatus::Running | InstanceStatus::Unknown | InstanceStatus::Orphaned => {
                self.deployment.stop_instance(self.id).await
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

    pub async fn ready(&mut self) -> anyhow::Result<()> {
        // TODO: Check status, error handling
        self.deployment.ready_instance(self.id).await
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

    pub async fn copy_to(&self, quest: SyncQuest, src: &Path, dst: &Path) -> anyhow::Result<()> {
        self.deployment
            .copy_to_instance(quest, self.id, src, dst)
            .await
    }

    pub async fn is_runnable(&self) -> anyhow::Result<bool> {
        self.deployment.is_instance_runnable(self.id).await
    }

    pub async fn is_running(&self) -> anyhow::Result<bool> {
        self.deployment.is_instance_running(self.id).await
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
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::jeweler::gem::manifest::tests::{create_test_manifest, create_test_manifest_full};
    use crate::jeweler::gem::manifest::PortRange;
    use crate::quest::Quest;
    use crate::tests::prepare_test_path;
    use std::fs::File;
    use std::io::Write;
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
        let instance = InstanceDeserializable {
            deployment_id,
            id: InstanceId::new(10),
            app_key,
            name: "TestInstance".to_string(),
            desired: InstanceStatus::Running,
            config: InstanceConfig::default(),
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
        let instance = InstanceDeserializable {
            deployment_id,
            id: InstanceId::new(10),
            app_key,
            name: "TestInstance".to_string(),
            desired: InstanceStatus::Running,
            config: InstanceConfig::default(),
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
        let instance = InstanceDeserializable {
            deployment_id,
            id: InstanceId::new(10),
            app_key,
            name: "TestInstance".to_string(),
            desired: InstanceStatus::Running,
            config: InstanceConfig::default(),
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
            desired: InstanceStatus::Created,
            name: "TestInstance".to_string(),
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
            .returning(|_| Err(anyhow::anyhow!("TestError")));
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
            .returning(|_| Ok(()));
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
            .returning(|_| Ok(()));
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
            .returning(|_| Ok(()));
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
            .returning(|_| Err(anyhow::anyhow!("TestError")));
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
        let path = prepare_test_path(module_path!(), "create_config_file_err").join("config");
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
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let instance = Instance::create(
            Quest::new_synced("TestQuest".to_string()),
            deployment,
            manifest.clone(),
            "TestInstance".to_string(),
        )
        .await
        .unwrap();
        assert_eq!(&instance.config.port_mapping, &manifest.ports);
        assert_eq!(instance.desired, InstanceStatus::Created);
        assert_eq!(
            &instance.config.environment_variables,
            &manifest.environment_variables
        );
        // TODO: Check volume_mounts
        // TODO: Check networks
    }

    #[tokio::test]
    async fn create_create_config_fails() {
        let manifest = Arc::new(create_test_manifest_full(None));
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_copy_from_app_image()
            .times(3)
            .returning(|_, _, _, _, _| Err(anyhow::anyhow!("TestError")));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        assert!(Instance::create(
            Quest::new_synced("TestQuest".to_string()),
            deployment,
            manifest.clone(),
            "TestInstance".to_string(),
        )
        .await
        .is_err());
    }
}
