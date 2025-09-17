pub use crate::Result;
use crate::jeweler::gem::app::{App, AppDeserializable, try_create_app};
use crate::lore::Lore;
use crate::vault::pouch::{AppKey, Pouch};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::error;

const APPS_FILE_NAME: &str = "apps.json";
pub type Gems = HashMap<AppKey, App>;

pub struct AppPouch {
    lore: Arc<Lore>,
    apps: Gems,
}

impl Pouch for AppPouch {
    type Gems = Gems;

    fn gems(&self) -> &Self::Gems {
        &self.apps
    }

    fn gems_mut(&mut self) -> &mut Self::Gems {
        &mut self.apps
    }
}

impl AppPouch {
    fn base_path(&self) -> &Path {
        &self.lore.app.base_path
    }

    pub(in super::super) fn close(&mut self) -> Result<()> {
        let base_path = self.base_path();
        fs::create_dir_all(base_path)?;
        let file = fs::File::create(base_path.join(APPS_FILE_NAME))?;
        let content: Vec<_> = self.apps.values().collect();
        serde_json::to_writer_pretty(file, &content)?;
        Ok(())
    }

    pub(in super::super) fn open(
        &mut self,
        manifests: &super::manifest::Gems,
        deployments: &super::deployment::Gems,
    ) -> Result<()> {
        self.apps = Self::create_apps(self.read_apps()?, manifests, deployments, self.lore.clone());
        Ok(())
    }

    fn read_apps(&self) -> Result<Vec<AppDeserializable>> {
        let file = fs::File::open(self.base_path().join(APPS_FILE_NAME))?;
        Ok(serde_json::from_reader(file)?)
    }

    fn create_apps(
        apps: Vec<AppDeserializable>,
        manifests: &super::manifest::Gems,
        deployments: &super::deployment::Gems,
        lore: Arc<Lore>,
    ) -> HashMap<AppKey, App> {
        apps.into_iter()
            .filter_map(|app| {
                let key = app.key.clone();
                match try_create_app(app, manifests, deployments, lore.clone()) {
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
    pub fn new(lore: Arc<Lore>) -> Self {
        Self {
            lore,
            apps: HashMap::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::jeweler::app::AppStatus;
    use crate::jeweler::gem::app::AppDataDeserializable;
    use crate::jeweler::gem::deployment::Deployment;
    use crate::jeweler::gem::deployment::docker::DockerDeploymentImpl;
    use crate::jeweler::gem::deployment::docker::tests::MockedDockerDeployment;
    use crate::jeweler::gem::manifest::AppManifest;
    use crate::relic::var::test::MockVarReader;
    use crate::{jeweler, lore};
    use serde_json::Value;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;
    use testdir::testdir;

    fn default_deployment() -> Deployment {
        let mut default_deployment = MockedDockerDeployment::default();
        default_deployment
            .expect_id()
            .return_const("DefaultMockedDeployment".to_string());
        Deployment::Docker(Arc::new(default_deployment))
    }

    pub fn test_app_pouch(
        manifests: &super::super::manifest::Gems,
        mut deployments: HashMap<AppKey, Deployment>,
        fallback_deployment: Option<Deployment>,
    ) -> AppPouch {
        let default_deployment = fallback_deployment.unwrap_or_else(default_deployment);
        let mut apps = test_apps();
        for app in apps.iter_mut() {
            let entry = deployments
                .entry(app.key.clone())
                .or_insert(default_deployment.clone());
            app.deployments.first_mut().unwrap().deployment_id = entry.id().clone();
        }
        let deployments = deployments
            .into_values()
            .map(|deployment| (deployment.id().clone(), deployment))
            .collect();
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        AppPouch {
            apps: AppPouch::create_apps(apps, manifests, &deployments, lore.clone()),
            lore,
        }
    }

    fn test_apps() -> Vec<AppDeserializable> {
        vec![
            minimal_app(),
            single_instance_app(),
            multi_instance_app(),
            minimal_app_with_instance(),
            minimal_app_2(),
            label_app(),
            editor_app(),
            network_app(),
            mount_app(),
        ]
    }

    pub fn existing_app_keys() -> Vec<AppKey> {
        test_apps().into_iter().map(|app| app.key).collect()
    }

    pub const UNKNOWN_APP_NAME: &str = "tech.flecs.unknown";
    pub const UNKNOWN_APP_VERSION: &str = "1.1.4";

    pub const MINIMAL_APP_WITH_INSTANCE_NAME: &str = "tech.flecs.min-app";
    pub const MINIMAL_APP_WITH_INSTANCE_VERSION: &str = "1.1.4";
    fn minimal_app_with_instance() -> AppDeserializable {
        AppDeserializable {
            key: AppKey {
                name: MINIMAL_APP_WITH_INSTANCE_NAME.to_string(),
                version: MINIMAL_APP_WITH_INSTANCE_VERSION.to_string(),
            },
            deployments: vec![AppDataDeserializable {
                id: Some("887665412".to_string()),
                deployment_id: "".to_string(),
                desired: AppStatus::Installed,
            }],
        }
    }

    pub const MINIMAL_APP_NAME: &str = "tech.flecs.min-app";
    pub const MINIMAL_APP_VERSION: &str = "1.0.0";
    fn minimal_app() -> AppDeserializable {
        AppDeserializable {
            key: AppKey {
                name: MINIMAL_APP_NAME.to_string(),
                version: MINIMAL_APP_VERSION.to_string(),
            },
            deployments: vec![AppDataDeserializable {
                id: Some("12345678".to_string()),
                deployment_id: "".to_string(),
                desired: AppStatus::Installed,
            }],
        }
    }

    pub const MINIMAL_APP_2_NAME: &str = "tech.flecs.min-app";
    pub const MINIMAL_APP_2_VERSION: &str = "2.4.5";
    fn minimal_app_2() -> AppDeserializable {
        AppDeserializable {
            key: AppKey {
                name: MINIMAL_APP_2_NAME.to_string(),
                version: MINIMAL_APP_2_VERSION.to_string(),
            },
            deployments: vec![AppDataDeserializable {
                id: Some("1234005678".to_string()),
                deployment_id: "".to_string(),
                desired: AppStatus::Installed,
            }],
        }
    }

    pub const SINGLE_INSTANCE_APP_NAME: &str = "tech.flecs.single-instance";
    pub const SINGLE_INSTANCE_APP_VERSION: &str = "1.0.0";
    fn single_instance_app() -> AppDeserializable {
        AppDeserializable {
            key: AppKey {
                name: SINGLE_INSTANCE_APP_NAME.to_string(),
                version: SINGLE_INSTANCE_APP_VERSION.to_string(),
            },
            deployments: vec![AppDataDeserializable {
                id: Some("ababababab".to_string()),
                deployment_id: "".to_string(),
                desired: AppStatus::Installed,
            }],
        }
    }

    pub const MULTI_INSTANCE_APP_NAME: &str = "tech.flecs.multi-instance";
    pub const MULTI_INSTANCE_APP_VERSION: &str = "1.0.0";
    fn multi_instance_app() -> AppDeserializable {
        AppDeserializable {
            key: AppKey {
                name: MULTI_INSTANCE_APP_NAME.to_string(),
                version: MULTI_INSTANCE_APP_VERSION.to_string(),
            },
            deployments: vec![AppDataDeserializable {
                id: Some("cdcdcdcdcdc".to_string()),
                deployment_id: "".to_string(),
                desired: AppStatus::Installed,
            }],
        }
    }

    pub const LABEL_APP_NAME: &str = "tech.flecs.label-app";
    pub const LABEL_APP_VERSION: &str = "7.6.2";
    fn label_app() -> AppDeserializable {
        AppDeserializable {
            key: AppKey {
                name: LABEL_APP_NAME.to_string(),
                version: LABEL_APP_VERSION.to_string(),
            },
            deployments: vec![AppDataDeserializable {
                id: Some("7171717171".to_string()),
                deployment_id: "".to_string(),
                desired: AppStatus::Installed,
            }],
        }
    }

    pub const EDITOR_APP_NAME: &str = "tech.flecs.editor-app";
    pub const EDITOR_APP_VERSION: &str = "5.2.1";
    fn editor_app() -> AppDeserializable {
        AppDeserializable {
            key: AppKey {
                name: EDITOR_APP_NAME.to_string(),
                version: EDITOR_APP_VERSION.to_string(),
            },
            deployments: vec![AppDataDeserializable {
                id: Some("67438969213497".to_string()),
                deployment_id: "".to_string(),
                desired: AppStatus::Installed,
            }],
        }
    }

    pub const NETWORK_APP_NAME: &str = "tech.flecs.network-app";
    pub const NETWORK_APP_VERSION: &str = "1.2.12";
    fn network_app() -> AppDeserializable {
        AppDeserializable {
            key: AppKey {
                name: NETWORK_APP_NAME.to_string(),
                version: NETWORK_APP_VERSION.to_string(),
            },
            deployments: vec![AppDataDeserializable {
                id: Some("7843584357".to_string()),
                deployment_id: "".to_string(),
                desired: AppStatus::Installed,
            }],
        }
    }

    pub const NO_MANIFEST_APP_NAME: &str = "tech.flecs.no-manifest";
    pub const NO_MANIFEST_APP_VERSION: &str = "1.0.0";

    pub const MOUNT_APP_NAME: &str = "tech.flecs.mount";
    pub const MOUNT_APP_VERSION: &str = "0.4.0";
    fn mount_app() -> AppDeserializable {
        AppDeserializable {
            key: AppKey {
                name: MOUNT_APP_NAME.to_string(),
                version: MOUNT_APP_VERSION.to_string(),
            },
            deployments: vec![AppDataDeserializable {
                id: Some("34c6af34572".to_string()),
                deployment_id: "".to_string(),
                desired: AppStatus::Installed,
            }],
        }
    }

    fn create_test_json() -> Value {
        serde_json::json!([
            {
                "key": {
                    "name": "test-app-1",
                    "version": "1.2.3"
                },
                "deployments": [{
                    "desired": "Installed",
                    "deployment_id": "test-deployment-id-1"
                }]
            }
        ])
    }

    fn create_test_manifests() -> HashMap<AppKey, AppManifest> {
        let manifest: flecs_app_manifest::generated::manifest_3_2_0::Single =
            flecs_app_manifest::generated::manifest_3_2_0::builder::Single::default()
                .app("test-app-1")
                .image("test-image")
                .version("1.2.3".to_string())
                .try_into()
                .unwrap();
        let manifest =
            flecs_app_manifest::generated::manifest_3_2_0::FlecsAppManifest::Single(manifest);
        let manifest = flecs_app_manifest::AppManifestVersion::V3_2_0(manifest);
        let manifest = flecs_app_manifest::AppManifest::try_from(manifest).unwrap();
        let manifest = AppManifest::try_from(manifest).unwrap();
        HashMap::from([(manifest.key().clone(), manifest)])
    }

    fn create_test_deployments() -> HashMap<jeweler::deployment::DeploymentId, Deployment> {
        let deployment = Deployment::Docker(Arc::new(DockerDeploymentImpl::new(
            "test-deployment-id-1".to_string(),
            PathBuf::from("/var/run/docker.sock"),
        )));
        HashMap::from([(deployment.id().clone(), deployment)])
    }

    fn create_test_app() -> App {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let manifests = create_test_manifests();
        let deployments = create_test_deployments();
        try_create_app(
            create_test_app_deserializable(),
            &manifests,
            &deployments,
            lore,
        )
        .unwrap()
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
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        fs::create_dir_all(&lore.app.base_path).unwrap();
        let path = lore.app.base_path.clone();
        let json = create_test_json();
        fs::write(
            path.join(APPS_FILE_NAME),
            serde_json::to_string_pretty(&json).unwrap(),
        )
        .unwrap();
        let mut app_pouch = AppPouch {
            apps: HashMap::default(),
            lore,
        };
        app_pouch
            .open(&create_test_manifests(), &create_test_deployments())
            .unwrap();
    }

    #[test]
    fn close_app_pouch() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let path = lore.app.base_path.clone();
        let json = create_test_json();
        let app = create_test_app();
        let mut app_pouch = AppPouch {
            apps: HashMap::from([(app.key.clone(), app)]),
            lore,
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
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut app_pouch = AppPouch { apps: gems, lore };
        assert_eq!(app_pouch.gems().len(), 1);
        assert_eq!(app_pouch.gems().get(&key).unwrap().key, key);
        assert_eq!(app_pouch.gems_mut().len(), 1);
        assert_eq!(app_pouch.gems_mut().get(&key).unwrap().key, key);
    }
}
