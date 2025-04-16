use super::{CreateInstanceError, InstanceCommon, InstanceId};
use crate::jeweler::deployment::DeploymentId;
use crate::jeweler::gem::deployment::compose::ComposeDeployment;
use crate::jeweler::gem::instance::status::InstanceStatus;
use crate::jeweler::gem::manifest::multi::AppManifestMulti;
use crate::jeweler::gem::manifest::{AppManifest, multi};
use crate::jeweler::{serialize_deployment_id, serialize_manifest_key};
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
    pub app_key: AppKey,
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
                app_key: self.app_key.clone().into(),
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
        self.deployment.instance_status(self.id).await
    }

    fn desired_status(&self) -> InstanceStatus {
        self.desired
    }

    fn taken_ipv4_addresses(&self) -> Vec<Ipv4Addr> {
        todo!()
    }
}

impl ComposeInstance {
    pub fn try_create_with_state(
        _instance: ComposeInstanceDeserializable,
        _manifests: &vault::pouch::manifest::Gems,
        _deployments: &vault::pouch::deployment::Gems,
    ) -> Result<Self, CreateInstanceError> {
        todo!()
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
            app_key: manifest.key.clone(),
            name,
            manifest,
            desired: InstanceStatus::Stopped,
            id: InstanceId::new_random(),
        })
    }
}
