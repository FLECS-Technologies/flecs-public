use crate::fsm::server_impl::ServerImpl;
use crate::relic::device::usb::UsbDeviceReader;
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
use flecsd_axum_server::apis::manifests::{
    Manifests, ManifestsAppNameVersionGetResponse, ManifestsGetResponse,
};
use flecsd_axum_server::models::ManifestsAppNameVersionGetPathParams;
use http::Method;
use net_spider::net_device::NetDeviceReader;
use net_spider::network_adapter::NetworkAdapterReader;

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
    T: UsbDeviceReader,
    NET: NetworkAdapterReader,
    NetDev: NetDeviceReader,
> Manifests for ServerImpl<APP, AUTH, I, L, Q, M, SYS, D, E, IMP, T, NET, NetDev>
{
    async fn manifests_app_name_version_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ManifestsAppNameVersionGetPathParams,
    ) -> Result<ManifestsAppNameVersionGetResponse, ()> {
        Ok(super::api::v2::manifests::app_name::version::get(
            self.vault.clone(),
            self.sorcerers.manifesto.clone(),
            path_params,
        )
        .await)
    }

    async fn manifests_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ManifestsGetResponse, ()> {
        Ok(
            super::api::v2::manifests::get(self.vault.clone(), self.sorcerers.manifesto.clone())
                .await,
        )
    }
}
