use crate::sorcerer::manifesto::Manifesto;
use crate::sorcerer::{spell, Sorcerer};
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::Vault;
use anyhow::Error;
use async_trait::async_trait;
use flecs_app_manifest::AppManifestVersion;
use flecs_console_client::apis::configuration::Configuration;
use std::sync::Arc;

#[derive(Default)]
pub struct ManifestoImpl {}

impl Sorcerer for ManifestoImpl {}

#[async_trait]
impl Manifesto for ManifestoImpl {
    async fn download_manifest(
        &self,
        vault: &Vault,
        app_key: AppKey,
        config: Arc<Configuration>,
    ) -> Result<AppManifestVersion, Error> {
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
        let session_id = session_id.unwrap_or_default();
        spell::manifest::download_manifest(config, &session_id, &app_key.name, &app_key.version)
            .await
    }
}
