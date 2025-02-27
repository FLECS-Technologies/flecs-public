mod manifesto_impl;
use super::spell::Error;
use super::Sorcerer;
use crate::vault::pouch::AppKey;
use crate::vault::Vault;
use async_trait::async_trait;
use flecs_app_manifest::AppManifestVersion;
use flecs_console_client::apis::configuration::Configuration;
pub use manifesto_impl::ManifestoImpl;
#[cfg(test)]
use mockall::automock;
use std::sync::Arc;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Manifesto: Sorcerer {
    async fn download_manifest(
        &self,
        vault: &Vault,
        app_key: AppKey,
        config: Arc<Configuration>,
    ) -> Result<AppManifestVersion, Error>;
}

#[cfg(test)]
impl Sorcerer for MockManifesto {}
