use crate::fsm::console_client::ConsoleClient;
use crate::jeweler::app::Token;
use crate::sorcerer::authmancer::Authmancer;
use crate::sorcerer::{spell, Sorcerer};
use crate::vault::pouch::Pouch;
use crate::vault::Vault;
use async_trait::async_trait;
use flecsd_axum_server::models::AuthResponseData;

#[derive(Default)]
pub struct AuthmancerImpl {}

impl Sorcerer for AuthmancerImpl {}

#[async_trait]
impl Authmancer for AuthmancerImpl {
    async fn delete_authentication(&self, vault: &Vault) {
        let mut grabbed_pouches = vault.reservation().reserve_secret_pouch_mut().grab().await;
        spell::auth::delete_authentication(
            grabbed_pouches
                .secret_pouch_mut
                .as_mut()
                .unwrap()
                .gems_mut(),
        )
    }

    async fn store_authentication(&self, auth: AuthResponseData, vault: &Vault) {
        let mut grabbed_pouches = vault.reservation().reserve_secret_pouch_mut().grab().await;
        spell::auth::store_authentication(
            auth,
            grabbed_pouches
                .secret_pouch_mut
                .as_mut()
                .unwrap()
                .gems_mut(),
        )
    }

    async fn acquire_download_token(
        &self,
        configuration: ConsoleClient,
        vault: &Vault,
        app: &str,
        version: &str,
    ) -> anyhow::Result<Option<Token>> {
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
            .id
            .unwrap_or_default();
        spell::auth::acquire_download_token(configuration, &session_id, app, version).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::pouch::Pouch;
    use crate::vault::tests::create_empty_test_vault;
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

    #[tokio::test]
    async fn delete_authentication_test() {
        let vault = create_empty_test_vault();
        vault
            .reservation()
            .reserve_secret_pouch_mut()
            .grab()
            .await
            .secret_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut()
            .authentication = Some(create_test_auth());
        AuthmancerImpl::default()
            .delete_authentication(&vault)
            .await;
        assert!(vault
            .reservation()
            .reserve_secret_pouch()
            .grab()
            .await
            .secret_pouch
            .as_ref()
            .unwrap()
            .gems()
            .authentication
            .is_none())
    }

    #[tokio::test]
    async fn store_authentication_test() {
        let vault = create_empty_test_vault();
        assert!(vault
            .reservation()
            .reserve_secret_pouch()
            .grab()
            .await
            .secret_pouch
            .as_ref()
            .unwrap()
            .gems()
            .authentication
            .is_none());
        AuthmancerImpl::default()
            .store_authentication(create_test_auth(), &vault)
            .await;
        assert_eq!(
            Some(create_test_auth()),
            vault
                .reservation()
                .reserve_secret_pouch()
                .grab()
                .await
                .secret_pouch
                .as_ref()
                .unwrap()
                .gems()
                .authentication
        );
    }
}
