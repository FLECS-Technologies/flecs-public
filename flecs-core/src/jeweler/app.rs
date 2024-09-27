pub use super::Result;
use crate::jeweler::deployment::{Deployment, DeploymentId};
use crate::jeweler::instance::{Instance, InstanceConfig, InstanceId, InstanceStatus};
use crate::vault::pouch::AppKey;
use anyhow::anyhow;
use async_trait::async_trait;
use flecs_app_manifest::AppManifest;
use std::collections::HashMap;
use std::sync::Arc;

type AppId = String;
#[async_trait]
pub trait AppDeployment {
    async fn install_app(&self, manifest: &AppManifest) -> Result<AppId>;
    async fn uninstall_app(&self, id: AppId) -> Result<()>;
    async fn is_app_installed(&self, id: AppId) -> Result<bool>;
}
#[derive(Default)]
pub enum AppStatus {
    #[default]
    None,
    Installed,
    NotInstalled,
}

pub struct AppData {
    desired: AppStatus,
    instances: HashMap<InstanceId, Instance>,
    id: Option<AppId>,
    deployment: Arc<dyn Deployment>,
}

pub struct App {
    key: AppKey,
    properties: HashMap<DeploymentId, AppData>,
    manifest: Option<Arc<AppManifest>>,
}

impl App {
    pub async fn install(&mut self) -> Result<()> {
        match &self.manifest {
            None => Err(anyhow!(
                "Can not install {:?}, no manifest present.",
                self.key
            ))?,
            Some(manifest) => {
                for data in self.properties.values_mut() {
                    data.desired = AppStatus::Installed;
                    // TODO: Installing app in one deployment should not fail the whole install process for all deployments
                    if let Some(id) = &data.id {
                        if !data.deployment.is_app_installed(id.clone()).await? {
                            data.id = Some(data.deployment.install_app(manifest.as_ref()).await?);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn uninstall(&mut self) -> Result<()> {
        for data in self.properties.values_mut() {
            data.desired = AppStatus::NotInstalled;
            // TODO: Uninstalling app in one deployment should not fail the whole uninstall process for all deployments
            if let Some(id) = &data.id {
                if data.deployment.is_app_installed(id.clone()).await? {
                    data.deployment.uninstall_app(id.clone()).await?;
                    data.id = None;
                }
            }
        }
        Ok(())
    }

    pub async fn create_instance(&mut self, instance_config: InstanceConfig) -> Result<()> {
        for data in self.properties.values_mut() {
            // TODO: Creating app in one deployment should not fail the whole creation process for all deployments
            let id = data
                .deployment
                .create_instance(instance_config.clone())
                .await?;
            data.instances.insert(
                id.clone(),
                Instance::new(
                    id,
                    instance_config.clone(),
                    data.deployment.clone(),
                    InstanceStatus::Created,
                ),
            );
        }
        Ok(())
    }
}
