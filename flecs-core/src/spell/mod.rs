use flecs_app_manifest::AppManifestVersion;
use flecs_console_client::apis::configuration::Configuration;
use flecs_console_client::apis::default_api::{
    get_api_v2_manifests_app_version, GetApiV2ManifestsAppVersionError,
    GetApiV2ManifestsAppVersionSuccess,
};
use flecs_console_client::models::ErrorDescription;
use http::StatusCode;
use std::fmt::{Display, Formatter};
#[derive(Debug)]
pub enum HttpError {
    ErrorDescription(ErrorDescription),
    UnknownError(serde_json::Value),
}

impl Display for HttpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::ErrorDescription(e) => write!(f, "HttpError: {:?}", e),
            HttpError::UnknownError(val) => write!(f, "HttpError::UnknownError: {:#}", val),
        }
    }
}

impl std::error::Error for HttpError {}

#[derive(Debug)]
pub enum Error {
    Http(HttpError),
    InvalidContent(String),
    NoData,
    UnexpectedData(serde_json::Value),
    UnexpectedResponse { status: StatusCode, content: String },
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
    Io(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Http(e) => write!(f, "{}", e),
            Error::InvalidContent(s) => write!(f, "InvalidContent: {}", &s[0..200]),
            Error::UnexpectedResponse { status, content } => write!(
                f,
                "UnexpectedResponse: {}, with content {}",
                status,
                &content[0..200]
            ),
            Error::NoData => write!(f, "No data"),
            Error::UnexpectedData(val) => write!(f, "UnexpectedData: {:#}", val),
            Error::Reqwest(e) => write!(f, "error in reqwest: {}", e),
            Error::Serde(e) => write!(f, "error in serde: {}", e),
            Error::Io(e) => write!(f, "error in IO: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Reqwest(e) => Some(e),
            Error::Serde(e) => Some(e),
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}
impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<flecs_console_client::apis::Error<GetApiV2ManifestsAppVersionError>> for Error {
    fn from(value: flecs_console_client::apis::Error<GetApiV2ManifestsAppVersionError>) -> Self {
        match value {
            flecs_console_client::apis::Error::Reqwest(e) => Error::Reqwest(e),
            flecs_console_client::apis::Error::Serde(e) => Error::Serde(e),
            flecs_console_client::apis::Error::Io(e) => Error::Io(e),
            flecs_console_client::apis::Error::ResponseError(content) => match content.entity {
                Some(GetApiV2ManifestsAppVersionError::Status403(v)) => {
                    Error::Http(HttpError::ErrorDescription(v))
                }
                Some(GetApiV2ManifestsAppVersionError::Status404(v)) => {
                    Error::Http(HttpError::ErrorDescription(ErrorDescription {
                        reason: None,
                        status_code: v.status_code,
                        status_text: v.status_text,
                    }))
                }
                Some(GetApiV2ManifestsAppVersionError::Status500(v)) => {
                    Error::Http(HttpError::ErrorDescription(v))
                }
                Some(GetApiV2ManifestsAppVersionError::UnknownValue(v)) => {
                    Error::Http(HttpError::UnknownError(v))
                }
                None => Error::Http(HttpError::ErrorDescription(ErrorDescription {
                    reason: None,
                    status_code: Some(content.status.as_u16() as i32),
                    status_text: None,
                })),
            },
        }
    }
}

pub async fn download_manifest(
    console_configuration: &Configuration,
    x_session_id: &str,
    app: &str,
    version: &str,
) -> Result<AppManifestVersion, Error> {
    let response =
        get_api_v2_manifests_app_version(console_configuration, x_session_id, app, version).await?;
    if response.status != StatusCode::OK {
        return Err(Error::UnexpectedResponse {
            status: response.status,
            content: response.content,
        });
    }
    match response
        .entity
        .ok_or_else(|| Error::InvalidContent(response.content))?
    {
        GetApiV2ManifestsAppVersionSuccess::Status200(val) => {
            let val = val.data.ok_or_else(|| Error::NoData)?;
            serde_json::from_value::<AppManifestVersion>(val).map_err(Error::Serde)
        }
        GetApiV2ManifestsAppVersionSuccess::UnknownValue(v) => Err(Error::UnexpectedData(v)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spell::Error::NoData;
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
        let path: String = format!("/api/v2/manifests/{}/{}", APP_NAME, APP_VERSION);
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
        let path: String = format!("/api/v2/manifests/{}/{}", APP_NAME, APP_VERSION);
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
            Err(NoData) => {}
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
        let path: String = format!("/api/v2/manifests/{}/{}", APP_NAME, APP_VERSION);
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
            Err(Error::UnexpectedResponse {
                status: StatusCode::ACCEPTED,
                ..
            }) => {}
            x => {
                panic!("Expected Error::UnexpectedResponse {{status: StatusCode::ACCEPTED, ..}}, got {:?}", x)
            }
        }
    }
}
