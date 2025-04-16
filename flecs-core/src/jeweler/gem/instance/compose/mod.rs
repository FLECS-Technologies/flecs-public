use super::{CreateInstanceError, InstanceCommon, InstanceId};
use crate::jeweler::deployment::DeploymentId;
use crate::jeweler::gem::deployment::Deployment;
use crate::jeweler::gem::deployment::compose::ComposeDeployment;
use crate::jeweler::gem::instance::status::InstanceStatus;
use crate::jeweler::gem::manifest::multi::AppManifestMulti;
use crate::jeweler::gem::manifest::{AppManifest, multi};
use crate::jeweler::{GetAppKey, serialize_deployment_id, serialize_manifest_key};
use crate::quest::SyncQuest;
use crate::vault;
use crate::vault::pouch::AppKey;
use async_trait::async_trait;
use flecsd_axum_server::models::{AppInstance, InstancesInstanceIdGet200Response};
use serde::{Deserialize, Serialize};
use std::mem::swap;
use std::net::Ipv4Addr;
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct ComposeInstance {
    pub id: InstanceId,
    #[serde(serialize_with = "serialize_manifest_key", rename = "app_key")]
    pub manifest: Arc<multi::AppManifestMulti>,
    #[serde(serialize_with = "serialize_deployment_id", rename = "deployment_id")]
    pub deployment: Arc<dyn ComposeDeployment>,
    pub name: String,
    pub desired: InstanceStatus,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ComposeInstanceDeserializable {
    pub id: InstanceId,
    pub app_key: AppKey,
    pub deployment_id: DeploymentId,
    pub name: String,
    pub desired: InstanceStatus,
}

#[async_trait]
impl InstanceCommon for ComposeInstance {
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
        AppManifest::Multi(self.manifest.clone())
    }

    fn replace_manifest(&mut self, manifest: AppManifest) -> AppManifest {
        let AppManifest::Multi(mut manifest) = manifest else {
            panic!("Can not replace manifest of ComposeInstance with {manifest:?}");
        };
        swap(&mut manifest, &mut self.manifest);
        AppManifest::Multi(manifest)
    }

    async fn generate_info(&self) -> anyhow::Result<AppInstance> {
        let status = self.status().await?;
        Ok(flecsd_axum_server::models::AppInstance {
            instance_id: format!("{}", self.id),
            instance_name: self.name.clone(),
            app_key: self.app_key().clone().into(),
            status: status.into(),
            desired: self.desired.into(),
            editors: None,
        })
    }

    async fn generate_detailed_info(&self) -> anyhow::Result<InstancesInstanceIdGet200Response> {
        let status = self.status().await?;
        Ok(
            flecsd_axum_server::models::InstancesInstanceIdGet200Response {
                instance_id: format!("{}", self.id),
                instance_name: self.name.clone(),
                app_key: self.manifest.app_key().clone().into(),
                status: status.into(),
                desired: self.desired.into(),
                config_files: Vec::new().into(),
                hostname: String::new(),
                ip_address: String::new(),
                ports: Vec::new(),
                volumes: Vec::new(),
                editors: None,
            },
        )
    }

    async fn status(&self) -> anyhow::Result<InstanceStatus> {
        let status = self.deployment.instance_status(&self.manifest).await?;
        let status = Self::aggregate_status(status);
        Ok(status)
    }

    fn desired_status(&self) -> InstanceStatus {
        self.desired
    }

    fn taken_ipv4_addresses(&self) -> Vec<Ipv4Addr> {
        // TODO
        Vec::new()
    }
}

impl ComposeInstance {
    fn aggregate_status(status_vec: Vec<InstanceStatus>) -> InstanceStatus {
        if status_vec.is_empty() {
            return InstanceStatus::Stopped;
        }
        let mut status_iter = status_vec.into_iter();
        let resulting_status = status_iter.next().unwrap_or(InstanceStatus::Unknown);
        for status in status_iter {
            if status != resulting_status {
                return InstanceStatus::Unknown;
            }
        }
        resulting_status
    }

    pub fn try_create_with_state(
        instance: ComposeInstanceDeserializable,
        manifests: &vault::pouch::manifest::Gems,
        deployments: &vault::pouch::deployment::Gems,
    ) -> Result<Self, CreateInstanceError> {
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
        let AppManifest::Multi(manifest) = manifest else {
            return Err(anyhow::anyhow!(
                "ComposeInstances can only be created with AppManifestSingle, not with {}",
                manifest.key()
            )
            .into());
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
        let Deployment::Compose(deployment) = deployment else {
            return Err(anyhow::anyhow!(
                "ComposeInstances can only be created with ComposeDeployments, not with {}",
                deployment.id()
            )
            .into());
        };
        Ok(Self {
            manifest,
            deployment,
            desired: instance.desired,
            id: instance.id,
            name: instance.name,
        })
    }

    pub async fn try_create_new(
        _quest: SyncQuest,
        deployment: Arc<dyn ComposeDeployment>,
        manifest: Arc<AppManifestMulti>,
        name: String,
    ) -> Result<Self, CreateInstanceError> {
        // TODO: Create volumes?
        let instance_id = InstanceId::new_random();
        tokio::fs::create_dir_all(crate::lore::instance_workdir_path(&instance_id.to_string()))
            .await?;
        Ok(Self {
            deployment,
            name,
            manifest,
            desired: InstanceStatus::Stopped,
            id: InstanceId::new_random(),
        })
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.desired = InstanceStatus::Running;
        if self.status().await? == InstanceStatus::Running {
            return Ok(());
        }
        let path = crate::lore::instance_workdir_path(&self.id.to_string());
        self.deployment
            .start_instance(&self.manifest, &path)
            .await?;
        Ok(())
    }

    pub async fn halt(&self) -> anyhow::Result<()> {
        if self.status().await? == InstanceStatus::Stopped {
            return Ok(());
        }
        self.deployment.stop_instance(&self.manifest).await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> anyhow::Result<()> {
        self.desired = InstanceStatus::Stopped;
        self.halt().await
    }

    pub async fn stop_and_delete(mut self) -> Result<(), (anyhow::Error, Self)> {
        self.desired = InstanceStatus::NotCreated;
        if let Err(e) = self.halt().await {
            return Err((e, self));
        };
        let path = crate::lore::instance_workdir_path(&self.id.to_string());
        if let Err(e) = tokio::fs::remove_dir_all(&path).await {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err((e.into(), self));
            }
        };
        Ok(())
    }
}
