use crate::jeweler::app::{AppId, AppStatus};
use crate::jeweler::deployment::{Deployment, DeploymentId};
use crate::jeweler::gem::instance::{Instance, InstanceDeserializable, InstanceId};
use crate::vault::pouch::AppKey;
use flecs_app_manifest::AppManifest;
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
    _deployment: Arc<dyn Deployment>,
}

impl AppData {
    pub fn new(_deployment: Arc<dyn Deployment>) -> Self {
        AppData {
            desired: AppStatus::None,
            instances: HashMap::new(),
            id: None,
            _deployment,
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
                    _deployment: deployment.clone(),
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
}
