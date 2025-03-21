mod deploymento_impl;

pub use super::Result;
use crate::jeweler::deployment::DeploymentId;
use crate::jeweler::network::{Network, NetworkConfig, NetworkId};
use crate::sorcerer::Sorcerer;
use crate::vault::Vault;
use async_trait::async_trait;
pub use deploymento_impl::DeploymentoImpl;
#[cfg(test)]
use mockall::automock;
use std::net::Ipv4Addr;
use std::sync::Arc;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Deploymento: Sorcerer {
    async fn create_network(
        &self,
        vault: Arc<Vault>,
        deployment_id: DeploymentId,
        config: NetworkConfig,
    ) -> Result<Network, CreateNetworkError>;

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

    async fn reserve_ipv4_address(
        &self,
        vault: Arc<Vault>,
        deployment_id: DeploymentId,
        network_id: NetworkId,
    ) -> Result<Ipv4Addr, ReserveIpv4AddressError>;
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

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum CreateNetworkError {
    #[error("Deployment not found: {0}")]
    DeploymentNotFound(DeploymentId),
    #[error(transparent)]
    Deployment(#[from] crate::jeweler::network::CreateNetworkError),
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum ReserveIpv4AddressError {
    #[error("Deployment not found: {0}")]
    DeploymentNotFound(DeploymentId),
    #[error("Network not found: {0}")]
    NetworkNotFound(NetworkId),
    #[error("No ip address free")]
    NoFreeIpAddress,
    #[error("Failed to reserve ip address in {network_id}: {reason}")]
    Other {
        network_id: NetworkId,
        reason: String,
    },
}

impl From<GetDeploymentNetworkError> for ReserveIpv4AddressError {
    fn from(value: GetDeploymentNetworkError) -> Self {
        match value {
            GetDeploymentNetworkError::DeploymentNotFound(id) => Self::DeploymentNotFound(id),
            GetDeploymentNetworkError::NetworkNotFound(id) => Self::NetworkNotFound(id),
            GetDeploymentNetworkError::Other { network_id, reason } => {
                Self::Other { network_id, reason }
            }
        }
    }
}
