pub use super::Result;
use crate::fsm::console_client::ConsoleClient;
use crate::jeweler::gem::manifest::AppManifest;
use crate::lore;
use crate::quest::SyncQuest;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::{GrabbedPouches, Vault};
use anyhow::anyhow;
use flecs_app_manifest::AppManifestVersion;
use flecs_console_client::apis::default_api::{
    GetApiV2ManifestsAppVersionSuccess, get_api_v2_manifests_app_version,
};
use http::StatusCode;
use std::sync::Arc;

pub async fn download_manifest(
    console_configuration: ConsoleClient,
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

pub async fn erase_manifest_if_unused(vault: Arc<Vault>, app_key: AppKey) -> Option<AppManifest> {
    let GrabbedPouches {
        manifest_pouch_mut: Some(ref mut manifests),
        app_pouch: Some(ref apps),
        instance_pouch: Some(ref instances),
        ..
    } = vault
        .reservation()
        .reserve_manifest_pouch_mut()
        .reserve_app_pouch()
        .reserve_instance_pouch()
        .grab()
        .await
    else {
        unreachable!("Reservation should never fail");
    };
    if apps.gems().contains_key(&app_key) {
        return None;
    }
    if instances
        .gems()
        .values()
        .any(|instance| *instance.app_key() == app_key)
    {
        return None;
    }
    manifests.gems_mut().remove(&app_key)
}

pub async fn replace_manifest(
    _quest: SyncQuest,
    vault: Arc<Vault>,
    manifest: AppManifest,
) -> Result<Option<AppManifest>> {
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
        .insert(manifest.key().clone(), manifest.clone());
    if let Some(app) = apps.gems_mut().get_mut(manifest.key()) {
        app.replace_manifest(manifest.clone());
    };
    for instance in instances
        .gems_mut()
        .values_mut()
        .filter(|instance| instance.app_key() == manifest.key())
    {
        instance.replace_manifest(manifest.clone());
    }
    Ok(old_manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::gem::manifest::single::Label;
    use crate::quest::Quest;
    use crate::vault;
    use crate::vault::pouch::instance::tests::LABEL_INSTANCE;
    use crate::vault::pouch::manifest::tests::label_manifest;
    use flecs_app_manifest::generated::manifest_3_1_0::{
        App as OtherApp, FlecsAppManifest, Image, Single, Version,
    };
    use std::collections::HashMap;
    use std::str::FromStr;

    #[tokio::test]
    async fn download_valid_manifest_test() {
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        const BODY: &str = r#"{
    "statusCode": 200,
    "statusText": "OK",
    "data": {
        "app": "tech.flecs.flunder",
        "_schemaVersion": "3.1.0",
        "version": "3.0.0",
        "image": "flecs.azurecr.io/tech.flecs.flunder"
    }
}"#;
        const APP_NAME: &str = "tech.flecs.flunder";
        const APP_VERSION: &str = "3.0.0";
        let expected_result = AppManifestVersion::V3_1_0(FlecsAppManifest::Single(Single {
            app: OtherApp::from_str(APP_NAME).unwrap(),
            args: None,
            capabilities: None,
            conffiles: None,
            devices: None,
            editors: None,
            env: None,
            hostname: None,
            image: Image::from_str("flecs.azurecr.io/tech.flecs.flunder").unwrap(),
            interactive: None,
            labels: None,
            minimum_flecs_version: None,
            multi_instance: None,
            ports: None,
            revision: None,
            schema: None,
            version: Version::from_str(APP_VERSION).unwrap(),
            volumes: None,
        }));
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
                panic!(
                    "Expected Error::UnexpectedResponse {{status: StatusCode::ACCEPTED, ..}}, got {:?}",
                    x
                )
            }
        }
    }
    #[tokio::test]
    async fn replace_manifest_test() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let AppManifest::Single(new_manifest) = label_manifest() else {
            panic!()
        };
        let mut new_manifest = Arc::try_unwrap(new_manifest).unwrap();
        let app_key = new_manifest.key.clone();
        new_manifest.labels.push(Label {
            label: "new.label".to_string(),
            value: None,
        });
        let new_manifest = AppManifest::Single(Arc::new(new_manifest));
        let old_manifest = replace_manifest(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            new_manifest.clone(),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(old_manifest, label_manifest());
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
        assert_eq!(apps.gems().get(&app_key).unwrap().manifest(), &new_manifest);
        assert_eq!(manifests.gems().get(&app_key), Some(&new_manifest.clone()));
        assert_eq!(
            instances.gems().get(&LABEL_INSTANCE).unwrap().manifest(),
            new_manifest
        );
    }
}
