pub use super::Result;
use crate::jeweler::gem::app::App;
use crate::quest::SyncQuest;
use crate::sorcerer::spell;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::{GrabbedPouches, Vault};
use flecs_app_manifest::AppManifest;
use flecs_console_client::apis::configuration::Configuration;
use std::collections::hash_map::Entry;
use std::sync::Arc;

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
    quest
        .lock()
        .await
        .create_sub_quest("Create app".to_string(), |_quest| {
            set_manifest_or_create_app(vault.clone(), manifest, app_key.clone())
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
        panic!("Reservation failed")
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

async fn set_manifest_or_create_app(
    vault: Arc<Vault>,
    manifest: Arc<AppManifest>,
    app_key: AppKey,
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
        panic!("Reservation failed")
    };
    match apps.gems_mut().entry(app_key.clone()) {
        Entry::Occupied(mut app) => {
            app.get_mut().set_manifest(manifest);
            Ok(())
        }
        Entry::Vacant(app_entry) => {
            let mut app = App::new(
                app_key,
                deployments.gems().values().map(Clone::clone).collect(),
            );
            app.set_manifest(manifest);
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
            let token = token.await?;
            install_app_in_vault(
                quest,
                vault.clone(),
                app_key.clone(),
                token.token.username,
                token.token.password,
            )
            .await
        })
        .await
        .2
        .await
}

pub async fn install_app_in_vault(
    quest: SyncQuest,
    vault: Arc<Vault>,
    app_key: AppKey,
    username: String,
    password: String,
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
        .install(quest, username, password)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::jeweler::gem::app::App;
    use crate::quest::{Progress, Quest};
    use crate::sorcerer::appraiser::{
        download_manifest, install_app, install_existing_app, set_manifest_or_create_app,
    };
    use crate::vault::pouch::{AppKey, Pouch};
    use crate::vault::{GrabbedPouches, Vault, VaultConfig};
    use flecs_app_manifest::generated::manifest_3_0_0;
    use flecs_app_manifest::{AppManifest, AppManifestVersion};
    use flecs_console_client::models::{
        GetApiV2ManifestsAppVersion200Response, PostApiV2Tokens200Response,
        PostApiV2Tokens200ResponseData, PostApiV2Tokens200ResponseDataToken,
    };
    use mockito::{Mock, ServerGuard};
    use std::str::FromStr;
    use std::sync::Arc;

    async fn token_mock_err(server: &mut ServerGuard, status: usize) -> Mock {
        server
            .mock("POST", "/api/v2/tokens")
            .with_status(status)
            .expect(1)
            .create_async()
            .await
    }
    async fn token_mock_ok(server: &mut ServerGuard) -> Mock {
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
            .expect(1)
            .create_async()
            .await
    }
    async fn manifest_mock_err(server: &mut ServerGuard, status: usize) -> Mock {
        server
            .mock(
                "GET",
                format!(
                    "/api/v2/manifests/{}/{}?max_manifest_version=3.0.0",
                    test_key().name,
                    test_key().version
                )
                .as_str(),
            )
            .with_status(status)
            .expect(1)
            .create_async()
            .await
    }
    async fn manifest_mock_ok(server: &mut ServerGuard, manifest: Arc<AppManifest>) -> Mock {
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
                    test_key().name,
                    test_key().version
                )
                .as_str(),
            )
            .with_status(200)
            .with_body(body.as_slice())
            .expect(1)
            .create_async()
            .await
    }

    fn create_test_manifest(revision: Option<String>) -> Arc<AppManifest> {
        let manifest = manifest_3_0_0::FlecsAppManifest {
            app: FromStr::from_str("some.test.app").unwrap(),
            args: vec![],
            capabilities: None,
            conffiles: vec![],
            devices: vec![],
            editors: vec![],
            env: vec![],
            image: FromStr::from_str("flecs.azurecr.io/io.anyviz.cloudadapter").unwrap(),
            interactive: None,
            labels: vec![],
            minimum_flecs_version: None,
            multi_instance: None,
            ports: vec![],
            revision,
            version: FromStr::from_str("1.2.3").unwrap(),
            volumes: vec![],
        };
        Arc::new(AppManifest::try_from(AppManifestVersion::V3_0_0(manifest)).unwrap())
    }

    fn test_key() -> AppKey {
        AppKey {
            name: "some.test.app".to_string(),
            version: "1.2.3".to_string(),
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
        set_manifest_or_create_app(vault.clone(), manifest.clone(), test_key())
            .await
            .unwrap();
        let grab = vault.reservation().reserve_app_pouch().grab().await;
        let apps = grab.app_pouch.as_ref().unwrap();
        assert_eq!(apps.gems().len(), 1);
        let app = apps.gems().get(&test_key()).unwrap();
        assert_eq!(app.key, test_key());
        assert_eq!(Some(manifest), app.manifest())
    }

    #[tokio::test]
    async fn test_set_manifest() {
        let manifest = create_test_manifest(None);
        let vault = Arc::new(Vault::new(VaultConfig::default()));
        set_manifest_or_create_app(vault.clone(), manifest.clone(), test_key())
            .await
            .unwrap();
        let grab = vault.reservation().reserve_app_pouch().grab().await;
        let apps = grab.app_pouch.as_ref().unwrap();
        assert_eq!(apps.gems().len(), 1);
        let app = apps.gems().get(&test_key()).unwrap();
        assert_eq!(app.key, test_key());
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
                panic!("Reservation failed")
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

        let token_mock = server
            .mock("POST", "/api/v2/tokens")
            .expect(0)
            .create_async()
            .await;
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
}
