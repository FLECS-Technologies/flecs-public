use crate::enchantment::floxy::Floxy;
use crate::fsm::server_impl::ServerImpl;
use crate::relic::device::usb::UsbDeviceReader;
use crate::sorcerer::appraiser::AppRaiser;
use crate::sorcerer::authmancer::Authmancer;
use async_trait::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecsd_axum_server::apis::jobs::{
    Jobs, JobsGetResponse, JobsJobIdDeleteResponse, JobsJobIdGetResponse,
};
use flecsd_axum_server::models::{JobsJobIdDeletePathParams, JobsJobIdGetPathParams};
use http::Method;

#[async_trait]
impl<APP: AppRaiser, AUTH: Authmancer, F: Floxy, T: UsbDeviceReader> Jobs
    for ServerImpl<APP, AUTH, F, T>
{
    async fn jobs_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<JobsGetResponse, ()> {
        Ok(JobsGetResponse::Status200_Success(
            crate::sorcerer::magequester::get_jobs().await,
        ))
    }

    async fn jobs_job_id_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: JobsJobIdDeletePathParams,
    ) -> Result<JobsJobIdDeleteResponse, ()> {
        match crate::sorcerer::magequester::delete_job(path_params.job_id as u64).await {
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
        match crate::sorcerer::magequester::get_job(path_params.job_id as u64).await {
            Some(job) => Ok(JobsJobIdGetResponse::Status200_Success(job)),
            None => Ok(JobsJobIdGetResponse::Status404_NotFound),
        }
    }
}
