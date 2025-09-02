mod manifesto_impl;
use super::Sorcerer;
use super::spell::Error;
use crate::fsm::console_client::ConsoleClient;
use crate::vault::Vault;
use crate::vault::pouch::AppKey;
use async_trait::async_trait;
use flecs_app_manifest::AppManifestVersion;
use flecs_app_manifest::generated::manifest_3_2_0::FlecsAppManifest;
pub use manifesto_impl::ManifestoImpl;
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Manifesto: Sorcerer {
    async fn download_manifest(
        &self,
        vault: &Vault,
        app_key: AppKey,
        config: ConsoleClient,
    ) -> Result<AppManifestVersion, Error>;
    async fn get_manifests(&self, vault: &Vault) -> Vec<FlecsAppManifest>;
    async fn get_manifest(&self, vault: &Vault, app_key: &AppKey) -> Option<FlecsAppManifest>;
}

#[cfg(test)]
impl Sorcerer for MockManifesto {}
