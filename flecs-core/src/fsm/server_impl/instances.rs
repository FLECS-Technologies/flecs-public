use crate::fsm::server_impl::ServerImpl;
use crate::jeweler::gem::instance::InstanceId;
use crate::quest::{Quest, QuestResult};
use crate::vault::pouch::AppKey;
use async_trait::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecsd_axum_server::apis::instances::{
    Instances, InstancesCreatePostResponse, InstancesGetResponse,
    InstancesInstanceIdConfigEnvironmentDeleteResponse,
    InstancesInstanceIdConfigEnvironmentGetResponse,
    InstancesInstanceIdConfigEnvironmentPutResponse, InstancesInstanceIdConfigGetResponse,
    InstancesInstanceIdConfigPortsDeleteResponse, InstancesInstanceIdConfigPortsGetResponse,
    InstancesInstanceIdConfigPortsTransportProtocolDeleteResponse,
    InstancesInstanceIdConfigPortsTransportProtocolGetResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeleteResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutResponse,
    InstancesInstanceIdConfigPortsTransportProtocolPutResponse,
    InstancesInstanceIdConfigPostResponse, InstancesInstanceIdDeleteResponse,
    InstancesInstanceIdEditorPortGetResponse, InstancesInstanceIdGetResponse,
    InstancesInstanceIdLogsGetResponse, InstancesInstanceIdPatchResponse,
    InstancesInstanceIdStartPostResponse, InstancesInstanceIdStopPostResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    AdditionalInfo, InstanceConfig, InstanceEnvironment, InstancePortMapping,
    InstancesCreatePostRequest, InstancesGetQueryParams,
    InstancesInstanceIdConfigEnvironmentDeletePathParams,
    InstancesInstanceIdConfigEnvironmentGetPathParams,
    InstancesInstanceIdConfigEnvironmentPutPathParams, InstancesInstanceIdConfigGetPathParams,
    InstancesInstanceIdConfigPortsDeletePathParams, InstancesInstanceIdConfigPortsGetPathParams,
    InstancesInstanceIdConfigPortsTransportProtocolDeletePathParams,
    InstancesInstanceIdConfigPortsTransportProtocolGetPathParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeletePathParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetPathParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutPathParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutRequest,
    InstancesInstanceIdConfigPortsTransportProtocolPutPathParams,
    InstancesInstanceIdConfigPostPathParams, InstancesInstanceIdDeletePathParams,
    InstancesInstanceIdEditorPortGetPathParams, InstancesInstanceIdGetPathParams,
    InstancesInstanceIdLogsGetPathParams, InstancesInstanceIdPatchPathParams,
    InstancesInstanceIdPatchRequest, InstancesInstanceIdStartPostPathParams,
    InstancesInstanceIdStopPostPathParams, JobMeta,
};
use http::Method;
use std::str::FromStr;

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

    async fn instances_instance_id_config_ports_transport_protocol_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigPortsTransportProtocolDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigPortsTransportProtocolDeleteResponse, ()> {
        todo!()
    }

    async fn instances_instance_id_config_ports_transport_protocol_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigPortsTransportProtocolGetPathParams,
    ) -> Result<InstancesInstanceIdConfigPortsTransportProtocolGetResponse, ()> {
        todo!()
    }

    async fn instances_instance_id_config_ports_transport_protocol_host_port_range_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeleteResponse, ()>
    {
        todo!()
    }

    async fn instances_instance_id_config_ports_transport_protocol_host_port_range_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetPathParams,
    ) -> Result<InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetResponse, ()> {
        todo!()
    }

    async fn instances_instance_id_config_ports_transport_protocol_host_port_range_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutPathParams,
        _body: InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutRequest,
    ) -> Result<InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutResponse, ()> {
        todo!()
    }

    async fn instances_instance_id_config_ports_transport_protocol_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: InstancesInstanceIdConfigPortsTransportProtocolPutPathParams,
        _body: Vec<InstancePortMapping>,
    ) -> Result<InstancesInstanceIdConfigPortsTransportProtocolPutResponse, ()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsm::server_impl::ServerImpl;
    use crate::jeweler::gem::app::{try_create_app, AppDeserializable};
    use crate::tests::prepare_test_path;
    use crate::vault::pouch::Pouch;
    use crate::vault::{Vault, VaultConfig};
    use axum::extract::Host;
    use axum_extra::extract::CookieJar;
    use flecsd_axum_server::apis::instances::{
        Instances, InstancesInstanceIdLogsGetResponse, InstancesInstanceIdStartPostResponse,
        InstancesInstanceIdStopPostResponse,
    };
    use flecsd_axum_server::models::{
        AppKey, InstancesInstanceIdLogsGetPathParams, InstancesInstanceIdStartPostPathParams,
        InstancesInstanceIdStopPostPathParams,
    };
    use http::Method;
    use std::collections::HashMap;
    use std::sync::Arc;

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
