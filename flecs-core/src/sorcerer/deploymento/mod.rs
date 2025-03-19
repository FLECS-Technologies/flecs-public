mod deploymento_impl;
pub use super::Result;
use crate::jeweler::deployment::DeploymentId;
use crate::jeweler::network::{Network, NetworkId};
use crate::sorcerer::Sorcerer;
use crate::vault::Vault;
use async_trait::async_trait;
pub use deploymento_impl::DeploymentoImpl;
#[cfg(test)]
use mockall::automock;
use std::sync::Arc;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Deploymento: Sorcerer {
    async fn get_deployment_networks(
        &self,
        vault: Arc<Vault>,
        deployment_id: DeploymentId,
    ) -> Result<Option<Vec<Network>>>;

    async fn get_deployment_network(
        &self,
        vault: Arc<Vault>,
        deployment_id: DeploymentId,
        network_id: NetworkId,
    ) -> Result<Network, GetDeploymentNetworkError>;
}

#[cfg(test)]
impl Sorcerer for MockDeploymento {}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetDeploymentNetworkError {
    #[error("Deployment not found: {0}")]
    DeploymentNotFound(DeploymentId),
    #[error("Network not found: {0}")]
    NetworkNotFound(NetworkId),
    #[error("Failed to get network {network_id}: {reason}")]
    Other {
        network_id: NetworkId,
        reason: String,
    },
}
