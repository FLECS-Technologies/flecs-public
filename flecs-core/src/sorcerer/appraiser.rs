pub use super::Result;
use crate::jeweler::app::{AppStatus, Token};
use crate::jeweler::gem::app::App;
use crate::jeweler::gem::manifest::AppManifest;
use crate::quest::SyncQuest;
use crate::sorcerer::spell;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::{GrabbedPouches, Vault};
use flecs_console_client::apis::configuration::Configuration;
use flecsd_axum_server::models::InstalledApp;
use futures_util::future::join_all;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use tracing::error;

pub async fn uninstall_app(quest: SyncQuest, vault: Arc<Vault>, app_key: AppKey) -> Result<()> {
    spell::app::uninstall_app(quest, vault, app_key).await
}

// TODO: Unit test
pub async fn does_app_exist(vault: Arc<Vault>, app_key: AppKey) -> bool {
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

pub async fn get_app(
    vault: Arc<Vault>,
    name: String,
    version: Option<String>,
) -> Result<Vec<InstalledApp>> {
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

pub async fn get_apps(vault: Arc<Vault>) -> Result<Vec<InstalledApp>> {
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

pub async fn install_app_from_manifest(
    quest: SyncQuest,
    vault: Arc<Vault>,
    manifest: Arc<AppManifest>,
    config: Arc<Configuration>,
) -> Result<()> {
    let app_key = manifest.key.clone();
    quest
        .lock()
        .await
        .create_sub_quest("Create app".to_string(), |_quest| {
            set_manifest_and_desired_or_create_app(
                vault.clone(),
                manifest,
                app_key.clone(),
                AppStatus::Installed,
            )
        })
        .await
        .2
        .await?;
    quest
        .lock()
        .await
        .create_sub_quest("Install app".to_string(), |quest| async {
            install_existing_app(quest, vault, app_key, config).await
        })
        .await
        .2
        .await
}

pub async fn install_apps(
    quest: SyncQuest,
    vault: Arc<Vault>,
    app_keys: Vec<AppKey>,
    config: Arc<Configuration>,
) -> Result<()> {
    let mut results = Vec::new();
    let mut keys = Vec::new();
    for app_key in app_keys {
        let result = quest
            .lock()
            .await
            .create_sub_quest(format!("Install app {app_key}"), |quest| {
                install_app(quest, vault.clone(), app_key.clone(), config.clone())
            })
            .await
            .2;
        results.push(result);
        keys.push(app_key);
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

pub async fn install_app(
    quest: SyncQuest,
    vault: Arc<Vault>,
    app_key: AppKey,
    config: Arc<Configuration>,
) -> Result<()> {
    let manifest = quest
        .lock()
        .await
        .create_sub_quest("Obtain manifest".to_string(), |_quest| {
            download_manifest(vault.clone(), app_key.clone(), config.clone())
        })
        .await
        .2
        .await?;
    install_app_from_manifest(quest, vault, manifest, config).await
}

async fn download_manifest(
    vault: Arc<Vault>,
    app_key: AppKey,
    config: Arc<Configuration>,
) -> Result<Arc<AppManifest>> {
    let session_id = vault
        .get_secrets()
        .await
        .get_session_id()
        .id
        .unwrap_or_default();
    let manifest: AppManifest =
        spell::manifest::download_manifest(config, &session_id, &app_key.name, &app_key.version)
            .await?
            .try_into()?;
    let manifest = Arc::new(manifest);
    let GrabbedPouches {
        manifest_pouch_mut: Some(ref mut manifests),
        app_pouch_mut: Some(ref mut apps),
        ..
    } = vault
        .reservation()
        .reserve_manifest_pouch_mut()
        .reserve_app_pouch_mut()
        .grab()
        .await
    else {
        unreachable!("Reservation should never fail");
    };

    if let Some(_previous_manifest) = manifests
        .gems_mut()
        .insert(app_key.clone(), manifest.clone())
    {
        println!("Previous manifest for {app_key}, was replaced.")
    };

    if let Some(app) = apps.gems_mut().get_mut(&app_key) {
        app.set_manifest(manifest.clone());
        println!("Previous manifest of {app_key}, was replaced.")
    };
    Ok(manifest)
}

async fn set_manifest_and_desired_or_create_app(
    vault: Arc<Vault>,
    manifest: Arc<AppManifest>,
    app_key: AppKey,
    desired: AppStatus,
) -> Result<()> {
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
            app.get_mut().set_manifest(manifest);
            app.get_mut().set_desired(desired);
            Ok(())
        }
        Entry::Vacant(app_entry) => {
            let mut app = App::new(
                app_key,
                deployments.gems().values().map(Clone::clone).collect(),
            );
            app.set_manifest(manifest);
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
    config: Arc<Configuration>,
) -> Result<()> {
    quest
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
        .2
        .await?;
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
    quest
        .lock()
        .await
        .create_sub_quest(format!("Install app {}", app_key), |quest| async move {
            let token = token.await?.map(Into::into);
            install_app_in_vault(quest, vault.clone(), app_key.clone(), token).await
        })
        .await
        .2
        .await
}

pub async fn install_app_in_vault(
    quest: SyncQuest,
    vault: Arc<Vault>,
    app_key: AppKey,
    token: Option<Token>,
) -> Result<()> {
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
    use crate::jeweler::app::{AppInfo, AppStatus};
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::jeweler::deployment::Deployment;
    use crate::jeweler::gem::app::tests::{test_key, test_key_numbered};
    use crate::jeweler::gem::app::App;
    use crate::jeweler::gem::manifest::tests::{
        create_test_manifest, create_test_manifest_numbered,
    };
    use crate::jeweler::gem::manifest::AppManifest;
    use crate::quest::{Progress, Quest};
    use crate::sorcerer::appraiser::{
        does_app_exist, download_manifest, get_app, get_apps, install_app,
        install_app_from_manifest, install_apps, install_existing_app,
        set_manifest_and_desired_or_create_app,
    };
    use crate::vault::pouch::Pouch;
    use crate::vault::{GrabbedPouches, Vault, VaultConfig};
    use flecs_console_client::models::{
        GetApiV2ManifestsAppVersion200Response, PostApiV2Tokens200Response,
        PostApiV2Tokens200ResponseData, PostApiV2Tokens200ResponseDataToken,
    };
    use mockito::{Mock, ServerGuard};
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
    async fn manifest_mock_err(server: &mut ServerGuard, status: usize) -> Mock {
        manifest_mock_err_numbered(server, status, 0, 0).await
    }
    async fn manifest_mock_err_numbered(
        server: &mut ServerGuard,
        status: usize,
        name_number: u8,
        version_number: u8,
    ) -> Mock {
        let key = test_key_numbered(name_number, version_number);
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
    async fn manifest_mock_ok(server: &mut ServerGuard, manifest: Arc<AppManifest>) -> Mock {
        manifest_mock_ok_numbered(server, manifest, 0, 0).await
    }
    async fn manifest_mock_ok_numbered(
        server: &mut ServerGuard,
        manifest: Arc<AppManifest>,
        name_number: u8,
        version_number: u8,
    ) -> Mock {
        let key = test_key_numbered(name_number, version_number);
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

    const CREATED_APPS_WITH_MANIFEST: [(u8, u8); 4] = [(1, 1), (1, 2), (1, 3), (1, 4)];
    const CREATED_APPS_WITHOUT_MANIFEST: [(u8, u8); 4] = [(2, 1), (2, 2), (2, 3), (2, 4)];
    const INSTALLED_APPS_WITHOUT_MANIFEST: [(u8, u8); 4] = [(3, 1), (3, 2), (3, 3), (3, 4)];
    const INSTALLED_APPS_WITH_MANIFEST: [(u8, u8); 4] = [(4, 1), (4, 2), (4, 3), (4, 4)];
    const UNKNOWN_APPS: [(u8, u8); 4] = [(5, 1), (5, 2), (5, 3), (5, 4)];
    fn create_test_deployment() -> Arc<dyn Deployment> {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_app_info()
            .withf(|_, id| id == "INSTALLED")
            .returning(|_, _| {
                Ok(AppInfo {
                    size: Some(2000),
                    ..AppInfo::default()
                })
            });
        deployment
            .expect_app_info()
            .returning(|_, _| Err(anyhow::anyhow!("test")));
        deployment.expect_id().returning(String::new);
        Arc::new(deployment) as Arc<dyn Deployment>
    }
    async fn create_test_vault() -> Arc<Vault> {
        let deployments = vec![create_test_deployment()];
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        {
            let mut reserved_pouch = vault.reservation().reserve_app_pouch_mut().grab().await;
            let apps = reserved_pouch.app_pouch_mut.as_mut().unwrap();
            for (name_number, version_number) in CREATED_APPS_WITH_MANIFEST {
                let mut app = App::new(
                    test_key_numbered(name_number, version_number),
                    deployments.clone(),
                );
                app.set_manifest(create_test_manifest_numbered(
                    name_number,
                    version_number,
                    None,
                ));
                app.deployments
                    .values_mut()
                    .next()
                    .unwrap()
                    .set_id("INSTALLED".to_string());
                apps.gems_mut().insert(app.key.clone(), app);
            }
            for (name_number, version_number) in INSTALLED_APPS_WITHOUT_MANIFEST {
                let mut app = App::new(
                    test_key_numbered(name_number, version_number),
                    deployments.clone(),
                );
                app.deployments
                    .values_mut()
                    .next()
                    .unwrap()
                    .set_id("INSTALLED".to_string());
                apps.gems_mut().insert(app.key.clone(), app);
            }
            for (name_number, version_number) in INSTALLED_APPS_WITH_MANIFEST {
                let mut app = App::new(
                    test_key_numbered(name_number, version_number),
                    deployments.clone(),
                );
                app.deployments
                    .values_mut()
                    .next()
                    .unwrap()
                    .set_id("INSTALLED".to_string());
                app.set_manifest(create_test_manifest_numbered(
                    name_number,
                    version_number,
                    None,
                ));
                apps.gems_mut().insert(app.key.clone(), app);
            }
            for (name_number, version_number) in CREATED_APPS_WITHOUT_MANIFEST {
                let app = App::new(
                    test_key_numbered(name_number, version_number),
                    deployments.clone(),
                );
                apps.gems_mut().insert(app.key.clone(), app);
            }
        }
        vault
    }

    #[tokio::test]
    async fn test_get_app_explicit_version() {
        let vault = create_test_vault().await;
        for (name_number, version_number) in INSTALLED_APPS_WITH_MANIFEST {
            let key = test_key_numbered(name_number, version_number);
            let result = get_app(vault.clone(), key.name, Some(key.version)).await;
            assert_eq!(result.unwrap().len(), 1);
        }
        for (name_number, version_number) in INSTALLED_APPS_WITHOUT_MANIFEST {
            let key = test_key_numbered(name_number, version_number);
            let result = get_app(vault.clone(), key.name, Some(key.version)).await;
            assert!(result.is_err());
        }
        for (name_number, version_number) in CREATED_APPS_WITH_MANIFEST {
            let key = test_key_numbered(name_number, version_number);
            let result = get_app(vault.clone(), key.name, Some(key.version)).await;
            assert_eq!(result.unwrap().len(), 1);
        }
        for (name_number, version_number) in CREATED_APPS_WITHOUT_MANIFEST {
            let key = test_key_numbered(name_number, version_number);
            let result = get_app(vault.clone(), key.name, Some(key.version)).await;
            assert!(result.is_err());
        }
        for (name_number, version_number) in UNKNOWN_APPS {
            let key = test_key_numbered(name_number, version_number);
            let result = get_app(vault.clone(), key.name, Some(key.version)).await;
            assert_eq!(result.unwrap().len(), 0);
        }
    }

    #[tokio::test]
    async fn test_get_app_no_version() {
        let vault = create_test_vault().await;
        {
            let name_number = INSTALLED_APPS_WITH_MANIFEST[0].0;
            let name = test_key_numbered(name_number, 0).name;
            let result = get_app(vault.clone(), name, None).await;
            assert_eq!(result.unwrap().len(), 4);
        }
        {
            let name_number = INSTALLED_APPS_WITHOUT_MANIFEST[0].0;
            let name = test_key_numbered(name_number, 0).name;
            let result = get_app(vault.clone(), name, None).await;
            assert_eq!(result.unwrap().len(), 0);
        }
        {
            let name_number = CREATED_APPS_WITH_MANIFEST[0].0;
            let name = test_key_numbered(name_number, 0).name;
            let result = get_app(vault.clone(), name, None).await;
            assert_eq!(result.unwrap().len(), 4);
        }
        {
            let name_number = CREATED_APPS_WITHOUT_MANIFEST[0].0;
            let name = test_key_numbered(name_number, 0).name;
            let result = get_app(vault.clone(), name, None).await;
            assert_eq!(result.unwrap().len(), 0);
        }
        {
            let name_number = UNKNOWN_APPS[0].0;
            let name = test_key_numbered(name_number, 0).name;
            let result = get_app(vault.clone(), name, None).await;
            assert_eq!(result.unwrap().len(), 0);
        }
    }

    #[tokio::test]
    async fn test_get_apps_no_version() {
        let vault = create_test_vault().await;
        let result = get_apps(vault).await.unwrap();
        assert_eq!(
            result.len(),
            INSTALLED_APPS_WITH_MANIFEST.len() + CREATED_APPS_WITH_MANIFEST.len()
        );
        for (name_number, version_number) in INSTALLED_APPS_WITH_MANIFEST
            .iter()
            .chain(CREATED_APPS_WITH_MANIFEST.iter())
        {
            let key = test_key_numbered(*name_number, *version_number).into();
            assert!(result.iter().any(|info| info.app_key == key));
        }
    }

    #[tokio::test]
    async fn test_create_app() {
        let app = App::new(test_key(), Vec::new());
        let manifest = create_test_manifest(None);
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        assert!(vault
            .reservation()
            .reserve_app_pouch_mut()
            .grab()
            .await
            .app_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut()
            .insert(test_key(), app)
            .is_none());
        set_manifest_and_desired_or_create_app(
            vault.clone(),
            manifest.clone(),
            test_key(),
            AppStatus::Installed,
        )
        .await
        .unwrap();
        let grab = vault.reservation().reserve_app_pouch().grab().await;
        let apps = grab.app_pouch.as_ref().unwrap();
        assert_eq!(apps.gems().len(), 1);
        let app = apps.gems().get(&test_key()).unwrap();
        assert_eq!(app.key, test_key());
        for data in app.deployments.values() {
            assert_eq!(data.desired, AppStatus::Installed);
        }
        assert_eq!(Some(manifest), app.manifest())
    }

    #[tokio::test]
    async fn test_set_manifest() {
        let manifest = create_test_manifest(None);
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        set_manifest_and_desired_or_create_app(
            vault.clone(),
            manifest.clone(),
            test_key(),
            AppStatus::NotInstalled,
        )
        .await
        .unwrap();
        let grab = vault.reservation().reserve_app_pouch().grab().await;
        let apps = grab.app_pouch.as_ref().unwrap();
        assert_eq!(apps.gems().len(), 1);
        let app = apps.gems().get(&test_key()).unwrap();
        assert_eq!(app.key, test_key());
        for data in app.deployments.values() {
            assert_eq!(data.desired, AppStatus::NotInstalled);
        }
        assert_eq!(Some(manifest), app.manifest())
    }

    #[tokio::test]
    async fn install_non_existing_app() {
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let (_, config) = crate::tests::create_test_server_and_config().await;
        assert!(install_existing_app(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            test_key(),
            config,
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn test_install_existing_app() {
        let mut app = App::new(test_key(), Vec::new());
        app.set_manifest(create_test_manifest(None));
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        vault
            .reservation()
            .reserve_app_pouch_mut()
            .grab()
            .await
            .app_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut()
            .insert(test_key(), app);
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = token_mock_ok(&mut server).await;
        install_existing_app(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            test_key(),
            config,
        )
        .await
        .unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn test_install_existing_app_token_failure() {
        let mut app = App::new(test_key(), Vec::new());
        app.set_manifest(create_test_manifest(None));
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        vault
            .reservation()
            .reserve_app_pouch_mut()
            .grab()
            .await
            .app_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut()
            .insert(test_key(), app);
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = token_mock_err(&mut server, 500).await;
        assert!(install_existing_app(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            test_key(),
            config,
        )
        .await
        .is_err());
        mock.assert();
    }

    #[tokio::test]
    async fn test_manifest_download() {
        let mut app = App::new(test_key(), Vec::new());
        let manifest = create_test_manifest(None);
        app.set_manifest(manifest.clone());
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        vault
            .reservation()
            .reserve_app_pouch_mut()
            .grab()
            .await
            .app_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut()
            .insert(test_key(), app);
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = manifest_mock_ok(&mut server, manifest.clone()).await;
        assert_eq!(
            download_manifest(vault.clone(), test_key(), config,)
                .await
                .unwrap(),
            manifest
        );
        assert_eq!(
            vault
                .reservation()
                .reserve_manifest_pouch()
                .grab()
                .await
                .manifest_pouch
                .as_ref()
                .unwrap()
                .gems()
                .get(&test_key()),
            Some(&manifest)
        );
        mock.assert();
    }

    #[tokio::test]
    async fn test_manifest_download_replace() {
        let mut app = App::new(test_key(), Vec::new());
        let manifest = create_test_manifest(None);
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        app.set_manifest(manifest.clone());
        {
            let GrabbedPouches {
                manifest_pouch_mut: Some(ref mut manifests),
                app_pouch_mut: Some(ref mut apps),
                ..
            } = vault
                .reservation()
                .reserve_manifest_pouch_mut()
                .reserve_app_pouch_mut()
                .grab()
                .await
            else {
                unreachable!("Reservation failed")
            };
            apps.gems_mut().insert(test_key(), app);
            manifests.gems_mut().insert(test_key(), manifest.clone());
        }
        let manifest = create_test_manifest(Some("10".to_string()));
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = manifest_mock_ok(&mut server, manifest.clone()).await;
        assert_eq!(
            download_manifest(vault.clone(), test_key(), config,)
                .await
                .unwrap(),
            manifest
        );
        assert_eq!(
            vault
                .reservation()
                .reserve_app_pouch()
                .grab()
                .await
                .app_pouch
                .as_ref()
                .unwrap()
                .gems()
                .get(&test_key())
                .unwrap()
                .manifest(),
            Some(manifest.clone())
        );
        assert_eq!(
            vault
                .reservation()
                .reserve_manifest_pouch()
                .grab()
                .await
                .manifest_pouch
                .as_ref()
                .unwrap()
                .gems()
                .get(&test_key()),
            Some(&manifest)
        );
        mock.assert();
    }

    #[tokio::test]
    async fn test_manifest_download_fail() {
        let mut app = App::new(test_key(), Vec::new());
        let manifest = create_test_manifest(None);
        app.set_manifest(manifest.clone());
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        vault
            .reservation()
            .reserve_app_pouch_mut()
            .grab()
            .await
            .app_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut()
            .insert(test_key(), app);
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = manifest_mock_err(&mut server, 500).await;
        assert!(download_manifest(vault, test_key(), config,).await.is_err());
        mock.assert();
    }

    #[tokio::test]
    async fn test_install_no_manifest() {
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let manifest_mock = manifest_mock_err(&mut server, 404).await;

        let token_mock = token_mock_uncalled(&mut server).await;
        let quest = Quest::new_synced("TestQuest".to_string());
        assert!(install_app(quest.clone(), vault, test_key(), config)
            .await
            .is_err());
        let quest = quest.lock().await;
        assert_eq!(
            quest.sub_quest_progress().await,
            Progress {
                total: Some(1),
                current: 1
            }
        );
        manifest_mock.assert();
        token_mock.assert();
    }

    #[tokio::test]
    async fn test_install_token_error() {
        let manifest = create_test_manifest(None);
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let (mut server, config) = crate::tests::create_test_server_and_config().await;

        let token_mock = token_mock_err(&mut server, 500).await;
        let manifest_mock = manifest_mock_ok(&mut server, manifest.clone()).await;
        let quest = Quest::new_synced("TestQuest".to_string());
        assert!(install_app(quest.clone(), vault, test_key(), config)
            .await
            .is_err());
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
        let manifest = create_test_manifest(None);
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let manifest_mock = manifest_mock_ok(&mut server, manifest.clone()).await;
        let token_mock = token_mock_ok(&mut server).await;
        let quest = Quest::new_synced("TestQuest".to_string());
        assert!(install_app(quest.clone(), vault, test_key(), config)
            .await
            .is_ok());
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
    async fn test_sideload_token_error() {
        let manifest = create_test_manifest(None);
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let (mut server, config) = crate::tests::create_test_server_and_config().await;

        let token_mock = token_mock_err(&mut server, 500).await;
        let quest = Quest::new_synced("TestQuest".to_string());
        assert!(
            install_app_from_manifest(quest.clone(), vault, manifest, config)
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
        let manifest = create_test_manifest(None);
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let token_mock = token_mock_ok(&mut server).await;
        let quest = Quest::new_synced("TestQuest".to_string());
        assert!(
            install_app_from_manifest(quest.clone(), vault, manifest, config)
                .await
                .is_ok()
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
    async fn test_install_apps_empty() {
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let token_mock = token_mock_uncalled(&mut server).await;
        let quest = Quest::new_synced("TestQuest".to_string());
        assert!(install_apps(quest.clone(), vault, Vec::new(), config)
            .await
            .is_ok());
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
        const APP_COUNT: u8 = 10;
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let token_mock = token_mock_ok_called(&mut server, APP_COUNT as usize).await;
        let mut manifest_mocks = Vec::new();
        let mut keys = Vec::new();
        for i in 0..APP_COUNT {
            manifest_mocks.push(
                manifest_mock_ok_numbered(
                    &mut server,
                    create_test_manifest_numbered(i, i, None),
                    i,
                    i,
                )
                .await,
            );
            keys.push(test_key_numbered(i, i));
        }
        let quest = Quest::new_synced("TestQuest".to_string());
        assert!(install_apps(quest.clone(), vault.clone(), keys, config)
            .await
            .is_ok());
        {
            let quest = quest.lock().await;
            assert_eq!(
                quest.sub_quest_progress().await,
                Progress {
                    total: Some(APP_COUNT as u64),
                    current: APP_COUNT as u64
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
        for i in 0..APP_COUNT {
            let key = test_key_numbered(i, i);
            let manifest = create_test_manifest_numbered(i, i, None);
            let app = apps.gems().get(&key).unwrap();
            assert_eq!(Some(&manifest), manifests.gems().get(&key));
            assert_eq!(Some(manifest), app.manifest());
            assert_eq!(key, app.key);
        }
    }

    #[tokio::test]
    async fn test_install_apps_err() {
        const OK_APP_COUNT: u8 = 10;
        const FAILING_APP_COUNT: u8 = 6;
        const TOTAL_APP_COUNT: u8 = OK_APP_COUNT + FAILING_APP_COUNT;
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let token_mock = token_mock_ok_called(&mut server, OK_APP_COUNT as usize).await;
        let mut manifest_mocks = Vec::new();
        let mut keys = Vec::new();
        for i in 0..OK_APP_COUNT {
            manifest_mocks.push(
                manifest_mock_ok_numbered(
                    &mut server,
                    create_test_manifest_numbered(i, i, None),
                    i,
                    i,
                )
                .await,
            );
            keys.push(test_key_numbered(i, i));
        }
        for i in OK_APP_COUNT..TOTAL_APP_COUNT {
            manifest_mocks.push(manifest_mock_err_numbered(&mut server, 404, i, i).await);
            keys.push(test_key_numbered(i, i));
        }
        let quest = Quest::new_synced("TestQuest".to_string());
        assert!(install_apps(quest.clone(), vault.clone(), keys, config)
            .await
            .is_err());
        {
            let quest = quest.lock().await;
            assert_eq!(
                quest.sub_quest_progress().await,
                Progress {
                    total: Some(TOTAL_APP_COUNT as u64),
                    current: TOTAL_APP_COUNT as u64
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
        for i in 0..OK_APP_COUNT {
            let key = test_key_numbered(i, i);
            let manifest = create_test_manifest_numbered(i, i, None);
            let app = apps.gems().get(&key).unwrap();
            assert_eq!(Some(&manifest), manifests.gems().get(&key));
            assert_eq!(Some(manifest), app.manifest());
            assert_eq!(key, app.key);
        }
        for i in OK_APP_COUNT..TOTAL_APP_COUNT {
            let key = test_key_numbered(i, i);
            assert!(apps.gems().get(&key).is_none());
            assert!(manifests.gems().get(&key).is_none());
        }
    }

    #[tokio::test]
    async fn app_exist() {
        let vault = create_test_vault().await;
        for (name_number, version_number) in CREATED_APPS_WITH_MANIFEST
            .iter()
            .chain(CREATED_APPS_WITHOUT_MANIFEST.iter())
            .chain(INSTALLED_APPS_WITHOUT_MANIFEST.iter())
            .chain(INSTALLED_APPS_WITH_MANIFEST.iter())
        {
            assert!(
                does_app_exist(
                    vault.clone(),
                    test_key_numbered(*name_number, *version_number)
                )
                .await
            )
        }
        for (name_number, version_number) in UNKNOWN_APPS.iter() {
            assert!(
                !does_app_exist(
                    vault.clone(),
                    test_key_numbered(*name_number, *version_number)
                )
                .await
            )
        }
    }
}
