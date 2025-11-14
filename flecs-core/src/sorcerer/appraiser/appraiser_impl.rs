use super::AppRaiser;
use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::fsm::console_client::ConsoleClient;
use crate::jeweler::app::{AppStatus, Token};
use crate::jeweler::gem::app::App;
use crate::jeweler::gem::deployment::Deployment;
use crate::jeweler::gem::manifest::AppManifest;
use crate::quest::SyncQuest;
use crate::sorcerer::{Sorcerer, spell};
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::{GrabbedPouches, Vault};
use async_trait::async_trait;
use flecsd_axum_server::models::InstalledApp;
use futures_util::TryFutureExt;
use futures_util::future::join_all;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use tracing::error;

#[derive(Default)]
pub struct AppraiserImpl {}

impl Sorcerer for AppraiserImpl {}

#[async_trait]
impl AppRaiser for AppraiserImpl {
    async fn uninstall_app<F: Floxy + 'static>(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        app_key: AppKey,
    ) -> anyhow::Result<()> {
        let instances_to_delete =
            spell::instance::get_instance_ids_by_app_key(vault.clone(), app_key.clone()).await;
        let no_dependents_result = {
            let vault = vault.clone();
            let instances_to_delete = instances_to_delete.clone();
            quest
                .lock()
                .await
                .create_sub_quest(
                    format!("Validate that no instances of {app_key} is needed as a provider"),
                    |_quest| async move {
                        let GrabbedPouches {
                            instance_pouch_mut: Some(ref mut instances),
                            provider_pouch: Some(ref providers),
                            ..
                        } = vault
                            .reservation()
                            .reserve_instance_pouch_mut()
                            .reserve_provider_pouch()
                            .grab()
                            .await
                        else {
                            unreachable!("Reservation should never fail");
                        };
                        spell::instance::validate_no_dependents_in_slice(
                            providers.gems(),
                            instances.gems(),
                            &instances_to_delete,
                        )
                    },
                )
                .await
                .2
        };
        no_dependents_result.await?;
        let delete_instances_result = quest
            .lock()
            .await
            .create_sub_quest(format!("Remove instances of {app_key}"), |quest| {
                spell::instance::delete_instances(quest, vault.clone(), floxy, instances_to_delete)
                    .map_err(|errors| {
                        anyhow::anyhow!(
                            errors
                                .into_iter()
                                .map(|(error, id)| format!(
                                    "Failed to delete instance {id}: {error}"
                                ))
                                .collect::<Vec<String>>()
                                .join(",")
                        )
                    })
            })
            .await
            .2;
        match (
            delete_instances_result.await,
            spell::app::uninstall_app(quest, vault.clone(), app_key.clone()).await,
        ) {
            (Ok(_), Ok(_)) => {
                spell::manifest::erase_manifest_if_unused(vault, app_key).await;
                Ok(())
            }
            (Err(e1), Err(e2)) => Err(anyhow::anyhow!(
                "Could not uninstall app ({e2}), could not remove all instances ({e1})"
            )),
            (Err(e), Ok(_)) => Err(anyhow::anyhow!(
                "App was uninstalled, but not all instances could be removed: {e}"
            )),
            (Ok(()), Err(e)) => Err(anyhow::anyhow!(
                "Instances were removed but app could not be uninstalled: {e}"
            )),
        }
    }

    // TODO: Unit test
    async fn does_app_exist(&self, vault: Arc<Vault>, app_key: AppKey) -> bool {
        vault
            .reservation()
            .reserve_app_pouch()
            .grab()
            .await
            .app_pouch
            .as_ref()
            .expect("Reservations should never fail")
            .gems()
            .contains_key(&app_key)
    }

    async fn get_app(
        &self,
        vault: Arc<Vault>,
        name: String,
        version: Option<String>,
    ) -> anyhow::Result<Vec<InstalledApp>> {
        let Some(ref apps) = vault
            .reservation()
            .reserve_app_pouch()
            .grab()
            .await
            .app_pouch
        else {
            unreachable!("Reservation failed")
        };
        if let Some(version) = version {
            if let Some(app) = apps.gems().get(&AppKey { name, version }) {
                Ok(vec![app.try_create_installed_info().await?])
            } else {
                Ok(Vec::new())
            }
        } else {
            let mut result = Vec::new();
            for app in apps.gems().values() {
                if app.key.name == name {
                    match app.try_create_installed_info().await {
                        Ok(installed_info) => result.push(installed_info),
                        Err(e) => {
                            error!("Failed to create installed info for app {}: {e}", app.key);
                        }
                    }
                }
            }
            Ok(result)
        }
    }

    async fn get_apps(&self, vault: Arc<Vault>) -> anyhow::Result<Vec<InstalledApp>> {
        let Some(ref mut apps) = vault
            .reservation()
            .reserve_app_pouch()
            .grab()
            .await
            .app_pouch
        else {
            unreachable!("Reservation failed")
        };
        let mut result = Vec::new();
        for app in apps.gems().values() {
            match app.try_create_installed_info().await {
                Ok(installed_info) => result.push(installed_info),
                Err(e) => {
                    error!("Failed to create installed info for app {}: {e}", app.key);
                }
            }
        }
        Ok(result)
    }

    async fn install_app_from_manifest(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        manifest: AppManifest,
        config: ConsoleClient,
    ) -> anyhow::Result<()> {
        let app_key = manifest.key().clone();
        let result = quest
            .lock()
            .await
            .create_sub_quest("Create app".to_string(), |_quest| {
                set_manifest_and_desired_or_create_app(
                    vault.clone(),
                    manifest.clone(),
                    app_key.clone(),
                    AppStatus::Installed,
                )
            })
            .await
            .2;
        result.await?;
        let result = quest
            .lock()
            .await
            .create_sub_quest("Install app".to_string(), |quest| {
                install_existing_app(quest, vault.clone(), app_key, config)
            })
            .await
            .2;
        match result.await {
            Err(e) => Err(e),
            Ok(()) => {
                let result = quest
                    .lock()
                    .await
                    .create_sub_quest(
                        format!("Replace manifest for {}", manifest.key()),
                        |quest| spell::manifest::replace_manifest(quest, vault, manifest),
                    )
                    .await
                    .2;
                result.await?;
                Ok(())
            }
        }
    }

    async fn install_apps(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        app_keys: Vec<AppKey>,
        config: ConsoleClient,
    ) -> anyhow::Result<()> {
        let mut results = Vec::new();
        let mut keys = Vec::new();
        for app_key in app_keys {
            let config = config.clone();
            let vault = vault.clone();
            keys.push(app_key.clone());
            let result = quest
                .lock()
                .await
                .create_sub_quest(format!("Install app {app_key}"), move |quest| async move {
                    Self::default()
                        .install_app(quest, vault, app_key, config)
                        .await
                })
                .await
                .2;
            results.push(result);
        }
        let errors = keys
            .iter()
            .zip(join_all(results).await)
            .filter_map(|(app_key, result)| result.err().map(|e| format!("[{app_key}: {e}]")))
            .collect::<Vec<String>>();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to install {} apps out of {}: {}",
                errors.len(),
                keys.len(),
                errors.join(",")
            ))
        }
    }

    async fn install_app(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        app_key: AppKey,
        config: ConsoleClient,
    ) -> anyhow::Result<()> {
        let manifest = quest
            .lock()
            .await
            .create_sub_quest("Obtain manifest".to_string(), |_quest| {
                download_manifest(vault.clone(), app_key.clone(), config.clone())
            })
            .await
            .2;
        let manifest = manifest.await?;
        self.install_app_from_manifest(quest, vault, manifest, config)
            .await
    }
}

async fn download_manifest(
    vault: Arc<Vault>,
    app_key: AppKey,
    config: ConsoleClient,
) -> anyhow::Result<AppManifest> {
    let session_id = vault
        .get_secrets()
        .await
        .get_session_id()
        .id
        .unwrap_or_default();
    let manifest =
        spell::manifest::download_manifest(config, &session_id, &app_key.name, &app_key.version)
            .await?;
    let manifest = flecs_app_manifest::AppManifest::try_from(manifest)?;
    let manifest = AppManifest::try_from(manifest)?;
    Ok(manifest)
}

async fn set_manifest_and_desired_or_create_app(
    vault: Arc<Vault>,
    manifest: AppManifest,
    app_key: AppKey,
    desired: AppStatus,
) -> anyhow::Result<()> {
    let GrabbedPouches {
        app_pouch_mut: Some(ref mut apps),
        deployment_pouch: Some(ref deployments),
        ..
    } = vault
        .reservation()
        .reserve_app_pouch_mut()
        .reserve_deployment_pouch()
        .grab()
        .await
    else {
        unreachable!("Reservation should never fail");
    };
    match apps.gems_mut().entry(app_key.clone()) {
        Entry::Occupied(mut app) => {
            app.get_mut().replace_manifest(manifest);
            app.get_mut().set_desired(desired);
            Ok(())
        }
        Entry::Vacant(app_entry) => {
            let is_multi_image_app = matches!(manifest, AppManifest::Multi(_));
            let deployments = deployments
                .gems()
                .values()
                .filter_map(|deployment| {
                    if is_multi_image_app == matches!(deployment, Deployment::Compose(_)) {
                        Some(deployment.clone())
                    } else {
                        None
                    }
                })
                .collect();
            let mut app = App::new(app_key, deployments, manifest);
            app.set_desired(desired);
            app_entry.insert(app);
            Ok(())
        }
    }
}

async fn install_existing_app(
    quest: SyncQuest,
    vault: Arc<Vault>,
    app_key: AppKey,
    config: ConsoleClient,
) -> anyhow::Result<()> {
    let result = quest
        .lock()
        .await
        .create_sub_quest(format!("Check if {app_key} exists"), |_quest| {
            let vault = vault.clone();
            let app_key = app_key.clone();
            async move {
                vault
                    .reservation()
                    .reserve_app_pouch()
                    .grab()
                    .await
                    .app_pouch
                    .as_ref()
                    .expect("Reservation failed")
                    .gems()
                    .get(&app_key)
                    .map(|_| ())
                    .ok_or_else(|| anyhow::anyhow!("Expected app {app_key} to already exist"))
            }
        })
        .await
        .2;
    result.await?;
    let token = quest
        .lock()
        .await
        .create_sub_quest(format!("Acquire download token for {app_key}"), |_quest| {
            let app_key = app_key.clone();
            let vault = vault.clone();
            async move {
                let session_id = vault
                    .get_secrets()
                    .await
                    .get_session_id()
                    .id
                    .unwrap_or_default();
                spell::auth::acquire_download_token(
                    config,
                    &session_id,
                    &app_key.name,
                    &app_key.version,
                )
                .await
            }
        })
        .await
        .2;
    let result = quest
        .lock()
        .await
        .create_sub_quest(format!("Install app {}", app_key), |quest| async move {
            let token = token.await?;
            install_app_in_vault(quest, vault.clone(), app_key.clone(), token).await
        })
        .await
        .2;
    result.await
}

async fn install_app_in_vault(
    quest: SyncQuest,
    vault: Arc<Vault>,
    app_key: AppKey,
    token: Option<Token>,
) -> anyhow::Result<()> {
    vault
        .reservation()
        .reserve_app_pouch_mut()
        .grab()
        .await
        .app_pouch_mut
        .as_mut()
        .expect("Reservation failed")
        .gems_mut()
        .get_mut(&app_key)
        .ok_or_else(|| anyhow::anyhow!("App {app_key} was unexpectedly removed"))?
        .install(quest, token)
        .await?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::jeweler::app::AppStatus;
    use crate::jeweler::gem::deployment::docker::tests::MockedDockerDeployment;
    use crate::quest::{Progress, Quest};
    use crate::vault::GrabbedPouches;
    use crate::vault::pouch::Pouch;
    use crate::vault::pouch::app::tests::{
        EDITOR_APP_NAME, LABEL_APP_NAME, MINIMAL_APP_NAME, MINIMAL_APP_VERSION,
        MULTI_INSTANCE_APP_NAME, NO_MANIFEST_APP_NAME, NO_MANIFEST_APP_VERSION,
        SINGLE_INSTANCE_APP_NAME, UNKNOWN_APP_NAME, UNKNOWN_APP_VERSION, existing_app_keys,
    };
    use crate::vault::pouch::manifest::tests::{editor_manifest, no_manifest, test_manifests};
    use crate::vault::tests::{create_empty_test_vault, create_test_vault};
    use flecs_console_client::models::{
        GetApiV2ManifestsAppVersion200Response, PostApiV2Tokens200Response,
        PostApiV2Tokens200ResponseData, PostApiV2Tokens200ResponseDataToken,
    };
    use mockito::{Mock, ServerGuard};
    use std::collections::HashMap;
    use std::sync::Arc;

    async fn token_mock_err(server: &mut ServerGuard, status: usize) -> Mock {
        server
            .mock("POST", "/api/v2/tokens")
            .with_status(status)
            .expect(1)
            .create_async()
            .await
    }

    async fn token_mock_uncalled(server: &mut ServerGuard) -> Mock {
        token_mock_ok_called(server, 0).await
    }

    async fn token_mock_ok_called(server: &mut ServerGuard, hits: usize) -> Mock {
        let body = serde_json::to_vec(&PostApiV2Tokens200Response {
            status_code: 200,
            status_text: None,
            data: Box::new(PostApiV2Tokens200ResponseData {
                token: Box::new(PostApiV2Tokens200ResponseDataToken {
                    username: "peter".to_string(),
                    password: "pw".to_string(),
                }),
            }),
        })
        .unwrap();
        server
            .mock("POST", "/api/v2/tokens")
            .with_status(200)
            .with_body(body.as_slice())
            .expect(hits)
            .create_async()
            .await
    }

    async fn token_mock_ok(server: &mut ServerGuard) -> Mock {
        token_mock_ok_called(server, 1).await
    }

    async fn manifest_mock_err(server: &mut ServerGuard, status: usize, key: &AppKey) -> Mock {
        server
            .mock(
                "GET",
                format!(
                    "/api/v2/manifests/{}/{}?max_manifest_version=3.0.0",
                    key.name, key.version
                )
                .as_str(),
            )
            .with_status(status)
            .expect(1)
            .create_async()
            .await
    }

    async fn manifest_mock_ok(
        server: &mut ServerGuard,
        manifest: AppManifest,
        key: &AppKey,
    ) -> Mock {
        let body = serde_json::to_vec(&GetApiV2ManifestsAppVersion200Response {
            status_code: Some(200),
            status_text: None,
            data: Some(serde_json::to_value(manifest).unwrap()),
        })
        .unwrap();
        server
            .mock(
                "GET",
                format!(
                    "/api/v2/manifests/{}/{}?max_manifest_version=3.0.0",
                    key.name, key.version
                )
                .as_str(),
            )
            .with_status(200)
            .with_body(body.as_slice())
            .expect(1)
            .create_async()
            .await
    }

    #[tokio::test]
    async fn test_get_app_explicit_version() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_is_app_installed()
            .returning(|_, _| Ok(true));
        deployment
            .expect_installed_app_size()
            .returning(|_, _| Ok(200));
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_test_vault(HashMap::new(), HashMap::new(), Some(deployment));
        let appraiser = AppraiserImpl::default();
        for app_key in existing_app_keys()
            .into_iter()
            .filter(|app_key| app_key.name != NO_MANIFEST_APP_NAME)
        {
            assert_eq!(
                appraiser
                    .get_app(vault.clone(), app_key.name, Some(app_key.version))
                    .await
                    .unwrap()
                    .len(),
                1
            );
        }
        assert!(
            appraiser
                .get_app(
                    vault.clone(),
                    UNKNOWN_APP_NAME.to_string(),
                    Some(UNKNOWN_APP_VERSION.to_string())
                )
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn test_get_app_no_version() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_is_app_installed()
            .returning(|_, _| Ok(true));
        deployment
            .expect_installed_app_size()
            .returning(|_, _| Ok(200));
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_test_vault(HashMap::new(), HashMap::new(), Some(deployment));
        let appraiser = AppraiserImpl::default();
        assert_eq!(
            appraiser
                .get_app(vault.clone(), MINIMAL_APP_NAME.to_string(), None)
                .await
                .unwrap()
                .len(),
            3
        );
        assert_eq!(
            appraiser
                .get_app(vault.clone(), UNKNOWN_APP_NAME.to_string(), None)
                .await
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            appraiser
                .get_app(vault.clone(), SINGLE_INSTANCE_APP_NAME.to_string(), None)
                .await
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            appraiser
                .get_app(vault.clone(), MULTI_INSTANCE_APP_NAME.to_string(), None)
                .await
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            appraiser
                .get_app(vault.clone(), LABEL_APP_NAME.to_string(), None)
                .await
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            appraiser
                .get_app(vault.clone(), EDITOR_APP_NAME.to_string(), None)
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(
            appraiser
                .get_app(vault.clone(), NO_MANIFEST_APP_NAME.to_string(), None)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn test_get_apps() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_is_app_installed()
            .returning(|_, _| Ok(true));
        deployment
            .expect_installed_app_size()
            .returning(|_, _| Ok(200));
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_test_vault(HashMap::new(), HashMap::new(), Some(deployment));
        let result = AppraiserImpl::default().get_apps(vault).await.unwrap();
        assert_eq!(result.len(), 9);
        for app_key in existing_app_keys()
            .into_iter()
            .filter(|app_key| app_key.name != NO_MANIFEST_APP_NAME)
        {
            assert!(
                result
                    .iter()
                    .any(|info| AppKey::from(info.app_key.clone()) == app_key)
            );
        }
        assert!(
            !result
                .iter()
                .any(|info| info.app_key.name == NO_MANIFEST_APP_NAME
                    && info.app_key.version == NO_MANIFEST_APP_VERSION)
        );
    }

    #[tokio::test]
    async fn set_manifest_and_desired_or_create_app_existing() {
        let manifest = no_manifest();
        let key = manifest.key().clone();
        let vault = create_test_vault(HashMap::new(), HashMap::new(), None);
        set_manifest_and_desired_or_create_app(
            vault.clone(),
            manifest.clone(),
            key.clone(),
            AppStatus::Installed,
        )
        .await
        .unwrap();
        let grab = vault.reservation().reserve_app_pouch().grab().await;
        let apps = grab.app_pouch.as_ref().unwrap();
        let app = apps.gems().get(&key).unwrap();
        assert_eq!(app.key, key);
        for data in app.deployments.values() {
            assert_eq!(data.desired, AppStatus::Installed);
        }
        assert_eq!(&manifest, app.manifest())
    }

    #[tokio::test]
    async fn set_manifest_and_desired_or_create_app_new() {
        let manifest = no_manifest();
        let key = manifest.key().clone();
        let vault = create_empty_test_vault();
        set_manifest_and_desired_or_create_app(
            vault.clone(),
            manifest.clone(),
            key.clone(),
            AppStatus::NotInstalled,
        )
        .await
        .unwrap();
        let grab = vault.reservation().reserve_app_pouch().grab().await;
        let apps = grab.app_pouch.as_ref().unwrap();
        assert_eq!(apps.gems().len(), 1);
        let app = apps.gems().get(&key).unwrap();
        assert_eq!(app.key, key);
        for data in app.deployments.values() {
            assert_eq!(data.desired, AppStatus::NotInstalled);
        }
        assert_eq!(&manifest, app.manifest())
    }

    #[tokio::test]
    async fn install_non_existing_app() {
        let vault = create_empty_test_vault();
        let (_, config) = crate::tests::create_test_server_and_config().await;
        let key = AppKey {
            name: UNKNOWN_APP_NAME.to_string(),
            version: UNKNOWN_APP_VERSION.to_string(),
        };
        assert!(
            install_existing_app(
                Quest::new_synced("TestQuest".to_string()),
                vault,
                key,
                config,
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn test_install_existing_app() {
        let key = AppKey {
            name: MINIMAL_APP_NAME.to_string(),
            version: MINIMAL_APP_VERSION.to_string(),
        };
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_is_app_installed()
            .once()
            .returning(|_, _| Ok(false));
        deployment
            .expect_install_app()
            .once()
            .returning(|_, _, _| Ok(()));
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_test_vault(
            HashMap::new(),
            HashMap::from([(key.clone(), deployment.clone())]),
            Some(deployment),
        );
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = token_mock_ok(&mut server).await;
        install_existing_app(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            key,
            config,
        )
        .await
        .unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn test_install_existing_app_token_failure() {
        let key = AppKey {
            name: MINIMAL_APP_NAME.to_string(),
            version: MINIMAL_APP_VERSION.to_string(),
        };
        let vault = create_test_vault(HashMap::new(), HashMap::new(), None);
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = token_mock_err(&mut server, 500).await;
        assert!(
            install_existing_app(
                Quest::new_synced("TestQuest".to_string()),
                vault,
                key,
                config,
            )
            .await
            .is_err()
        );
        mock.assert();
    }

    #[tokio::test]
    async fn test_manifest_download() {
        let key = AppKey {
            name: NO_MANIFEST_APP_NAME.to_string(),
            version: NO_MANIFEST_APP_VERSION.to_string(),
        };
        let manifest = no_manifest();
        let vault = create_test_vault(HashMap::new(), HashMap::new(), None);
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = manifest_mock_ok(&mut server, manifest.clone(), &key).await;
        assert_eq!(
            download_manifest(vault.clone(), key, config,)
                .await
                .unwrap(),
            manifest
        );
        mock.assert();
    }

    #[tokio::test]
    async fn test_manifest_download_fail() {
        let key = AppKey {
            name: NO_MANIFEST_APP_NAME.to_string(),
            version: NO_MANIFEST_APP_VERSION.to_string(),
        };
        let vault = create_test_vault(HashMap::new(), HashMap::new(), None);
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = manifest_mock_err(&mut server, 500, &key).await;
        assert!(download_manifest(vault, key, config).await.is_err());
        mock.assert();
    }

    #[tokio::test]
    async fn test_install_token_error() {
        let key = AppKey {
            name: NO_MANIFEST_APP_NAME.to_string(),
            version: NO_MANIFEST_APP_VERSION.to_string(),
        };
        let manifest = no_manifest();
        let vault = create_test_vault(HashMap::new(), HashMap::new(), None);
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let token_mock = token_mock_err(&mut server, 500).await;
        let manifest_mock = manifest_mock_ok(&mut server, manifest.clone(), &key).await;
        let quest = Quest::new_synced("TestQuest".to_string());
        assert!(
            AppraiserImpl::default()
                .install_app(quest.clone(), vault, key, config)
                .await
                .is_err()
        );
        let quest = quest.lock().await;
        assert_eq!(
            quest.sub_quest_progress().await,
            Progress {
                total: Some(3),
                current: 3
            }
        );
        manifest_mock.assert();
        token_mock.assert();
    }

    #[tokio::test]
    async fn test_install() {
        let key = AppKey {
            name: NO_MANIFEST_APP_NAME.to_string(),
            version: NO_MANIFEST_APP_VERSION.to_string(),
        };
        let manifest = editor_manifest();
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_deployment_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_is_app_installed()
            .once()
            .returning(|_, _| Ok(false));
        deployment
            .expect_install_app()
            .once()
            .returning(|_, _, _| Ok(()));
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_test_vault(HashMap::new(), HashMap::new(), Some(deployment));
        let manifest_mock = manifest_mock_ok(&mut server, manifest.clone(), &key).await;
        let token_mock = token_mock_ok(&mut server).await;
        let quest = Quest::new_synced("TestQuest".to_string());
        assert!(
            AppraiserImpl::default()
                .install_app(quest.clone(), vault, key, config)
                .await
                .is_ok()
        );
        let quest = quest.lock().await;
        assert_eq!(
            quest.sub_quest_progress().await,
            Progress {
                total: Some(4),
                current: 4
            }
        );
        manifest_mock.assert();
        token_mock.assert();
    }

    #[tokio::test]
    async fn test_sideload_token_error() {
        let manifest = no_manifest();
        let vault = create_empty_test_vault();
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let token_mock = token_mock_err(&mut server, 500).await;
        let quest = Quest::new_synced("TestQuest".to_string());
        assert!(
            AppraiserImpl::default()
                .install_app_from_manifest(quest.clone(), vault, manifest, config)
                .await
                .is_err()
        );
        let quest = quest.lock().await;
        assert_eq!(
            quest.sub_quest_progress().await,
            Progress {
                total: Some(2),
                current: 2
            }
        );
        token_mock.assert();
    }

    #[tokio::test]
    async fn test_sideload() {
        let mut deployment = MockedDockerDeployment::new();
        let manifest = no_manifest();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_install_app()
            .once()
            .returning(|_, _, _| Ok(()));
        deployment
            .expect_is_app_installed()
            .once()
            .returning(|_, _| Ok(false));
        deployment.expect_is_default().return_const(true);
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_empty_test_vault();
        vault
            .reservation()
            .reserve_deployment_pouch_mut()
            .grab()
            .await
            .deployment_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut()
            .insert(deployment.id().clone(), deployment);
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let token_mock = token_mock_ok(&mut server).await;
        let quest = Quest::new_synced("TestQuest".to_string());
        assert!(
            AppraiserImpl::default()
                .install_app_from_manifest(quest.clone(), vault.clone(), manifest.clone(), config)
                .await
                .is_ok()
        );
        let quest = quest.lock().await;
        assert_eq!(
            quest.sub_quest_progress().await,
            Progress {
                total: Some(3),
                current: 3
            }
        );
        token_mock.assert();
        let GrabbedPouches {
            manifest_pouch: Some(ref manifest_pouch),
            app_pouch: Some(ref app_pouch),
            ..
        } = vault
            .reservation()
            .reserve_manifest_pouch()
            .reserve_app_pouch()
            .grab()
            .await
        else {
            unreachable!("Reservation should never fail")
        };
        let key = manifest.key().clone();
        let app = app_pouch.gems().get(&key).unwrap();
        assert_eq!(manifest_pouch.gems().get(&key), Some(&manifest));
        assert_eq!(*app.manifest(), manifest);
        assert_eq!(key, app.key);
    }

    #[tokio::test]
    async fn test_install_apps_empty() {
        let vault = create_empty_test_vault();
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let token_mock = token_mock_uncalled(&mut server).await;
        let quest = Quest::new_synced("TestQuest".to_string());
        assert!(
            AppraiserImpl::default()
                .install_apps(quest.clone(), vault, Vec::new(), config)
                .await
                .is_ok()
        );
        let quest = quest.lock().await;
        assert_eq!(
            quest.sub_quest_progress().await,
            Progress {
                total: Some(0),
                current: 0
            }
        );
        token_mock.assert();
    }

    #[tokio::test]
    async fn test_install_apps_ok() {
        let manifests = test_manifests();
        let app_count = manifests.len();
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_install_app()
            .times(app_count)
            .returning(|_, _, _| Ok(()));
        deployment
            .expect_is_app_installed()
            .times(app_count)
            .returning(|_, _| Ok(false));
        deployment.expect_is_default().return_const(true);
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_empty_test_vault();
        vault
            .reservation()
            .reserve_deployment_pouch_mut()
            .grab()
            .await
            .deployment_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut()
            .insert(deployment.id().clone(), deployment);
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let token_mock = token_mock_ok_called(&mut server, app_count).await;
        let mut manifest_mocks = Vec::new();
        let mut keys = Vec::new();
        for manifest in manifests.iter() {
            manifest_mocks
                .push(manifest_mock_ok(&mut server, manifest.clone(), manifest.key()).await);
            keys.push(manifest.key().clone());
        }
        let quest = Quest::new_synced("TestQuest".to_string());
        assert!(
            AppraiserImpl::default()
                .install_apps(quest.clone(), vault.clone(), keys.clone(), config)
                .await
                .is_ok()
        );
        {
            let quest = quest.lock().await;
            assert_eq!(
                quest.sub_quest_progress().await,
                Progress {
                    total: Some(app_count as u64),
                    current: app_count as u64
                }
            );
        }
        token_mock.assert();
        let GrabbedPouches {
            manifest_pouch: Some(ref manifest_pouch),
            app_pouch: Some(ref app_pouch),
            ..
        } = vault
            .reservation()
            .reserve_manifest_pouch()
            .reserve_app_pouch()
            .grab()
            .await
        else {
            unreachable!("Reservation should never fail")
        };
        for manifest_mock in manifest_mocks {
            manifest_mock.assert();
        }
        for key in keys {
            let app = app_pouch.gems().get(&key).unwrap();
            let manifest = manifests
                .iter()
                .find(|manifest| *manifest.key() == key)
                .cloned()
                .unwrap();
            assert_eq!(manifest_pouch.gems().get(&key), Some(&manifest));
            assert_eq!(*app.manifest(), manifest);
            assert_eq!(key, app.key);
        }
    }

    #[tokio::test]
    async fn test_install_apps_err() {
        let mut manifests = test_manifests().into_iter();
        let failing_manifests: Vec<_> = manifests.by_ref().take(4).collect();
        let installed_manifests: Vec<_> = manifests.collect();
        let installed_app_count = installed_manifests.len();
        let total_app_count = installed_app_count + failing_manifests.len();
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_install_app()
            .times(installed_app_count)
            .returning(|_, _, _| Ok(()));
        deployment
            .expect_is_app_installed()
            .times(installed_app_count)
            .returning(|_, _| Ok(false));
        deployment.expect_is_default().return_const(true);
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = create_empty_test_vault();
        vault
            .reservation()
            .reserve_deployment_pouch_mut()
            .grab()
            .await
            .deployment_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut()
            .insert(deployment.id().clone(), deployment);
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let token_mock = token_mock_ok_called(&mut server, installed_app_count).await;
        let mut manifest_mocks = Vec::new();
        let mut keys = Vec::new();
        for manifest in installed_manifests.iter() {
            manifest_mocks
                .push(manifest_mock_ok(&mut server, manifest.clone(), manifest.key()).await);
            keys.push(manifest.key().clone());
        }
        for manifest in failing_manifests.iter() {
            manifest_mocks.push(manifest_mock_err(&mut server, 404, manifest.key()).await);
            keys.push(manifest.key().clone());
        }
        let quest = Quest::new_synced("TestQuest".to_string());
        assert!(
            AppraiserImpl::default()
                .install_apps(quest.clone(), vault.clone(), keys, config)
                .await
                .is_err()
        );
        {
            let quest = quest.lock().await;
            assert_eq!(
                quest.sub_quest_progress().await,
                Progress {
                    total: Some(total_app_count as u64),
                    current: total_app_count as u64
                }
            );
        }
        token_mock.assert();
        let GrabbedPouches {
            manifest_pouch: Some(ref manifests),
            app_pouch: Some(ref apps),
            ..
        } = vault
            .reservation()
            .reserve_manifest_pouch()
            .reserve_app_pouch()
            .grab()
            .await
        else {
            unreachable!("Reservation should never fail")
        };
        for manifest_mock in manifest_mocks {
            manifest_mock.assert();
        }
        for manifest in installed_manifests {
            let key = manifest.key().clone();
            let app = apps.gems().get(&key).unwrap();
            assert_eq!(manifests.gems().get(&key), Some(&manifest));
            assert_eq!(*app.manifest(), manifest);
            assert_eq!(key, app.key);
        }
        for manifest in failing_manifests {
            let key = manifest.key().clone();
            assert!(apps.gems().get(&key).is_none());
            assert!(manifests.gems().get(&key).is_none());
        }
    }

    #[tokio::test]
    async fn app_exist() {
        let vault = create_test_vault(HashMap::new(), HashMap::new(), None);
        for key in existing_app_keys() {
            assert!(
                AppraiserImpl::default()
                    .does_app_exist(vault.clone(), key.clone())
                    .await,
                "Expected app '{key}' to exist"
            )
        }
        assert!(
            !AppraiserImpl::default()
                .does_app_exist(
                    vault.clone(),
                    AppKey {
                        name: UNKNOWN_APP_NAME.to_string(),
                        version: UNKNOWN_APP_VERSION.to_string(),
                    }
                )
                .await
        )
    }
}
