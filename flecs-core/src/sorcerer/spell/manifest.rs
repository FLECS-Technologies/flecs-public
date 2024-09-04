pub use super::Result;
use crate::lore;
use anyhow::anyhow;
use flecs_app_manifest::AppManifestVersion;
use flecs_console_client::apis::configuration::Configuration;
use flecs_console_client::apis::default_api::{
    get_api_v2_manifests_app_version, GetApiV2ManifestsAppVersionSuccess,
};
use http::StatusCode;

pub async fn download_manifest(
    console_configuration: &Configuration,
    x_session_id: &str,
    app: &str,
    version: &str,
) -> Result<AppManifestVersion> {
    let response = get_api_v2_manifests_app_version(
        console_configuration,
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

#[cfg(test)]
mod tests {
    use super::*;
    use flecs_app_manifest::generated::manifest_3_0_0::{
        FlecsAppManifest, FlecsAppManifestApp, FlecsAppManifestImage,
    };
    use std::str::FromStr;

    #[tokio::test]
    async fn download_valid_manifest_test() {
        let mut server = mockito::Server::new_async().await;
        let config = Configuration {
            base_path: server.url(),
            ..Configuration::default()
        };
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
        let result = download_manifest(&config, "", APP_NAME, APP_VERSION).await;
        mock.assert();
        assert_eq!(result.unwrap(), expected_result);
    }

    #[tokio::test]
    async fn download_no_data_manifest_test() {
        let mut server = mockito::Server::new_async().await;
        let config = Configuration {
            base_path: server.url(),
            ..Configuration::default()
        };
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
        let result = download_manifest(&config, "", APP_NAME, APP_VERSION).await;
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
        let mut server = mockito::Server::new_async().await;
        let config = Configuration {
            base_path: server.url(),
            ..Configuration::default()
        };
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
        let result = download_manifest(&config, "", APP_NAME, APP_VERSION).await;
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
}
