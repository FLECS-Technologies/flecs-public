use crate::jeweler::gem::instance::InstanceId;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::Vault;
use anyhow::Error;
use axum::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecs_app_manifest::{AppManifest, AppManifestVersion};
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
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    AdditionalInfo, AppsAppDeletePathParams, AppsAppDeleteQueryParams, AppsAppGetPathParams,
    AppsAppGetQueryParams, AppsInstallPostRequest, AppsSideloadPostRequest, AuthResponseData,
    DeviceLicenseActivationStatusGet200Response, DeviceLicenseInfoGet200Response, Dosschema,
    FlunderBrowseGetQueryParams, InstanceConfig, InstanceEnvironment, InstancePorts,
    InstancesCreatePostRequest, InstancesGetQueryParams,
    InstancesInstanceIdConfigEnvironmentDeletePathParams,
    InstancesInstanceIdConfigEnvironmentGetPathParams,
    InstancesInstanceIdConfigEnvironmentPutPathParams, InstancesInstanceIdConfigGetPathParams,
    InstancesInstanceIdConfigPortsDeletePathParams, InstancesInstanceIdConfigPortsGetPathParams,
    InstancesInstanceIdConfigPortsPutPathParams, InstancesInstanceIdConfigPostPathParams,
    InstancesInstanceIdDeletePathParams, InstancesInstanceIdEditorPortGetPathParams,
    InstancesInstanceIdGetPathParams, InstancesInstanceIdLogsGetPathParams,
    InstancesInstanceIdPatchPathParams, InstancesInstanceIdPatchRequest,
    InstancesInstanceIdStartPostPathParams, InstancesInstanceIdStopPostPathParams, JobMeta,
    JobsJobIdDeletePathParams, JobsJobIdGetPathParams,
};
use http::Method;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{error, warn};

fn additional_info_from_error(error: Error) -> AdditionalInfo {
    AdditionalInfo {
        additional_info: format!("{error:#}"),
    }
}

fn ok() -> AdditionalInfo {
    AdditionalInfo {
        additional_info: "OK".to_string(),
    }
}

pub struct ServerImpl {
    vault: Arc<Vault>,
}

impl ServerImpl {
    pub async fn default() -> Self {
        Self {
            vault: crate::lore::vault::default().await,
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
        path_params: AppsAppDeletePathParams,
        query_params: AppsAppDeleteQueryParams,
    ) -> Result<AppsAppDeleteResponse, String> {
        match query_params.version {
            Some(app_version) => {
                let key = AppKey {
                    name: path_params.app,
                    version: app_version,
                };
                if !self
                    .vault
                    .reservation()
                    .reserve_app_pouch()
                    .grab()
                    .await
                    .app_pouch
                    .as_ref()
                    .expect("Vault reservations should never fail")
                    .gems()
                    .contains_key(&key)
                {
                    return Ok(AppsAppDeleteResponse::Status404_NoSuchAppOrApp);
                }
                let (id, _) = crate::lore::quest::default()
                    .await
                    .lock()
                    .await
                    .schedule_quest(format!("Uninstall {key}"), |quest| {
                        crate::sorcerer::appraiser::uninstall_app(quest, self.vault.clone(), key)
                    })
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(AppsAppDeleteResponse::Status202_Accepted(JobMeta {
                    job_id: id.0 as i32,
                }))
            }
            None => Err("Missing query parameter 'version'".to_string()),
        }
    }

    async fn apps_app_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: AppsAppGetPathParams,
        query_params: AppsAppGetQueryParams,
    ) -> Result<AppsAppGetResponse, String> {
        let apps = crate::sorcerer::appraiser::get_app(
            self.vault.clone(),
            path_params.app,
            query_params.version,
        )
        .await
        .map_err(|e| e.to_string())?;
        if apps.is_empty() {
            Ok(AppsAppGetResponse::Status404_NoSuchAppOrApp)
        } else {
            Ok(AppsAppGetResponse::Status200_Success(apps))
        }
    }

    async fn apps_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<AppsGetResponse, String> {
        let apps = crate::sorcerer::appraiser::get_apps(self.vault.clone())
            .await
            .map_err(|e| e.to_string())?;
        Ok(AppsGetResponse::Status200_Success(apps))
    }

    async fn apps_install_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: AppsInstallPostRequest,
    ) -> Result<AppsInstallPostResponse, String> {
        let app_key = body.app_key.into();
        let config = crate::lore::console_client_config::default().await;
        match crate::lore::quest::default()
            .await
            .lock()
            .await
            .schedule_quest(format!("Install {}", app_key), |quest| {
                crate::sorcerer::appraiser::install_app(quest, self.vault.clone(), app_key, config)
            })
            .await
        {
            Ok((id, _)) => Ok(AppsInstallPostResponse::Status202_Accepted(JobMeta {
                job_id: id.0 as i32,
            })),
            Err(e) => Ok(AppsInstallPostResponse::Status500_InternalServerError(
                AdditionalInfo {
                    additional_info: e.to_string(),
                },
            )),
        }
    }

    async fn apps_sideload_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: AppsSideloadPostRequest,
    ) -> Result<AppsSideloadPostResponse, String> {
        match serde_json::from_str::<AppManifestVersion>(&body.manifest).map(AppManifest::try_from)
        {
            Err(e) => Ok(AppsSideloadPostResponse::Status400_MalformedRequest(
                AdditionalInfo {
                    additional_info: e.to_string(),
                },
            )),
            Ok(Err(e)) => Ok(AppsSideloadPostResponse::Status400_MalformedRequest(
                AdditionalInfo {
                    additional_info: e.to_string(),
                },
            )),
            Ok(Ok(manifest)) => {
                let config = crate::lore::console_client_config::default().await;
                match crate::lore::quest::default()
                    .await
                    .lock()
                    .await
                    .schedule_quest(
                        format!("Sideloading {}-{}", manifest.app.deref(), manifest.version),
                        |quest| {
                            crate::sorcerer::appraiser::install_app_from_manifest(
                                quest,
                                self.vault.clone(),
                                Arc::new(manifest),
                                config,
                            )
                        },
                    )
                    .await
                {
                    Ok((id, _)) => Ok(AppsSideloadPostResponse::Status202_Accepted(JobMeta {
                        job_id: id.0 as i32,
                    })),
                    Err(e) => Err(e.to_string()),
                }
            }
        }
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
        crate::sorcerer::authmancer::delete_authentication(&self.vault).await;
        Ok(ConsoleAuthenticationDeleteResponse::Status204_NoContent)
    }

    async fn console_authentication_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: AuthResponseData,
    ) -> Result<ConsoleAuthenticationPutResponse, String> {
        crate::sorcerer::authmancer::store_authentication(body, &self.vault).await;
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
            crate::lore::console_client_config::default().await,
        )
        .await
        {
            Ok(()) => Ok(DeviceLicenseActivationPostResponse::Status200_Success(ok())),
            Err(e) => Ok(
                DeviceLicenseActivationPostResponse::Status500_InternalServerError(
                    additional_info_from_error(e),
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
        match crate::sorcerer::licenso::validate_license(
            &self.vault,
            crate::lore::console_client_config::default().await,
        )
        .await
        {
            Ok(is_valid) => Ok(DeviceLicenseActivationStatusGetResponse::Status200_Success(
                DeviceLicenseActivationStatusGet200Response { is_valid },
            )),
            Err(e) => Ok(
                DeviceLicenseActivationStatusGetResponse::Status500_InternalServerError({
                    additional_info_from_error(e)
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
        let secrets = self.vault.get_secrets().await;
        Ok(DeviceLicenseInfoGetResponse::Status200_Success(
            DeviceLicenseInfoGet200Response {
                // TODO: Use correct type, as soon as serial numbers are implemented
                r#type: "Via user license".to_string(),
                session_id: Some(console_session_id_to_core_session_id(
                    secrets.get_session_id(),
                )),
                license: secrets.license_key,
            },
        ))
    }

    async fn device_onboarding_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: Dosschema,
    ) -> Result<DeviceOnboardingPostResponse, String> {
        if body.apps.is_empty() {
            return Ok(DeviceOnboardingPostResponse::Status400_MalformedRequest(
                AdditionalInfo {
                    additional_info: "No apps to install given (field 'apps' is empty)".to_string(),
                },
            ));
        }
        let app_keys = body
            .apps
            .into_iter()
            .filter_map(|app| {
                if let Some(version) = app.version {
                    Some(crate::vault::pouch::AppKey {
                        name: app.name,
                        version,
                    })
                } else {
                    warn!(
                        "Skip installing newest version of app {}, not implemented yet",
                        app.name
                    );
                    None
                }
            })
            .collect();
        let config = crate::lore::console_client_config::default().await;
        match crate::lore::quest::default()
            .await
            .lock()
            .await
            .schedule_quest("Install apps via device onboarding".to_string(), |quest| {
                crate::sorcerer::appraiser::install_apps(
                    quest,
                    self.vault.clone(),
                    app_keys,
                    config,
                )
            })
            .await
        {
            Ok((id, _)) => Ok(DeviceOnboardingPostResponse::Status202_Accepted(JobMeta {
                job_id: id.0 as i32,
            })),
            Err(e) => Err(e.to_string()),
        }
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
        path_params: InstancesInstanceIdDeletePathParams,
    ) -> Result<InstancesInstanceIdDeleteResponse, String> {
        let instance_id =
            InstanceId::from_str(path_params.instance_id.as_str()).map_err(|e| e.to_string())?;
        if !crate::sorcerer::instancius::does_instance_exist(self.vault.clone(), instance_id).await
        {
            return Ok(InstancesInstanceIdDeleteResponse::Status404_NoInstanceWithThisInstance);
        }
        let quest_id = crate::lore::quest::default()
            .await
            .lock()
            .await
            .schedule_quest(format!("Delete instance {instance_id}"), |quest| {
                crate::sorcerer::instancius::delete_instance(quest, self.vault.clone(), instance_id)
            })
            .await
            .map_err(|e| e.to_string())?
            .0;
        Ok(InstancesInstanceIdDeleteResponse::Status202_Accepted(
            JobMeta {
                job_id: quest_id.0 as i32,
            },
        ))
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
    ) -> Result<JobsJobIdDeleteResponse, String> {
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
    ) -> Result<JobsJobIdGetResponse, String> {
        match crate::sorcerer::magequester::get_job(path_params.job_id as u64).await {
            Some(job) => Ok(JobsJobIdGetResponse::Status200_Success(job)),
            None => Ok(JobsJobIdGetResponse::Status404_NotFound),
        }
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
        Ok(SystemInfoGetResponse::Status200_Sucess(
            crate::relic::system::info::try_create_system_info().map_err(|e| {
                let e = e.to_string();
                error!("Could not create SystemInfo: {e}");
                e
            })?,
        ))
    }

    async fn system_ping_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemPingGetResponse, String> {
        Ok(SystemPingGetResponse::Status200_Success(ok()))
    }

    async fn system_version_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemVersionGetResponse, String> {
        Ok(SystemVersionGetResponse::Status200_Success(
            models::SystemVersionGet200Response {
                api: crate::lore::API_VERSION.to_string(),
                core: crate::lore::CORE_VERSION.to_string(),
            },
        ))
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

#[cfg(test)]
mod tests {
    use crate::fsm::server_impl::ServerImpl;
    use crate::vault::{Vault, VaultConfig};
    use axum::extract::Host;
    use axum_extra::extract::CookieJar;
    use flecsd_axum_server::apis::apps::{Apps, AppsAppDeleteResponse};
    use flecsd_axum_server::models::{AppsAppDeletePathParams, AppsAppDeleteQueryParams};
    use http::Method;
    use std::path::Path;
    use std::sync::Arc;

    #[tokio::test]
    async fn uninstall_404() {
        let server = ServerImpl {
            vault: Arc::new(Vault::new(VaultConfig {
                path: Path::new("/tmp/flecs-tests/uninstall_404/").to_path_buf(),
            })),
        };
        assert_eq!(
            Ok(AppsAppDeleteResponse::Status404_NoSuchAppOrApp),
            server
                .apps_app_delete(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    AppsAppDeletePathParams {
                        app: "app".to_string(),
                    },
                    AppsAppDeleteQueryParams {
                        version: Some("version".to_string())
                    },
                )
                .await
        )
    }

    #[tokio::test]
    async fn uninstall_no_version() {
        let server = ServerImpl {
            vault: Arc::new(Vault::new(VaultConfig {
                path: Path::new("/tmp/flecs-tests/uninstall_404/").to_path_buf(),
            })),
        };
        assert!(server
            .apps_app_delete(
                Method::default(),
                Host("host".to_string()),
                CookieJar::default(),
                AppsAppDeletePathParams {
                    app: "app".to_string(),
                },
                AppsAppDeleteQueryParams { version: None },
            )
            .await
            .is_err())
    }
}
