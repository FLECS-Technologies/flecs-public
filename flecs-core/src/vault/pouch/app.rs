use crate::jeweler;
use crate::jeweler::deployment::Deployment;
use crate::jeweler::gem::app::{try_create_app, App, AppDeserializable};
use crate::jeweler::gem::manifest::AppManifest;
use crate::vault::pouch::{AppKey, DeploymentId, Pouch};
pub use crate::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::error;

const APPS_FILE_NAME: &str = "apps.json";
pub type AppStatus = jeweler::app::AppStatus;

pub struct AppPouch {
    path: PathBuf,
    apps: HashMap<AppKey, App>,
}

impl Pouch for AppPouch {
    type Gems = HashMap<AppKey, App>;

    fn gems(&self) -> &Self::Gems {
        &self.apps
    }

    fn gems_mut(&mut self) -> &mut Self::Gems {
        &mut self.apps
    }
}

impl AppPouch {
    #[allow(dead_code)] // TODO: We currently can not close the pouch as this would overwrite data of C++ core
    pub(in super::super) fn close(&mut self) -> Result<()> {
        let file = fs::File::create(self.path.join(APPS_FILE_NAME))?;
        let content: Vec<_> = self.apps.values().collect();
        serde_json::to_writer_pretty(file, &content)?;
        Ok(())
    }

    pub(in super::super) fn open(
        &mut self,
        manifests: &HashMap<AppKey, Arc<AppManifest>>,
        deployments: &HashMap<DeploymentId, Arc<dyn Deployment>>,
    ) -> Result<()> {
        self.apps = Self::create_apps(self.read_apps()?, manifests, deployments);
        Ok(())
    }

    fn read_apps(&self) -> Result<Vec<AppDeserializable>> {
        let file = fs::File::open(self.path.join(APPS_FILE_NAME))?;
        Ok(serde_json::from_reader(file)?)
    }

    fn create_apps(
        apps: Vec<AppDeserializable>,
        manifests: &HashMap<AppKey, Arc<AppManifest>>,
        deployments: &HashMap<DeploymentId, Arc<dyn Deployment>>,
    ) -> HashMap<AppKey, App> {
        apps.into_iter()
            .filter_map(|app| {
                let key = app.key.clone();
                match try_create_app(app, manifests, deployments) {
                    Ok(app) => Some((key, app)),
                    Err(e) => {
                        error!("Could not create app {key}: {e}");
                        None
                    }
                }
            })
            .collect()
    }
}

impl AppPouch {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            apps: HashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::gem::app::AppDataDeserializable;
    use crate::jeweler::gem::deployment::docker::DockerDeployment;
    use crate::tests::prepare_test_path;
    use crate::vault::pouch;
    use flecs_app_manifest::AppManifestVersion;
    use serde_json::Value;
    use std::fs;

    fn create_test_json() -> Value {
        serde_json::json!([
            {
                "key": {
                    "name": "test-app-1",
                    "version": "1.2.3"
                },
                "deployments": [{
                    "id": "test-app-id-1",
                    "desired": "Installed",
                    "deployment_id": "test-deployment-id-1"
                }]
            }
        ])
    }

    fn create_test_manifests() -> HashMap<AppKey, Arc<AppManifest>> {
        let manifest: flecs_app_manifest::generated::manifest_3_0_0::FlecsAppManifest =
            flecs_app_manifest::generated::manifest_3_0_0::builder::FlecsAppManifest::default()
                .app("test-app-1")
                .image("test-image")
                .version("1.2.3")
                .try_into()
                .unwrap();
        HashMap::from([(
            AppKey {
                name: manifest.app.to_string(),
                version: manifest.version.clone(),
            },
            Arc::new(AppManifestVersion::V3_0_0(manifest).try_into().unwrap()),
        )])
    }

    fn create_test_deployments() -> HashMap<jeweler::deployment::DeploymentId, Arc<dyn Deployment>>
    {
        let deployment: Arc<dyn Deployment> =
            pouch::deployment::Deployment::Docker(DockerDeployment::new(
                "test-deployment-id-1".to_string(),
                PathBuf::from("/var/run/docker.sock"),
            ))
            .into();
        HashMap::from([(deployment.id(), deployment)])
    }

    fn create_test_app() -> App {
        let manifests = create_test_manifests();
        let deployments = create_test_deployments();
        try_create_app(create_test_app_deserializable(), &manifests, &deployments).unwrap()
    }

    fn create_test_app_deserializable() -> AppDeserializable {
        AppDeserializable {
            key: AppKey {
                name: "test-app-1".to_string(),
                version: "1.2.3".to_string(),
            },
            deployments: vec![AppDataDeserializable {
                id: Some("test-app-id-1".to_string()),
                desired: AppStatus::Installed,
                deployment_id: "test-deployment-id-1".to_string(),
            }],
        }
    }

    #[test]
    fn open_app_pouch() {
        let path = prepare_test_path(module_path!(), "open_pouch");
        let json = create_test_json();
        fs::write(
            path.join(APPS_FILE_NAME),
            serde_json::to_string_pretty(&json).unwrap(),
        )
        .unwrap();
        let mut app_pouch = AppPouch {
            apps: HashMap::default(),
            path,
        };
        app_pouch
            .open(&create_test_manifests(), &create_test_deployments())
            .unwrap();
    }

    #[test]
    fn close_app_pouch() {
        let path = prepare_test_path(module_path!(), "close_pouch");
        let json = create_test_json();
        let app = create_test_app();
        let mut app_pouch = AppPouch {
            apps: HashMap::from([(app.key.clone(), app)]),
            path: path.clone(),
        };
        app_pouch.close().unwrap();
        let file = fs::File::open(path.join(APPS_FILE_NAME)).unwrap();
        let content: Value = serde_json::from_reader(file).unwrap();
        assert_eq!(content, json);
    }

    #[test]
    fn app_gems() {
        let app = create_test_app();
        let key = app.key.clone();
        let gems = HashMap::from([(key.clone(), app)]);
        let mut app_pouch = AppPouch {
            apps: gems,
            path: prepare_test_path(module_path!(), "gems"),
        };
        assert_eq!(app_pouch.gems().len(), 1);
        assert_eq!(app_pouch.gems().get(&key).unwrap().key, key);
        assert_eq!(app_pouch.gems_mut().len(), 1);
        assert_eq!(app_pouch.gems_mut().get(&key).unwrap().key, key);
    }
}
