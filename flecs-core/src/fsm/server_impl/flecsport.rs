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
use axum_extra::extract::{CookieJar, Multipart};
use flecsd_axum_server::apis::flecsport::{
    ExportsExportIdDeleteResponse, ExportsExportIdGetResponse, ExportsGetResponse,
    ExportsPostResponse, Flecsport, ImportsPostResponse,
};
use flecsd_axum_server::models::{
    ExportRequest, ExportsExportIdDeletePathParams, ExportsExportIdGetPathParams,
    ImportsPostHeaderParams,
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
    F: Floxy + 'static,
    T: UsbDeviceReader + 'static,
    NET: NetworkAdapterReader,
    NetDev: NetDeviceReader,
> Flecsport for ServerImpl<APP, AUTH, I, L, Q, M, SYS, D, E, IMP, F, T, NET, NetDev>
{
    async fn exports_export_id_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ExportsExportIdDeletePathParams,
    ) -> Result<ExportsExportIdDeleteResponse, ()> {
        Ok(super::api::v2::exports::export_id::delete(
            self.sorcerers.exportius.clone(),
            self.lore.clone(),
            path_params,
        )
        .await)
    }

    async fn exports_export_id_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ExportsExportIdGetPathParams,
    ) -> Result<ExportsExportIdGetResponse, ()> {
        Ok(super::api::v2::exports::export_id::get(
            self.sorcerers.exportius.clone(),
            self.lore.clone(),
            path_params,
        )
        .await)
    }

    async fn exports_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ExportsGetResponse, ()> {
        Ok(super::api::v2::exports::get(self.sorcerers.exportius.clone(), self.lore.clone()).await)
    }

    async fn exports_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: ExportRequest,
    ) -> Result<ExportsPostResponse, ()> {
        Ok(super::api::v2::exports::post(
            self.vault.clone(),
            self.sorcerers.exportius.clone(),
            self.enchantments.floxy.clone(),
            self.enchantments.quest_master.clone(),
            self.lore.clone(),
            body,
        )
        .await)
    }

    async fn imports_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _header_params: ImportsPostHeaderParams,
        body: Multipart,
    ) -> Result<ImportsPostResponse, ()> {
        Ok(super::api::v2::imports::post(
            self.vault.clone(),
            self.lore.clone(),
            self.sorcerers.importius.clone(),
            self.enchantments.floxy.clone(),
            self.usb_reader.clone(),
            self.enchantments.quest_master.clone(),
            body,
        )
        .await)
    }
}
