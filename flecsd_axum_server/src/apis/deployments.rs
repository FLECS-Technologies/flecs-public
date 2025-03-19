use async_trait::async_trait;
use axum::extract::*;
use axum_extra::extract::{CookieJar, Multipart};
use bytes::Bytes;
use http::Method;
use serde::{Deserialize, Serialize};

use crate::{models, types::*};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum DeploymentsDeploymentIdNetworksGetResponse {
    /// Success
    Status200_Success(Vec<models::DeploymentNetwork>),
    /// No deployment with this deployment_id found
    Status404_NoDeploymentWithThisDeployment,
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4PostResponse {
    /// Success
    Status200_Success(models::DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4Post200Response),
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// Resource not found
    Status404_ResourceNotFound(models::OptionalAdditionalInfo),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum DeploymentsDeploymentIdNetworksNetworkIdGetResponse {
    /// Success
    Status200_Success(models::DeploymentNetwork),
    /// Resource not found
    Status404_ResourceNotFound(models::OptionalAdditionalInfo),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

/// Deployments
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Deployments {
    /// DeploymentsDeploymentIdNetworksGet - GET /v2/deployments/{deployment_id}/networks
    async fn deployments_deployment_id_networks_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::DeploymentsDeploymentIdNetworksGetPathParams,
    ) -> Result<DeploymentsDeploymentIdNetworksGetResponse, ()>;

    /// DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4Post - POST /v2/deployments/{deployment_id}/networks/{network_id}/dhcp/ipv4
    async fn deployments_deployment_id_networks_network_id_dhcp_ipv4_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4PostPathParams,
    ) -> Result<DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4PostResponse, ()>;

    /// DeploymentsDeploymentIdNetworksNetworkIdGet - GET /v2/deployments/{deployment_id}/networks/{network_id}
    async fn deployments_deployment_id_networks_network_id_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::DeploymentsDeploymentIdNetworksNetworkIdGetPathParams,
    ) -> Result<DeploymentsDeploymentIdNetworksNetworkIdGetResponse, ()>;
}
