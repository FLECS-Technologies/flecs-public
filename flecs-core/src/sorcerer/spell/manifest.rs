pub use super::Result;
use crate::jeweler::gem::manifest::AppManifest;
use crate::lore;
use crate::quest::SyncQuest;
use crate::vault::pouch::Pouch;
use crate::vault::{GrabbedPouches, Vault};
use anyhow::anyhow;
use flecs_app_manifest::AppManifestVersion;
use flecs_console_client::apis::configuration::Configuration;
use flecs_console_client::apis::default_api::{
    get_api_v2_manifests_app_version, GetApiV2ManifestsAppVersionSuccess,
};
use http::StatusCode;
use std::sync::Arc;

pub async fn download_manifest(
    console_configuration: Arc<Configuration>,
    x_session_id: &str,
    app: &str,
    version: &str,
) -> Result<AppManifestVersion> {
    let response = get_api_v2_manifests_app_version(
        &console_configuration,
        x_session_id,
        app,
        version,
        Some(lore::MAX_SUPPORTED_APP_MANIFEST_VERSION),
        None,
    )
    .await?;
    if response.status != StatusCode::OK {
        return Err(anyhow!(
            "Unexpected response (status {}): {}",
            response.status,
            response.content
        ));
    }
    match response.entity.ok_or_else(|| {
        anyhow!(
            "Invalid response (status {}): {}",
            response.status,
            response.content
        )
    })? {
        GetApiV2ManifestsAppVersionSuccess::Status200(val) => {
            let val = val.data.ok_or_else(|| anyhow!("No data"))?;
            serde_json::from_value::<AppManifestVersion>(val).map_err(|e| e.into())
        }
        GetApiV2ManifestsAppVersionSuccess::UnknownValue(v) => Err(anyhow!("Unknown value {v}")),
    }
}

pub async fn replace_manifest(
    _quest: SyncQuest,
    vault: Arc<Vault>,
    manifest: Arc<AppManifest>,
) -> Result<Option<Arc<AppManifest>>> {
    let GrabbedPouches {
        manifest_pouch_mut: Some(ref mut manifests),
        app_pouch_mut: Some(ref mut apps),
        instance_pouch_mut: Some(ref mut instances),
        ..
    } = vault
        .reservation()
        .reserve_manifest_pouch_mut()
        .reserve_app_pouch_mut()
        .reserve_instance_pouch_mut()
        .grab()
        .await
    else {
        unreachable!("Reservation should never fail");
    };
    let old_manifest = manifests
        .gems_mut()
        .insert(manifest.key.clone(), manifest.clone());
    if let Some(app) = apps.gems_mut().get_mut(&manifest.key) {
        app.set_manifest(manifest.clone());
    };
    for instance in instances
        .gems_mut()
        .values_mut()
        .filter(|instance| instance.app_key() == manifest.key)
    {
        instance.replace_manifest(manifest.clone());
    }
    Ok(old_manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::gem::app::App;
    use crate::jeweler::gem::manifest::Label;
    use crate::quest::Quest;
    use crate::sorcerer::spell::instance::tests::create_test_vault;
    use crate::vault::pouch::manifest::tests::create_test_manifest;
    use crate::vault::pouch::AppKey;
    use flecs_app_manifest::generated::manifest_3_0_0::{
        FlecsAppManifest, FlecsAppManifestApp, FlecsAppManifestImage,
    };
    use std::str::FromStr;

    #[tokio::test]
    async fn download_valid_manifest_test() {
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        const BODY: &str = r#"{
    "statusCode": 200,
    "statusText": "OK",
    "data": {
        "app": "tech.flecs.flunder",
        "_schemaVersion": "3.0.0",
        "version": "3.0.0",
        "image": "flecs.azurecr.io/tech.flecs.flunder"
    }
}"#;
        const APP_NAME: &str = "tech.flecs.flunder";
        const APP_VERSION: &str = "3.0.0";
        let expected_result = AppManifestVersion::V3_0_0(FlecsAppManifest {
            app: FlecsAppManifestApp::from_str(APP_NAME).unwrap(),
            args: vec![],
            capabilities: None,
            conffiles: vec![],
            devices: vec![],
            editors: vec![],
            env: vec![],
            image: FlecsAppManifestImage::from_str("flecs.azurecr.io/tech.flecs.flunder").unwrap(),
            interactive: None,
            labels: vec![],
            minimum_flecs_version: None,
            multi_instance: None,
            ports: vec![],
            revision: None,
            version: APP_VERSION.to_string(),
            volumes: vec![],
        });
        let path: String = format!(
            "/api/v2/manifests/{}/{}?max_manifest_version=3.0.0",
            APP_NAME, APP_VERSION
        );
        let mock = server
            .mock("GET", path.as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(BODY)
            .create_async()
            .await;
        let result = download_manifest(config, "", APP_NAME, APP_VERSION).await;
        mock.assert();
        assert_eq!(result.unwrap(), expected_result);
    }

    #[tokio::test]
    async fn download_no_data_manifest_test() {
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        const BODY: &str = r#"{
        "statusCode": 200,
        "statusText": "OK"
    }"#;
        const APP_NAME: &str = "my.no-data.app";
        const APP_VERSION: &str = "3.0.0";
        let path: String = format!(
            "/api/v2/manifests/{}/{}?max_manifest_version=3.0.0",
            APP_NAME, APP_VERSION
        );
        let mock = server
            .mock("GET", path.as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(BODY)
            .create_async()
            .await;
        let result = download_manifest(config, "", APP_NAME, APP_VERSION).await;
        mock.assert();
        match result {
            Err(e) => {
                assert!(e.to_string().contains("No data"))
            }
            x => {
                panic!("Expected Error::NoData, got {:?}", x)
            }
        }
    }
    #[tokio::test]
    async fn download_manifest_unexpected_response_test() {
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        const BODY: &str = r#"{
        "statusCode": 202,
        "statusText": "OK"
    }"#;
        const APP_NAME: &str = "my.no-data.app";
        const APP_VERSION: &str = "3.0.0";
        let path: String = format!(
            "/api/v2/manifests/{}/{}?max_manifest_version=3.0.0",
            APP_NAME, APP_VERSION
        );
        let mock = server
            .mock("GET", path.as_str())
            .with_status(202)
            .with_header("content-type", "application/json")
            .with_body(BODY)
            .create_async()
            .await;
        let result = download_manifest(config, "", APP_NAME, APP_VERSION).await;
        mock.assert();
        match result {
            Err(e) => {
                assert!(e.to_string().contains("Unexpected response (status"))
            }
            x => {
                panic!("Expected Error::UnexpectedResponse {{status: StatusCode::ACCEPTED, ..}}, got {:?}", x)
            }
        }
    }
    #[tokio::test]
    async fn replace_manifest_test() {
        let vault = create_test_vault(module_path!(), "replace_manifest_test", None).await;
        let mut manifest = create_test_manifest("some.test.app-1", "1.2.3");
        let labels = vec![Label {
            label: "Replacing".to_string(),
            value: None,
        }];
        manifest.labels = labels.clone();
        let manifest = Arc::new(manifest);
        let app_key_1 = AppKey {
            name: "some.test.app-1".to_string(),
            version: "1.2.3".to_string(),
        };
        let app_key_2 = AppKey {
            name: "some.test.app-2".to_string(),
            version: "1.2.4".to_string(),
        };
        {
            let app_1 = App::new(app_key_1.clone(), Vec::new());
            let app_2 = App::new(app_key_2.clone(), Vec::new());
            let mut grab = vault.reservation().reserve_app_pouch_mut().grab().await;
            let apps = grab.app_pouch_mut.as_mut().unwrap();
            apps.gems_mut().insert(app_key_1.clone(), app_1);
            apps.gems_mut().insert(app_key_2.clone(), app_2);
        }
        let old_manifest = replace_manifest(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            manifest.clone(),
        )
        .await
        .unwrap();
        assert_ne!(old_manifest.unwrap().labels, labels);
        let GrabbedPouches {
            manifest_pouch: Some(ref manifests),
            app_pouch: Some(ref apps),
            instance_pouch: Some(ref instances),
            ..
        } = vault
            .reservation()
            .reserve_manifest_pouch()
            .reserve_app_pouch()
            .reserve_instance_pouch()
            .grab()
            .await
        else {
            unreachable!("Reservation should never fail");
        };
        assert!(apps.gems().get(&app_key_2).unwrap().manifest().is_none());
        assert_eq!(
            apps.gems()
                .get(&app_key_1)
                .unwrap()
                .manifest()
                .unwrap()
                .labels,
            labels
        );
        assert_eq!(manifests.gems().get(&app_key_1).unwrap().labels, labels);
        for instance in instances.gems().values() {
            if instance.app_key() == app_key_1 {
                assert_eq!(instance.manifest.labels, labels);
            } else {
                assert_ne!(instance.manifest.labels, labels);
            }
        }
    }
}
