use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::manifest::AppManifest;
use crate::quest::Quest;
use crate::quest::QuestResult;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::Vault;
use anyhow::Error;
use axum::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecs_app_manifest::AppManifestVersion;
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
    ) -> Result<AppsAppDeleteResponse, ()> {
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
                let vault = self.vault.clone();
                let (id, _) = crate::lore::quest::default()
                    .await
                    .lock()
                    .await
                    .schedule_quest(format!("Uninstall {key}"), |quest| {
                        crate::sorcerer::appraiser::uninstall_app(quest, vault, key)
                    })
                    .await
                    // TODO: Add 500 Response to API
                    .map_err(|_| ())?;
                Ok(AppsAppDeleteResponse::Status202_Accepted(JobMeta {
                    job_id: id.0 as i32,
                }))
            }
            // TODO: Add 400 Response to API
            None => Err(()),
        }
    }

    async fn apps_app_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: AppsAppGetPathParams,
        query_params: AppsAppGetQueryParams,
    ) -> Result<AppsAppGetResponse, ()> {
        let apps = crate::sorcerer::appraiser::get_app(
            self.vault.clone(),
            path_params.app,
            query_params.version,
        )
        .await
        // TODO: Add 500 Response to API
        .map_err(|_| ())?;
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
    ) -> Result<AppsGetResponse, ()> {
        let apps = crate::sorcerer::appraiser::get_apps(self.vault.clone())
            .await
            // TODO: Add 500 Response to API
            .map_err(|_| ())?;
        Ok(AppsGetResponse::Status200_Success(apps))
    }

    async fn apps_install_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: AppsInstallPostRequest,
    ) -> Result<AppsInstallPostResponse, ()> {
        let app_key = body.app_key.into();
        let config = crate::lore::console_client_config::default().await;
        let vault = self.vault.clone();
        match crate::lore::quest::default()
            .await
            .lock()
            .await
            .schedule_quest(format!("Install {}", app_key), |quest| {
                crate::sorcerer::appraiser::install_app(quest, vault, app_key, config)
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
    ) -> Result<AppsSideloadPostResponse, ()> {
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
                let vault = self.vault.clone();
                match crate::lore::quest::default()
                    .await
                    .lock()
                    .await
                    .schedule_quest(format!("Sideloading {}", manifest.key), |quest| {
                        crate::sorcerer::appraiser::install_app_from_manifest(
                            quest,
                            vault,
                            Arc::new(manifest),
                            config,
                        )
                    })
                    .await
                {
                    Ok((id, _)) => Ok(AppsSideloadPostResponse::Status202_Accepted(JobMeta {
                        job_id: id.0 as i32,
                    })),
                    // TODO: Add 500 Response to API
                    Err(_) => Err(()),
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
    ) -> Result<ConsoleAuthenticationDeleteResponse, ()> {
        crate::sorcerer::authmancer::delete_authentication(&self.vault).await;
        Ok(ConsoleAuthenticationDeleteResponse::Status204_NoContent)
    }

    async fn console_authentication_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: AuthResponseData,
    ) -> Result<ConsoleAuthenticationPutResponse, ()> {
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
    ) -> Result<DeviceLicenseActivationPostResponse, ()> {
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
    ) -> Result<DeviceLicenseActivationStatusGetResponse, ()> {
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
    ) -> Result<DeviceLicenseInfoGetResponse, ()> {
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
    ) -> Result<DeviceOnboardingPostResponse, ()> {
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
        let vault = self.vault.clone();
        match crate::lore::quest::default()
            .await
            .lock()
            .await
            .schedule_quest("Install apps via device onboarding".to_string(), |quest| {
                crate::sorcerer::appraiser::install_apps(quest, vault, app_keys, config)
            })
            .await
        {
            Ok((id, _)) => Ok(DeviceOnboardingPostResponse::Status202_Accepted(JobMeta {
                job_id: id.0 as i32,
            })),
            // TODO: Add 500 Response to API
            Err(_) => Err(()),
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
    ) -> Result<FlunderBrowseGetResponse, ()> {
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
        body: InstancesCreatePostRequest,
    ) -> Result<InstancesCreatePostResponse, ()> {
        let app_key: AppKey = body.app_key.into();
        if !crate::sorcerer::appraiser::does_app_exist(self.vault.clone(), app_key.clone()).await {
            return Ok(InstancesCreatePostResponse::Status400_MalformedRequest(
                AdditionalInfo {
                    additional_info: format!("App {app_key} does not exist"),
                },
            ));
        }
        let vault = self.vault.clone();
        let instance_name = body.instance_name;
        let (id, _quest) = crate::lore::quest::default()
            .await
            .lock()
            .await
            .schedule_quest_with_result(
                format!("Create instance for {app_key}"),
                |quest| async move {
                    let id = crate::sorcerer::instancius::create_instance(
                        quest,
                        vault,
                        app_key,
                        instance_name.unwrap_or_default(),
                    )
                    .await?;
                    Ok(QuestResult::InstanceId(id))
                },
            )
            .await
            // TODO: Add 500 Response to API
            .map_err(|_| ())?;
        Ok(InstancesCreatePostResponse::Status202_Accepted(
            JobMeta::new(id.0 as i32),
        ))
    }

    async fn instances_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        query_params: InstancesGetQueryParams,
    ) -> Result<InstancesGetResponse, ()> {
        let instances = match query_params {
            InstancesGetQueryParams {
                version: None,
                app: None,
            } => {
                crate::sorcerer::instancius::get_all_instances(
                    Quest::new_synced("Get info for all instances".to_string()),
                    self.vault.clone(),
                )
                .await
            }
            InstancesGetQueryParams { version, app } => {
                crate::sorcerer::instancius::get_instances_filtered(
                    Quest::new_synced(format!(
                        "Get all instances matching {:?} in version {:?}",
                        app, version
                    )),
                    self.vault.clone(),
                    app,
                    version,
                )
                .await
            }
        };
        Ok(InstancesGetResponse::Status200_Success(instances))
    }

    async fn instances_instance_id_config_environment_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigEnvironmentDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigEnvironmentDeleteResponse, ()> {
        todo!()
    }

    async fn instances_instance_id_config_environment_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigEnvironmentGetPathParams,
    ) -> Result<InstancesInstanceIdConfigEnvironmentGetResponse, ()> {
        todo!()
    }

    async fn instances_instance_id_config_environment_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigEnvironmentPutPathParams,
        _body: InstanceEnvironment,
    ) -> Result<InstancesInstanceIdConfigEnvironmentPutResponse, ()> {
        todo!()
    }

    async fn instances_instance_id_config_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigGetPathParams,
    ) -> Result<InstancesInstanceIdConfigGetResponse, ()> {
        todo!()
    }

    async fn instances_instance_id_config_ports_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigPortsDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigPortsDeleteResponse, ()> {
        todo!()
    }

    async fn instances_instance_id_config_ports_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigPortsGetPathParams,
    ) -> Result<InstancesInstanceIdConfigPortsGetResponse, ()> {
        todo!()
    }

    async fn instances_instance_id_config_ports_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigPortsPutPathParams,
        _body: InstancePorts,
    ) -> Result<InstancesInstanceIdConfigPortsPutResponse, ()> {
        todo!()
    }

    async fn instances_instance_id_config_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigPostPathParams,
        _body: InstanceConfig,
    ) -> Result<InstancesInstanceIdConfigPostResponse, ()> {
        todo!()
    }

    async fn instances_instance_id_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdDeletePathParams,
    ) -> Result<InstancesInstanceIdDeleteResponse, ()> {
        // TODO: Add 400 Response to API
        let instance_id = InstanceId::from_str(path_params.instance_id.as_str()).map_err(|_| ())?;
        if !crate::sorcerer::instancius::does_instance_exist(self.vault.clone(), instance_id).await
        {
            return Ok(InstancesInstanceIdDeleteResponse::Status404_NoInstanceWithThisInstance);
        }
        let vault = self.vault.clone();
        let quest_id = crate::lore::quest::default()
            .await
            .lock()
            .await
            .schedule_quest(format!("Delete instance {instance_id}"), move |quest| {
                crate::sorcerer::instancius::delete_instance(quest, vault, instance_id)
            })
            .await
            // TODO: Add 500 Response to API
            .map_err(|_| ())?
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
    ) -> Result<InstancesInstanceIdEditorPortGetResponse, ()> {
        todo!()
    }

    async fn instances_instance_id_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdGetPathParams,
    ) -> Result<InstancesInstanceIdGetResponse, ()> {
        let instance_id = match InstanceId::from_str(path_params.instance_id.as_str()) {
            Err(e) => {
                return Ok(
                    InstancesInstanceIdGetResponse::Status500_InternalServerError(AdditionalInfo {
                        additional_info: format!("Failed to parse instance id: {e}"),
                    }),
                )
            }
            Ok(instance_id) => instance_id,
        };
        match crate::sorcerer::instancius::get_instance_detailed(self.vault.clone(), instance_id)
            .await
        {
            Ok(Some(details)) => Ok(InstancesInstanceIdGetResponse::Status200_Success(details)),
            Ok(None) => Ok(InstancesInstanceIdGetResponse::Status404_NoInstanceWithThisInstance),
            Err(e) => Ok(
                InstancesInstanceIdGetResponse::Status500_InternalServerError(AdditionalInfo {
                    additional_info: e.to_string(),
                }),
            ),
        }
    }

    async fn instances_instance_id_logs_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdLogsGetPathParams,
    ) -> Result<InstancesInstanceIdLogsGetResponse, ()> {
        // TODO: Add 400 Response to API
        let instance_id = InstanceId::from_str(path_params.instance_id.as_str()).map_err(|_| ())?;
        if !crate::sorcerer::instancius::does_instance_exist(self.vault.clone(), instance_id).await
        {
            return Ok(InstancesInstanceIdLogsGetResponse::Status404_NoInstanceWithThisInstance);
        }
        match crate::sorcerer::instancius::get_instance_logs(self.vault.clone(), instance_id).await
        {
            Err(e) => Ok(
                InstancesInstanceIdLogsGetResponse::Status500_InternalServerError(AdditionalInfo {
                    additional_info: e.to_string(),
                }),
            ),
            Ok(logs) => Ok(InstancesInstanceIdLogsGetResponse::Status200_Success(
                models::InstancesInstanceIdLogsGet200Response {
                    stdout: logs.stdout,
                    stderr: logs.stderr,
                },
            )),
        }
    }

    async fn instances_instance_id_patch(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdPatchPathParams,
        _body: InstancesInstanceIdPatchRequest,
    ) -> Result<InstancesInstanceIdPatchResponse, ()> {
        todo!()
    }

    async fn instances_instance_id_start_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdStartPostPathParams,
    ) -> Result<InstancesInstanceIdStartPostResponse, ()> {
        // TODO: Add 400 Response to API
        let instance_id = InstanceId::from_str(path_params.instance_id.as_str()).map_err(|_| ())?;
        if !crate::sorcerer::instancius::does_instance_exist(self.vault.clone(), instance_id).await
        {
            return Ok(InstancesInstanceIdStartPostResponse::Status404_NoInstanceWithThisInstance);
        }
        let vault = self.vault.clone();
        let quest_id = crate::lore::quest::default()
            .await
            .lock()
            .await
            .schedule_quest(format!("Start instance {instance_id}"), move |quest| {
                crate::sorcerer::instancius::start_instance(quest, vault, instance_id)
            })
            .await
            // TODO: Add 500 Response to API
            .map_err(|_| ())?
            .0;
        Ok(InstancesInstanceIdStartPostResponse::Status202_Accepted(
            JobMeta {
                job_id: quest_id.0 as i32,
            },
        ))
    }

    async fn instances_instance_id_stop_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdStopPostPathParams,
    ) -> Result<InstancesInstanceIdStopPostResponse, ()> {
        // TODO: Add 400 Response to API
        let instance_id = InstanceId::from_str(path_params.instance_id.as_str()).map_err(|_| ())?;
        if !crate::sorcerer::instancius::does_instance_exist(self.vault.clone(), instance_id).await
        {
            return Ok(InstancesInstanceIdStopPostResponse::Status404_NoInstanceWithThisInstance);
        }
        let vault = self.vault.clone();
        let quest_id = crate::lore::quest::default()
            .await
            .lock()
            .await
            .schedule_quest(format!("Stop instance {instance_id}"), move |quest| {
                crate::sorcerer::instancius::stop_instance(quest, vault, instance_id)
            })
            .await
            // TODO: Add 500 Response to API
            .map_err(|_| ())?
            .0;
        Ok(InstancesInstanceIdStopPostResponse::Status202_Accepted(
            JobMeta {
                job_id: quest_id.0 as i32,
            },
        ))
    }
}

#[async_trait]
impl Jobs for ServerImpl {
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

#[async_trait]
impl System for ServerImpl {
    async fn system_info_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemInfoGetResponse, ()> {
        Ok(SystemInfoGetResponse::Status200_Sucess(
            crate::relic::system::info::try_create_system_info().map_err(|e| {
                error!("Could not create SystemInfo: {e}");
            })?,
        ))
    }

    async fn system_ping_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemPingGetResponse, ()> {
        Ok(SystemPingGetResponse::Status200_Success(ok()))
    }

    async fn system_version_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemVersionGetResponse, ()> {
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
    use crate::jeweler::gem::app::{try_create_app, AppDeserializable};
    use crate::tests::prepare_test_path;
    use crate::vault::pouch::Pouch;
    use crate::vault::{Vault, VaultConfig};
    use axum::extract::Host;
    use axum_extra::extract::CookieJar;
    use flecsd_axum_server::apis::apps::{Apps, AppsAppDeleteResponse};
    use flecsd_axum_server::apis::instances::{
        Instances, InstancesCreatePostResponse, InstancesInstanceIdLogsGetResponse,
        InstancesInstanceIdStartPostResponse, InstancesInstanceIdStopPostResponse,
    };
    use flecsd_axum_server::models::{
        AppKey, AppsAppDeletePathParams, AppsAppDeleteQueryParams, InstancesCreatePostRequest,
        InstancesInstanceIdLogsGetPathParams, InstancesInstanceIdStartPostPathParams,
        InstancesInstanceIdStopPostPathParams,
    };
    use http::Method;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn uninstall_404() {
        let path = prepare_test_path(module_path!(), "uninstall_404");
        let server = ServerImpl {
            vault: Arc::new(Vault::new(VaultConfig { path })),
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
    async fn start_404() {
        let path = prepare_test_path(module_path!(), "start_404");
        let server = ServerImpl {
            vault: Arc::new(Vault::new(VaultConfig { path })),
        };
        assert_eq!(
            Ok(InstancesInstanceIdStartPostResponse::Status404_NoInstanceWithThisInstance),
            server
                .instances_instance_id_start_post(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdStartPostPathParams {
                        instance_id: "00001234".to_string(),
                    },
                )
                .await
        )
    }

    #[tokio::test]
    async fn stop_404() {
        let path = prepare_test_path(module_path!(), "stop_404");
        let server = ServerImpl {
            vault: Arc::new(Vault::new(VaultConfig { path })),
        };
        assert_eq!(
            Ok(InstancesInstanceIdStopPostResponse::Status404_NoInstanceWithThisInstance),
            server
                .instances_instance_id_stop_post(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdStopPostPathParams {
                        instance_id: "00001234".to_string(),
                    },
                )
                .await
        )
    }

    #[tokio::test]
    async fn logs_404() {
        let path = prepare_test_path(module_path!(), "logs_404");
        let server = ServerImpl {
            vault: Arc::new(Vault::new(VaultConfig { path })),
        };
        assert_eq!(
            Ok(InstancesInstanceIdLogsGetResponse::Status404_NoInstanceWithThisInstance),
            server
                .instances_instance_id_logs_get(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdLogsGetPathParams {
                        instance_id: "00001234".to_string(),
                    },
                )
                .await
        )
    }

    #[tokio::test]
    async fn uninstall_no_version() {
        let path = prepare_test_path(module_path!(), "uninstall_no_version");
        let server = ServerImpl {
            vault: Arc::new(Vault::new(VaultConfig { path })),
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

    #[tokio::test]
    async fn create_instance_no_app() {
        let path = prepare_test_path(module_path!(), "create_instance_no_app");
        let server = ServerImpl {
            vault: Arc::new(Vault::new(VaultConfig { path })),
        };
        assert!(matches!(
            server
                .instances_create_post(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesCreatePostRequest {
                        app_key: AppKey {
                            name: "TestName".to_string(),
                            version: "1.2.3".to_string()
                        },
                        instance_name: None,
                    },
                )
                .await,
            Ok(InstancesCreatePostResponse::Status400_MalformedRequest(_))
        ))
    }

    #[tokio::test]
    async fn create_instance_ok() {
        let path = prepare_test_path(module_path!(), "create_instance_ok");
        let vault = Arc::new(Vault::new(VaultConfig { path }));
        let test_key = AppKey {
            name: "TestName".to_string(),
            version: "1.2.3".to_string(),
        };
        let app = AppDeserializable {
            key: test_key.clone().into(),
            deployments: Vec::new(),
        };
        let app = try_create_app(app, &HashMap::new(), &HashMap::new()).unwrap();
        vault
            .reservation()
            .reserve_app_pouch_mut()
            .grab()
            .await
            .app_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut()
            .insert(test_key.into(), app);
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_create_post(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesCreatePostRequest {
                        app_key: AppKey {
                            name: "TestName".to_string(),
                            version: "1.2.3".to_string()
                        },
                        instance_name: None,
                    },
                )
                .await,
            Ok(InstancesCreatePostResponse::Status202_Accepted(_))
        ))
    }
}
