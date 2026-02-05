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
use flecsd_axum_server::apis::apps::{
    Apps, AppsAppDeleteResponse, AppsAppGetResponse, AppsGetResponse, AppsInstallPostResponse,
    AppsSideloadPostResponse,
};
use flecsd_axum_server::models::{
    AppsAppDeletePathParams, AppsAppDeleteQueryParams, AppsAppGetPathParams, AppsAppGetQueryParams,
    AppsInstallPostRequest, AppsSideloadPostRequest,
};
use http::Method;
use net_spider::net_device::NetDeviceReader;
use net_spider::network_adapter::NetworkAdapterReader;

#[async_trait]
impl<
    APP: AppRaiser + 'static,
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
> Apps for ServerImpl<APP, AUTH, I, L, Q, M, SYS, D, E, IMP, T, NET, NetDev>
{
    async fn apps_app_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: AppsAppDeletePathParams,
        query_params: AppsAppDeleteQueryParams,
    ) -> Result<AppsAppDeleteResponse, ()> {
        super::api::v2::apps::app::delete(
            self.vault.clone(),
            self.relics.floxy.clone(),
            self.sorcerers.app_raiser.clone(),
            self.enchantments.quest_master.clone(),
            path_params,
            query_params,
        )
        .await
    }

    async fn apps_app_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: AppsAppGetPathParams,
        query_params: AppsAppGetQueryParams,
    ) -> Result<AppsAppGetResponse, ()> {
        Ok(super::api::v2::apps::app::get(
            self.vault.clone(),
            self.sorcerers.app_raiser.clone(),
            path_params,
            query_params,
        )
        .await)
    }

    async fn apps_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<AppsGetResponse, ()> {
        Ok(super::api::v2::apps::get(self.vault.clone(), self.sorcerers.app_raiser.clone()).await)
    }

    async fn apps_install_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: AppsInstallPostRequest,
    ) -> Result<AppsInstallPostResponse, ()> {
        Ok(super::api::v2::apps::install::post(
            self.vault.clone(),
            self.sorcerers.app_raiser.clone(),
            self.console_client.clone(),
            self.enchantments.quest_master.clone(),
            body,
        )
        .await)
    }

    async fn apps_sideload_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: AppsSideloadPostRequest,
    ) -> Result<AppsSideloadPostResponse, ()> {
        super::api::v2::apps::sideload::post(
            self.vault.clone(),
            self.sorcerers.app_raiser.clone(),
            self.enchantments.quest_master.clone(),
            self.console_client.clone(),
            body,
        )
        .await
    }
}
