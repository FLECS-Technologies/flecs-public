use crate::enchantment::floxy::Floxy;
use crate::fsm::server_impl::ServerImpl;
use crate::relic::device::net::NetDeviceReader;
use crate::relic::device::usb::UsbDeviceReader;
use crate::relic::network::NetworkAdapterReader;
use crate::sorcerer::appraiser::AppRaiser;
use crate::sorcerer::authmancer::Authmancer;
use crate::sorcerer::deploymento::Deploymento;
use crate::sorcerer::instancius::Instancius;
use crate::sorcerer::licenso::Licenso;
use crate::sorcerer::mage_quester::MageQuester;
use crate::sorcerer::manifesto::Manifesto;
use crate::sorcerer::systemus::Systemus;
use async_trait::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecsd_axum_server::apis::flecsport::{
    ExportsExportIdDeleteResponse, ExportsExportIdGetResponse, ExportsGetResponse,
    ExportsPostResponse, Flecsport,
};
use flecsd_axum_server::models::{
    ExportRequest, ExportsExportIdDeletePathParams, ExportsExportIdGetPathParams,
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
        F: Floxy,
        T: UsbDeviceReader,
        NET: NetworkAdapterReader,
        NetDev: NetDeviceReader,
    > Flecsport for ServerImpl<APP, AUTH, I, L, Q, M, SYS, D, F, T, NET, NetDev>
{
    async fn exports_export_id_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: ExportsExportIdDeletePathParams,
    ) -> Result<ExportsExportIdDeleteResponse, ()> {
        todo!()
    }

    async fn exports_export_id_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: ExportsExportIdGetPathParams,
    ) -> Result<ExportsExportIdGetResponse, ()> {
        todo!()
    }

    async fn exports_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ExportsGetResponse, ()> {
        todo!()
    }

    async fn exports_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _body: ExportRequest,
    ) -> Result<ExportsPostResponse, ()> {
        todo!()
    }
}
