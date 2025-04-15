pub use super::Result;
use crate::fsm::console_client::ConsoleClient;
use anyhow::{Context, anyhow};
use flecs_console_client::apis::device_api::{
    PostApiV2DeviceLicenseActivateError, PostApiV2DeviceLicenseActivateSuccess,
    PostApiV2DeviceLicenseValidateSuccess,
};
use flecs_console_client::apis::{Error, ResponseContent};
use flecs_console_client::models::{
    ActivationData, PostApiV2DeviceLicenseActivateRequest, SessionId,
};
use http::StatusCode;

pub async fn validate_license(
    session_id: Option<String>,
    configuration: ConsoleClient,
) -> Result<bool> {
    match &session_id {
        Some(session_id) => {
            match flecs_console_client::apis::device_api::post_api_v2_device_license_validate(
                &configuration,
                session_id,
            )
            .await
            {
                Ok(ResponseContent {
                    entity: Some(PostApiV2DeviceLicenseValidateSuccess::Status200(response)),
                    ..
                }) => Ok(response.data.is_valid),
                Ok(ResponseContent {
                    entity: Some(PostApiV2DeviceLicenseValidateSuccess::UnknownValue(value)),
                    ..
                }) => Err(anyhow!("Received unknown value from console: {value:?}")),
                Ok(ResponseContent { entity: None, .. }) => {
                    Err(anyhow!("Received no data from console".to_string()))
                }
                Err(e) => Err(anyhow!(e)),
            }
        }
        None => Ok(false),
    }
    .with_context(|| format!("Could not validate session id: {session_id:?}"))
}

pub fn read_serial_number() -> Option<String> {
    None
}

#[derive(Debug, PartialEq)]
pub enum ActivationResult {
    Activated(ActivationData),
    AlreadyActive,
}

fn handle_activation_response(
    response: Result<
        ResponseContent<PostApiV2DeviceLicenseActivateSuccess>,
        Error<PostApiV2DeviceLicenseActivateError>,
    >,
) -> Result<ActivationResult> {
    match response {
        Ok(ResponseContent {
            entity: Some(PostApiV2DeviceLicenseActivateSuccess::Status200(activation_data)),
            ..
        }) => Ok(ActivationResult::Activated(*activation_data.data)),
        Ok(ResponseContent {
            entity: Some(PostApiV2DeviceLicenseActivateSuccess::Status204()),
            ..
        }) => Ok(ActivationResult::AlreadyActive),
        Ok(ResponseContent {
            entity: Some(PostApiV2DeviceLicenseActivateSuccess::UnknownValue(value)),
            ..
        }) => Err(anyhow!("Received unknown value from console: {value:?}")),
        Ok(ResponseContent {
            entity: None,
            status: StatusCode::NO_CONTENT,
            ..
        }) => Ok(ActivationResult::AlreadyActive),
        Ok(ResponseContent {
            entity: None,
            content,
            ..
        }) => Err(anyhow!("Received invalid data from console: {content}")),
        Err(e) => Err(anyhow!(e)),
    }
}

pub async fn activate_via_license_key(
    license_key: &str,
    session_id: SessionId,
    configuration: ConsoleClient,
) -> Result<ActivationResult> {
    let response = flecs_console_client::apis::device_api::post_api_v2_device_license_activate(
        &configuration,
        None,
        session_id.id.as_deref(),
        Some(PostApiV2DeviceLicenseActivateRequest {
            license_key: Some(license_key.to_string()),
        }),
    )
    .await;
    handle_activation_response(response)
        .with_context(|| format!("Could not activate via license key {license_key}"))
}

pub async fn activate_via_user_license(
    configuration: ConsoleClient,
    authorization_token: &str,
) -> Result<ActivationResult> {
    let bearer_token = format!("Bearer {authorization_token}");
    let response = flecs_console_client::apis::device_api::post_api_v2_device_license_activate(
        &configuration,
        Some(&bearer_token),
        None,
        None,
    )
    .await;
    handle_activation_response(response).context("Could not activate via user license")
}

#[cfg(test)]
mod tests {
    use super::*;
    use flecs_console_client::apis::ResponseContent;
    use flecs_console_client::apis::device_api::PostApiV2DeviceLicenseActivateSuccess;
    use flecs_console_client::models::{
        ActivationData, ErrorDescription, PostApiV2DeviceLicenseActivate200Response, SessionId,
    };
    use http::StatusCode;

    const LICENSE_KEY: &str = "1234-ABCD-5678-EFGH";
    const SESSION_ID: &str = "74c3b620-6048-4bfd-9bf7-c9857a001694";
    const TIMESTAMP: u64 = 17243237291234u64;

    #[tokio::test]
    async fn activate_via_license_key_already_active_test() {
        let session = SessionId {
            id: Some(SESSION_ID.to_string()),
            timestamp: Some(TIMESTAMP),
        };
        let (mut server, config) = crate::tests::create_test_server_and_config().await;

        let mock = server
            .mock("POST", "/api/v2/device/license/activate")
            .with_status(204)
            .create_async()
            .await;
        assert_eq!(
            ActivationResult::AlreadyActive,
            activate_via_license_key(LICENSE_KEY, session, config)
                .await
                .unwrap()
        );
        mock.assert();
    }

    #[tokio::test]
    async fn activate_via_license_key_test() {
        let session = SessionId {
            id: None,
            timestamp: None,
        };
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let body = serde_json::json!({
            "statusCode": 200,
            "statusTest": "OK",
            "data": {
                "sessionId": {
                    "id": SESSION_ID,
                    "timestamp": TIMESTAMP
                },
                "licenseKey": LICENSE_KEY
            }
        });
        let body = serde_json::to_string(&body).unwrap();
        let resulting_session_id = SessionId {
            id: Some(SESSION_ID.to_string()),
            timestamp: Some(TIMESTAMP),
        };
        let mock = server
            .mock("POST", "/api/v2/device/license/activate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&body)
            .create_async()
            .await;
        assert_eq!(
            ActivationResult::Activated(ActivationData {
                session_id: Box::new(resulting_session_id),
                license_key: LICENSE_KEY.to_string(),
            }),
            activate_via_license_key(LICENSE_KEY, session, config)
                .await
                .unwrap()
        );
        mock.assert();
    }

    #[tokio::test]
    async fn activate_via_user_test() {
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let body = serde_json::json!({
            "statusCode": 200,
            "statusTest": "OK",
            "data": {
                "sessionId": {
                    "id": SESSION_ID,
                    "timestamp": TIMESTAMP
                },
                "licenseKey": LICENSE_KEY
            }
        });
        let body = serde_json::to_string(&body).unwrap();
        let resulting_session_id = SessionId {
            id: Some(SESSION_ID.to_string()),
            timestamp: Some(TIMESTAMP),
        };
        let mock = server
            .mock("POST", "/api/v2/device/license/activate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&body)
            .create_async()
            .await;
        assert_eq!(
            ActivationResult::Activated(ActivationData {
                session_id: Box::new(resulting_session_id),
                license_key: LICENSE_KEY.to_string(),
            }),
            activate_via_user_license(config, "valid authorization token")
                .await
                .unwrap()
        );
        mock.assert();
    }

    #[test]
    fn handle_200_response_test() {
        let activation_data = ActivationData {
            session_id: Box::new(SessionId {
                id: Some(SESSION_ID.to_string()),
                timestamp: Some(TIMESTAMP),
            }),
            license_key: LICENSE_KEY.to_string(),
        };
        let response = Ok(ResponseContent {
            status: StatusCode::OK,
            content: String::new(),
            entity: Some(PostApiV2DeviceLicenseActivateSuccess::Status200(
                PostApiV2DeviceLicenseActivate200Response {
                    status_text: None,
                    status_code: Some(200),
                    data: Box::new(activation_data.clone()),
                },
            )),
        });
        assert_eq!(
            ActivationResult::Activated(activation_data),
            handle_activation_response(response).unwrap()
        );
    }
    #[test]
    fn handle_204_response_test() {
        let response = Ok(ResponseContent {
            status: StatusCode::NO_CONTENT,
            content: String::new(),
            entity: Some(PostApiV2DeviceLicenseActivateSuccess::Status204()),
        });
        assert_eq!(
            ActivationResult::AlreadyActive,
            handle_activation_response(response).unwrap()
        );
    }
    #[test]
    fn handle_empty_204_response_test() {
        let response = Ok(ResponseContent {
            status: StatusCode::NO_CONTENT,
            content: String::new(),
            entity: None,
        });
        assert_eq!(
            ActivationResult::AlreadyActive,
            handle_activation_response(response).unwrap()
        );
    }
    #[test]
    fn handle_unknown_response_test() {
        let unknown_value = serde_json::json!({"randomValue": 1234});
        let response = Ok(ResponseContent {
            status: StatusCode::OK,
            content: String::new(),
            entity: Some(PostApiV2DeviceLicenseActivateSuccess::UnknownValue(
                unknown_value.clone(),
            )),
        });
        assert!(
            handle_activation_response(response)
                .err()
                .unwrap()
                .to_string()
                .contains(&format!(
                    "Received unknown value from console: {unknown_value:?}"
                ))
        );
    }
    #[test]
    fn handle_invalid_response_test() {
        let invalid_content = "invalid content";
        let response = Ok(ResponseContent {
            status: StatusCode::OK,
            content: invalid_content.to_string(),
            entity: None,
        });
        assert!(
            handle_activation_response(response)
                .err()
                .unwrap()
                .to_string()
                .contains(&format!(
                    "Received invalid data from console: {}",
                    invalid_content
                ))
        );
    }
    #[test]
    fn handle_error_response_test() {
        let error_content = ResponseContent {
            status: StatusCode::FORBIDDEN,
            content: String::new(),
            entity: Some(PostApiV2DeviceLicenseActivateError::Status403(
                ErrorDescription {
                    reason: None,
                    status_code: None,
                    status_text: None,
                },
            )),
        };
        let response = Err(Error::ResponseError(error_content.clone()));
        assert!(
            handle_activation_response(response)
                .err()
                .unwrap()
                .to_string()
                .contains(&Error::ResponseError(error_content).to_string())
        );
    }

    #[tokio::test]
    async fn validate_license_valid() {
        let session_id = Some(SESSION_ID.to_string());
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let body = serde_json::json!({
            "statusCode": 200,
            "statusTest": "OK",
            "data": {
                "isValid": true
            }
        });
        let body = serde_json::to_string(&body).unwrap();
        let mock = server
            .mock("POST", "/api/v2/device/license/validate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&body)
            .create_async()
            .await;
        assert!(validate_license(session_id, config).await.unwrap());
        mock.assert();
    }
    #[tokio::test]
    async fn validate_license_invalid() {
        let session_id = Some(SESSION_ID.to_string());
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let body = serde_json::json!({
            "statusCode": 200,
            "statusTest": "OK",
            "data": {
                "isValid": false
            }
        });
        let body = serde_json::to_string(&body).unwrap();
        let mock = server
            .mock("POST", "/api/v2/device/license/validate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&body)
            .create_async()
            .await;
        assert!(!validate_license(session_id, config).await.unwrap());
        mock.assert();
    }

    #[tokio::test]
    async fn validate_license_missing() {
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = server
            .mock("POST", "/api/v2/device/license/validate")
            .with_status(200)
            .expect(0)
            .create_async()
            .await;
        assert!(!validate_license(None, config).await.unwrap());
        mock.assert();
    }

    #[tokio::test]
    async fn validate_license_unknown_value() {
        let session_id = Some(SESSION_ID.to_string());
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let body_json = serde_json::json!({
            "statusCode": 200,
            "statusTest": "OK",
            "dataa": {
                "isValid": false
            }
        });
        let body = serde_json::to_string(&body_json).unwrap();
        let mock = server
            .mock("POST", "/api/v2/device/license/validate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&body)
            .create_async()
            .await;
        assert!(
            format!(
                "{:#}",
                validate_license(session_id, config).await.err().unwrap()
            )
            .contains(&format!(
                "Received unknown value from console: {body_json:?}"
            ))
        );
        mock.assert();
    }
    #[tokio::test]
    async fn validate_license_no_data() {
        let session_id = Some(SESSION_ID.to_string());
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = server
            .mock("POST", "/api/v2/device/license/validate")
            .with_status(200)
            .create_async()
            .await;
        assert!(
            format!(
                "{:#}",
                validate_license(session_id, config).await.err().unwrap()
            )
            .contains("Received no data from console")
        );
        mock.assert();
    }
    #[tokio::test]
    async fn validate_license_error() {
        let session_id = Some(SESSION_ID.to_string());
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = server
            .mock("POST", "/api/v2/device/license/validate")
            .with_status(500)
            .create_async()
            .await;
        assert!(validate_license(session_id, config).await.is_err());
        mock.assert();
    }
}
