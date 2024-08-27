use super::spell;
use crate::vault::pouch::Pouch;
use crate::vault::Vault;
use flecs_console_client::apis::configuration::Configuration;
use flecs_console_client::models::PostApiV2Tokens200ResponseData;
use flecsd_axum_server::models::AuthResponseData;

pub fn delete_authentication(vault: &Vault) {
    let mut grabbed_pouches = vault.reservation().reserve_secret_pouch_mut().grab();
    spell::auth::delete_authentication(
        grabbed_pouches
            .secret_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut(),
    )
}

pub fn store_authentication(auth: AuthResponseData, vault: &Vault) {
    let mut grabbed_pouches = vault.reservation().reserve_secret_pouch_mut().grab();
    spell::auth::store_authentication(
        auth,
        grabbed_pouches
            .secret_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut(),
    )
}

pub async fn acquire_download_token(
    configuration: &Configuration,
    vault: &Vault,
    app: &str,
    version: &str,
) -> Result<PostApiV2Tokens200ResponseData, String> {
    let session_id = vault
        .reservation()
        .reserve_secret_pouch()
        .grab()
        .secret_pouch
        .as_ref()
        .unwrap()
        .gems()
        .get_session_id()
        .id
        .unwrap_or_default();
    spell::auth::acquire_download_token(
        configuration,
        &session_id,
        app.to_string(),
        version.to_string(),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::pouch::Pouch;
    use crate::vault::{Vault, VaultConfig};
    use flecsd_axum_server::models::{AuthResponseData, FeatureFlags, Jwt, User};
    use std::fs;
    use std::path::Path;

    const TEST_PATH: &str = "/tmp/flecs-tests/auth";

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
        let test_path = Path::new(TEST_PATH).join("delete_authentication");
        fs::create_dir_all(&test_path).unwrap();
        let vault = Vault::new(VaultConfig { path: test_path });
        vault
            .reservation()
            .reserve_secret_pouch_mut()
            .grab()
            .secret_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut()
            .authentication = Some(create_test_auth());
        delete_authentication(&vault);
        assert!(vault
            .reservation()
            .reserve_secret_pouch()
            .grab()
            .secret_pouch
            .as_ref()
            .unwrap()
            .gems()
            .authentication
            .is_none())
    }

    #[test]
    fn store_authentication_test() {
        let test_path = Path::new(TEST_PATH).join("store_authentication");
        fs::create_dir_all(&test_path).unwrap();
        let vault = Vault::new(VaultConfig { path: test_path });
        assert!(vault
            .reservation()
            .reserve_secret_pouch()
            .grab()
            .secret_pouch
            .as_ref()
            .unwrap()
            .gems()
            .authentication
            .is_none());
        store_authentication(create_test_auth(), &vault);
        assert_eq!(
            Some(create_test_auth()),
            vault
                .reservation()
                .reserve_secret_pouch()
                .grab()
                .secret_pouch
                .as_ref()
                .unwrap()
                .gems()
                .authentication
        );
    }
}
