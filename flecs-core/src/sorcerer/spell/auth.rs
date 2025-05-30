pub use super::Result;
use crate::fsm::console_client::ConsoleClient;
use crate::jeweler::app::Token;
use crate::vault::pouch::secret::Secrets;
use anyhow::{Context, anyhow};
use flecs_console_client::apis::ResponseContent;
use flecs_console_client::apis::default_api::{PostApiV2TokensSuccess, post_api_v2_tokens};
use flecs_console_client::models::PostApiV2TokensRequest;
use flecsd_axum_server::models::AuthResponseData;
use http::StatusCode;

pub fn delete_authentication(secrets: &mut Secrets) {
    secrets.authentication = None;
}

pub fn store_authentication(auth: AuthResponseData, secrets: &mut Secrets) {
    secrets.authentication = Some(auth);
}

pub async fn acquire_download_token(
    console_configuration: ConsoleClient,
    x_session_id: &str,
    app: &str,
    version: &str,
) -> Result<Option<Token>> {
    match post_api_v2_tokens(
        &console_configuration,
        x_session_id,
        Some(PostApiV2TokensRequest {
            app: app.to_string(),
            version: version.to_string(),
        }),
    )
    .await
    {
        Ok(ResponseContent {
            entity: Some(PostApiV2TokensSuccess::Status200(response)),
            ..
        }) => Ok(Some((*response.data).into())),
        Ok(ResponseContent {
            status: StatusCode::NO_CONTENT,
            ..
        }) => Ok(None),
        Ok(ResponseContent {
            status, content, ..
        }) => Err(anyhow!(
            "Received invalid response for tokens request with status {status}: {content}"
        )),
        Err(e) => Err(anyhow!(e)),
    }
    .with_context(|| format!("Failed to acquire download token for app {app}-{version}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use flecs_console_client::models::SessionId;
    use flecsd_axum_server::models::{AuthResponseData, FeatureFlags, Jwt, User};

    fn create_test_auth() -> AuthResponseData {
        AuthResponseData {
            user: User {
                id: 123,
                user_email: "name@some-mail.com".to_string(),
                user_login: "my_name".to_string(),
                display_name: "My Name".to_string(),
            },
            jwt: Jwt {
                token: "secret value 1234&/(".to_string(),
                token_expires: 1234565678,
            },
            feature_flags: FeatureFlags {
                is_vendor: false,
                is_whitelabeled: false,
            },
        }
    }

    #[test]
    fn delete_authentication_test() {
        let mut secrets = Secrets::new(None, SessionId::default(), Some(create_test_auth()));
        assert!(secrets.authentication.is_some());
        delete_authentication(&mut secrets);
        assert!(secrets.authentication.is_none());
    }

    #[test]
    fn store_authentication_test() {
        let mut secrets = Secrets::new(None, SessionId::default(), None);
        store_authentication(create_test_auth(), &mut secrets);
        assert_eq!(Some(create_test_auth()), secrets.authentication);
    }

    const USERNAME: &str = "some user";
    const PASSWORD: &str = "some secret password";

    #[tokio::test]
    async fn acquire_download_token_test() {
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let body = serde_json::json!({
            "statusCode": 200,
            "statusTest": "OK",
            "data": {
                "token": {
                    "username": USERNAME,
                    "password": PASSWORD,
                },
            }
        });
        let body = serde_json::to_string(&body).unwrap();
        let expected_result = Token {
            username: USERNAME.to_string(),
            password: PASSWORD.to_string(),
        };
        let mock = server
            .mock("POST", "/api/v2/tokens")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&body)
            .create_async()
            .await;
        assert_eq!(
            acquire_download_token(config, "", "", "").await.unwrap(),
            Some(expected_result)
        );
        mock.assert();
    }

    #[tokio::test]
    async fn acquire_download_token_no_content_test() {
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = server
            .mock("POST", "/api/v2/tokens")
            .with_status(204)
            .create_async()
            .await;
        assert_eq!(
            acquire_download_token(config, "", "", "").await.unwrap(),
            None
        );
        mock.assert();
    }

    #[tokio::test]
    async fn acquire_download_token_invalid_data_test() {
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let body = serde_json::json!({
            "statusCode": 200,
            "statusTest": "OK",
            "data": {
                "username": USERNAME,
                "password": PASSWORD,
            }
        });
        let body = serde_json::to_string(&body).unwrap();
        let mock = server
            .mock("POST", "/api/v2/tokens")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&body)
            .create_async()
            .await;
        assert!(acquire_download_token(config, "", "", "").await.is_err());
        mock.assert();
    }

    #[tokio::test]
    async fn acquire_download_token_error_test() {
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let body = serde_json::json!({
            "additionalInfo": "Access denied"
        });
        let body = serde_json::to_string(&body).unwrap();
        let mock = server
            .mock("POST", "/api/v2/tokens")
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(&body)
            .create_async()
            .await;
        assert!(acquire_download_token(config, "", "", "").await.is_err());
        mock.assert();
    }
}
