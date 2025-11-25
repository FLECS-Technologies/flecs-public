mod appraiser_impl;
pub use super::Result;
use crate::enchantment::floxy::Floxy;
use crate::fsm::console_client::ConsoleClient;
use crate::jeweler::gem::manifest::AppManifest;
use crate::quest::SyncQuest;
use crate::sorcerer::Sorcerer;
use crate::vault::Vault;
use crate::vault::pouch::AppKey;
pub use appraiser_impl::AppraiserImpl;
use async_trait::async_trait;
use flecsd_axum_server::models::InstalledApp;
#[cfg(test)]
use mockall::automock;
use std::sync::Arc;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait AppRaiser: Sorcerer {
    async fn uninstall_app(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<dyn Floxy>,
        app_key: AppKey,
    ) -> Result<()>;

    async fn does_app_exist(&self, vault: Arc<Vault>, app_key: AppKey) -> bool;

    async fn get_app(
        &self,
        vault: Arc<Vault>,
        name: String,
        version: Option<String>,
    ) -> Result<Vec<InstalledApp>>;

    async fn get_apps(&self, vault: Arc<Vault>) -> Result<Vec<InstalledApp>>;

    async fn install_app_from_manifest(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        manifest: AppManifest,
        config: ConsoleClient,
    ) -> Result<()>;

    async fn install_apps(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        app_keys: Vec<AppKey>,
        config: ConsoleClient,
    ) -> Result<()>;

    async fn install_app(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        app_key: AppKey,
        config: ConsoleClient,
    ) -> Result<()>;
}

#[cfg(test)]
impl Sorcerer for MockAppRaiser {}
