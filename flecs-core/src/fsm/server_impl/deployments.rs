use crate::enchantment::floxy::Floxy;
use crate::fsm::server_impl::ServerImpl;
use crate::relic::device::net::NetDeviceReader;
use crate::relic::device::usb::UsbDeviceReader;
use crate::relic::network::NetworkAdapterReader;
use crate::sorcerer::appraiser::AppRaiser;
use crate::sorcerer::authmancer::Authmancer;
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
    DeploymentsDeploymentIdNetworksNetworkIdGetResponse,
};
use flecsd_axum_server::models::{
    DeploymentsDeploymentIdNetworksGetPathParams,
    DeploymentsDeploymentIdNetworksNetworkIdGetPathParams,
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
        F: Floxy,
        T: UsbDeviceReader,
        NET: NetworkAdapterReader,
        NetDev: NetDeviceReader,
    > Deployments for ServerImpl<APP, AUTH, I, L, Q, M, SYS, F, T, NET, NetDev>
{
    async fn deployments_deployment_id_networks_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: DeploymentsDeploymentIdNetworksGetPathParams,
    ) -> Result<DeploymentsDeploymentIdNetworksGetResponse, ()> {
        todo!()
    }

    async fn deployments_deployment_id_networks_network_id_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: DeploymentsDeploymentIdNetworksNetworkIdGetPathParams,
    ) -> Result<DeploymentsDeploymentIdNetworksNetworkIdGetResponse, ()> {
        todo!()
    }
}
