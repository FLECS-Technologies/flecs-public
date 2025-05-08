use crate::enchantment::floxy::Floxy;
use crate::fsm::server_impl::ServerImpl;
use crate::relic::device::net::NetDeviceReader;
use crate::relic::device::usb::UsbDeviceReader;
use crate::relic::network::NetworkAdapterReader;
use crate::sorcerer::appraiser::AppRaiser;
use crate::sorcerer::authmancer::Authmancer;
use crate::sorcerer::deploymento::Deploymento;
use crate::sorcerer::exportius::Exportius;
use crate::sorcerer::importius::Importius;
use crate::sorcerer::instancius::Instancius;
use crate::sorcerer::licenso::Licenso;
use crate::sorcerer::mage_quester::MageQuester;
use crate::sorcerer::manifesto::Manifesto;
use crate::sorcerer::systemus::Systemus;
use async_trait::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecsd_axum_server::apis::deployments::{
    Deployments, DeploymentsDeploymentIdNetworksGetResponse,
    DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4PostResponse,
    DeploymentsDeploymentIdNetworksNetworkIdGetResponse,
    DeploymentsDeploymentIdNetworksPostResponse,
};
use flecsd_axum_server::models::{
    DeploymentsDeploymentIdNetworksGetPathParams,
    DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4PostPathParams,
    DeploymentsDeploymentIdNetworksNetworkIdGetPathParams,
    DeploymentsDeploymentIdNetworksPostPathParams, PostDeploymentNetwork,
};
use http::Method;

#[async_trait]
impl<
    APP: AppRaiser,
    AUTH: Authmancer,
    I: Instancius,
    L: Licenso,
    Q: MageQuester,
    M: Manifesto,
    SYS: Systemus,
    D: Deploymento,
    E: Exportius,
    IMP: Importius,
    F: Floxy,
    T: UsbDeviceReader,
    NET: NetworkAdapterReader,
    NetDev: NetDeviceReader,
> Deployments for ServerImpl<APP, AUTH, I, L, Q, M, SYS, D, E, IMP, F, T, NET, NetDev>
{
    async fn deployments_deployment_id_networks_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: DeploymentsDeploymentIdNetworksGetPathParams,
    ) -> Result<DeploymentsDeploymentIdNetworksGetResponse, ()> {
        Ok(super::api::v2::deployments::deployment_id::networks::get(
            self.vault.clone(),
            self.sorcerers.deploymento.clone(),
            path_params,
        )
        .await)
    }

    async fn deployments_deployment_id_networks_network_id_dhcp_ipv4_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4PostPathParams,
    ) -> Result<DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4PostResponse, ()> {
        Ok(
            super::api::v2::deployments::deployment_id::networks::dhcp::ipv4::post(
                self.vault.clone(),
                self.sorcerers.deploymento.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn deployments_deployment_id_networks_network_id_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: DeploymentsDeploymentIdNetworksNetworkIdGetPathParams,
    ) -> Result<DeploymentsDeploymentIdNetworksNetworkIdGetResponse, ()> {
        Ok(
            super::api::v2::deployments::deployment_id::networks::network_id::get(
                self.vault.clone(),
                self.sorcerers.deploymento.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn deployments_deployment_id_networks_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: DeploymentsDeploymentIdNetworksPostPathParams,
        body: PostDeploymentNetwork,
    ) -> Result<DeploymentsDeploymentIdNetworksPostResponse, ()> {
        Ok(super::api::v2::deployments::deployment_id::networks::post(
            self.vault.clone(),
            self.sorcerers.deploymento.clone(),
            path_params,
            body,
        )
        .await)
    }
}
