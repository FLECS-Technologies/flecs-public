pub use crate::Result;
use crate::jeweler::app::{AppId, AppStatus, Token};
use crate::jeweler::deployment::DeploymentId;
use crate::jeweler::gem::deployment::Deployment;
use crate::jeweler::gem::manifest::AppManifest;
use crate::jeweler::{GetDeploymentId, serialize_hashmap_values};
use crate::quest::{Quest, State, SyncQuest};
use crate::vault::pouch;
use crate::vault::pouch::AppKey;
use crate::vault::pouch::deployment::DefaultDeployments;
use flecsd_axum_server::models::InstalledApp;
use futures_util::future::join_all;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use std::mem::swap;
use std::path::PathBuf;
use tracing::error;

#[derive(Debug, Serialize, Clone)]
pub struct AppData {
    pub desired: AppStatus,
    #[serde(serialize_with = "serialize_deployment_id", rename = "deployment_id")]
    deployment: Deployment,
}

fn serialize_deployment_id<S, D>(
    deployment_id_provider: &D,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    D: GetDeploymentId,
    S: Serializer,
{
    serializer.serialize_str(deployment_id_provider.deployment_id().as_str())
}

impl AppData {
    pub fn new(deployment: Deployment) -> Self {
        AppData {
            desired: AppStatus::None,
            deployment,
        }
    }

    pub fn deployment(&self) -> &Deployment {
        &self.deployment
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct App {
    pub key: AppKey,
    #[serde(serialize_with = "serialize_hashmap_values")]
    pub(crate) deployments: HashMap<DeploymentId, AppData>,
    #[serde(skip)]
    manifest: AppManifest,
}

#[derive(Debug, Deserialize)]
pub struct AppDataDeserializable {
    pub desired: AppStatus,
    pub deployment_id: DeploymentId,
    pub id: Option<AppId>,
}

#[derive(Debug, Deserialize)]
pub struct AppDeserializable {
    pub key: AppKey,
    pub deployments: Vec<AppDataDeserializable>,
}

pub fn try_create_app(
    app: AppDeserializable,
    manifests: &pouch::manifest::Gems,
    deployments: &pouch::deployment::Gems,
) -> anyhow::Result<App> {
    let deployments = app
        .deployments
        .into_iter()
        .filter_map(|data| match deployments.get(&data.deployment_id) {
            Some(deployment) => Some((
                data.deployment_id,
                AppData {
                    desired: data.desired,
                    deployment: deployment.clone(),
                },
            )),
            None => {
                // TODO: Decide if returning an error would be better
                error!(
                    "Ignoring unknown deployment {} of {}",
                    data.deployment_id, app.key
                );
                None
            }
        })
        .collect();
    let Some(manifest) = manifests.get(&app.key).cloned() else {
        anyhow::bail!("No manifest found for {}", app.key);
    };
    Ok(App {
        deployments,
        manifest,
        key: app.key,
    })
}

pub fn try_create_legacy_app(
    app_key: AppKey,
    manifests: &pouch::manifest::Gems,
    default_deployments: DefaultDeployments,
) -> anyhow::Result<App> {
    let Some(manifest) = manifests.get(&app_key).cloned() else {
        anyhow::bail!("No manifest found for {}", app_key);
    };
    let deployment = match (&manifest, default_deployments) {
        (
            AppManifest::Single(_),
            DefaultDeployments {
                docker: Some(deployment),
                ..
            },
        ) => deployment,
        (
            AppManifest::Multi(_),
            DefaultDeployments {
                compose: Some(deployment),
                ..
            },
        ) => deployment,
        _ => anyhow::bail!("No deployment found for {}", app_key),
    };
    Ok(App {
        deployments: HashMap::from([(
            deployment.id().to_string(),
            AppData {
                deployment,
                desired: AppStatus::Installed,
            },
        )]),
        manifest,
        key: app_key,
    })
}

impl App {
    pub fn set_desired(&mut self, status: AppStatus) {
        for data in self.deployments.values_mut() {
            data.desired = status;
        }
    }

    pub fn replace_manifest(&mut self, mut manifest: AppManifest) -> AppManifest {
        swap(&mut self.manifest, &mut manifest);
        manifest
    }

    pub fn manifest(&self) -> &AppManifest {
        &self.manifest
    }

    pub fn new(key: AppKey, deployments: Vec<Deployment>, manifest: AppManifest) -> Self {
        Self {
            key,
            manifest,
            deployments: deployments
                .into_iter()
                .map(|deployment| (deployment.id().clone(), AppData::new(deployment)))
                .collect(),
        }
    }

    pub async fn try_create_installed_info(&self) -> anyhow::Result<InstalledApp> {
        Ok(Self::create_installed_info(
            self.deployments
                .values()
                .next()
                .ok_or_else(|| anyhow::anyhow!("No data available"))?,
            self.key.clone(),
            self.status().await.ok(),
            self.installed_size().await.ok(),
            &self.manifest,
        ))
    }

    fn create_installed_info(
        data: &AppData,
        app_key: AppKey,
        status: Option<AppStatus>,
        installed_size: Option<usize>,
        manifest: &AppManifest,
    ) -> InstalledApp {
        InstalledApp {
            app_key: app_key.into(),
            status: status.unwrap_or_default().into(),
            desired: data.desired.into(),
            installed_size: installed_size.unwrap_or_default() as i32,
            multi_instance: match manifest {
                AppManifest::Multi(_) => false,
                AppManifest::Single(single) => single.multi_instance(),
            },
        }
    }

    pub async fn status(&self) -> crate::Result<AppStatus> {
        let data = self
            .deployments
            .values()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No data available"))?;
        let manifest = self.manifest.clone();
        match data
            .deployment
            .is_app_installed(
                Quest::new_synced("Check app installation status".to_string()),
                manifest,
            )
            .await?
        {
            true => Ok(AppStatus::Installed),
            false => Ok(AppStatus::NotInstalled),
        }
    }

    pub async fn installed_size(&self) -> crate::Result<usize> {
        let data = self
            .deployments
            .values()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No data available"))?;
        let manifest = self.manifest.clone();
        data.deployment
            .installed_app_size(
                Quest::new_synced("Check app installation size".to_string()),
                manifest,
            )
            .await
    }

    pub async fn install(&mut self, quest: SyncQuest, token: Option<Token>) -> anyhow::Result<()> {
        let mut deployment_ids = Vec::new();
        let mut install_app_results = Vec::new();
        for data in self.deployments.values_mut() {
            data.desired = AppStatus::Installed;
            let deployment = data.deployment.clone();
            let manifest = self.manifest.clone();
            let token = token.clone();
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
                        let is_installed = deployment
                            .is_app_installed(quest.clone(), manifest.clone())
                            .await?;
                        if is_installed {
                            quest.lock().await.state = State::Skipped;
                            quest.lock().await.detail = Some("Already installed".to_string());
                            Ok(())
                        } else {
                            deployment.install_app(quest, manifest.clone(), token).await
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
                    error!(
                        "Failed to install {} to deployment {}: {e}",
                        self.key, deployment_id
                    )
                }
                Ok(_) => {
                    if self.deployments.contains_key(&deployment_id) {
                        success_count += 1;
                    } else {
                        error!("No app data for deployment {} found ", deployment_id)
                    }
                }
            }
        }
        if success_count == 0 && !self.deployments.is_empty() {
            anyhow::bail!(
                "Failed to install {} in any of the {} deployments",
                self.key,
                self.deployments.len()
            );
        } else {
            Ok(())
        }
    }

    async fn uninstall_from_deployment(
        quest: SyncQuest,
        mut data: AppData,
        manifest: AppManifest,
    ) -> Result<(), (anyhow::Error, AppData)> {
        data.desired = AppStatus::NotInstalled;
        let app_key = manifest.key().clone();
        let app_installed = quest
            .lock()
            .await
            .create_sub_quest(
                format!(
                    "Check if app {app_key} is installed in deployment {}",
                    data.deployment.id()
                ),
                |quest| {
                    let deployment = data.deployment.clone();
                    let manifest = manifest.clone();
                    async move { deployment.is_app_installed(quest, manifest).await }
                },
            )
            .await
            .2;
        let app_installed = app_installed.await;
        match app_installed {
            Ok(false) => {
                let mut quest = quest.lock().await;
                quest.state = State::Skipped;
                quest.detail = Some(format!(
                    "App is not installed on deployment {}",
                    data.deployment.id()
                ));
                return Ok(());
            }
            Err(e) => {
                return Err((e, data));
            }
            Ok(true) => {}
        }
        let result = quest
            .lock()
            .await
            .create_sub_quest(
                format!(
                    "Uninstall app {app_key} from deployment {}",
                    data.deployment.id()
                ),
                |quest| async move {
                    match data.deployment.uninstall_app(quest, manifest).await {
                        Ok(()) => Ok::<_, anyhow::Error>(None),
                        Err(e) => Ok::<_, anyhow::Error>(Some((e, data))),
                    }
                },
            )
            .await
            .2;
        let result = result.await;
        match result {
            Ok(None) => Ok(()),
            Ok(Some((e, data))) => Err((e, data)),
            Err(e) => {
                error!("Unexpected error during uninstallation of app {app_key}: {e}");
                Ok(())
            }
        }
    }

    pub async fn uninstall(mut self, quest: SyncQuest) -> Result<(), (anyhow::Error, Self)> {
        let mut deployment_ids = Vec::new();
        let mut uninstall_results = Vec::new();
        let mut app_data = HashMap::new();
        swap(&mut app_data, &mut self.deployments);
        for data in app_data.into_values() {
            deployment_ids.push(data.deployment.id().clone());
            let manifest = self.manifest().clone();
            let uninstall_result = quest
                .lock()
                .await
                .create_sub_quest(
                    format!(
                        "Uninstall {} from deployment {}",
                        self.key,
                        data.deployment.id()
                    ),
                    |quest| async move {
                        match Self::uninstall_from_deployment(quest, data, manifest).await {
                            Ok(()) => Ok::<_, anyhow::Error>(None),
                            Err((e, data)) => Ok::<_, anyhow::Error>(Some((e, data))),
                        }
                    },
                )
                .await
                .2;
            uninstall_results.push(uninstall_result);
        }
        let mut errors = Vec::new();
        for (deployment_id, result) in deployment_ids
            .into_iter()
            .zip(join_all(uninstall_results).await)
        {
            match result {
                Err(e) => errors.push(format!("from deployment {deployment_id}: {e}")),
                Ok(None) => {}
                Ok(Some((e, app_data))) => {
                    errors.push(format!("from deployment {deployment_id}: {e}"));
                    self.deployments.insert(deployment_id, app_data);
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err((
                anyhow::anyhow!("Failed to uninstall {}, {}", self.key, errors.join(", ")),
                self,
            ))
        }
    }

    pub async fn export(&self, quest: SyncQuest, path: PathBuf) -> Result<(), anyhow::Error> {
        let Some((_, app_data)) = self.deployments.iter().next() else {
            anyhow::bail!("App {} is not installed in any deployment", self.key);
        };
        let path = path.join(format!("{}_{}", self.key.name, self.key.version));
        tokio::fs::create_dir_all(&path).await?;
        app_data
            .deployment
            .export_app(quest, self.manifest.clone(), path.clone())
            .await?;
        let manifest_path = path.join(format!(
            "{}_{}.manifest.json",
            self.key.name, self.key.version
        ));
        tokio::fs::write(&manifest_path, serde_json::to_vec_pretty(&self.manifest)?).await?;
        let app_path = path.join(format!("{}_{}.json", self.key.name, self.key.version));
        tokio::fs::write(&app_path, serde_json::to_vec_pretty(&self)?).await?;
        Ok(())
    }

    pub async fn import(&self, quest: SyncQuest, path: PathBuf) -> Result<(), anyhow::Error> {
        let mut results = Vec::new();
        {
            let mut quest = quest.lock().await;
            for data in self.deployments.values() {
                let deployment = data.deployment().clone();
                let path = path.clone();
                let manifest = self.manifest().clone();
                let result =
                    quest
                        .create_sub_quest(
                            format!("Import {} to {}", self.key, deployment.id()),
                            move |quest| async move {
                                deployment.import_app(quest, manifest, path).await
                            },
                        )
                        .await
                        .2;
                results.push(result);
            }
        }
        for result in join_all(results).await {
            result?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::jeweler::gem::deployment::Deployment;
    use crate::jeweler::gem::deployment::docker::tests::MockedDockerDeployment;
    use crate::jeweler::gem::manifest::single::tests::{
        create_test_manifest, create_test_manifest_raw,
    };
    use crate::vault::pouch::manifest::tests::min_app_1_0_0_manifest;
    use std::sync::Arc;

    pub fn test_key() -> AppKey {
        test_key_numbered(0, 0)
    }

    pub fn test_key_numbered(name_number: u8, version_number: u8) -> AppKey {
        AppKey {
            name: format!("some.test.app-{name_number}"),
            version: format!("1.2.{version_number}"),
        }
    }
    #[tokio::test]
    async fn status_no_deployment() {
        let app = App::new(test_key(), vec![], min_app_1_0_0_manifest());
        assert!(app.status().await.is_err());
    }

    #[test]
    fn set_desired() {
        let deployments = [
            "MockedDeployment#1",
            "MockedDeployment#2",
            "MockedDeployment#3",
            "MockedDeployment#4",
        ]
        .into_iter()
        .map(|id| {
            let mut deployment = MockedDockerDeployment::new();
            deployment.expect_id().return_const(id.to_string());
            Deployment::Docker(Arc::new(deployment))
        })
        .collect();
        let mut app = App::new(test_key(), deployments, min_app_1_0_0_manifest());
        assert_eq!(app.deployments.len(), 4);
        for data in app.deployments.values() {
            assert_eq!(data.desired, AppStatus::None);
        }
        app.set_desired(AppStatus::Installed);
        for data in app.deployments.values() {
            assert_eq!(data.desired, AppStatus::Installed);
        }
    }

    #[tokio::test]
    async fn status_installed() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_is_app_installed()
            .times(1)
            .returning(|_, _| Ok(true));
        deployment.expect_id().return_const("id".to_string());
        let deployment = Deployment::Docker(Arc::new(deployment));
        let app = App::new(test_key(), vec![deployment], min_app_1_0_0_manifest());
        assert_eq!(app.status().await.unwrap(), AppStatus::Installed);
    }

    #[tokio::test]
    async fn status_not_installed() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_is_app_installed()
            .times(1)
            .returning(|_, _| Ok(false));
        deployment.expect_id().return_const("id".to_string());
        let deployment = Deployment::Docker(Arc::new(deployment));
        let app = App::new(test_key(), vec![deployment], min_app_1_0_0_manifest());
        assert_eq!(app.status().await.unwrap(), AppStatus::NotInstalled);
    }

    #[tokio::test]
    async fn size_no_deployment() {
        let app = App::new(test_key(), vec![], min_app_1_0_0_manifest());
        assert!(app.installed_size().await.is_err());
    }

    #[tokio::test]
    async fn test_size_installed() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_installed_app_size()
            .times(1)
            .returning(|_, _| Ok(6520));
        deployment.expect_id().return_const("id".to_string());
        let deployment = Deployment::Docker(Arc::new(deployment));
        let app = App::new(test_key(), vec![deployment], min_app_1_0_0_manifest());
        assert_eq!(app.installed_size().await.unwrap(), 6520);
    }

    #[tokio::test]
    async fn size_no_size_test() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_installed_app_size()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("No size")));
        deployment.expect_id().return_const("id".to_string());
        let deployment = Deployment::Docker(Arc::new(deployment));
        let app = App::new(test_key(), vec![deployment], min_app_1_0_0_manifest());
        assert!(app.installed_size().await.is_err());
    }

    #[tokio::test]
    async fn try_create_app_info_no_deployment() {
        let app = App::new(test_key(), vec![], min_app_1_0_0_manifest());
        assert!(app.try_create_installed_info().await.is_err());
    }

    #[tokio::test]
    async fn create_app_info_defaults() {
        let data = AppData {
            desired: AppStatus::Installed,
            deployment: Deployment::Docker(Arc::new(MockedDockerDeployment::new())),
        };
        let info =
            App::create_installed_info(&data, test_key(), None, None, &create_test_manifest(None));
        assert_eq!(info.installed_size, 0);
        assert_eq!(info.status, flecsd_axum_server::models::AppStatus::Unknown);
        assert!(!info.multi_instance);
    }

    #[tokio::test]
    async fn create_app_info() {
        let data = AppData {
            desired: AppStatus::Installed,
            deployment: Deployment::Docker(Arc::new(MockedDockerDeployment::new())),
        };
        let info = App::create_installed_info(
            &data,
            test_key(),
            Some(AppStatus::Installed),
            Some(78990),
            &create_test_manifest(None),
        );
        assert_eq!(info.installed_size, 78990);
        assert_eq!(
            info.status,
            flecsd_axum_server::models::AppStatus::Installed
        );
        assert!(!info.multi_instance);
    }

    #[tokio::test]
    async fn try_create_app_info_ok() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_installed_app_size()
            .once()
            .returning(|_, _| Ok(1234));
        deployment
            .expect_is_app_installed()
            .once()
            .returning(|_, _| Ok(true));
        deployment.expect_id().return_const("id".to_string());
        let deployment = Deployment::Docker(Arc::new(deployment));
        let mut app = App::new(test_key(), vec![deployment], min_app_1_0_0_manifest());
        let mut manifest = create_test_manifest_raw(None);
        if let flecs_app_manifest::generated::manifest_3_1_0::FlecsAppManifest::Single(single) =
            &mut manifest
        {
            single.multi_instance = Some(true.into());
        }
        let manifest = AppManifest::try_from(
            flecs_app_manifest::AppManifest::try_from(
                flecs_app_manifest::AppManifestVersion::V3_1_0(manifest),
            )
            .unwrap(),
        )
        .unwrap();
        app.replace_manifest(manifest);
        let info = app.try_create_installed_info().await.unwrap();
        assert_eq!(
            info.status,
            flecsd_axum_server::models::AppStatus::Installed
        );
        assert_eq!(info.installed_size, 1234);
        assert!(info.multi_instance);
    }

    #[tokio::test]
    async fn uninstall_app_not_installed() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_is_app_installed()
            .times(1)
            .returning(|_, _| Ok(false));
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        let deployment = Deployment::Docker(Arc::new(deployment));
        let app_data = AppData::new(deployment);
        let quest = Quest::new_synced("TestQuest".to_string());
        App::uninstall_from_deployment(quest.clone(), app_data, min_app_1_0_0_manifest())
            .await
            .unwrap();
        assert_eq!(quest.lock().await.state, State::Skipped);
    }

    #[tokio::test]
    async fn uninstall_app_data_ok() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_is_app_installed()
            .times(1)
            .returning(|_, _| Ok(true));
        deployment
            .expect_uninstall_app()
            .times(1)
            .returning(|_, _| Ok(()));
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        let deployment = Deployment::Docker(Arc::new(deployment));
        let mut app_data = AppData::new(deployment);
        app_data.desired = AppStatus::Installed;
        App::uninstall_from_deployment(
            Quest::new_synced("TestQuest".to_string()),
            app_data,
            min_app_1_0_0_manifest(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn uninstall_app_data_err() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_is_app_installed()
            .times(1)
            .returning(|_, _| Ok(true));
        deployment
            .expect_uninstall_app()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        let deployment = Deployment::Docker(Arc::new(deployment));
        let mut app_data = AppData::new(deployment);
        app_data.desired = AppStatus::Installed;
        let result = App::uninstall_from_deployment(
            Quest::new_synced("TestQuest".to_string()),
            app_data,
            min_app_1_0_0_manifest(),
        )
        .await;
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().1.desired, AppStatus::NotInstalled);
    }

    #[tokio::test]
    async fn uninstall_ok() {
        const DEPLOYMENT_COUNT: usize = 5;
        let mut deployments = HashMap::new();
        for i in 0..DEPLOYMENT_COUNT {
            let mut deployment = MockedDockerDeployment::new();
            let deployment_id = format!("MockedDeployment-{}", i);
            deployment.expect_id().return_const(deployment_id.clone());
            deployment
                .expect_is_app_installed()
                .once()
                .returning(|_, _| Ok(true));
            deployment
                .expect_uninstall_app()
                .times(1)
                .returning(|_, _| Ok(()));
            let deployment = Deployment::Docker(Arc::new(deployment));
            deployments.insert(deployment_id, deployment);
        }
        let manifest = min_app_1_0_0_manifest();
        let mut app = AppDeserializable {
            key: manifest.key().clone(),
            deployments: Vec::new(),
        };
        for i in 0..DEPLOYMENT_COUNT {
            let deployment_id = format!("MockedDeployment-{}", i);
            app.deployments.push(AppDataDeserializable {
                desired: AppStatus::Installed,
                deployment_id,
                id: Some(format!("MockedAppId-{}", i)),
            });
        }
        let app = try_create_app(
            app,
            &HashMap::from([(manifest.key().clone(), manifest)]),
            &deployments,
        )
        .unwrap();
        assert_eq!(app.deployments.len(), DEPLOYMENT_COUNT);
        app.uninstall(Quest::new_synced("TestQuest".to_string()))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn uninstall_err() {
        const DEPLOYMENT_COUNT: usize = 5;
        const ERR_DEPLOYMENT_COUNT: usize = 3;
        let mut deployments = HashMap::new();
        for i in 0..DEPLOYMENT_COUNT {
            let mut deployment = MockedDockerDeployment::new();
            let deployment_id = format!("MockedDeployment-{}", i);
            deployment.expect_id().return_const(deployment_id.clone());
            deployment
                .expect_is_app_installed()
                .times(1)
                .returning(|_, _| Ok(true));
            deployment
                .expect_uninstall_app()
                .times(1)
                .returning(|_, _| Ok(()));
            let deployment = deployment;
            deployments.insert(deployment_id, Deployment::Docker(Arc::new(deployment)));
        }
        for i in DEPLOYMENT_COUNT..DEPLOYMENT_COUNT + ERR_DEPLOYMENT_COUNT {
            let mut deployment = MockedDockerDeployment::new();
            let deployment_id = format!("MockedDeployment-{}", i);
            let closure_deployment_id = deployment_id.clone();
            deployment
                .expect_id()
                .return_const(closure_deployment_id.clone());
            deployment
                .expect_is_app_installed()
                .once()
                .returning(|_, _| Ok(true));
            deployment
                .expect_uninstall_app()
                .times(1)
                .returning(|_, _| Err(anyhow::anyhow!("TestError")));
            let deployment = deployment;
            deployments.insert(deployment_id, Deployment::Docker(Arc::new(deployment)));
        }
        let manifest = min_app_1_0_0_manifest();
        let mut app = AppDeserializable {
            key: manifest.key().clone(),
            deployments: Vec::new(),
        };
        for i in 0..DEPLOYMENT_COUNT + ERR_DEPLOYMENT_COUNT {
            let deployment_id = format!("MockedDeployment-{}", i);
            app.deployments.push(AppDataDeserializable {
                desired: AppStatus::Installed,
                deployment_id,
                id: Some(format!("MockedAppId-{}", i)),
            });
        }
        let app = try_create_app(
            app,
            &HashMap::from([(manifest.key().clone(), manifest)]),
            &deployments,
        )
        .unwrap();
        assert_eq!(
            app.deployments.len(),
            DEPLOYMENT_COUNT + ERR_DEPLOYMENT_COUNT
        );
        let result = app
            .uninstall(Quest::new_synced("TestQuest".to_string()))
            .await;
        assert!(result.is_err());
        let (_error, app) = result.err().unwrap();
        assert_eq!(app.deployments.len(), ERR_DEPLOYMENT_COUNT);
        for i in DEPLOYMENT_COUNT..DEPLOYMENT_COUNT + ERR_DEPLOYMENT_COUNT {
            assert!(
                app.deployments
                    .contains_key(&format!("MockedDeployment-{}", i))
            );
        }
    }
}
