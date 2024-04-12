use crate::ffi2;
use flecs_console_api_client_rs::apis::configuration::Configuration;
use flecs_console_api_client_rs::apis::default_api::{
    get_api_v2_manifests_app_version, post_api_v2_tokens,
};
use flecs_console_api_client_rs::apis::device_api::{
    post_api_v2_device_license_activate, post_api_v2_device_license_validate,
};
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
    authentication: Option<ffi2::Authentication>,
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
        Ok(resp
            .data
            .ok_or(GenericError::new("No response data".to_string()))?
            .session_id
            .ok_or(GenericError::new("No session id in response".to_string()))?)
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
        ));

        Ok(resp?
            .data
            .ok_or(GenericError::new("No response data".to_string()))?
            .is_valid
            .ok_or(GenericError::new("No 'isValid' in response".to_string()))?)
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
        Ok(serde_json::to_string(&resp.data)?)
    }
    pub fn acquire_download_token(
        self: &Console,
        app: String,
        version: String,
        session_id: String,
    ) -> anyhow::Result<ffi2::DownloadToken> {
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
        ));
        let token = resp?
            .data
            .ok_or(GenericError::new("No data present".to_string()))?
            .token
            .ok_or(GenericError::new("No token present".to_string()))?;
        Ok(ffi2::DownloadToken {
            username: token
                .username
                .ok_or(GenericError::new("No username in token".to_string()))?,
            password: token
                .password
                .ok_or(GenericError::new("No password in token".to_string()))?,
        })
    }
    pub fn authentication(&self) -> ffi2::Authentication {
        match &self.authentication {
            Some(auth) => auth.clone(),
            _ => ffi2::Authentication::default(),
        }
    }
    pub fn store_authentication(&mut self, authentication: ffi2::Authentication) -> u16 {
        self.authentication = Some(authentication);
        204
    }
    pub fn delete_authentication(&mut self) -> u16 {
        self.authentication = None;
        204
    }
}
