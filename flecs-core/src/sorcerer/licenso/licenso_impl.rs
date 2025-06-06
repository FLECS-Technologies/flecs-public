use crate::fsm::console_client::ConsoleClient;
use crate::sorcerer::licenso::Licenso;
use crate::sorcerer::spell::license::ActivationResult;
use crate::sorcerer::{Sorcerer, spell};
use crate::vault::pouch::Pouch;
use crate::vault::{GrabbedPouches, Vault};
use anyhow::{Context, anyhow};
use async_trait::async_trait;

#[derive(Default)]
pub struct LicensoImpl {}

impl Sorcerer for LicensoImpl {}

#[async_trait]
impl Licenso for LicensoImpl {
    async fn activate_license(
        &self,
        vault: &Vault,
        configuration: ConsoleClient,
    ) -> anyhow::Result<()> {
        let secrets = vault.get_secrets().await;

        let activation_result = match (
            secrets.license_key.as_ref(),
            spell::license::read_serial_number(),
        ) {
            (Some(&ref key), _) | (None, Some(ref key)) => {
                spell::license::activate_via_license_key(key, secrets.get_session_id(), configuration)
                    .await
            }
            (None, None) => spell::license::activate_via_user_license(
                configuration,
                &secrets
                    .authentication
                    .as_ref()
                    .ok_or_else(|| {
                        anyhow!("Can not activate license, as no license key or user authentication is present")
                    })?
                    .jwt
                    .token,
            )
                .await,
        };
        match activation_result? {
            ActivationResult::Activated(activation_data) => {
                match vault.reservation().reserve_secret_pouch_mut().grab().await
                { GrabbedPouches {
                    secret_pouch_mut: Some(ref mut secret_pouch),
                    ..
                } => {
                    secret_pouch.gems_mut().license_key = Some(activation_data.license_key);
                    secret_pouch
                        .gems_mut()
                        .set_session_id(*activation_data.session_id);
                    Ok(())
                } _ => {
                    panic!("Failed to reserve secret pouch mut");
                }}
            }
            ActivationResult::AlreadyActive => {
                match vault.reservation().reserve_secret_pouch().grab().await
                { GrabbedPouches {
                    secret_pouch: Some(ref secret_pouch),
                    ..
                } => {
                    match (&secret_pouch.gems().license_key, secret_pouch.gems().get_session_id().id) {
                        (None, None) => Err(anyhow!("Console responded with already active, but license and session id are not set")),
                        (None, Some(_)) => Err(anyhow!("Console responded with already active, but license is not set")),
                        (Some(_), None)=> Err(anyhow!("Console responded with already active, but session id is not set")),
                        _ => Ok(()),
                    }
                } _ => {
                    panic!("Failed to reserve secret pouch");
                }}
            }
        }.context("Could not activate license")
    }

    async fn validate_license(
        &self,
        vault: &Vault,
        configuration: ConsoleClient,
    ) -> anyhow::Result<bool> {
        let session_id = vault
            .reservation()
            .reserve_secret_pouch()
            .grab()
            .await
            .secret_pouch
            .as_ref()
            .unwrap()
            .gems()
            .get_session_id()
            .id;
        spell::license::validate_license(session_id, configuration).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::pouch::secret::Secrets;
    use crate::vault::tests::create_empty_test_vault;
    use flecs_console_client::models::SessionId;
    use flecsd_axum_server::models::{AuthResponseData, FeatureFlags, Jwt, User};
    use mockito::Matcher;

    const LICENSE_KEY: &str = "1234-ABCD-5678-EFGH";
    const SESSION_ID: &str = "74c3b620-6048-4bfd-9bf7-c9857a001694";
    const TIMESTAMP: u64 = 17243237291234u64;

    async fn setup_secrets(vault: &Vault, secrets: Secrets) {
        let mut pouches = vault.reservation().reserve_secret_pouch_mut().grab().await;
        let secret_pouch = pouches.secret_pouch_mut.as_mut().unwrap();
        assert!(secret_pouch.gems().license_key.is_none());
        assert!(secret_pouch.gems().get_session_id().id.is_none());
        assert!(secret_pouch.gems().get_session_id().timestamp.is_none());
        assert!(secret_pouch.gems().authentication.is_none());
        secret_pouch
            .gems_mut()
            .set_session_id(secrets.get_session_id());
        secret_pouch.gems_mut().authentication = secrets.authentication;
        secret_pouch.gems_mut().license_key = secrets.license_key;
    }

    fn create_auth_for_token(token: String) -> Option<AuthResponseData> {
        Some(AuthResponseData {
            jwt: Jwt {
                token,
                token_expires: 23,
            },
            user: User {
                id: 1,
                user_login: String::new(),
                user_email: String::new(),
                display_name: String::new(),
            },
            feature_flags: FeatureFlags {
                is_vendor: false,
                is_whitelabeled: false,
            },
        })
    }

    #[tokio::test]
    async fn activate_via_user_test() {
        let auth = "some_valid_auth";
        let vault = create_empty_test_vault();
        setup_secrets(
            &vault,
            Secrets::new(
                None,
                SessionId::default(),
                create_auth_for_token(auth.to_string()),
            ),
        )
        .await;
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
            .match_header("Authorization", format!("Bearer {}", auth).as_str())
            .create_async()
            .await;
        LicensoImpl::default()
            .activate_license(&vault, config)
            .await
            .unwrap();
        mock.assert();
        let mut secrets = vault.reservation().reserve_secret_pouch_mut().grab().await;
        let secrets = secrets.secret_pouch_mut.as_mut().unwrap();
        assert_eq!(secrets.gems().get_session_id(), resulting_session_id);
        assert_eq!(secrets.gems().license_key, Some(LICENSE_KEY.to_string()));
    }

    #[tokio::test]
    async fn activate_via_license_test() {
        let vault = create_empty_test_vault();
        setup_secrets(
            &vault,
            Secrets::new(Some(LICENSE_KEY.to_string()), SessionId::default(), None),
        )
        .await;
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
            .match_header("Authorization", Matcher::Missing)
            .create_async()
            .await;
        LicensoImpl::default()
            .activate_license(&vault, config)
            .await
            .unwrap();
        mock.assert();
        let mut secrets = vault.reservation().reserve_secret_pouch_mut().grab().await;
        let secrets = secrets.secret_pouch_mut.as_mut().unwrap();
        assert_eq!(secrets.gems().get_session_id(), resulting_session_id);
        assert_eq!(secrets.gems().license_key, Some(LICENSE_KEY.to_string()));
    }

    #[tokio::test]
    async fn activate_already_active_test() {
        let vault = create_empty_test_vault();
        setup_secrets(
            &vault,
            Secrets::new(
                Some(LICENSE_KEY.to_string()),
                SessionId {
                    id: Some(SESSION_ID.to_string()),
                    timestamp: Some(TIMESTAMP),
                },
                None,
            ),
        )
        .await;
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let resulting_session_id = SessionId {
            id: Some(SESSION_ID.to_string()),
            timestamp: Some(TIMESTAMP),
        };
        let mock = server
            .mock("POST", "/api/v2/device/license/activate")
            .with_status(204)
            .create_async()
            .await;
        LicensoImpl::default()
            .activate_license(&vault, config)
            .await
            .unwrap();
        mock.assert();
        let mut secrets = vault.reservation().reserve_secret_pouch_mut().grab().await;
        let secrets = secrets.secret_pouch_mut.as_mut().unwrap();
        assert_eq!(secrets.gems().get_session_id(), resulting_session_id);
        assert_eq!(secrets.gems().license_key, Some(LICENSE_KEY.to_string()));
    }

    #[tokio::test]
    async fn activate_already_active_no_license_no_session_test() {
        let vault = create_empty_test_vault();
        setup_secrets(
            &vault,
            Secrets::new(
                None,
                SessionId::default(),
                create_auth_for_token("irrelevant".to_string()),
            ),
        )
        .await;
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = server
            .mock("POST", "/api/v2/device/license/activate")
            .with_status(204)
            .create_async()
            .await;
        assert!(
            format!(
                "{:#}",
                LicensoImpl::default()
                    .activate_license(&vault, config)
                    .await
                    .err()
                    .unwrap()
            )
            .contains(
                "Console responded with already active, but license and session id are not set"
            )
        );
        mock.assert();
        let mut secrets = vault.reservation().reserve_secret_pouch_mut().grab().await;
        let secrets = secrets.secret_pouch_mut.as_mut().unwrap();
        assert_eq!(secrets.gems().get_session_id(), SessionId::default());
        assert_eq!(secrets.gems().license_key, None);
    }

    #[tokio::test]
    async fn activate_already_active_no_license_test() {
        let vault = create_empty_test_vault();
        setup_secrets(
            &vault,
            Secrets::new(
                None,
                SessionId {
                    id: Some(SESSION_ID.to_string()),
                    timestamp: Some(TIMESTAMP),
                },
                create_auth_for_token("irrelevant".to_string()),
            ),
        )
        .await;
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = server
            .mock("POST", "/api/v2/device/license/activate")
            .with_status(204)
            .create_async()
            .await;
        assert!(
            format!(
                "{:#}",
                LicensoImpl::default()
                    .activate_license(&vault, config)
                    .await
                    .err()
                    .unwrap()
            )
            .contains("Console responded with already active, but license is not set")
        );
        mock.assert();
        let mut secrets = vault.reservation().reserve_secret_pouch_mut().grab().await;
        let secrets = secrets.secret_pouch_mut.as_mut().unwrap();
        assert_eq!(
            secrets.gems().get_session_id(),
            SessionId {
                id: Some(SESSION_ID.to_string()),
                timestamp: Some(TIMESTAMP),
            }
        );
        assert_eq!(secrets.gems().license_key, None);
    }

    #[tokio::test]
    async fn activate_already_active_no_session_test() {
        let vault = create_empty_test_vault();
        setup_secrets(
            &vault,
            Secrets::new(Some(LICENSE_KEY.to_string()), SessionId::default(), None),
        )
        .await;
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = server
            .mock("POST", "/api/v2/device/license/activate")
            .with_status(204)
            .create_async()
            .await;
        assert!(
            format!(
                "{:#}",
                LicensoImpl::default()
                    .activate_license(&vault, config)
                    .await
                    .err()
                    .unwrap()
            )
            .contains("Console responded with already active, but session id is not set")
        );
        mock.assert();
        let mut secrets = vault.reservation().reserve_secret_pouch_mut().grab().await;
        let secrets = secrets.secret_pouch_mut.as_mut().unwrap();
        assert_eq!(secrets.gems().get_session_id(), SessionId::default());
        assert_eq!(secrets.gems().license_key, Some(LICENSE_KEY.to_string()));
    }

    #[tokio::test]
    async fn activate_without_secrets_test() {
        let vault = create_empty_test_vault();
        setup_secrets(&vault, Secrets::new(None, SessionId::default(), None)).await;
        let (mut server, config) = crate::tests::create_test_server_and_config().await;
        let mock = server
            .mock("POST", "/api/v2/device/license/activate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .expect(0)
            .create_async()
            .await;
        assert!(
            LicensoImpl::default()
                .activate_license(&vault, config)
                .await
                .err()
                .unwrap()
                .to_string()
                .contains(
                    "Can not activate license, as no license key or user authentication is present"
                ),
        );
        mock.assert();
        let mut secrets = vault.reservation().reserve_secret_pouch_mut().grab().await;
        let secrets = secrets.secret_pouch_mut.as_mut().unwrap();
        assert_eq!(secrets.gems().get_session_id(), SessionId::default());
        assert_eq!(secrets.gems().license_key, None);
        assert_eq!(secrets.gems().authentication, None);
    }
}
