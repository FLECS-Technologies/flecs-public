use crate::jeweler::gem::instance::{InstanceId, TransportProtocol};
use crate::jeweler::gem::manifest::{AppManifest, PortMapping, PortRange};
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
    InstancesInstanceIdConfigPortsTransportProtocolDeleteResponse as DeleteInstanceConfigPortMappingsResponse,
    InstancesInstanceIdConfigPortsTransportProtocolGetResponse as GetInstanceConfigProtocolPortMappingsResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortDeleteResponse as DeleteInstanceConfigPortMappingResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortGetResponse as GetInstanceConfigPortMappingResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortPutResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndDeleteResponse as DeleteInstanceConfigPortMappingRangeResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndGetResponse as GetInstanceConfigPortMappingRangeResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutResponse as PutInstanceConfigPortMappingRangeResponse,
    InstancesInstanceIdConfigPortsTransportProtocolPutResponse,
    InstancesInstanceIdConfigPostResponse, InstancesInstanceIdDeleteResponse,
    InstancesInstanceIdEditorPortGetResponse, InstancesInstanceIdGetResponse,
    InstancesInstanceIdLogsGetResponse, InstancesInstanceIdPatchResponse,
    InstancesInstanceIdStartPostResponse, InstancesInstanceIdStopPostResponse,
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
    FlunderBrowseGetQueryParams, InstanceConfig, InstanceEnvironment, InstancePortMapping,
    InstancePortMappingRange, InstancePortMappingSingle, InstancesCreatePostRequest,
    InstancesGetQueryParams, InstancesInstanceIdConfigEnvironmentDeletePathParams,
    InstancesInstanceIdConfigEnvironmentGetPathParams,
    InstancesInstanceIdConfigEnvironmentPutPathParams, InstancesInstanceIdConfigGetPathParams,
    InstancesInstanceIdConfigPortsDeletePathParams, InstancesInstanceIdConfigPortsGetPathParams,
    InstancesInstanceIdConfigPortsTransportProtocolDeletePathParams,
    InstancesInstanceIdConfigPortsTransportProtocolGetPathParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortDeletePathParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortGetPathParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortPutPathParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortPutRequest,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndDeletePathParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndGetPathParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutPathParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutRequest,
    InstancesInstanceIdConfigPortsTransportProtocolPutPathParams,
    InstancesInstanceIdConfigPostPathParams, InstancesInstanceIdDeletePathParams,
    InstancesInstanceIdEditorPortGetPathParams, InstancesInstanceIdGetPathParams,
    InstancesInstanceIdLogsGetPathParams, InstancesInstanceIdPatchPathParams,
    InstancesInstanceIdPatchRequest, InstancesInstanceIdStartPostPathParams,
    InstancesInstanceIdStopPostPathParams, JobMeta, JobsJobIdDeletePathParams,
    JobsJobIdGetPathParams, OptionalAdditionalInfo,
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
        path_params: InstancesInstanceIdConfigPortsDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigPortsDeleteResponse, ()> {
        let Ok(instance_id) = InstanceId::from_str(&path_params.instance_id) else {
            return Ok(
                InstancesInstanceIdConfigPortsDeleteResponse::Status400_MalformedRequest(
                    invalid_instance_id_additional_info(&path_params.instance_id),
                ),
            );
        };
        if crate::sorcerer::instancius::delete_instance_config_port_mappings(
            self.vault.clone(),
            instance_id,
        )
        .await
        {
            Ok(InstancesInstanceIdConfigPortsDeleteResponse::Status200_ExposedPortsOfInstanceWithThisInstance)
        } else {
            Ok(InstancesInstanceIdConfigPortsDeleteResponse::Status404_NoInstanceWithThisInstance)
        }
    }

    async fn instances_instance_id_config_ports_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigPortsGetPathParams,
    ) -> Result<InstancesInstanceIdConfigPortsGetResponse, ()> {
        let Ok(instance_id) = InstanceId::from_str(&path_params.instance_id) else {
            return Ok(
                InstancesInstanceIdConfigPortsGetResponse::Status400_MalformedRequest(
                    invalid_instance_id_additional_info(&path_params.instance_id),
                ),
            );
        };
        match crate::sorcerer::instancius::get_instance_config_port_mappings(
            self.vault.clone(),
            instance_id,
        )
        .await
        {
            None => {
                Ok(InstancesInstanceIdConfigPortsGetResponse::Status404_NoInstanceWithThisInstance)
            }
            Some(mapping) => Ok(
                InstancesInstanceIdConfigPortsGetResponse::Status200_Success(
                    models::InstancePorts {
                        tcp: port_mappings_to_instance_ports(&mapping.tcp),
                        udp: port_mappings_to_instance_ports(&mapping.udp),
                        sctp: port_mappings_to_instance_ports(&mapping.sctp),
                    },
                ),
            ),
        }
    }

    async fn instances_instance_id_config_ports_transport_protocol_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigPortsTransportProtocolDeletePathParams,
    ) -> Result<DeleteInstanceConfigPortMappingsResponse, ()> {
        let Ok(instance_id) = InstanceId::from_str(&path_params.instance_id) else {
            return Ok(
                DeleteInstanceConfigPortMappingsResponse::Status400_MalformedRequest(
                    invalid_instance_id_additional_info(&path_params.instance_id),
                ),
            );
        };
        if crate::sorcerer::instancius::delete_instance_config_port_mappings(
            self.vault.clone(),
            instance_id,
        )
        .await
        {
            Ok(DeleteInstanceConfigPortMappingsResponse::Status200_RemovedAllPublishedPortsOfInstanceWithThisInstance)
        } else {
            Ok(
                DeleteInstanceConfigPortMappingsResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo::new(),
                ),
            )
        }
    }

    async fn instances_instance_id_config_ports_transport_protocol_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigPortsTransportProtocolGetPathParams,
    ) -> Result<GetInstanceConfigProtocolPortMappingsResponse, ()> {
        let Ok(instance_id) = InstanceId::from_str(&path_params.instance_id) else {
            return Ok(
                GetInstanceConfigProtocolPortMappingsResponse::Status400_MalformedRequest(
                    invalid_instance_id_additional_info(&path_params.instance_id),
                ),
            );
        };
        if let Some(port_mapping) =
            crate::sorcerer::instancius::get_instance_config_protocol_port_mappings(
                self.vault.clone(),
                instance_id,
                path_params.transport_protocol.into(),
            )
            .await
        {
            Ok(GetInstanceConfigProtocolPortMappingsResponse::Status200_PublishedPortsForInstanceWithThisInstance(port_mappings_to_instance_ports(&port_mapping)))
        } else {
            Ok(
                GetInstanceConfigProtocolPortMappingsResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo::new(),
                ),
            )
        }
    }

    async fn instances_instance_id_config_ports_transport_protocol_host_port_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigPortsTransportProtocolHostPortDeletePathParams,
    ) -> Result<DeleteInstanceConfigPortMappingResponse, ()> {
        let Ok(instance_id) = InstanceId::from_str(&path_params.instance_id) else {
            return Ok(
                DeleteInstanceConfigPortMappingResponse::Status400_MalformedRequest(
                    invalid_instance_id_additional_info(&path_params.instance_id),
                ),
            );
        };
        match crate::sorcerer::instancius::delete_instance_config_port_mapping(
            self.vault.clone(),
            instance_id,
            path_params.host_port as u16,
            path_params.transport_protocol.into(),
        )
        .await
        {
            None => Ok(
                DeleteInstanceConfigPortMappingResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo {
                        additional_info: Some(format!("Instance {instance_id} does not exist")),
                    },
                ),
            ),
            Some(false) => Ok(
                DeleteInstanceConfigPortMappingResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo {
                        additional_info: Some(format!(
                            "Host port {} is not mapped to {instance_id}",
                            path_params.host_port
                        )),
                    },
                ),
            ),
            Some(true) => Ok(DeleteInstanceConfigPortMappingResponse::Status200_Success),
        }
    }

    async fn instances_instance_id_config_ports_transport_protocol_host_port_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigPortsTransportProtocolHostPortGetPathParams,
    ) -> Result<GetInstanceConfigPortMappingResponse, ()> {
        let Ok(instance_id) = InstanceId::from_str(&path_params.instance_id) else {
            return Ok(
                GetInstanceConfigPortMappingResponse::Status400_MalformedRequest(
                    invalid_instance_id_additional_info(&path_params.instance_id),
                ),
            );
        };
        match crate::sorcerer::instancius::get_instance_config_port_mapping(
            self.vault.clone(),
            instance_id,
            path_params.host_port as u16,
            path_params.transport_protocol.into(),
        )
        .await
        {
            None => Ok(
                GetInstanceConfigPortMappingResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo {
                        additional_info: Some(format!("Instance {instance_id} does not exist")),
                    },
                ),
            ),
            Some(None) => Ok(
                GetInstanceConfigPortMappingResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo {
                        additional_info: Some(format!(
                            "Host port {} is not mapped to {instance_id}",
                            path_params.host_port,
                        )),
                    },
                ),
            ),
            Some(Some(PortMapping::Single(_, container_port))) => Ok(
                GetInstanceConfigPortMappingResponse::Status200_Success(container_port as i32),
            ),
            Some(Some(PortMapping::Range { from, .. })) => {
                Ok(GetInstanceConfigPortMappingResponse::Status200_Success(
                    *from.range().start() as i32,
                ))
            }
        }
    }

    async fn instances_instance_id_config_ports_transport_protocol_host_port_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigPortsTransportProtocolHostPortPutPathParams,
        body: InstancesInstanceIdConfigPortsTransportProtocolHostPortPutRequest,
    ) -> Result<InstancesInstanceIdConfigPortsTransportProtocolHostPortPutResponse, ()> {
        let Ok(instance_id) = InstanceId::from_str(&path_params.instance_id) else {
            return Ok(
                InstancesInstanceIdConfigPortsTransportProtocolHostPortPutResponse::Status400_MalformedRequest(
                    invalid_instance_id_additional_info(&path_params.instance_id),
                ),
            );
        };
        match crate::sorcerer::instancius::put_instance_config_port_mapping(
            self.vault.clone(),
            instance_id,
            PortMapping::Single(path_params.host_port as u16, body.container_port),
            path_params.transport_protocol.into(),
        )
            .await
        {
            Err(e) => Ok(InstancesInstanceIdConfigPortsTransportProtocolHostPortPutResponse::Status400_MalformedRequest(AdditionalInfo::new(e.to_string()))),
            Ok(None) => Ok(
                InstancesInstanceIdConfigPortsTransportProtocolHostPortPutResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo {
                        additional_info: Some(format!("Instance {instance_id} does not exist")),
                    },
                ),
            ),
            Ok(Some(false)) => Ok(InstancesInstanceIdConfigPortsTransportProtocolHostPortPutResponse::Status201_TheSpecifiedPortMappingWasCreated),
            Ok(Some(true)) => Ok(InstancesInstanceIdConfigPortsTransportProtocolHostPortPutResponse::Status200_TheSpecifiedPortMappingWasSet),
        }
    }

    async fn instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndDeletePathParams,
    ) -> Result<DeleteInstanceConfigPortMappingRangeResponse, ()> {
        let Ok(instance_id) = InstanceId::from_str(&path_params.instance_id) else {
            return Ok(
                DeleteInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(
                    invalid_instance_id_additional_info(&path_params.instance_id),
                ),
            );
        };
        let host_port_range = match PortRange::try_new(
            path_params.host_port_start as u16,
            path_params.host_port_end as u16,
        ) {
            Err(e) => {
                return Ok(
                    DeleteInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(
                        AdditionalInfo::new(format!(
                            "Invalid host port range ({}-{}): {e}",
                            path_params.host_port_start, path_params.host_port_end
                        )),
                    ),
                )
            }
            Ok(host_port_range) => host_port_range,
        };
        match crate::sorcerer::instancius::delete_instance_config_port_mapping_range(
            self.vault.clone(),
            instance_id,
            host_port_range,
            path_params.transport_protocol.into(),
        )
        .await
        {
            None => Ok(
                DeleteInstanceConfigPortMappingRangeResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo {
                        additional_info: Some(format!("Instance {instance_id} does not exist")),
                    },
                ),
            ),
            Some(false) => Ok(
                DeleteInstanceConfigPortMappingRangeResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo {
                        additional_info: Some(format!(
                            "Host port range ({}-{}) is not mapped to {instance_id}",
                            path_params.host_port_start, path_params.host_port_end
                        )),
                    },
                ),
            ),
            Some(true) => Ok(DeleteInstanceConfigPortMappingRangeResponse::Status200_Success),
        }
    }

    async fn instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndGetPathParams,
    ) -> Result<GetInstanceConfigPortMappingRangeResponse, ()> {
        let Ok(instance_id) = InstanceId::from_str(&path_params.instance_id) else {
            return Ok(
                GetInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(
                    invalid_instance_id_additional_info(&path_params.instance_id),
                ),
            );
        };
        let host_port_range = match PortRange::try_new(
            path_params.host_port_start as u16,
            path_params.host_port_end as u16,
        ) {
            Err(e) => {
                return Ok(
                    GetInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(
                        AdditionalInfo::new(format!(
                            "Invalid host port range ({}-{}): {e}",
                            path_params.host_port_start, path_params.host_port_end
                        )),
                    ),
                )
            }
            Ok(host_port_range) => host_port_range,
        };
        match crate::sorcerer::instancius::get_instance_config_port_mapping_range(
            self.vault.clone(),
            instance_id,
            host_port_range,
            path_params.transport_protocol.into(),
        )
        .await
        {
            None => Ok(
                GetInstanceConfigPortMappingRangeResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo {
                        additional_info: Some(format!("Instance {instance_id} does not exist")),
                    },
                ),
            ),
            Some(None) => Ok(
                GetInstanceConfigPortMappingRangeResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo {
                        additional_info: Some(format!(
                            "Host port range ({}-{}) is not mapped to {instance_id}",
                            path_params.host_port_start, path_params.host_port_end
                        )),
                    },
                ),
            ),
            Some(Some(PortMapping::Single(_, container_port))) => Ok(
                GetInstanceConfigPortMappingRangeResponse::Status200_Success(models::PortRange {
                    start: container_port,
                    end: container_port,
                }),
            ),
            Some(Some(PortMapping::Range { to, .. })) => {
                Ok(GetInstanceConfigPortMappingRangeResponse::Status200_Success(to.into()))
            }
        }
    }

    async fn instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutPathParams,
        body: InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutRequest,
    ) -> Result<PutInstanceConfigPortMappingRangeResponse, ()> {
        let Ok(instance_id) = InstanceId::from_str(&path_params.instance_id) else {
            return Ok(
                PutInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(
                    invalid_instance_id_additional_info(&path_params.instance_id),
                ),
            );
        };
        let host_port_range = match PortRange::try_new(
            path_params.host_port_start as u16,
            path_params.host_port_end as u16,
        ) {
            Err(e) => {
                return Ok(
                    PutInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(
                        AdditionalInfo::new(format!(
                            "Invalid host port range ({}-{}): {e}",
                            path_params.host_port_start, path_params.host_port_end
                        )),
                    ),
                )
            }
            Ok(host_port_range) => host_port_range,
        };
        let container_port_range = match PortRange::try_from(body.container_port_range.clone()) {
            Err(e) => {
                return Ok(
                    PutInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(
                        AdditionalInfo::new(format!(
                            "Invalid container port range ({:?}): {e}",
                            body.container_port_range
                        )),
                    ),
                )
            }
            Ok(host_port_range) => host_port_range,
        };
        if container_port_range.range().len() != host_port_range.range().len() {
            return Ok(
                PutInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(
                    AdditionalInfo::new(format!(
                        "The size of the container port range ({container_port_range}) and host port range ({host_port_range}) has to be equal",
                    )),
                ),
            );
        }
        match crate::sorcerer::instancius::put_instance_config_port_mapping(
            self.vault.clone(),
            instance_id,
            PortMapping::Range { from: host_port_range, to: container_port_range },
            path_params.transport_protocol.into(),
        )
            .await
        {
            Err(e) => Ok(PutInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(AdditionalInfo::new(e.to_string()))),
            Ok(None) => Ok(
                PutInstanceConfigPortMappingRangeResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo {
                        additional_info: Some(format!("Instance {instance_id} does not exist")),
                    },
                ),
            ),
            Ok(Some(false)) => Ok(PutInstanceConfigPortMappingRangeResponse::Status201_TheSpecifiedPortMappingWasCreated),
            Ok(Some(true)) => Ok(PutInstanceConfigPortMappingRangeResponse::Status200_TheSpecifiedPortMappingWasSet),
        }
    }

    async fn instances_instance_id_config_ports_transport_protocol_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigPortsTransportProtocolPutPathParams,
        body: Vec<InstancePortMapping>,
    ) -> Result<InstancesInstanceIdConfigPortsTransportProtocolPutResponse, ()> {
        let Ok(instance_id) = InstanceId::from_str(&path_params.instance_id) else {
            return Ok(
                InstancesInstanceIdConfigPortsTransportProtocolPutResponse::Status400_MalformedRequest(
                    invalid_instance_id_additional_info(&path_params.instance_id)
                ),
            );
        };
        let port_mapping = match body.into_iter().map(PortMapping::try_from).collect::<Result<Vec<_>, _>>() {
            Err(e) => return Ok(
                InstancesInstanceIdConfigPortsTransportProtocolPutResponse::Status400_MalformedRequest(
                    AdditionalInfo::new(format!("Invalid port mapping: {e}"))
                ),
            ),
            Ok(port_mapping) => port_mapping,
        };
        if let Err(errors) = validate_port_mappings(&port_mapping) {
            return Ok(
                InstancesInstanceIdConfigPortsTransportProtocolPutResponse::Status400_MalformedRequest(
                    AdditionalInfo::new(format!("Invalid port mapping: {}", errors.join("\n")))
                ),
            );
        }
        let instance_found =
            crate::sorcerer::instancius::put_instance_config_protocol_port_mappings(
                self.vault.clone(),
                instance_id,
                port_mapping,
                path_params.transport_protocol.into(),
            )
            .await;
        if instance_found {
            Ok(InstancesInstanceIdConfigPortsTransportProtocolPutResponse::Status200_PublishedPortsOfInstanceWithThisInstance)
        } else {
            Ok(
                InstancesInstanceIdConfigPortsTransportProtocolPutResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo::new()
                )
            )
        }
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

impl TryFrom<models::PortRange> for PortRange {
    type Error = Error;

    fn try_from(value: models::PortRange) -> Result<Self, Self::Error> {
        Self::try_new(value.start, value.end)
    }
}

impl From<PortRange> for models::PortRange {
    fn from(value: PortRange) -> Self {
        Self {
            start: *value.range().start(),
            end: *value.range().end(),
        }
    }
}

impl TryFrom<InstancePortMapping> for PortMapping {
    type Error = Error;

    fn try_from(value: InstancePortMapping) -> Result<Self, Self::Error> {
        match value {
            InstancePortMapping::InstancePortMappingRange(mapping) => Ok(Self::Range {
                from: PortRange::try_from(mapping.host_ports)?,
                to: PortRange::try_from(mapping.container_ports)?,
            }),
            InstancePortMapping::InstancePortMappingSingle(mapping) => {
                Ok(Self::Single(mapping.host_port, mapping.container_port))
            }
        }
    }
}

impl From<&PortMapping> for InstancePortMapping {
    fn from(value: &PortMapping) -> Self {
        match value {
            PortMapping::Single(host, container) => InstancePortMapping::InstancePortMappingSingle(
                Box::new(InstancePortMappingSingle {
                    host_port: *host,
                    container_port: *container,
                }),
            ),
            PortMapping::Range { from, to } => {
                InstancePortMapping::InstancePortMappingRange(Box::new(InstancePortMappingRange {
                    host_ports: models::PortRange {
                        start: *from.range().start(),
                        end: *from.range().end(),
                    },
                    container_ports: models::PortRange {
                        start: *to.range().start(),
                        end: *to.range().end(),
                    },
                }))
            }
        }
    }
}

fn port_mappings_to_instance_ports(port_mappings: &[PortMapping]) -> Vec<InstancePortMapping> {
    port_mappings
        .iter()
        .map(InstancePortMapping::from)
        .collect()
}

fn invalid_instance_id_additional_info(instance_id: &str) -> AdditionalInfo {
    AdditionalInfo {
        additional_info: format!("Invalid instance_id: {}", instance_id),
    }
}

impl From<models::TransportProtocol> for TransportProtocol {
    fn from(value: models::TransportProtocol) -> Self {
        match value {
            models::TransportProtocol::Tcp => TransportProtocol::Tcp,
            models::TransportProtocol::Udp => TransportProtocol::Udp,
            models::TransportProtocol::Sctp => TransportProtocol::Sctp,
        }
    }
}

fn validate_port_mappings(port_mappings: &[PortMapping]) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for port_mapping in port_mappings {
        if let PortMapping::Range { from, to } = port_mapping {
            if from.range().len() != to.range().len() {
                errors.push(
                    format!("The size of the container port range ({to}) and host port range ({from}) has to be equal")
                )
            }
        }
    }
    for (i, one) in port_mappings.iter().enumerate() {
        for (j, two) in port_mappings.iter().enumerate() {
            if i != j && one.do_host_ports_overlap(two) {
                errors.push(format!(
                    "Host ports of mapping {one} overlaps with host ports of mapping {two}"
                ))
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::gem::app::{try_create_app, AppDeserializable};
    use crate::jeweler::gem::manifest::{PortMapping, PortRange};
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
    use flecsd_axum_server::models;
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

    #[tokio::test]
    async fn delete_instance_config_ports_400() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "delete_instance_config_ports_400",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsDeletePathParams {
                        instance_id: "invalid_instance_id".to_string(),
                    },
                )
                .await,
            Ok(InstancesInstanceIdConfigPortsDeleteResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_404() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "delete_instance_config_ports_404",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsDeletePathParams {
                        instance_id: "12341234".to_string(),
                    },
                )
                .await,
            Ok(InstancesInstanceIdConfigPortsDeleteResponse::Status404_NoInstanceWithThisInstance)
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_200() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "delete_instance_config_ports_200",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsDeletePathParams {
                        instance_id: "00000006".to_string(),
                    },
                )
                .await,
            Ok(InstancesInstanceIdConfigPortsDeleteResponse::Status200_ExposedPortsOfInstanceWithThisInstance)
        ));
        assert!(server
            .vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .get(&InstanceId::new(6))
            .unwrap()
            .config
            .port_mapping
            .is_empty())
    }

    #[tokio::test]
    async fn get_instance_config_ports_400() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "get_instance_config_ports_400",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsGetPathParams {
                        instance_id: "invalid_instance_id".to_string(),
                    },
                )
                .await,
            Ok(InstancesInstanceIdConfigPortsGetResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_404() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "get_instance_config_ports_404",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsGetPathParams {
                        instance_id: "12341234".to_string(),
                    },
                )
                .await,
            Ok(InstancesInstanceIdConfigPortsGetResponse::Status404_NoInstanceWithThisInstance)
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_200() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "get_instance_config_ports_200",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert_eq!(
            server
                .instances_instance_id_config_ports_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsGetPathParams {
                        instance_id: "00000006".to_string(),
                    },
                )
                .await,
            Ok(
                InstancesInstanceIdConfigPortsGetResponse::Status200_Success(
                    models::InstancePorts {
                        tcp: vec![models::InstancePortMapping::InstancePortMappingSingle(
                            Box::new(models::InstancePortMappingSingle {
                                host_port: 80,
                                container_port: 8080,
                            })
                        )],
                        udp: vec![models::InstancePortMapping::InstancePortMappingRange(
                            Box::new(models::InstancePortMappingRange {
                                host_ports: models::PortRange {
                                    start: 50,
                                    end: 100,
                                },
                                container_ports: models::PortRange {
                                    start: 150,
                                    end: 200,
                                },
                            })
                        )],
                        sctp: vec![],
                    }
                )
            )
        );
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_200() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "delete_instance_config_ports_transport_protocol_200",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolDeletePathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp
                    },
                )
                .await,
            Ok(DeleteInstanceConfigPortMappingsResponse::Status200_RemovedAllPublishedPortsOfInstanceWithThisInstance)
        );
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_400() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "delete_instance_config_ports_transport_protocol_400",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolDeletePathParams {
                        instance_id: "blablaa".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp
                    },
                )
                .await,
            Ok(DeleteInstanceConfigPortMappingsResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_404() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "delete_instance_config_ports_transport_protocol_404",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolDeletePathParams {
                        instance_id: "aaaaaaaa".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp
                    },
                )
                .await,
            Ok(DeleteInstanceConfigPortMappingsResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_400() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "get_instance_config_ports_transport_protocol_400",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolGetPathParams {
                        instance_id: "invalid instance id".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp
                    },
                )
                .await,
            Ok(GetInstanceConfigProtocolPortMappingsResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_404() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "get_instance_config_ports_transport_protocol_404",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolGetPathParams {
                        instance_id: "abcdabcd".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp
                    },
                )
                .await,
            Ok(GetInstanceConfigProtocolPortMappingsResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_200() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "get_instance_config_ports_transport_protocol_200",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolGetPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp
                    },
                )
                .await,
            Ok(GetInstanceConfigProtocolPortMappingsResponse::Status200_PublishedPortsForInstanceWithThisInstance(
                vec![InstancePortMapping::InstancePortMappingSingle(Box::new(
                    models::InstancePortMappingSingle { host_port: 80, container_port: 8080 })
                )]
            ))
        );
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_host_port_400() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "delete_instance_config_ports_transport_protocol_host_port_400",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortDeletePathParams {
                        instance_id: "invalid instance id".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port: 80,
                    },
                )
                .await,
            Ok(DeleteInstanceConfigPortMappingResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_host_port_404_instance() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "delete_instance_config_ports_transport_protocol_host_port_404_instance",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortDeletePathParams {
                        instance_id: "ffffffff".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port: 80,
                    },
                )
                .await,
            Ok(DeleteInstanceConfigPortMappingResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_host_port_404_host() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "delete_instance_config_ports_transport_protocol_host_port_404_host",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortDeletePathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port: 90,
                    },
                )
                .await,
            Ok(DeleteInstanceConfigPortMappingResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_host_port_200_host() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "delete_instance_config_ports_transport_protocol_host_port_200_host",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortDeletePathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port: 80,
                    },
                )
                .await,
            Ok(DeleteInstanceConfigPortMappingResponse::Status200_Success)
        );
        assert!(server
            .vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .get(&InstanceId::new(6))
            .unwrap()
            .config
            .port_mapping
            .tcp
            .is_empty())
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_host_port_400() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "get_instance_config_ports_transport_protocol_host_port_400",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortGetPathParams {
                        instance_id: "invalid instance id".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port: 90,
                    },
                )
                .await,
            Ok(GetInstanceConfigPortMappingResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_host_port_404_instance() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "get_instance_config_ports_transport_protocol_host_port_404_instance",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortGetPathParams {
                        instance_id: "ffffffff".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port: 80,
                    },
                )
                .await,
            Ok(GetInstanceConfigPortMappingResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_host_port_404_host() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "get_instance_config_ports_transport_protocol_host_port_404_host",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortGetPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port: 90,
                    },
                )
                .await,
            Ok(GetInstanceConfigPortMappingResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_host_port_200_single() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "get_instance_config_ports_transport_protocol_host_port_200_single",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortGetPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port: 80,
                    },
                )
                .await,
            Ok(GetInstanceConfigPortMappingResponse::Status200_Success(
                8080
            ))
        );
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_host_port_400_instance_id() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_host_port_400_instance_id",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortPutPathParams {
                        instance_id: "invalid instance id".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port: 80,
                    },
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortPutRequest{
                        container_port: 20
                    },
                )
                .await,
            Ok(InstancesInstanceIdConfigPortsTransportProtocolHostPortPutResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_host_port_400_overlap() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_host_port_400_overlap",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortPutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port: 80,
                    },
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortPutRequest{
                        container_port: 20
                    },
                )
                .await,
            Ok(InstancesInstanceIdConfigPortsTransportProtocolHostPortPutResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_host_port_404() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_host_port_404",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortPutPathParams {
                        instance_id: "ffffffff".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port: 80,
                    },
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortPutRequest{
                        container_port: 20
                    },
                )
                .await,
            Ok(InstancesInstanceIdConfigPortsTransportProtocolHostPortPutResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_host_port_201() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_host_port_201",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortPutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port: 70,
                    },
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortPutRequest{
                        container_port: 20
                    },
                )
                .await,
            Ok(InstancesInstanceIdConfigPortsTransportProtocolHostPortPutResponse::Status201_TheSpecifiedPortMappingWasCreated)
        ));
        assert!(server
            .vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .get(&InstanceId::new(6))
            .unwrap()
            .config
            .port_mapping
            .tcp
            .contains(&PortMapping::Single(70, 20)))
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_host_port_200() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_host_port_200",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortPutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port: 80,
                    },
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortPutRequest{
                        container_port: 20
                    },
                )
                .await,
            Ok(InstancesInstanceIdConfigPortsTransportProtocolHostPortPutResponse::Status200_TheSpecifiedPortMappingWasSet)
        ));
        let resulting_port_mapping = server
            .vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .get(&InstanceId::new(6))
            .unwrap()
            .config
            .port_mapping
            .tcp
            .clone();
        assert_eq!(resulting_port_mapping, vec![PortMapping::Single(80, 20)])
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_range_400_range() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "delete_instance_config_ports_transport_protocol_range_400_range",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndDeletePathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_start: 20,
                        host_port_end: 1,
                    },
                )
                .await,
            Ok(DeleteInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_range_400_instance_id() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "delete_instance_config_ports_transport_protocol_range_400_instance_id",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndDeletePathParams {
                        instance_id: "invalid instance id".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_start: 20,
                        host_port_end: 70,
                    },
                )
                .await,
            Ok(DeleteInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_range_404_range() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "delete_instance_config_ports_transport_protocol_range_404_range",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndDeletePathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_start: 20,
                        host_port_end: 70,
                    },
                )
                .await,
            Ok(DeleteInstanceConfigPortMappingRangeResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_range_404_instance() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "delete_instance_config_ports_transport_protocol_range_404_instance",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndDeletePathParams {
                        instance_id: "aabbccdd".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_start: 50,
                        host_port_end: 100,
                    },
                )
                .await,
            Ok(DeleteInstanceConfigPortMappingRangeResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_range_200() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "delete_instance_config_ports_transport_protocol_range_200",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndDeletePathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_start: 50,
                        host_port_end: 100,
                    },
                )
                .await,
            Ok(DeleteInstanceConfigPortMappingRangeResponse::Status200_Success)
        );
        assert!(server
            .vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .get(&InstanceId::new(6))
            .unwrap()
            .config
            .port_mapping
            .udp
            .is_empty())
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_range_400_instance_id() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "get_instance_config_ports_transport_protocol_range_400_instance_id",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndGetPathParams {
                        instance_id: "invalid instance id".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_start: 70,
                        host_port_end: 100,
                    },
                )
                .await,
            Ok(GetInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_range_400_range() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "get_instance_config_ports_transport_protocol_range_400_range",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndGetPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_start: 70,
                        host_port_end: 4,
                    },
                )
                .await,
            Ok(GetInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_range_404_range() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "get_instance_config_ports_transport_protocol_range_404_range",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndGetPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_start: 70,
                        host_port_end: 100,
                    },
                )
                .await,
            Ok(GetInstanceConfigPortMappingRangeResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_range_404_instance() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "get_instance_config_ports_transport_protocol_range_404_instance",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndGetPathParams {
                        instance_id: "12345678".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_start: 50,
                        host_port_end: 100,
                    },
                )
                .await,
            Ok(GetInstanceConfigPortMappingRangeResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_range_200_range() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "get_instance_config_ports_transport_protocol_range_200_range",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndGetPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_start: 50,
                        host_port_end: 100,
                    },
                )
                .await,
            Ok(GetInstanceConfigPortMappingRangeResponse::Status200_Success(models::PortRange {
                start: 150,
                end: 200,
            }))
        );
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_range_200_single() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "get_instance_config_ports_transport_protocol_range_200",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndGetPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_start: 80,
                        host_port_end: 80,
                    },
                )
                .await,
            Ok(GetInstanceConfigPortMappingRangeResponse::Status200_Success(models::PortRange {
                start: 8080,
                end: 8080,
            }))
        );
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_range_400_instance_id() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_range_400_instance_id",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutPathParams {
                        instance_id: "invalid instance id".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                        host_port_start: 70,
                        host_port_end: 90,
                    },
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutRequest {
                        container_port_range: models::PortRange {
                            start: 200,
                            end: 220,
                        }
                    }
                )
                .await,
            Ok(PutInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_range_400_host_range() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_range_400_host_range",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                        host_port_start: 70,
                        host_port_end: 50,
                    },
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutRequest {
                        container_port_range: models::PortRange {
                            start: 200,
                            end: 220,
                        }
                    }
                )
                .await,
            Ok(PutInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_range_400_container_range() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_range_400_container_range",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                        host_port_start: 70,
                        host_port_end: 90,
                    },
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutRequest {
                        container_port_range: models::PortRange {
                            start: 200,
                            end: 180,
                        }
                    }
                )
                .await,
            Ok(PutInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_range_400_range_mismatch() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_range_400_range_mismatch",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                        host_port_start: 70,
                        host_port_end: 90,
                    },
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutRequest {
                        container_port_range: models::PortRange {
                            start: 200,
                            end: 400,
                        }
                    }
                )
                .await,
            Ok(PutInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_range_400_overlap() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_range_400_overlap",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_start: 70,
                        host_port_end: 90,
                    },
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutRequest {
                        container_port_range: models::PortRange {
                            start: 200,
                            end: 220,
                        }
                    }
                )
                .await,
            Ok(PutInstanceConfigPortMappingRangeResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_range_404() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_range_404",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutPathParams {
                        instance_id: "ffeeddcc".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                        host_port_start: 1000,
                        host_port_end: 1100,
                    },
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutRequest {
                        container_port_range: models::PortRange {
                            start: 200,
                            end: 300,
                        }
                    }
                )
                .await,
            Ok(PutInstanceConfigPortMappingRangeResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_range_201() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_range_201",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                        host_port_start: 1000,
                        host_port_end: 1100,
                    },
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutRequest {
                        container_port_range: models::PortRange {
                            start: 200,
                            end: 300,
                        }
                    }
                )
                .await,
            Ok(PutInstanceConfigPortMappingRangeResponse::Status201_TheSpecifiedPortMappingWasCreated)
        );
        assert!(server
            .vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .get(&InstanceId::new(6))
            .unwrap()
            .config
            .port_mapping
            .sctp
            .contains(&PortMapping::Range {
                from: PortRange::new(1000..=1100),
                to: PortRange::new(200..=300),
            }));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_range_200() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_range_200",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_start_host_port_end_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_start: 50,
                        host_port_end: 100,
                    },
                    InstancesInstanceIdConfigPortsTransportProtocolHostPortStartHostPortEndPutRequest {
                        container_port_range: models::PortRange {
                            start: 200,
                            end: 250,
                        }
                    }
                )
                .await,
            Ok(PutInstanceConfigPortMappingRangeResponse::Status200_TheSpecifiedPortMappingWasSet)
        );
        assert_eq!(
            server
                .vault
                .reservation()
                .reserve_instance_pouch()
                .grab()
                .await
                .instance_pouch
                .as_ref()
                .unwrap()
                .gems()
                .get(&InstanceId::new(6))
                .unwrap()
                .config
                .port_mapping
                .udp,
            vec![PortMapping::Range {
                from: PortRange::new(50..=100),
                to: PortRange::new(200..=250),
            }]
        );
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_400_instance_id() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_400_instance_id",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolPutPathParams {
                        instance_id: "invalid instance id".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                    },
                    vec![
                        InstancePortMapping::InstancePortMappingRange(Box::new(
                            models::InstancePortMappingRange {
                                host_ports: models::PortRange {
                                    start: 2000,
                                    end: 3000,
                                },
                                container_ports: models::PortRange {
                                    start: 6000,
                                    end: 7000,
                                },
                            },
                        ))
                    ],
                )
                .await,
            Ok(InstancesInstanceIdConfigPortsTransportProtocolPutResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_400_overlap() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_400_overlap",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        let port_mappings = vec![
            InstancePortMapping::InstancePortMappingRange(Box::new(
                models::InstancePortMappingRange {
                    host_ports: models::PortRange {
                        start: 2000,
                        end: 3000,
                    },
                    container_ports: models::PortRange {
                        start: 6000,
                        end: 7000,
                    },
                },
            )),
            InstancePortMapping::InstancePortMappingSingle(Box::new(
                models::InstancePortMappingSingle {
                    host_port: 2500,
                    container_port: 10000,
                },
            )),
        ];
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolPutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                    },
                    port_mappings,
                )
                .await,
            Ok(InstancesInstanceIdConfigPortsTransportProtocolPutResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_400_port_mapping() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_400_port_mapping",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolPutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                    },
                    vec![
                        InstancePortMapping::InstancePortMappingRange(Box::new(
                            models::InstancePortMappingRange {
                                host_ports: models::PortRange {
                                    start: 2000,
                                    end: 1000,
                                },
                                container_ports: models::PortRange {
                                    start: 6000,
                                    end: 7000,
                                },
                            },
                        ))
                    ],
                )
                .await,
            Ok(InstancesInstanceIdConfigPortsTransportProtocolPutResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_404() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_404",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolPutPathParams {
                        instance_id: "77778888".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                    },
                    vec![],
                )
                .await,
            Ok(InstancesInstanceIdConfigPortsTransportProtocolPutResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_200() {
        let vault = crate::sorcerer::instancius::tests::spell_test_vault(
            module_path!(),
            "put_instance_config_ports_transport_protocol_200",
            None,
        )
        .await;
        let server = ServerImpl { vault };
        let port_mappings = vec![
            InstancePortMapping::InstancePortMappingSingle(Box::new(
                models::InstancePortMappingSingle {
                    host_port: 100,
                    container_port: 20,
                },
            )),
            InstancePortMapping::InstancePortMappingRange(Box::new(
                models::InstancePortMappingRange {
                    host_ports: models::PortRange {
                        start: 2000,
                        end: 3000,
                    },
                    container_ports: models::PortRange {
                        start: 6000,
                        end: 7000,
                    },
                },
            )),
            InstancePortMapping::InstancePortMappingSingle(Box::new(
                models::InstancePortMappingSingle {
                    host_port: 60,
                    container_port: 70,
                },
            )),
        ];
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    InstancesInstanceIdConfigPortsTransportProtocolPutPathParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                    },
                    port_mappings
                )
                .await,
            Ok(InstancesInstanceIdConfigPortsTransportProtocolPutResponse::Status200_PublishedPortsOfInstanceWithThisInstance)
        );
        assert_eq!(
            server
                .vault
                .reservation()
                .reserve_instance_pouch()
                .grab()
                .await
                .instance_pouch
                .as_ref()
                .unwrap()
                .gems()
                .get(&InstanceId::new(6))
                .unwrap()
                .config
                .port_mapping
                .udp,
            vec![
                PortMapping::Single(100, 20),
                PortMapping::Range {
                    from: PortRange::new(2000..=3000),
                    to: PortRange::new(6000..=7000),
                },
                PortMapping::Single(60, 70)
            ]
        );
    }

    #[test]
    fn try_from_port_range_ok() {
        assert_eq!(
            PortRange::try_from(models::PortRange { start: 10, end: 20 }).unwrap(),
            PortRange::new(10..=20)
        );
    }

    #[test]
    fn try_from_port_range_err() {
        assert!(PortRange::try_from(models::PortRange { start: 10, end: 6 }).is_err());
    }

    #[test]
    fn from_port_range_test() {
        assert_eq!(
            models::PortRange::from(PortRange::new(9..=20)),
            models::PortRange { start: 9, end: 20 }
        )
    }

    #[test]
    fn try_from_instance_port_mapping_range_ok() {
        let instance_port_mapping =
            InstancePortMapping::InstancePortMappingRange(Box::new(InstancePortMappingRange {
                host_ports: models::PortRange { start: 7, end: 10 },
                container_ports: models::PortRange { start: 17, end: 20 },
            }));
        let expected_mapping = PortMapping::Range {
            from: PortRange::new(7..=10),
            to: PortRange::new(17..=20),
        };
        assert_eq!(
            PortMapping::try_from(instance_port_mapping).unwrap(),
            expected_mapping
        );
    }

    #[test]
    fn try_from_instance_port_mapping_range_host_err() {
        let instance_port_mapping =
            InstancePortMapping::InstancePortMappingRange(Box::new(InstancePortMappingRange {
                host_ports: models::PortRange { start: 70, end: 20 },
                container_ports: models::PortRange { start: 17, end: 20 },
            }));
        assert!(PortMapping::try_from(instance_port_mapping).is_err(),);
    }

    #[test]
    fn try_from_instance_port_mapping_range_container_err() {
        let instance_port_mapping =
            InstancePortMapping::InstancePortMappingRange(Box::new(InstancePortMappingRange {
                host_ports: models::PortRange { start: 70, end: 80 },
                container_ports: models::PortRange { start: 60, end: 40 },
            }));
        assert!(PortMapping::try_from(instance_port_mapping).is_err(),);
    }

    #[test]
    fn try_from_instance_port_mapping_single_ok() {
        let instance_port_mapping =
            InstancePortMapping::InstancePortMappingSingle(Box::new(InstancePortMappingSingle {
                host_port: 10,
                container_port: 17,
            }));
        let expected_mapping = PortMapping::Single(10, 17);
        assert_eq!(
            PortMapping::try_from(instance_port_mapping).unwrap(),
            expected_mapping
        );
    }

    #[test]
    fn from_port_mapping_range() {
        let port_mapping = PortMapping::Range {
            from: PortRange::new(6..=9),
            to: PortRange::new(11..=14),
        };
        assert_eq!(
            InstancePortMapping::from(&port_mapping),
            InstancePortMapping::InstancePortMappingRange(Box::new(InstancePortMappingRange {
                host_ports: models::PortRange { start: 6, end: 9 },
                container_ports: models::PortRange { start: 11, end: 14 },
            }))
        )
    }

    #[test]
    fn from_port_mapping_single() {
        let port_mapping = PortMapping::Single(100, 1000);
        assert_eq!(
            InstancePortMapping::from(&port_mapping),
            InstancePortMapping::InstancePortMappingSingle(Box::new(InstancePortMappingSingle {
                host_port: 100,
                container_port: 1000,
            }))
        )
    }

    #[test]
    fn port_mappings_to_instance_ports_test() {
        let port_mappings = [
            PortMapping::Single(100, 1000),
            PortMapping::Single(6, 110),
            PortMapping::Range {
                from: PortRange::new(10..=20),
                to: PortRange::new(20..=30),
            },
        ];
        assert_eq!(port_mappings_to_instance_ports(&[]), vec![]);
        assert_eq!(
            port_mappings_to_instance_ports(&port_mappings),
            vec![
                InstancePortMapping::InstancePortMappingSingle(Box::new(
                    InstancePortMappingSingle {
                        host_port: 100,
                        container_port: 1000,
                    }
                )),
                InstancePortMapping::InstancePortMappingSingle(Box::new(
                    InstancePortMappingSingle {
                        host_port: 6,
                        container_port: 110,
                    }
                )),
                InstancePortMapping::InstancePortMappingRange(Box::new(InstancePortMappingRange {
                    host_ports: models::PortRange { start: 10, end: 20 },
                    container_ports: models::PortRange { start: 20, end: 30 },
                }))
            ]
        );
    }

    #[test]
    fn invalid_instance_id_info() {
        assert_eq!(
            invalid_instance_id_additional_info("test_instance_id"),
            AdditionalInfo {
                additional_info: "Invalid instance_id: test_instance_id".to_string()
            }
        );
    }

    #[test]
    fn transport_protocol_from() {
        assert_eq!(
            TransportProtocol::from(models::TransportProtocol::Tcp),
            TransportProtocol::Tcp
        );
        assert_eq!(
            TransportProtocol::from(models::TransportProtocol::Udp),
            TransportProtocol::Udp
        );
        assert_eq!(
            TransportProtocol::from(models::TransportProtocol::Sctp),
            TransportProtocol::Sctp
        );
    }

    #[test]
    fn validate_port_mappings_empty() {
        assert!(validate_port_mappings(&[]).is_ok());
    }

    #[test]
    fn validate_port_mappings_ok() {
        assert!(validate_port_mappings(&[PortMapping::Single(10, 20)]).is_ok());
        assert!(validate_port_mappings(&[PortMapping::Range {
            from: PortRange::new(10..=20),
            to: PortRange::new(70..=80)
        }])
        .is_ok());
        assert!(validate_port_mappings(&[
            PortMapping::Range {
                from: PortRange::new(600..=700),
                to: PortRange::new(800..=900)
            },
            PortMapping::Range {
                from: PortRange::new(10..=20),
                to: PortRange::new(70..=80)
            },
            PortMapping::Single(1, 20),
        ])
        .is_ok());
    }

    #[test]
    fn validate_port_mappings_err_invalid_range() {
        let errors = validate_port_mappings(&[PortMapping::Range {
            from: PortRange::new(10..=20),
            to: PortRange::new(30..=80),
        }])
        .err()
        .unwrap();
        assert_eq!(errors.len(), 1, "{errors:?}");
    }

    #[test]
    fn validate_port_mappings_err_multiple_invalid_range() {
        let errors = validate_port_mappings(&[
            PortMapping::Range {
                from: PortRange::new(10..=20),
                to: PortRange::new(30..=80),
            },
            PortMapping::Range {
                from: PortRange::new(70..=700),
                to: PortRange::new(30..=80),
            },
            PortMapping::Single(1000, 2000),
        ])
        .err()
        .unwrap();
        assert_eq!(errors.len(), 2, "{errors:?}");
    }

    #[test]
    fn validate_port_mappings_err_overlap() {
        let errors = validate_port_mappings(&[
            PortMapping::Range {
                from: PortRange::new(10..=20),
                to: PortRange::new(30..=40),
            },
            PortMapping::Single(15, 50),
        ])
        .err()
        .unwrap();
        assert_eq!(errors.len(), 2, "{errors:?}");
    }

    #[test]
    fn validate_port_mappings_err_multiple_overlap() {
        let errors = validate_port_mappings(&[
            PortMapping::Range {
                from: PortRange::new(10..=20),
                to: PortRange::new(30..=40),
            },
            PortMapping::Range {
                from: PortRange::new(12..=17),
                to: PortRange::new(60..=65),
            },
            PortMapping::Single(15, 50),
        ])
        .err()
        .unwrap();
        assert_eq!(errors.len(), 6, "{errors:?}");
    }

    #[test]
    fn validate_port_mappings_err_multiple() {
        let errors = validate_port_mappings(&[
            PortMapping::Range {
                from: PortRange::new(10..=20),
                to: PortRange::new(30..=80),
            },
            PortMapping::Range {
                from: PortRange::new(12..=17),
                to: PortRange::new(60..=90),
            },
            PortMapping::Single(15, 50),
        ])
        .err()
        .unwrap();
        assert_eq!(errors.len(), 8, "{errors:?}");
    }
}
