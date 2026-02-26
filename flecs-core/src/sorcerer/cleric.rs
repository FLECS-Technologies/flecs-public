pub mod cleric_impl;

use super::Sorcerer;
pub use super::{Error, Result};
use crate::lore::MargoLoreRef;
use crate::quest::SyncQuest;
use crate::sorcerer::appraiser::ManifestSource;
use crate::vault::Vault;
use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Cleric: Sorcerer {
    async fn onboarding(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        lore: MargoLoreRef,
    ) -> Result<HashMap<String, Vec<ManifestSource>>>;
}

#[cfg(test)]
impl Sorcerer for MockCleric {}
