use super::spell;
use super::spell::Error;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::Vault;
use flecs_app_manifest::AppManifestVersion;

pub async fn download_manifest(
    vault: &Vault,
    app_key: AppKey,
) -> Result<AppManifestVersion, Error> {
    let session_id = vault
        .reservation()
        .reserve_secret_pouch()
        .grab()
        .secret_pouch
        .as_ref()
        .unwrap()
        .gems()
        .get_session_id()
        .id;
    let session_id = session_id.unwrap_or_default();
    spell::manifest::download_manifest(
        crate::lore::console_client_config::default(),
        &session_id,
        &app_key.name,
        &app_key.version,
    )
    .await
}
