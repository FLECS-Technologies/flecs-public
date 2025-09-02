use crate::fsm::console_client::ConsoleClient;
use crate::sorcerer::manifesto::Manifesto;
use crate::sorcerer::{Sorcerer, spell};
use crate::vault::Vault;
use crate::vault::pouch::{AppKey, Pouch};
use anyhow::Error;
use async_trait::async_trait;
use flecs_app_manifest::AppManifestVersion;
use flecs_app_manifest::generated::manifest_3_2_0::FlecsAppManifest;

#[derive(Default)]
pub struct ManifestoImpl {}

impl Sorcerer for ManifestoImpl {}

#[async_trait]
impl Manifesto for ManifestoImpl {
    async fn download_manifest(
        &self,
        vault: &Vault,
        app_key: AppKey,
        config: ConsoleClient,
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

    async fn get_manifests(&self, vault: &Vault) -> Vec<FlecsAppManifest> {
        vault
            .reservation()
            .reserve_manifest_pouch()
            .grab()
            .await
            .manifest_pouch
            .as_ref()
            .expect("Vault reservations should never fail")
            .gems()
            .values()
            .cloned()
            .map(FlecsAppManifest::from)
            .collect()
    }

    async fn get_manifest(&self, vault: &Vault, app_key: &AppKey) -> Option<FlecsAppManifest> {
        vault
            .reservation()
            .reserve_manifest_pouch()
            .grab()
            .await
            .manifest_pouch
            .as_ref()
            .expect("Vault reservations should never fail")
            .gems()
            .get(app_key)
            .cloned()
            .map(FlecsAppManifest::from)
    }
}
