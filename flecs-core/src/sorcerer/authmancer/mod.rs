mod authmancer_impl;
use super::Sorcerer;
pub use super::{Error, Result};
use crate::fsm::console_client::ConsoleClient;
use crate::jeweler::app::Token;
use crate::vault::Vault;
use async_trait::async_trait;
pub use authmancer_impl::AuthmancerImpl;
use flecsd_axum_server::models::AuthResponseData;
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Authmancer: Sorcerer {
    async fn delete_authentication(&self, vault: &Vault);
    async fn store_authentication(&self, auth: AuthResponseData, vault: &Vault);
    async fn acquire_download_token(
        &self,
        configuration: ConsoleClient,
        vault: &Vault,
        app: &str,
        version: &str,
    ) -> Result<Option<Token>>;
}

#[cfg(test)]
impl Sorcerer for MockAuthmancer {}
