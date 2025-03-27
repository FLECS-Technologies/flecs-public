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
use flecsd_axum_server::apis::jobs::{
    Jobs, JobsGetResponse, JobsJobIdDeleteResponse, JobsJobIdGetResponse,
};
use flecsd_axum_server::models::{JobsJobIdDeletePathParams, JobsJobIdGetPathParams};
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
    > Jobs for ServerImpl<APP, AUTH, I, L, Q, M, SYS, D, F, T, NET, NetDev>
{
    async fn jobs_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<JobsGetResponse, ()> {
        Ok(super::api::v2::jobs::get(
            self.sorcerers.mage_quester.clone(),
            self.enchantments.quest_master.clone(),
        )
        .await)
    }

    async fn jobs_job_id_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: JobsJobIdDeletePathParams,
    ) -> Result<JobsJobIdDeleteResponse, ()> {
        Ok(super::api::v2::jobs::job_id::delete(
            self.sorcerers.mage_quester.clone(),
            self.enchantments.quest_master.clone(),
            path_params,
        )
        .await)
    }

    async fn jobs_job_id_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: JobsJobIdGetPathParams,
    ) -> Result<JobsJobIdGetResponse, ()> {
        Ok(super::api::v2::jobs::job_id::get(
            self.sorcerers.mage_quester.clone(),
            self.enchantments.quest_master.clone(),
            path_params,
        )
        .await)
    }
}
