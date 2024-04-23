use crate::ffi;
use flecs_console_api_client_rs::apis::configuration::Configuration;
use flecs_console_api_client_rs::apis::default_api::{
    get_api_v2_manifests_app_version, post_api_v2_tokens, GetApiV2ManifestsAppVersionSuccess,
    PostApiV2TokensSuccess,
};
use flecs_console_api_client_rs::apis::device_api::{
    post_api_v2_device_license_activate, post_api_v2_device_license_validate,
    PostApiV2DeviceLicenseActivateSuccess, PostApiV2DeviceLicenseValidateSuccess,
};
use flecs_console_api_client_rs::apis::ResponseContent;
use flecs_console_api_client_rs::models::PostApiV2TokensRequest;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use tokio::runtime::Runtime;

pub fn new_console(base_url: String) -> Box<Console> {
    let mut configuration = Configuration::new();
    configuration.base_path = base_url;
    Box::new(Console {
        authentication: None,
        configuration,
        runtime: Runtime::new().unwrap(),
    })
}

pub struct Console {
    authentication: Option<ffi::Authentication>,
    configuration: Configuration,
    runtime: Runtime,
}

struct GenericError {
    message: String,
}

impl Debug for GenericError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.message, f)
    }
}

impl Display for GenericError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.message, f)
    }
}

impl Error for GenericError {}

impl GenericError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl Console {
    pub fn activate_license(self: &Console, session_id: String) -> anyhow::Result<String> {
        let auth = &self
            .authentication
            .as_ref()
            .ok_or(GenericError::new("No authentication available".to_string()))?;
        let resp = self.runtime.block_on(post_api_v2_device_license_activate(
            &self.configuration,
            &format!("Bearer {}", &auth.jwt.token),
            &session_id,
        ))?;
        let session_id = match resp {
            ResponseContent {
                entity: Some(PostApiV2DeviceLicenseActivateSuccess::Status200(val)),
                ..
            } => val
                .data
                .ok_or(GenericError::new("No response data".to_string()))?
                .session_id
                .ok_or(GenericError::new("No session id in response".to_string()))?,
            ResponseContent {
                entity: Some(PostApiV2DeviceLicenseActivateSuccess::Status204()),
                ..
            } => session_id,
            ResponseContent {
                status: http::StatusCode::NO_CONTENT,
                ..
            } => session_id,
            ResponseContent {
                entity: Some(PostApiV2DeviceLicenseActivateSuccess::UnknownValue(val)),
                ..
            } => Err(GenericError::new(format!("Unexpected body: {:#}", val)))?,
            _ => Err(GenericError::new("No response data".to_string()))?,
        };
        Ok(session_id)
    }

    pub fn validate_license(self: &Console, session_id: String) -> anyhow::Result<bool> {
        let auth = &self
            .authentication
            .as_ref()
            .ok_or(GenericError::new("No authentication available".to_string()))?;
        eprintln!(
            "Validating license with token {} and session id {}",
            &auth.jwt.token, &session_id
        );
        let resp = self.runtime.block_on(post_api_v2_device_license_validate(
            &self.configuration,
            &format!("Bearer {}", &auth.jwt.token),
            &session_id,
        ))?;
        let is_valid = match resp.entity {
            Some(PostApiV2DeviceLicenseValidateSuccess::Status200(val)) => val
                .data
                .ok_or(GenericError::new("No response data".to_string()))?
                .is_valid
                .ok_or(GenericError::new("No session id in response".to_string()))?,
            _ => Err(GenericError::new("No response data".to_string()))?,
        };
        Ok(is_valid)
    }
    pub fn download_manifest(
        self: &Console,
        app: String,
        version: String,
        session_id: String,
    ) -> anyhow::Result<String> {
        let auth = &self
            .authentication
            .as_ref()
            .ok_or(GenericError::new("No authentication available".to_string()))?;
        let resp = self.runtime.block_on(get_api_v2_manifests_app_version(
            &self.configuration,
            &format!("Bearer {}", &auth.jwt.token),
            &session_id,
            &app,
            &version,
        ))?;
        let manifest = match resp.entity {
            Some(GetApiV2ManifestsAppVersionSuccess::Status200(val)) => val
                .data
                .ok_or(GenericError::new("No response data".to_string()))?,
            _ => Err(GenericError::new("No response data".to_string()))?,
        };
        Ok(serde_json::to_string(&manifest)?)
    }
    pub fn acquire_download_token(
        self: &Console,
        app: String,
        version: String,
        session_id: String,
    ) -> anyhow::Result<ffi::DownloadToken> {
        let auth = &self
            .authentication
            .as_ref()
            .ok_or(GenericError::new("No authentication available".to_string()))?;
        let request = PostApiV2TokensRequest {
            version: version.clone(),
            app: app.clone(),
        };
        let resp = self.runtime.block_on(post_api_v2_tokens(
            &self.configuration,
            &format!("Bearer {}", &auth.jwt.token),
            &session_id,
            Some(request),
        ))?;
        let token = match resp.entity {
            Some(PostApiV2TokensSuccess::Status200(val)) => val
                .data
                .ok_or(GenericError::new("No data present".to_string()))?
                .token
                .ok_or(GenericError::new("No token present".to_string()))?,
            _ => Err(GenericError::new("No response data".to_string()))?,
        };
        Ok(ffi::DownloadToken {
            username: token
                .username
                .ok_or(GenericError::new("No username in token".to_string()))?,
            password: token
                .password
                .ok_or(GenericError::new("No password in token".to_string()))?,
        })
    }
    pub fn authentication(&self) -> ffi::Authentication {
        match &self.authentication {
            Some(auth) => auth.clone(),
            _ => ffi::Authentication::default(),
        }
    }
    pub fn store_authentication(&mut self, authentication: ffi::Authentication) -> u16 {
        self.authentication = Some(authentication);
        204
    }
    pub fn delete_authentication(&mut self) -> u16 {
        self.authentication = None;
        204
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::Authentication;
    use flecs_console_api_client_rs::models::{
        PostApiV2DeviceLicenseActivate200Response, PostApiV2DeviceLicenseActivate200ResponseData,
    };

    const ACTIVATE_PATH: &str = "/api/v2/device/license/activate";

    #[test]
    fn test_no_authentication() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", ACTIVATE_PATH)
            .with_status(200)
            .with_body("invalid body")
            .expect(0)
            .create();
        let console = new_console(server.url());
        let session_id = "some_id".to_string();

        let result = console.activate_license(session_id.clone());
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.err().unwrap()),
            "No authentication available".to_string()
        );

        let result = console.validate_license(session_id.clone());
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.err().unwrap()),
            "No authentication available".to_string()
        );

        let result =
            console.download_manifest("my.app".to_string(), "0.1".to_string(), session_id.clone());
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.err().unwrap()),
            "No authentication available".to_string()
        );

        let result = console.acquire_download_token(
            "my.app".to_string(),
            "0.1".to_string(),
            session_id.clone(),
        );
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.err().unwrap()),
            "No authentication available".to_string()
        );
        mock.assert()
    }

    #[test]
    fn test_activate_license_invalid_data() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", ACTIVATE_PATH)
            .with_status(200)
            .with_body("invalid body")
            .expect(1)
            .create();
        let mut console = new_console(server.url());
        console.authentication = Some(Authentication::default());

        let session_id = "some_id".to_string();
        let result = console.activate_license(session_id.clone());
        assert!(result.is_err());
        mock.assert()
    }

    #[test]
    fn test_activate_license_valid_data() {
        let session_id = "some_id".to_string();
        let mut server = mockito::Server::new();
        let resp = PostApiV2DeviceLicenseActivate200Response {
            data: Some(Box::new(PostApiV2DeviceLicenseActivate200ResponseData {
                session_id: Some(session_id.clone()),
            })),
            status_code: Some(200),
            status_text: Some("Ok".to_string()),
        };
        let mock = server
            .mock("POST", ACTIVATE_PATH)
            .with_status(200)
            .with_body(serde_json::to_string(&resp).unwrap())
            .expect(1)
            .create();
        let mut console = new_console(server.url());
        console.authentication = Some(Authentication::default());

        let result = console.activate_license(session_id.clone());
        assert_eq!(result.unwrap(), session_id);
        mock.assert()
    }

    #[test]
    fn test_activate_license_valid_already_activated() {
        let session_id = "some_id".to_string();
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", ACTIVATE_PATH)
            .with_status(204)
            .expect(1)
            .create();
        let mut console = new_console(server.url());
        console.authentication = Some(Authentication::default());

        let result = console.activate_license(session_id.clone());
        assert_eq!(result.unwrap(), session_id);
        mock.assert()
    }
}
