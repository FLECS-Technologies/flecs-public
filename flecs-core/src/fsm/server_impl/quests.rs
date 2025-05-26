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
use flecsd_axum_server::apis::quests::{
    Quests, QuestsGetResponse, QuestsIdDeleteResponse, QuestsIdGetResponse,
};
use flecsd_axum_server::models::{QuestsIdDeletePathParams, QuestsIdGetPathParams};
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
> Quests for ServerImpl<APP, AUTH, I, L, Q, M, SYS, D, E, IMP, F, T, NET, NetDev>
{
    async fn quests_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<QuestsGetResponse, ()> {
        Ok(super::api::v2::quests::get(
            self.sorcerers.mage_quester.clone(),
            self.enchantments.quest_master.clone(),
        )
        .await)
    }

    async fn quests_id_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: QuestsIdDeletePathParams,
    ) -> Result<QuestsIdDeleteResponse, ()> {
        Ok(super::api::v2::quests::id::delete(
            self.sorcerers.mage_quester.clone(),
            self.enchantments.quest_master.clone(),
            path_params,
        )
        .await)
    }

    async fn quests_id_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: QuestsIdGetPathParams,
    ) -> Result<QuestsIdGetResponse, ()> {
        Ok(super::api::v2::quests::id::get(
            self.sorcerers.mage_quester.clone(),
            self.enchantments.quest_master.clone(),
            path_params,
        )
        .await)
    }
}
