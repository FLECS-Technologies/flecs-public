pub mod cleric_impl;

use super::Sorcerer;
pub use super::{Error, Result};
use crate::lore::MargoLoreRef;
use crate::quest::SyncQuest;
use crate::sorcerer::appraiser::ManifestSource;
use crate::vault::Vault;
use async_trait::async_trait;
use margo_workload_management_api_client_rs::apis::configuration::Configuration;
#[cfg(test)]
use mockall::automock;
use std::collections::HashMap;
use std::sync::Arc;

pub type ClientId = String;

#[derive(Debug, Clone)]
pub struct Client {
    pub id: ClientId,
    pub config: Configuration,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Cleric: Sorcerer {
    async fn onboarding(&self, quest: SyncQuest, lore: MargoLoreRef) -> Result<Client>;

    async fn receive_bundle(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        client: Client,
    ) -> Result<HashMap<String, HashMap<String, ManifestSource>>>;
}

#[cfg(test)]
impl Sorcerer for MockCleric {}
