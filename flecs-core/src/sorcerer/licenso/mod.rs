mod licenso_impl;
pub use super::Result;
use super::Sorcerer;
use crate::fsm::console_client::ConsoleClient;
use crate::vault::Vault;
use async_trait::async_trait;
pub use licenso_impl::LicensoImpl;
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Licenso: Sorcerer {
    async fn activate_license(&self, vault: &Vault, configuration: ConsoleClient) -> Result<()>;

    async fn validate_license(&self, vault: &Vault, configuration: ConsoleClient) -> Result<bool>;
}

#[cfg(test)]
impl Sorcerer for MockLicenso {}
