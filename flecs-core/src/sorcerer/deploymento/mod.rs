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
    ) -> Result<GetDeploymentNetworkResult>;
}

#[cfg(test)]
impl Sorcerer for MockDeploymento {}

#[derive(Debug, PartialEq)]
pub enum GetDeploymentNetworkResult {
    DeploymentNotFound,
    NetworkNotFound,
    Network(Box<Network>),
}
