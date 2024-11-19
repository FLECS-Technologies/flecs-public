use crate::jeweler::app::{AppId, AppStatus};
use crate::jeweler::deployment::{Deployment, DeploymentId};
use crate::jeweler::gem::instance::{Instance, InstanceDeserializable, InstanceId};
use crate::quest::{State, SyncQuest};
use crate::vault::pouch::AppKey;
use flecs_app_manifest::AppManifest;
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct AppData {
    desired: AppStatus,
    instances: HashMap<InstanceId, Instance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<AppId>,
    #[serde(skip_serializing)]
    deployment: Arc<dyn Deployment>,
}

impl AppData {
    pub fn new(deployment: Arc<dyn Deployment>) -> Self {
        AppData {
            desired: AppStatus::None,
            instances: HashMap::new(),
            id: None,
            deployment,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct App {
    pub key: AppKey,
    pub(crate) properties: HashMap<DeploymentId, AppData>,
    #[serde(skip)]
    manifest: Option<Arc<AppManifest>>, // TODO: Can we remove the Option and always have a manifest?
}

#[derive(Debug, Deserialize)]
pub struct AppDataDeserializable {
    pub desired: AppStatus,
    pub instances: HashMap<InstanceId, InstanceDeserializable>,
    pub id: Option<AppId>,
}

#[derive(Debug, Deserialize)]
pub struct AppDeserializable {
    pub key: AppKey,
    pub properties: HashMap<DeploymentId, AppDataDeserializable>,
}

pub fn try_create_app(
    app: AppDeserializable,
    manifests: &HashMap<AppKey, Arc<AppManifest>>,
    deployments: &HashMap<DeploymentId, Arc<dyn Deployment>>,
) -> anyhow::Result<App> {
    let properties = app
        .properties
        .into_iter()
        .filter_map(|(key, data)| match deployments.get(&key) {
            Some(deployment) => Some((
                key,
                AppData {
                    desired: data.desired,
                    instances: data
                        .instances
                        .into_iter()
                        .map(|(id, instance)| {
                            (
                                id,
                                Instance::new(
                                    instance.id,
                                    instance.name,
                                    instance.config,
                                    deployment.clone(),
                                    instance.desired,
                                ),
                            )
                        })
                        .collect(),
                    id: None,
                    deployment: deployment.clone(),
                },
            )),
            None => {
                // TODO: Decide if returning an error would be better
                eprintln!("Ignoring unknown deployment {key} of {}", app.key);
                None
            }
        })
        .collect();
    Ok(App {
        properties,
        manifest: manifests.get(&app.key).cloned(),
        key: app.key,
    })
}

impl App {
    pub fn set_manifest(&mut self, manifest: Arc<AppManifest>) {
        self.manifest = Some(manifest)
    }

    pub fn manifest(&self) -> Option<Arc<AppManifest>> {
        self.manifest.clone()
    }

    pub fn new(key: AppKey, deployments: Vec<Arc<dyn Deployment>>) -> Self {
        Self {
            key,
            manifest: None,
            properties: deployments
                .into_iter()
                .map(|deployment| (deployment.id(), AppData::new(deployment)))
                .collect(),
        }
    }

    pub async fn install(
        &mut self,
        quest: SyncQuest,
        username: String,
        password: String,
    ) -> anyhow::Result<()> {
        match &self.manifest {
            None => anyhow::bail!("Can not install {:?}, no manifest present.", self.key),
            Some(manifest) => {
                let mut deployment_ids = Vec::new();
                let mut install_app_results = Vec::new();
                for data in self.properties.values_mut() {
                    data.desired = AppStatus::Installed;
                    let deployment = data.deployment.clone();
                    let manifest = manifest.clone();
                    let id = data.id.clone();
                    let username = username.clone();
                    let password = password.clone();
                    let (.., id) = quest
                        .lock()
                        .await
                        .create_sub_quest(
                            format!(
                                "Installing app {}-{} to {}",
                                self.key.name,
                                self.key.version,
                                deployment.id()
                            ),
                            |quest| async move {
                                let is_installed = match &id {
                                    Some(id) => {
                                        deployment
                                            .is_app_installed(quest.clone(), id.clone())
                                            .await?
                                    }
                                    None => false,
                                };
                                if is_installed {
                                    quest.lock().await.state = State::Skipped;
                                    quest.lock().await.detail =
                                        Some("Already installed".to_string());
                                    Ok(id.unwrap())
                                } else {
                                    deployment
                                        .install_app(quest, manifest.clone(), username, password)
                                        .await
                                }
                            },
                        )
                        .await;
                    deployment_ids.push(data.deployment.id().clone());
                    install_app_results.push(id);
                }
                let mut success_count = 0;
                for (deployment_id, result) in deployment_ids
                    .into_iter()
                    .zip(join_all(install_app_results).await)
                {
                    match result {
                        Err(e) => {
                            eprintln!(
                                "Failed to install {} to deployment {}: {e}",
                                self.key, deployment_id
                            )
                        }
                        Ok(app_id) => {
                            if let Some(app_data) = self.properties.get_mut(&deployment_id) {
                                success_count += 1;
                                app_data.id = Some(app_id);
                            } else {
                                eprintln!("No app data for deployment {} found ", deployment_id)
                            }
                        }
                    }
                }
                if success_count == 0 && !self.properties.is_empty() {
                    anyhow::bail!(
                        "Failed to install {} in any of the {} deployments",
                        self.key,
                        self.properties.len()
                    );
                } else {
                    Ok(())
                }
            }
        }
    }
}
