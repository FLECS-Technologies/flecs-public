mod licenso_impl;
pub use super::Result;
use super::Sorcerer;
use crate::vault::Vault;
use async_trait::async_trait;
use flecs_console_client::apis::configuration::Configuration;
pub use licenso_impl::LicensoImpl;
#[cfg(test)]
use mockall::automock;
use std::sync::Arc;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Licenso: Sorcerer {
    async fn activate_license(
        &self,
        vault: &Vault,
        configuration: Arc<Configuration>,
    ) -> Result<()>;

    async fn validate_license(
        &self,
        vault: &Vault,
        configuration: Arc<Configuration>,
    ) -> Result<bool>;
}

#[cfg(test)]
impl Sorcerer for MockLicenso {}
