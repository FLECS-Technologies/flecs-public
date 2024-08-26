use crate::vault::Vault;
use axum::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecs_console_client::models::SessionId;
use flecsd_axum_server::apis::apps::{
    Apps, AppsAppDeleteResponse, AppsAppGetResponse, AppsGetResponse, AppsInstallPostResponse,
    AppsSideloadPostResponse,
};
use flecsd_axum_server::apis::console::{
    Console, ConsoleAuthenticationDeleteResponse, ConsoleAuthenticationPutResponse,
};
use flecsd_axum_server::apis::device::{
    Device, DeviceLicenseActivationPostResponse, DeviceLicenseActivationStatusGetResponse,
    DeviceLicenseInfoGetResponse, DeviceOnboardingPostResponse,
};
use flecsd_axum_server::apis::flunder::{Flunder, FlunderBrowseGetResponse};
use flecsd_axum_server::apis::instances::{
    Instances, InstancesCreatePostResponse, InstancesGetResponse,
    InstancesInstanceIdConfigEnvironmentDeleteResponse,
    InstancesInstanceIdConfigEnvironmentGetResponse,
    InstancesInstanceIdConfigEnvironmentPutResponse, InstancesInstanceIdConfigGetResponse,
    InstancesInstanceIdConfigPortsDeleteResponse, InstancesInstanceIdConfigPortsGetResponse,
    InstancesInstanceIdConfigPortsPutResponse, InstancesInstanceIdConfigPostResponse,
    InstancesInstanceIdDeleteResponse, InstancesInstanceIdEditorPortGetResponse,
    InstancesInstanceIdGetResponse, InstancesInstanceIdLogsGetResponse,
    InstancesInstanceIdPatchResponse, InstancesInstanceIdStartPostResponse,
    InstancesInstanceIdStopPostResponse,
};
use flecsd_axum_server::apis::jobs::{
    Jobs, JobsGetResponse, JobsJobIdDeleteResponse, JobsJobIdGetResponse,
};
use flecsd_axum_server::apis::system::{
    System, SystemInfoGetResponse, SystemPingGetResponse, SystemVersionGetResponse,
};
use flecsd_axum_server::models::{
    AdditionalInfo, AppEditor, AppKey, AppStatus, AppsAppDeletePathParams,
    AppsAppDeleteQueryParams, AppsAppGetPathParams, AppsAppGetQueryParams, AppsInstallPostRequest,
    AppsSideloadPostRequest, AuthResponseData, DeviceLicenseActivationStatusGet200Response,
    DeviceLicenseInfoGet200Response, Dosschema, FlunderBrowseGetQueryParams, InstalledApp,
    InstanceConfig, InstanceEnvironment, InstancePorts, InstancesCreatePostRequest,
    InstancesGetQueryParams, InstancesInstanceIdConfigEnvironmentDeletePathParams,
    InstancesInstanceIdConfigEnvironmentGetPathParams,
    InstancesInstanceIdConfigEnvironmentPutPathParams, InstancesInstanceIdConfigGetPathParams,
    InstancesInstanceIdConfigPortsDeletePathParams, InstancesInstanceIdConfigPortsGetPathParams,
    InstancesInstanceIdConfigPortsPutPathParams, InstancesInstanceIdConfigPostPathParams,
    InstancesInstanceIdDeletePathParams, InstancesInstanceIdEditorPortGetPathParams,
    InstancesInstanceIdGetPathParams, InstancesInstanceIdLogsGetPathParams,
    InstancesInstanceIdPatchPathParams, InstancesInstanceIdPatchRequest,
    InstancesInstanceIdStartPostPathParams, InstancesInstanceIdStopPostPathParams,
    JobsJobIdDeletePathParams, JobsJobIdGetPathParams,
};
use http::Method;
use std::sync::Arc;

pub struct ServerImpl {
    vault: Arc<Vault>,
}

impl Default for ServerImpl {
    fn default() -> Self {
        Self {
            vault: crate::lore::vault::default(),
        }
    }
}

#[async_trait]
impl Apps for ServerImpl {
    async fn apps_app_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: AppsAppDeletePathParams,
        _query_params: AppsAppDeleteQueryParams,
    ) -> Result<AppsAppDeleteResponse, String> {
        todo!()
    }

    async fn apps_app_get(
        &self,
        _method: Method,
        host: Host,
        _cookies: CookieJar,
        path_params: AppsAppGetPathParams,
        query_params: AppsAppGetQueryParams,
    ) -> Result<AppsAppGetResponse, String> {
        println!(
            "Received app request from {} for app {} in version {}",
            host.0,
            path_params.app,
            query_params.version.unwrap_or("unknown".to_string())
        );
        Ok(AppsAppGetResponse::Status200_Success(vec![InstalledApp {
            app_key: AppKey {
                name: "some app".into(),
                version: "1.0.2".into(),
            },
            status: AppStatus::Installed,
            desired: AppStatus::Installed,
            editors: vec![AppEditor {
                name: "Some".to_string(),
                port: 123,
                supports_reverse_proxy: Some(true),
            }],
            installed_size: 1234,
            multi_instance: false,
        }]))
    }

    async fn apps_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<AppsGetResponse, String> {
        todo!()
    }

    async fn apps_install_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _body: AppsInstallPostRequest,
    ) -> Result<AppsInstallPostResponse, String> {
        todo!()
    }

    async fn apps_sideload_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _body: AppsSideloadPostRequest,
    ) -> Result<AppsSideloadPostResponse, String> {
        todo!()
    }
}

#[async_trait]
impl Console for ServerImpl {
    async fn console_authentication_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ConsoleAuthenticationDeleteResponse, String> {
        crate::sorcerer::authmancer::delete_authentication(&self.vault);
        Ok(ConsoleAuthenticationDeleteResponse::Status204_NoContent)
    }

    async fn console_authentication_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: AuthResponseData,
    ) -> Result<ConsoleAuthenticationPutResponse, String> {
        crate::sorcerer::authmancer::store_authentication(body, &self.vault);
        Ok(ConsoleAuthenticationPutResponse::Status204_NoContent)
    }
}

#[async_trait]
impl Device for ServerImpl {
    async fn device_license_activation_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<DeviceLicenseActivationPostResponse, String> {
        match crate::sorcerer::licenso::activate_license(
            &self.vault,
            crate::lore::console_client_config::default(),
        )
        .await
        {
            Ok(()) => Ok(DeviceLicenseActivationPostResponse::Status200_Success(
                AdditionalInfo {
                    additional_info: "OK".to_string(),
                },
            )),
            Err(additional_info) => Ok(
                DeviceLicenseActivationPostResponse::Status500_InternalServerError(
                    AdditionalInfo { additional_info },
                ),
            ),
        }
    }

    async fn device_license_activation_status_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<DeviceLicenseActivationStatusGetResponse, String> {
        match crate::sorcerer::licenso::validate_license(&self.vault).await {
            Ok(is_valid) => Ok(DeviceLicenseActivationStatusGetResponse::Status200_Success(
                DeviceLicenseActivationStatusGet200Response { is_valid },
            )),
            Err(additional_info) => Ok(
                DeviceLicenseActivationStatusGetResponse::Status500_InternalServerError({
                    AdditionalInfo { additional_info }
                }),
            ),
        }
    }

    async fn device_license_info_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<DeviceLicenseInfoGetResponse, String> {
        let secrets = self.vault.get_secrets();
        Ok(DeviceLicenseInfoGetResponse::Status200_Success(
            DeviceLicenseInfoGet200Response {
                // TODO: Use correct type, as soon as serial numbers are implemented
                r#type: "Via user license".to_string(),
                session_id: Some(console_session_id_to_core_session_id(
                    secrets.get_session_id(),
                )),
                value: secrets.license_key,
            },
        ))
    }

    async fn device_onboarding_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _body: Dosschema,
    ) -> Result<DeviceOnboardingPostResponse, String> {
        todo!()
    }
}

#[async_trait]
impl Flunder for ServerImpl {
    async fn flunder_browse_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _query_params: FlunderBrowseGetQueryParams,
    ) -> Result<FlunderBrowseGetResponse, String> {
        todo!()
    }
}

#[async_trait]
impl Instances for ServerImpl {
    async fn instances_create_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _body: InstancesCreatePostRequest,
    ) -> Result<InstancesCreatePostResponse, String> {
        todo!()
    }

    async fn instances_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _query_params: InstancesGetQueryParams,
    ) -> Result<InstancesGetResponse, String> {
        todo!()
    }

    async fn instances_instance_id_config_environment_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigEnvironmentDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigEnvironmentDeleteResponse, String> {
        todo!()
    }

    async fn instances_instance_id_config_environment_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigEnvironmentGetPathParams,
    ) -> Result<InstancesInstanceIdConfigEnvironmentGetResponse, String> {
        todo!()
    }

    async fn instances_instance_id_config_environment_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigEnvironmentPutPathParams,
        _body: InstanceEnvironment,
    ) -> Result<InstancesInstanceIdConfigEnvironmentPutResponse, String> {
        todo!()
    }

    async fn instances_instance_id_config_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigGetPathParams,
    ) -> Result<InstancesInstanceIdConfigGetResponse, String> {
        todo!()
    }

    async fn instances_instance_id_config_ports_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigPortsDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigPortsDeleteResponse, String> {
        todo!()
    }

    async fn instances_instance_id_config_ports_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigPortsGetPathParams,
    ) -> Result<InstancesInstanceIdConfigPortsGetResponse, String> {
        todo!()
    }

    async fn instances_instance_id_config_ports_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigPortsPutPathParams,
        _body: InstancePorts,
    ) -> Result<InstancesInstanceIdConfigPortsPutResponse, String> {
        todo!()
    }

    async fn instances_instance_id_config_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigPostPathParams,
        _body: InstanceConfig,
    ) -> Result<InstancesInstanceIdConfigPostResponse, String> {
        todo!()
    }

    async fn instances_instance_id_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdDeletePathParams,
    ) -> Result<InstancesInstanceIdDeleteResponse, String> {
        todo!()
    }

    async fn instances_instance_id_editor_port_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdEditorPortGetPathParams,
    ) -> Result<InstancesInstanceIdEditorPortGetResponse, String> {
        todo!()
    }

    async fn instances_instance_id_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdGetPathParams,
    ) -> Result<InstancesInstanceIdGetResponse, String> {
        todo!()
    }

    async fn instances_instance_id_logs_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdLogsGetPathParams,
    ) -> Result<InstancesInstanceIdLogsGetResponse, String> {
        todo!()
    }

    async fn instances_instance_id_patch(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdPatchPathParams,
        _body: InstancesInstanceIdPatchRequest,
    ) -> Result<InstancesInstanceIdPatchResponse, String> {
        todo!()
    }

    async fn instances_instance_id_start_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdStartPostPathParams,
    ) -> Result<InstancesInstanceIdStartPostResponse, String> {
        todo!()
    }

    async fn instances_instance_id_stop_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdStopPostPathParams,
    ) -> Result<InstancesInstanceIdStopPostResponse, String> {
        todo!()
    }
}

#[async_trait]
impl Jobs for ServerImpl {
    async fn jobs_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<JobsGetResponse, String> {
        todo!()
    }

    async fn jobs_job_id_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: JobsJobIdDeletePathParams,
    ) -> Result<JobsJobIdDeleteResponse, String> {
        todo!()
    }

    async fn jobs_job_id_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: JobsJobIdGetPathParams,
    ) -> Result<JobsJobIdGetResponse, String> {
        todo!()
    }
}

#[async_trait]
impl System for ServerImpl {
    async fn system_info_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemInfoGetResponse, String> {
        todo!()
    }

    async fn system_ping_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemPingGetResponse, String> {
        Ok(SystemPingGetResponse::Status200_Success(
            AdditionalInfo::new(String::from("Ok")),
        ))
    }

    async fn system_version_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemVersionGetResponse, String> {
        todo!()
    }
}

fn console_session_id_to_core_session_id(
    session_id: SessionId,
) -> flecsd_axum_server::models::SessionId {
    flecsd_axum_server::models::SessionId {
        id: session_id.id,
        timestamp: session_id.timestamp,
    }
}
