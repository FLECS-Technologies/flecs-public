mod manifesto_impl;
use super::Sorcerer;
use super::spell::Error;
use crate::fsm::console_client::ConsoleClient;
use crate::vault::Vault;
use crate::vault::pouch::AppKey;
use async_trait::async_trait;
use flecs_app_manifest::AppManifestVersion;
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
}

#[cfg(test)]
impl Sorcerer for MockManifesto {}
