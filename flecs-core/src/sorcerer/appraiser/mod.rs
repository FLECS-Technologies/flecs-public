mod appraiser_impl;

pub use super::Result;
use crate::fsm::console_client::ConsoleClient;
use crate::jeweler::gem::manifest::AppManifest;
use crate::quest::SyncQuest;
use crate::relic::floxy::Floxy;
use crate::sorcerer::Sorcerer;
use crate::sorcerer::cleric::Client;
use crate::vault::Vault;
use crate::vault::pouch::AppKey;
pub use appraiser_impl::AppraiserImpl;
use async_trait::async_trait;
use flecsd_axum_server::models::InstalledApp;
#[cfg(test)]
use mockall::automock;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum ManifestSource {
    AppKey(AppKey),
    Url(url::Url),
}

impl Display for ManifestSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AppKey(key) => key.fmt(f),
            Self::Url(url) => url.fmt(f),
        }
    }
}

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
        sources: Vec<ManifestSource>,
        config: ConsoleClient,
    ) -> Result<()>;

    async fn install_app(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        source: ManifestSource,
        config: ConsoleClient,
    ) -> Result<()>;

    async fn install_application_deployments(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        source: HashMap<String, HashMap<String, ManifestSource>>,
        config: ConsoleClient,
        margo_client: Client,
    ) -> Result<()>;

    async fn install_application_deployment(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        id: String,
        sources: HashMap<String, ManifestSource>,
        config: ConsoleClient,
        margo_client: Client,
    ) -> Result<()>;
}

#[cfg(test)]
impl Sorcerer for MockAppRaiser {}
