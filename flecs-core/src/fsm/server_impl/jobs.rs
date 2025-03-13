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
        Ok(JobsGetResponse::Status200_Success(
            self.sorcerers.mage_quester.get_jobs().await,
        ))
    }

    async fn jobs_job_id_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: JobsJobIdDeletePathParams,
    ) -> Result<JobsJobIdDeleteResponse, ()> {
        match self
            .sorcerers
            .mage_quester
            .delete_job(path_params.job_id as u64)
            .await
        {
            Ok(_) => Ok(JobsJobIdDeleteResponse::Status200_Success),
            Err(crate::quest::quest_master::DeleteQuestError::StillRunning) => {
                Ok(JobsJobIdDeleteResponse::Status400_JobNotFinished(format!(
                    "Not removing unfinished job {}",
                    path_params.job_id
                )))
            }
            Err(crate::quest::quest_master::DeleteQuestError::Unknown) => {
                Ok(JobsJobIdDeleteResponse::Status404_NotFound)
            }
        }
    }

    async fn jobs_job_id_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: JobsJobIdGetPathParams,
    ) -> Result<JobsJobIdGetResponse, ()> {
        match self
            .sorcerers
            .mage_quester
            .get_job(path_params.job_id as u64)
            .await
        {
            Some(job) => Ok(JobsJobIdGetResponse::Status200_Success(job)),
            None => Ok(JobsJobIdGetResponse::Status404_NotFound),
        }
    }
}
