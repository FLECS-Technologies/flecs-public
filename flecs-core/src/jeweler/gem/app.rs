use crate::jeweler::app::{AppId, AppStatus};
use crate::jeweler::deployment::{Deployment, DeploymentId};
use crate::jeweler::gem::instance::{Instance, InstanceDeserializable, InstanceId};
use crate::quest::{Quest, State, SyncQuest};
use crate::vault::pouch::AppKey;
use flecs_app_manifest::AppManifest;
use flecsd_axum_server::models::InstalledApp;
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

    pub fn set_id(&mut self, id: AppId) {
        self.id = Some(id);
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
                    id: data.id,
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

    pub async fn try_create_installed_info(&self) -> anyhow::Result<InstalledApp> {
        Ok(Self::create_installed_info(
            self.properties
                .values()
                .next()
                .ok_or_else(|| anyhow::anyhow!("No data available"))?,
            self.key.clone(),
            self.status().await.ok(),
            self.installed_size().await.ok(),
            self.manifest
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Manifest not available"))?,
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
            multi_instance: manifest.multi_instance.unwrap_or(false),
            editors: vec![],
        }
    }

    pub async fn status(&self) -> crate::Result<AppStatus> {
        let data = self
            .properties
            .values()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No data available"))?;
        match data
            .deployment
            .is_app_installed(
                Quest::new_synced("Check app installation status".to_string()),
                data.id
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("No app id available"))?,
            )
            .await?
        {
            true => Ok(AppStatus::Installed),
            false => Ok(AppStatus::NotInstalled),
        }
    }

    pub async fn installed_size(&self) -> crate::Result<usize> {
        let data = self
            .properties
            .values()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No data available"))?;
        data.deployment
            .installed_app_size(
                Quest::new_synced("Check app installation size".to_string()),
                data.id
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("No app id available"))?,
            )
            .await
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::app::AppInfo;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::sorcerer::appraiser::tests::create_test_manifest;
    use flecs_app_manifest::AppManifestVersion;

    fn test_key() -> AppKey {
        AppKey {
            name: "test-app".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    #[tokio::test]
    async fn status_no_deployment() {
        let app = App::new(test_key(), vec![]);
        assert!(app.status().await.is_err());
    }

    #[tokio::test]
    async fn status_no_id() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_app_info().times(0);
        deployment.expect_id().returning(|| "id".to_string());
        let app = App::new(
            test_key(),
            vec![Arc::new(deployment) as Arc<dyn Deployment>],
        );
        assert!(app.status().await.is_err());
    }

    #[tokio::test]
    async fn status_installed() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_app_info()
            .times(1)
            .returning(|_, _| Ok(AppInfo::default()));
        deployment.expect_id().returning(|| "id".to_string());
        let mut app = App::new(
            test_key(),
            vec![Arc::new(deployment) as Arc<dyn Deployment>],
        );
        app.properties.values_mut().next().unwrap().id = Some("DataId".to_string());
        assert_eq!(app.status().await.unwrap(), AppStatus::Installed);
    }

    #[tokio::test]
    async fn status_not_installed() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_app_info()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("App not found")));
        deployment.expect_id().returning(|| "id".to_string());
        let mut app = App::new(
            test_key(),
            vec![Arc::new(deployment) as Arc<dyn Deployment>],
        );
        app.properties.values_mut().next().unwrap().id = Some("DataId".to_string());
        assert_eq!(app.status().await.unwrap(), AppStatus::NotInstalled);
    }

    #[tokio::test]
    async fn size_no_deployment() {
        let app = App::new(test_key(), vec![]);
        assert!(app.installed_size().await.is_err());
    }

    #[tokio::test]
    async fn size_no_id() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_app_info().times(0);
        deployment.expect_id().returning(|| "id".to_string());
        let app = App::new(
            test_key(),
            vec![Arc::new(deployment) as Arc<dyn Deployment>],
        );
        assert!(app.installed_size().await.is_err());
    }

    #[tokio::test]
    async fn test_size_installed() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_app_info().times(1).returning(|_, _| {
            Ok(AppInfo {
                size: Some(6520),
                ..AppInfo::default()
            })
        });
        deployment.expect_id().returning(|| "id".to_string());
        let mut app = App::new(
            test_key(),
            vec![Arc::new(deployment) as Arc<dyn Deployment>],
        );
        app.properties.values_mut().next().unwrap().id = Some("DataId".to_string());
        assert_eq!(app.installed_size().await.unwrap(), 6520);
    }

    #[tokio::test]
    async fn size_no_size_test() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_app_info()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("No size")));
        deployment.expect_id().returning(|| "id".to_string());
        let mut app = App::new(
            test_key(),
            vec![Arc::new(deployment) as Arc<dyn Deployment>],
        );
        app.properties.values_mut().next().unwrap().id = Some("DataId".to_string());
        assert!(app.installed_size().await.is_err());
    }

    #[tokio::test]
    async fn try_create_app_info_no_deployment() {
        let app = App::new(test_key(), vec![]);
        assert!(app.try_create_installed_info().await.is_err());
    }

    #[tokio::test]
    async fn create_app_info_defaults() {
        let data = AppData {
            desired: AppStatus::Installed,
            instances: Default::default(),
            id: Some("DataId".to_string()),
            deployment: Arc::new(MockedDeployment::new()),
        };
        let info = App::create_installed_info(
            &data,
            test_key(),
            None,
            None,
            create_test_manifest(None).as_ref(),
        );
        assert_eq!(info.installed_size, 0);
        assert_eq!(info.status, flecsd_axum_server::models::AppStatus::Unknown);
        assert!(!info.multi_instance);
    }

    #[tokio::test]
    async fn create_app_info() {
        let data = AppData {
            desired: AppStatus::Installed,
            instances: Default::default(),
            id: Some("DataId".to_string()),
            deployment: Arc::new(MockedDeployment::new()),
        };
        let info = App::create_installed_info(
            &data,
            test_key(),
            Some(AppStatus::Installed),
            Some(78990),
            create_test_manifest(None).as_ref(),
        );
        assert_eq!(info.installed_size, 78990);
        assert_eq!(
            info.status,
            flecsd_axum_server::models::AppStatus::Installed
        );
        assert!(!info.multi_instance);
    }

    #[tokio::test]
    async fn try_create_app_info_no_manifest() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_app_info()
            .times(2)
            .returning(|_, _| Ok(AppInfo::default()));
        deployment.expect_id().returning(|| "id".to_string());
        let mut app = App::new(
            test_key(),
            vec![Arc::new(deployment) as Arc<dyn Deployment>],
        );
        app.properties.values_mut().next().unwrap().id = Some("DataId".to_string());
        assert!(app.try_create_installed_info().await.is_err());
    }

    #[tokio::test]
    async fn try_create_app_info_ok() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_app_info().times(2).returning(|_, _| {
            Ok(AppInfo {
                size: Some(1234),
                ..AppInfo::default()
            })
        });
        deployment.expect_id().returning(|| "id".to_string());
        let mut app = App::new(
            test_key(),
            vec![Arc::new(deployment) as Arc<dyn Deployment>],
        );
        app.properties.values_mut().next().unwrap().id = Some("DataId".to_string());
        let mut manifest = crate::sorcerer::appraiser::tests::create_test_manifest_raw(None);
        manifest.multi_instance = Some(true);
        let manifest =
            Arc::new(AppManifest::try_from(AppManifestVersion::V3_0_0(manifest)).unwrap());
        app.set_manifest(manifest);
        let info = app.try_create_installed_info().await.unwrap();
        assert_eq!(
            info.status,
            flecsd_axum_server::models::AppStatus::Installed
        );
        assert_eq!(info.installed_size, 1234);
        assert!(info.multi_instance);
    }
}
