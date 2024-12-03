use crate::jeweler::deployment::{Deployment, DeploymentId};
use crate::jeweler::gem::manifest::AppManifest;
use crate::jeweler::volume::VolumeId;
use crate::jeweler::{serialize_deployment_id, serialize_manifest_key};
use crate::quest::SyncQuest;
use crate::vault::pouch::AppKey;
use bollard::container::Config;
use bollard::models::ContainerStateStatusEnum;
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::path::Path;
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
        write!(f, "{:01x}", self.value)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct InstanceConfig {
    // TODO: Add more info (e.g. from manifest)
    #[serde(skip_serializing_if = "HashSet::is_empty", default)]
    pub volume_ids: HashSet<VolumeId>,
}

impl From<InstanceConfig> for Config<String> {
    fn from(_value: InstanceConfig) -> Self {
        // TODO: Add more info from instance config if available
        Config {
            image: None,
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

    pub async fn create() -> anyhow::Result<Self> {
        // TODO: Create portmapping
        // TODO: Set env and ports
        // TODO: Create volumes
        // TODO: Create networks
        // TODO: Create conffiles
        todo!()
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.desired = InstanceStatus::Running;
        match self.deployment.instance_status(self.id).await? {
            InstanceStatus::Running => {}
            _ => {
                self.id = self
                    .deployment
                    .start_instance(self.config.clone(), Some(self.id))
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
        for volume_id in self.config.volume_ids.iter() {
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

    pub async fn copy_from(&self, quest: SyncQuest, src: &Path, dst: &Path) -> anyhow::Result<()> {
        self.deployment
            .copy_from_instance(quest, self.id, src, dst)
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
    use crate::jeweler::gem::manifest::tests::create_test_manifest;
    use crate::quest::Quest;
    use std::num::IntErrorKind;

    #[test]
    fn display_instance_id() {
        assert_eq!("0", InstanceId { value: 0 }.to_string());
        assert_eq!("1", InstanceId { value: 1 }.to_string());
        assert_eq!("f0", InstanceId { value: 240 }.to_string());
        assert_eq!("3da33", InstanceId { value: 252467 }.to_string());
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
                volume_ids: HashSet::from([
                    format!("Instance#{id}Volume#1"),
                    format!("Instance#{id}Volume#2"),
                    format!("Instance#{id}Volume#3"),
                    format!("Instance#{id}Volume#4"),
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
}
