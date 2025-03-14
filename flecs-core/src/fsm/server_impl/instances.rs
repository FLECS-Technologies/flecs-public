use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::fsm::server_impl::ServerImpl;
use crate::jeweler::gem::instance::{InstanceId, TransportProtocol, UsbPathConfig};
use crate::jeweler::gem::manifest::{EnvironmentVariable, Label, PortMapping, PortRange};
use crate::quest::{Quest, QuestResult};
use crate::relic::device::usb::{UsbDevice, UsbDeviceReader};
use crate::sorcerer::appraiser::AppRaiser;
use crate::sorcerer::authmancer::Authmancer;
use crate::sorcerer::instancius::{
    GetInstanceUsbDeviceResult, Instancius, PutInstanceUsbDeviceResult,
};
use crate::sorcerer::licenso::Licenso;
use crate::sorcerer::mage_quester::MageQuester;
use crate::sorcerer::manifesto::Manifesto;
use crate::sorcerer::systemus::Systemus;
use crate::vault::pouch::AppKey;
use anyhow::Error;
use async_trait::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecsd_axum_server::apis::instances::{
    Instances, InstancesCreatePostResponse, InstancesGetResponse,
    InstancesInstanceIdConfigDevicesUsbDeleteResponse,
    InstancesInstanceIdConfigDevicesUsbGetResponse,
    InstancesInstanceIdConfigDevicesUsbPortDeleteResponse,
    InstancesInstanceIdConfigDevicesUsbPortGetResponse,
    InstancesInstanceIdConfigDevicesUsbPortPutResponse,
    InstancesInstanceIdConfigEnvironmentDeleteResponse as DeleteEnvironmentResponse,
    InstancesInstanceIdConfigEnvironmentGetResponse as GetEnvironmentResponse,
    InstancesInstanceIdConfigEnvironmentPutResponse as PutEnvironmentResponse,
    InstancesInstanceIdConfigEnvironmentVariableNameDeleteResponse as DeleteEnvironmentVariableResponse,
    InstancesInstanceIdConfigEnvironmentVariableNameGetResponse as GetEnvironmentVariableResponse,
    InstancesInstanceIdConfigEnvironmentVariableNamePutResponse as PutEnvironmentVariableResponse,
    InstancesInstanceIdConfigGetResponse,
    InstancesInstanceIdConfigLabelsGetResponse as GetLabelsResponse,
    InstancesInstanceIdConfigLabelsLabelNameGetResponse as GetLabelResponse,
    InstancesInstanceIdConfigPortsDeleteResponse as DeletePortsResponse,
    InstancesInstanceIdConfigPortsGetResponse as GetPortsResponse,
    InstancesInstanceIdConfigPortsTransportProtocolDeleteResponse as DeleteProtocolPortsResponse,
    InstancesInstanceIdConfigPortsTransportProtocolGetResponse as GetProtocolPortsResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeleteResponse as DeletePortRangeResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetResponse as GetPortRangeResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutResponse as PutPortRangeResponse,
    InstancesInstanceIdConfigPortsTransportProtocolPutResponse as PutProtocolPortsResponse,
    InstancesInstanceIdConfigPostResponse, InstancesInstanceIdDeleteResponse,
    InstancesInstanceIdEditorPortGetResponse, InstancesInstanceIdGetResponse,
    InstancesInstanceIdLogsGetResponse, InstancesInstanceIdPatchResponse,
    InstancesInstanceIdStartPostResponse, InstancesInstanceIdStopPostResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    AdditionalInfo, InstanceConfig, InstanceEnvironment, InstancesCreatePostRequest,
    InstancesGetQueryParams, InstancesInstanceIdConfigDevicesUsbDeletePathParams,
    InstancesInstanceIdConfigDevicesUsbGetPathParams,
    InstancesInstanceIdConfigDevicesUsbPortDeletePathParams,
    InstancesInstanceIdConfigDevicesUsbPortGetPathParams,
    InstancesInstanceIdConfigDevicesUsbPortPutPathParams,
    InstancesInstanceIdConfigEnvironmentDeletePathParams as DeleteEnvironmentParams,
    InstancesInstanceIdConfigEnvironmentGetPathParams as GetEnvironmentParams,
    InstancesInstanceIdConfigEnvironmentPutPathParams as PutEnvironmentParams,
    InstancesInstanceIdConfigEnvironmentVariableNameDeletePathParams as DeleteEnvironmentVariableParams,
    InstancesInstanceIdConfigEnvironmentVariableNameGet200Response as PutEnvironmentVariableRequest,
    InstancesInstanceIdConfigEnvironmentVariableNameGetPathParams as GetEnvironmentVariableParams,
    InstancesInstanceIdConfigEnvironmentVariableNamePutPathParams as PutEnvironmentVariableParams,
    InstancesInstanceIdConfigGetPathParams,
    InstancesInstanceIdConfigLabelsGetPathParams as GetLabelsParams,
    InstancesInstanceIdConfigLabelsLabelNameGetPathParams as GetLabelParams,
    InstancesInstanceIdConfigPortsDeletePathParams as DeletePortsParams,
    InstancesInstanceIdConfigPortsGetPathParams as GetPortsParams,
    InstancesInstanceIdConfigPortsTransportProtocolDeletePathParams as DeleteProtocolPortsParams,
    InstancesInstanceIdConfigPortsTransportProtocolGetPathParams as GetProtocolPortsParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeletePathParams as DeletePortRangeParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetPathParams as GetPortRangeParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutPathParams as PutPortRangeParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutRequest as PutPortRangeRequest,
    InstancesInstanceIdConfigPortsTransportProtocolPutPathParams as PutProtocolPortsParams,
    InstancesInstanceIdConfigPostPathParams, InstancesInstanceIdDeletePathParams,
    InstancesInstanceIdEditorPortGetPathParams, InstancesInstanceIdGetPathParams,
    InstancesInstanceIdLogsGetPathParams, InstancesInstanceIdPatchPathParams,
    InstancesInstanceIdPatchRequest, InstancesInstanceIdStartPostPathParams,
    InstancesInstanceIdStopPostPathParams, JobMeta, OptionalAdditionalInfo,
};
use http::Method;
use std::collections::HashSet;
use std::num::NonZeroU16;
use std::str::FromStr;

#[async_trait]
impl<
        APP: AppRaiser,
        AUTH: Authmancer,
        I: Instancius + 'static,
        L: Licenso,
        Q: MageQuester,
        M: Manifesto,
        SYS: Systemus,
        F: Floxy + 'static,
        T: UsbDeviceReader + 'static,
    > Instances for ServerImpl<APP, AUTH, I, L, Q, M, SYS, F, T>
{
    async fn instances_create_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: InstancesCreatePostRequest,
    ) -> Result<InstancesCreatePostResponse, ()> {
        let app_key: AppKey = body.app_key.into();
        if !self
            .sorcerers
            .app_raiser
            .does_app_exist(self.vault.clone(), app_key.clone())
            .await
        {
            return Ok(InstancesCreatePostResponse::Status400_MalformedRequest(
                AdditionalInfo {
                    additional_info: format!("App {app_key} does not exist"),
                },
            ));
        }
        let vault = self.vault.clone();
        let instance_name = body.instance_name;
        let instancius = self.sorcerers.instancius.clone();
        let (id, _quest) = crate::lore::quest::default()
            .await
            .lock()
            .await
            .schedule_quest_with_result(
                format!("Create instance for {app_key}"),
                |quest| async move {
                    let id = instancius
                        .create_instance(quest, vault, app_key, instance_name.unwrap_or_default())
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
                self.sorcerers
                    .instancius
                    .get_all_instances(
                        Quest::new_synced("Get info for all instances".to_string()),
                        self.vault.clone(),
                    )
                    .await
            }
            InstancesGetQueryParams { version, app } => {
                self.sorcerers
                    .instancius
                    .get_instances_filtered(
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

    async fn instances_instance_id_config_devices_usb_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigDevicesUsbDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigDevicesUsbDeleteResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        match self.sorcerers.instancius.delete_instance_usb_devices(self.vault.clone(), instance_id)
            .await
        {
            Some(_) => Ok(InstancesInstanceIdConfigDevicesUsbDeleteResponse::Status200_Success),
            None => Ok(InstancesInstanceIdConfigDevicesUsbDeleteResponse::Status404_NoInstanceWithThisInstance),
        }
    }

    async fn instances_instance_id_config_devices_usb_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigDevicesUsbGetPathParams,
    ) -> Result<InstancesInstanceIdConfigDevicesUsbGetResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        match self.sorcerers.instancius.get_instance_usb_devices(self.vault.clone(), instance_id, self.usb_reader.clone())
            .await
        {
            Err(e) => Ok(
                InstancesInstanceIdConfigDevicesUsbGetResponse::Status500_InternalServerError(
                    AdditionalInfo {
                        additional_info: e.to_string(),
                    },
                ),
            ),
            Ok(None) => Ok(InstancesInstanceIdConfigDevicesUsbGetResponse::Status404_NoInstanceWithThisInstance),
            Ok(Some(usb_devices)) => {
                Ok(InstancesInstanceIdConfigDevicesUsbGetResponse::Status200_Success(
                    usb_devices.into_iter().map(instance_config_usb_device_from).collect(),
                ))
            }
        }
    }

    async fn instances_instance_id_config_devices_usb_port_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigDevicesUsbPortDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigDevicesUsbPortDeleteResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        match self
            .sorcerers
            .instancius
            .delete_instance_usb_device(self.vault.clone(), instance_id, path_params.port.clone())
            .await
        {
            Some(Some(_)) => {
                Ok(InstancesInstanceIdConfigDevicesUsbPortDeleteResponse::Status200_Success)
            }
            Some(None) => Ok(
                InstancesInstanceIdConfigDevicesUsbPortDeleteResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo {
                        additional_info: Some(format!("No instance with id {instance_id}")),
                    },
                ),
            ),
            None => Ok(
                InstancesInstanceIdConfigDevicesUsbPortDeleteResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo {
                        additional_info: Some(format!(
                            "Usb port '{}' not mapped to instance {instance_id}",
                            path_params.port
                        )),
                    },
                ),
            ),
        }
    }

    async fn instances_instance_id_config_devices_usb_port_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigDevicesUsbPortGetPathParams,
    ) -> Result<InstancesInstanceIdConfigDevicesUsbPortGetResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        match self.sorcerers.instancius.get_instance_usb_device(
            self.vault.clone(),
            instance_id,
            path_params.port.clone(),
            self.usb_reader.clone(),
        )
        .await
        {
            Ok(GetInstanceUsbDeviceResult::DeviceActive(config, device)) => Ok(
                InstancesInstanceIdConfigDevicesUsbPortGetResponse::Status200_Success(
                    instance_config_usb_device_from((config, Some(device))),
                ),
            ),
            Ok(GetInstanceUsbDeviceResult::DeviceInactive(config)) => Ok(
                InstancesInstanceIdConfigDevicesUsbPortGetResponse::Status200_Success(
                    instance_config_usb_device_from((config, None)),
                ),
            ),
            Ok(GetInstanceUsbDeviceResult::InstanceNotFound) => Ok(
                InstancesInstanceIdConfigDevicesUsbPortGetResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo {
                        additional_info: Some(format!("No instance with id {instance_id}")),
                    },
                ),
            ),
            Ok(GetInstanceUsbDeviceResult::DeviceNotMapped) => Ok(
                InstancesInstanceIdConfigDevicesUsbPortGetResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo {
                        additional_info: Some(format!(
                            "Usb port '{}' not mapped to instance {instance_id}",
                            path_params.port
                        )),
                    },
                ),
            ),
            Ok(GetInstanceUsbDeviceResult::UnknownDevice) => Ok(
                InstancesInstanceIdConfigDevicesUsbPortGetResponse::Status404_ResourceNotFound(
                    OptionalAdditionalInfo {
                        additional_info: Some(format!(
                            "Usb port '{}' not mapped to instance {instance_id} and not corresponding to any known device",
                            path_params.port
                        )),
                    },
                ),
            ),
            Err(e) => Ok(
                InstancesInstanceIdConfigDevicesUsbPortGetResponse::Status500_InternalServerError(
                    AdditionalInfo::new(e.to_string()),
                ),
            ),
        }
    }

    async fn instances_instance_id_config_devices_usb_port_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigDevicesUsbPortPutPathParams,
    ) -> Result<InstancesInstanceIdConfigDevicesUsbPortPutResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        match self.sorcerers.instancius.put_instance_usb_device(
            self.vault.clone(),
            instance_id,
            path_params.port.clone(),
            self.usb_reader.clone(),
        )
        .await
        {
            Ok(PutInstanceUsbDeviceResult::InstanceNotFound) => Ok(
                InstancesInstanceIdConfigDevicesUsbPortPutResponse::Status404_ResourceNotFound(OptionalAdditionalInfo{
                    additional_info: Some(format!("No instance with id {instance_id}")),
                }),
            ),
            Ok(PutInstanceUsbDeviceResult::DeviceNotFound) => Ok(
                InstancesInstanceIdConfigDevicesUsbPortPutResponse::Status404_ResourceNotFound(OptionalAdditionalInfo{
                    additional_info: Some(format!("No usb device with port {}", path_params.port)),
                }),
            ),
            Ok(PutInstanceUsbDeviceResult::DeviceMappingCreated) => Ok(
                InstancesInstanceIdConfigDevicesUsbPortPutResponse::Status201_UsbDeviceWasPassedThrough,
            ),
            Ok(PutInstanceUsbDeviceResult::DeviceMappingUpdated(_)) => Ok(
                InstancesInstanceIdConfigDevicesUsbPortPutResponse::Status200_AlreadyPassedThrough,
            ),
            Err(e) => Ok(
                InstancesInstanceIdConfigDevicesUsbPortPutResponse::Status500_InternalServerError(
                    AdditionalInfo::new(e.to_string()),
                ),
            ),
        }
    }

    async fn instances_instance_id_config_environment_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: DeleteEnvironmentParams,
    ) -> Result<DeleteEnvironmentResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        match self
            .sorcerers
            .instancius
            .delete_instance_config_environment(self.vault.clone(), instance_id)
            .await
        {
            None => Ok(DeleteEnvironmentResponse::Status404_NoInstanceWithThisInstance),
            Some(_) => {
                Ok(DeleteEnvironmentResponse::Status200_EnvironmentOfInstanceWithThisInstance)
            }
        }
    }

    async fn instances_instance_id_config_environment_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: GetEnvironmentParams,
    ) -> Result<GetEnvironmentResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        match self
            .sorcerers
            .instancius
            .get_instance_config_environment(self.vault.clone(), instance_id)
            .await
        {
            None => Ok(GetEnvironmentResponse::Status404_NoInstanceWithThisInstance),
            Some(environment) => Ok(GetEnvironmentResponse::Status200_Success(
                InstanceEnvironment::from(
                    environment
                        .into_iter()
                        .map(models::InstanceEnvironmentVariable::from)
                        .collect::<Vec<models::InstanceEnvironmentVariable>>(),
                ),
            )),
        }
    }

    async fn instances_instance_id_config_environment_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: PutEnvironmentParams,
        body: InstanceEnvironment,
    ) -> Result<PutEnvironmentResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        let environment: Vec<_> = body.into_iter().map(EnvironmentVariable::from).collect();
        if let Err(errors) = validate_environment_variables(&environment) {
            return Ok(PutEnvironmentResponse::Status400_MalformedRequest(
                AdditionalInfo {
                    additional_info: errors.join("\n"),
                },
            ));
        };
        match self.sorcerers.instancius.put_instance_config_environment(self.vault.clone(), instance_id, environment).await {
            None => Ok(PutEnvironmentResponse::Status404_NoInstanceWithThisInstance),
            Some(previous_environment) if previous_environment.is_empty() => Ok(PutEnvironmentResponse::Status201_EnvironmentForInstanceWithThisInstanceIdWasCreated),
            Some(_) => Ok(PutEnvironmentResponse::Status200_EnvironmentForInstanceWithThisInstanceIdIsSet),
        }
    }

    async fn instances_instance_id_config_environment_variable_name_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: DeleteEnvironmentVariableParams,
    ) -> Result<DeleteEnvironmentVariableResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        match self.sorcerers.instancius.delete_instance_config_environment_variable_value(
            self.vault.clone(), instance_id, path_params.variable_name.clone())
            .await
        {
            None => Ok(DeleteEnvironmentVariableResponse::Status404_ResourceNotFound(
                OptionalAdditionalInfo {
                    additional_info: Some(format!("No instance with id {instance_id}"))
                })),
            Some(None) => Ok(DeleteEnvironmentVariableResponse::Status404_ResourceNotFound(
                OptionalAdditionalInfo {
                    additional_info: Some(format!("No environment variable with name {}", path_params.variable_name))
                })),
            Some(Some(_)) => {
                Ok(DeleteEnvironmentVariableResponse::Status200_EnvironmentVariableOfInstanceWithThisInstance)
            }
        }
    }

    async fn instances_instance_id_config_environment_variable_name_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: GetEnvironmentVariableParams,
    ) -> Result<GetEnvironmentVariableResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        match self
            .sorcerers
            .instancius
            .get_instance_config_environment_variable_value(
                self.vault.clone(),
                instance_id,
                path_params.variable_name.clone(),
            )
            .await
        {
            None => Ok(GetEnvironmentVariableResponse::Status404_ResourceNotFound(
                OptionalAdditionalInfo {
                    additional_info: Some(format!("No instance with id {instance_id}")),
                },
            )),
            Some(None) => Ok(GetEnvironmentVariableResponse::Status404_ResourceNotFound(
                OptionalAdditionalInfo {
                    additional_info: Some(format!(
                        "No environment variable with name {}",
                        path_params.variable_name
                    )),
                },
            )),
            Some(Some(value)) => Ok(GetEnvironmentVariableResponse::Status200_Success(
                models::InstancesInstanceIdConfigEnvironmentVariableNameGet200Response { value },
            )),
        }
    }

    async fn instances_instance_id_config_environment_variable_name_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: PutEnvironmentVariableParams,
        body: PutEnvironmentVariableRequest,
    ) -> Result<PutEnvironmentVariableResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        match self.sorcerers.instancius.put_instance_config_environment_variable_value(
            self.vault.clone(),
            instance_id,
            EnvironmentVariable {
                name: path_params.variable_name,
                value: body.value,
            },
        )
            .await {
            None => Ok(PutEnvironmentVariableResponse::Status404_NoInstanceWithThisInstance),
            Some(None) => Ok(PutEnvironmentVariableResponse::Status201_EnvironmentForInstanceWithThisInstanceIdWasCreated),
            Some(Some(_)) => Ok(PutEnvironmentVariableResponse::Status200_EnvironmentForInstanceWithThisInstanceIdIsSet),
        }
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

    async fn instances_instance_id_config_labels_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: GetLabelsParams,
    ) -> Result<GetLabelsResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        match self
            .sorcerers
            .instancius
            .get_instance_labels(self.vault.clone(), instance_id)
            .await
        {
            None => Ok(GetLabelsResponse::Status404_NoInstanceWithThisInstance),
            Some(labels) => Ok(GetLabelsResponse::Status200_Success(
                labels
                    .into_iter()
                    .map(models::InstanceLabel::from)
                    .collect(),
            )),
        }
    }

    async fn instances_instance_id_config_labels_label_name_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: GetLabelParams,
    ) -> Result<GetLabelResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        match self
            .sorcerers
            .instancius
            .get_instance_label_value(
                self.vault.clone(),
                instance_id,
                path_params.label_name.clone(),
            )
            .await
        {
            None => Ok(GetLabelResponse::Status404_ResourceNotFound(
                OptionalAdditionalInfo {
                    additional_info: Some(format!("No instance with id {}", instance_id)),
                },
            )),
            Some(None) => Ok(GetLabelResponse::Status404_ResourceNotFound(
                OptionalAdditionalInfo {
                    additional_info: Some(format!(
                        "No environment label with name {}",
                        path_params.label_name
                    )),
                },
            )),
            Some(Some(value)) => Ok(GetLabelResponse::Status200_Success(
                models::InstancesInstanceIdConfigLabelsLabelNameGet200Response { value },
            )),
        }
    }

    async fn instances_instance_id_config_ports_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: DeletePortsParams,
    ) -> Result<DeletePortsResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        if self
            .sorcerers
            .instancius
            .delete_instance_config_port_mappings(self.vault.clone(), instance_id)
            .await
        {
            Ok(DeletePortsResponse::Status200_ExposedPortsOfInstanceWithThisInstance)
        } else {
            Ok(DeletePortsResponse::Status404_NoInstanceWithThisInstance)
        }
    }

    async fn instances_instance_id_config_ports_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: GetPortsParams,
    ) -> Result<GetPortsResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        match self
            .sorcerers
            .instancius
            .get_instance_config_port_mappings(self.vault.clone(), instance_id)
            .await
        {
            None => Ok(GetPortsResponse::Status404_NoInstanceWithThisInstance),
            Some(mapping) => Ok(GetPortsResponse::Status200_Success(models::InstancePorts {
                tcp: port_mappings_to_instance_ports(&mapping.tcp),
                udp: port_mappings_to_instance_ports(&mapping.udp),
                sctp: port_mappings_to_instance_ports(&mapping.sctp),
            })),
        }
    }

    async fn instances_instance_id_config_ports_transport_protocol_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: DeleteProtocolPortsParams,
    ) -> Result<DeleteProtocolPortsResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        match self.sorcerers.instancius.delete_instance_config_protocol_port_mappings(
            self.vault.clone(),
            instance_id,
            path_params.transport_protocol.into(),
        )
        .await
        {
            Some(_) => Ok(DeleteProtocolPortsResponse::Status200_RemovedAllPublishedPortsOfInstanceWithThisInstance),
            None => Ok(DeleteProtocolPortsResponse::Status404_ResourceNotFound(
                OptionalAdditionalInfo::new(),
            ))
        }
    }

    async fn instances_instance_id_config_ports_transport_protocol_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: GetProtocolPortsParams,
    ) -> Result<GetProtocolPortsResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        if let Some(port_mapping) = self
            .sorcerers
            .instancius
            .get_instance_config_protocol_port_mappings(
                self.vault.clone(),
                instance_id,
                path_params.transport_protocol.into(),
            )
            .await
        {
            Ok(
                GetProtocolPortsResponse::Status200_PublishedPortsForInstanceWithThisInstance(
                    port_mappings_to_instance_ports(&port_mapping),
                ),
            )
        } else {
            Ok(GetProtocolPortsResponse::Status404_ResourceNotFound(
                OptionalAdditionalInfo::new(),
            ))
        }
    }

    async fn instances_instance_id_config_ports_transport_protocol_host_port_range_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: DeletePortRangeParams,
    ) -> Result<DeletePortRangeResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        let host_port_range = match parse_host_port_path_parameter(&path_params.host_port_range) {
            Ok(host_port_range) => host_port_range,
            Err(e) => return Ok(DeletePortRangeResponse::Status400_MalformedRequest(e)),
        };
        match self
            .sorcerers
            .instancius
            .delete_instance_config_port_mapping_range(
                self.vault.clone(),
                instance_id,
                host_port_range,
                path_params.transport_protocol.into(),
            )
            .await
        {
            None => Ok(DeletePortRangeResponse::Status404_ResourceNotFound(
                OptionalAdditionalInfo {
                    additional_info: Some(format!("Instance {instance_id} does not exist")),
                },
            )),
            Some(false) => Ok(DeletePortRangeResponse::Status404_ResourceNotFound(
                OptionalAdditionalInfo {
                    additional_info: Some(format!(
                        "Host port range ({}) is not mapped to {instance_id}",
                        host_port_range
                    )),
                },
            )),
            Some(true) => Ok(DeletePortRangeResponse::Status200_Success),
        }
    }

    async fn instances_instance_id_config_ports_transport_protocol_host_port_range_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: GetPortRangeParams,
    ) -> Result<GetPortRangeResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        let host_port_range = match parse_host_port_path_parameter(&path_params.host_port_range) {
            Ok(host_port_range) => host_port_range,
            Err(e) => return Ok(GetPortRangeResponse::Status400_MalformedRequest(e)),
        };
        match self
            .sorcerers
            .instancius
            .get_instance_config_port_mapping_range(
                self.vault.clone(),
                instance_id,
                host_port_range,
                path_params.transport_protocol.into(),
            )
            .await
        {
            None => Ok(GetPortRangeResponse::Status404_ResourceNotFound(
                OptionalAdditionalInfo {
                    additional_info: Some(format!("Instance {instance_id} does not exist")),
                },
            )),
            Some(None) => Ok(GetPortRangeResponse::Status404_ResourceNotFound(
                OptionalAdditionalInfo {
                    additional_info: Some(format!(
                        "Host port range ({}) is not mapped to {instance_id}",
                        host_port_range
                    )),
                },
            )),
            Some(Some(PortMapping::Single(host_port, container_port))) => {
                Ok(GetPortRangeResponse::Status200_Success(
                    models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                        models::InstancePortMappingSingle {
                            host_port,
                            container_port,
                        },
                    )),
                ))
            }
            Some(Some(PortMapping::Range { from, to })) => {
                Ok(GetPortRangeResponse::Status200_Success(
                    models::InstancePortMapping::InstancePortMappingRange(Box::new(
                        models::InstancePortMappingRange {
                            host_ports: from.into(),
                            container_ports: to.into(),
                        },
                    )),
                ))
            }
        }
    }

    async fn instances_instance_id_config_ports_transport_protocol_host_port_range_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: PutPortRangeParams,
        body: PutPortRangeRequest,
    ) -> Result<PutPortRangeResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        let host_port_range = match parse_host_port_path_parameter(&path_params.host_port_range) {
            Ok(host_port_range) => host_port_range,
            Err(e) => return Ok(PutPortRangeResponse::Status400_MalformedRequest(e)),
        };
        let container_port_range = match PortRange::try_from(body.clone()) {
            Err(e) => {
                return Ok(PutPortRangeResponse::Status400_MalformedRequest(
                    AdditionalInfo::new(format!("Invalid container port range: {e}")),
                ))
            }
            Ok(host_port_range) => host_port_range,
        };
        if container_port_range.range().len() != host_port_range.range().len() {
            return Ok(PutPortRangeResponse::Status400_MalformedRequest(
                AdditionalInfo::new(format!(
                    "The size of the container port range ({container_port_range}) \
                        and host port range ({host_port_range}) has to be equal",
                )),
            ));
        }
        match self
            .sorcerers
            .instancius
            .put_instance_config_port_mapping(
                self.vault.clone(),
                instance_id,
                PortMapping::Range {
                    from: host_port_range,
                    to: container_port_range,
                }
                .normalize(),
                path_params.transport_protocol.into(),
            )
            .await
        {
            Err(e) => Ok(PutPortRangeResponse::Status400_MalformedRequest(
                AdditionalInfo::new(e.to_string()),
            )),
            Ok(None) => Ok(PutPortRangeResponse::Status404_ResourceNotFound(
                OptionalAdditionalInfo {
                    additional_info: Some(format!("Instance {instance_id} does not exist")),
                },
            )),
            Ok(Some(false)) => {
                Ok(PutPortRangeResponse::Status201_TheSpecifiedPortMappingWasCreated)
            }
            Ok(Some(true)) => Ok(PutPortRangeResponse::Status200_TheSpecifiedPortMappingWasSet),
        }
    }

    async fn instances_instance_id_config_ports_transport_protocol_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: PutProtocolPortsParams,
        body: Vec<models::InstancePortMapping>,
    ) -> Result<PutProtocolPortsResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        let port_mapping = match body
            .into_iter()
            .map(PortMapping::try_from)
            .collect::<Result<Vec<_>, _>>()
        {
            Err(e) => {
                return Ok(PutProtocolPortsResponse::Status400_MalformedRequest(
                    AdditionalInfo::new(format!("Invalid port mapping: {e}")),
                ))
            }
            Ok(port_mapping) => port_mapping,
        };
        if let Err(errors) = validate_port_mappings(&port_mapping) {
            return Ok(PutProtocolPortsResponse::Status400_MalformedRequest(
                AdditionalInfo::new(format!("Invalid port mapping: {}", errors.join("\n"))),
            ));
        }
        let instance_found = self
            .sorcerers
            .instancius
            .put_instance_config_protocol_port_mappings(
                self.vault.clone(),
                instance_id,
                port_mapping,
                path_params.transport_protocol.into(),
            )
            .await;
        if instance_found {
            Ok(PutProtocolPortsResponse::Status200_PublishedPortsOfInstanceWithThisInstance)
        } else {
            Ok(PutProtocolPortsResponse::Status404_ResourceNotFound(
                OptionalAdditionalInfo::new(),
            ))
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
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        if !self
            .sorcerers
            .instancius
            .does_instance_exist(self.vault.clone(), instance_id)
            .await
        {
            return Ok(InstancesInstanceIdDeleteResponse::Status404_NoInstanceWithThisInstance);
        }
        let vault = self.vault.clone();
        let floxy = FloxyOperation::new_arc(self.enchantments.floxy.clone());
        let instancius = self.sorcerers.instancius.clone();
        let quest_id = crate::lore::quest::default()
            .await
            .lock()
            .await
            .schedule_quest(
                format!("Delete instance {instance_id}"),
                move |quest| async move {
                    instancius
                        .delete_instance(quest, vault, floxy, instance_id)
                        .await
                },
            )
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
        host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdEditorPortGetPathParams,
    ) -> Result<InstancesInstanceIdEditorPortGetResponse, ()> {
        let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
        let port = NonZeroU16::new(path_params.port as u16).unwrap();
        super::route_impl::instances::instance_id::editor::port::get(
            self.vault.clone(),
            self.enchantments.floxy.clone(),
            self.sorcerers.instancius.clone(),
            host,
            instance_id,
            port,
        )
        .await
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
        match self
            .sorcerers
            .instancius
            .get_instance_detailed(self.vault.clone(), instance_id)
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
        if !self
            .sorcerers
            .instancius
            .does_instance_exist(self.vault.clone(), instance_id)
            .await
        {
            return Ok(InstancesInstanceIdLogsGetResponse::Status404_NoInstanceWithThisInstance);
        }
        match self
            .sorcerers
            .instancius
            .get_instance_logs(self.vault.clone(), instance_id)
            .await
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
        if !self
            .sorcerers
            .instancius
            .does_instance_exist(self.vault.clone(), instance_id)
            .await
        {
            return Ok(InstancesInstanceIdStartPostResponse::Status404_NoInstanceWithThisInstance);
        }
        let vault = self.vault.clone();
        let floxy = FloxyOperation::new_arc(self.enchantments.floxy.clone());
        let instancius = self.sorcerers.instancius.clone();
        let quest_id = crate::lore::quest::default()
            .await
            .lock()
            .await
            .schedule_quest(
                format!("Start instance {instance_id}"),
                move |quest| async move {
                    instancius
                        .start_instance(quest, vault, floxy, instance_id)
                        .await
                },
            )
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
        if !self
            .sorcerers
            .instancius
            .does_instance_exist(self.vault.clone(), instance_id)
            .await
        {
            return Ok(InstancesInstanceIdStopPostResponse::Status404_NoInstanceWithThisInstance);
        }
        let vault = self.vault.clone();
        let floxy = FloxyOperation::new_arc(self.enchantments.floxy.clone());
        let instancius = self.sorcerers.instancius.clone();
        let quest_id = crate::lore::quest::default()
            .await
            .lock()
            .await
            .schedule_quest(
                format!("Stop instance {instance_id}"),
                move |quest| async move {
                    instancius
                        .stop_instance(quest, vault, floxy, instance_id)
                        .await
                },
            )
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

impl TryFrom<models::InstancePortMapping> for PortMapping {
    type Error = Error;

    fn try_from(value: models::InstancePortMapping) -> Result<Self, Self::Error> {
        match value {
            models::InstancePortMapping::InstancePortMappingRange(mapping) => Ok(Self::Range {
                from: PortRange::try_from(mapping.host_ports)?,
                to: PortRange::try_from(mapping.container_ports)?,
            }),
            models::InstancePortMapping::InstancePortMappingSingle(mapping) => {
                Ok(Self::Single(mapping.host_port, mapping.container_port))
            }
        }
    }
}

impl TryFrom<PutPortRangeRequest> for PortRange {
    type Error = Error;

    fn try_from(value: PutPortRangeRequest) -> Result<Self, Self::Error> {
        match value {
            PutPortRangeRequest::PortRange(range) => Self::try_from(*range),
            PutPortRangeRequest::I32(port) => {
                let port = u16::try_from(*port)?;
                Ok(Self::new(port..=port))
            }
        }
    }
}

impl From<&PortMapping> for models::InstancePortMapping {
    fn from(value: &PortMapping) -> Self {
        match value {
            PortMapping::Single(host, container) => {
                models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                    models::InstancePortMappingSingle {
                        host_port: *host,
                        container_port: *container,
                    },
                ))
            }
            PortMapping::Range { from, to } => {
                models::InstancePortMapping::InstancePortMappingRange(Box::new(
                    models::InstancePortMappingRange {
                        host_ports: models::PortRange {
                            start: *from.range().start(),
                            end: *from.range().end(),
                        },
                        container_ports: models::PortRange {
                            start: *to.range().start(),
                            end: *to.range().end(),
                        },
                    },
                ))
            }
        }
    }
}

fn port_mappings_to_instance_ports(
    port_mappings: &[PortMapping],
) -> Vec<models::InstancePortMapping> {
    port_mappings
        .iter()
        .map(models::InstancePortMapping::from)
        .collect()
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

fn parse_host_port_path_parameter(path_parameter: &str) -> Result<PortRange, AdditionalInfo> {
    match (
        PortRange::from_str(path_parameter),
        u16::from_str(path_parameter),
    ) {
        (Ok(host_port_range), _) => Ok(host_port_range),
        (_, Ok(host_port)) => Ok(PortRange::new(host_port..=host_port)),
        (Err(e1), Err(e2)) => Err(AdditionalInfo {
            additional_info: format!(
                "Could not parse path parameter for host port range ({path_parameter}), expected \
                either one non-zero unsigned 16 bit integer ({e2}) or two non-zero unsigned 16 bit \
                integers seperated by dash ({e1})"
            ),
        }),
    }
}

fn validate_environment_variables(
    environment_variables: &[EnvironmentVariable],
) -> Result<(), Vec<String>> {
    let mut set = HashSet::new();
    let mut errors = Vec::new();
    for environment_variable in environment_variables {
        if !set.insert(environment_variable.name.as_str()) {
            errors.push(format!(
                "Duplicate environment variable name: {}",
                environment_variable.name
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

impl From<EnvironmentVariable> for models::InstanceEnvironmentVariable {
    fn from(value: EnvironmentVariable) -> Self {
        Self {
            name: value.name,
            value: value.value,
        }
    }
}

impl From<models::InstanceEnvironmentVariable> for EnvironmentVariable {
    fn from(value: models::InstanceEnvironmentVariable) -> Self {
        Self {
            name: value.name,
            value: value.value,
        }
    }
}

impl From<Label> for models::InstanceLabel {
    fn from(value: Label) -> Self {
        Self {
            name: value.label,
            value: value.value,
        }
    }
}

fn instance_config_usb_device_from(
    (config, device): (UsbPathConfig, Option<UsbDevice>),
) -> models::InstanceConfigUsbDevice {
    match device {
        Some(device) => models::InstanceConfigUsbDevice {
            port: config.port,
            device_connected: true,
            pid: Some(device.pid as i32),
            name: Some(device.device),
            vendor: Some(device.vendor),
            vid: Some(device.vid as i32),
        },
        None => models::InstanceConfigUsbDevice {
            port: config.port,
            device_connected: false,
            name: None,
            vid: None,
            pid: None,
            vendor: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsm::server_impl::ServerImpl;
    use crate::jeweler::gem::instance::InstancePortMapping;
    use crate::relic::device::usb::MockUsbDeviceReader;
    use crate::sorcerer::appraiser::MockAppRaiser;
    use crate::sorcerer::instancius::MockInstancius;
    use crate::sorcerer::MockSorcerers;
    use axum::extract::Host;
    use axum_extra::extract::CookieJar;
    use flecsd_axum_server::apis::instances::{
        InstancesInstanceIdLogsGetResponse, InstancesInstanceIdStartPostResponse,
        InstancesInstanceIdStopPostResponse,
    };
    use flecsd_axum_server::models::{
        AppKey, InstancesInstanceIdLogsGetPathParams, InstancesInstanceIdStartPostPathParams,
        InstancesInstanceIdStopPostPathParams,
    };
    use http::Method;
    use std::collections::HashMap;
    use std::sync::Arc;

    async fn await_quest_completion() {
        let quest = crate::lore::quest::default()
            .await
            .lock()
            .await
            .schedule_quest("Wait for quests to complete".to_string(), |_quest| async {
                Ok(())
            })
            .await
            .unwrap()
            .1;
        quest
            .lock()
            .await
            .create_infallible_sub_quest(
                "Subquest: Wait for quests to complete".to_string(),
                |_quest| async {},
            )
            .await
            .2
            .await;
    }

    #[tokio::test]
    async fn start_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_does_instance_exist()
            .withf(|_, id| id.value == 0x1234)
            .once()
            .returning(|_, _| false);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_start_post(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdStartPostPathParams {
                        instance_id: "00001234".to_string(),
                    },
                )
                .await,
            Ok(InstancesInstanceIdStartPostResponse::Status404_NoInstanceWithThisInstance)
        )
    }

    #[tokio::test]
    async fn stop_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_does_instance_exist()
            .withf(|_, id| id.value == 0x1234)
            .once()
            .returning(|_, _| false);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_stop_post(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdStopPostPathParams {
                        instance_id: "00001234".to_string(),
                    },
                )
                .await,
            Ok(InstancesInstanceIdStopPostResponse::Status404_NoInstanceWithThisInstance)
        )
    }

    #[tokio::test]
    async fn logs_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_does_instance_exist()
            .withf(|_, id| id.value == 0x1234)
            .once()
            .returning(|_, _| false);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_logs_get(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdLogsGetPathParams {
                        instance_id: "00001234".to_string(),
                    },
                )
                .await,
            Ok(InstancesInstanceIdLogsGetResponse::Status404_NoInstanceWithThisInstance)
        )
    }

    #[tokio::test]
    async fn create_instance_no_app() {
        let mut appraiser = MockAppRaiser::new();
        appraiser.expect_does_app_exist().once().return_const(false);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                app_raiser: Arc::new(appraiser),
                ..Default::default()
            },
        );
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
        let test_key = AppKey {
            name: "TestName".to_string(),
            version: "1.2.3".to_string(),
        };
        let expected_key = test_key.clone();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_create_instance()
            .withf(move |_, _, app_key, name| {
                app_key.name == expected_key.name
                    && app_key.version == expected_key.version
                    && name.is_empty()
            })
            .once()
            .returning(|_, _, _, _| Ok(InstanceId::new(1)));
        let mut appraiser = MockAppRaiser::new();
        appraiser
            .expect_does_app_exist()
            .once()
            .returning(|_, _| true);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                app_raiser: Arc::new(appraiser),
                ..MockSorcerers::default()
            },
        );
        let result = server
            .instances_create_post(
                Method::default(),
                Host("host".to_string()),
                CookieJar::default(),
                InstancesCreatePostRequest {
                    app_key: test_key.clone(),
                    instance_name: None,
                },
            )
            .await;
        match result {
            Ok(InstancesCreatePostResponse::Status202_Accepted(_)) => {
                await_quest_completion().await;
            }
            _ => panic!("Expected InstancesCreatePostResponse::Status202_Accepted"),
        }
    }

    #[tokio::test]
    async fn delete_instance_config_ports_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_port_mappings()
            .withf(move |_, id| id.value == 0x12341234)
            .once()
            .returning(|_, _| false);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    DeletePortsParams {
                        instance_id: "12341234".to_string(),
                    },
                )
                .await,
            Ok(DeletePortsResponse::Status404_NoInstanceWithThisInstance)
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_port_mappings()
            .withf(move |_, id| id.value == 6)
            .once()
            .returning(|_, _| true);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    DeletePortsParams {
                        instance_id: "00000006".to_string(),
                    },
                )
                .await,
            Ok(DeletePortsResponse::Status200_ExposedPortsOfInstanceWithThisInstance)
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_port_mappings()
            .withf(move |_, id| id.value == 0x12341234)
            .once()
            .returning(|_, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetPortsParams {
                        instance_id: "12341234".to_string(),
                    },
                )
                .await,
            Ok(GetPortsResponse::Status404_NoInstanceWithThisInstance)
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_port_mappings()
            .withf(move |_, id| id.value == 6)
            .once()
            .returning(|_, _| {
                Some(InstancePortMapping {
                    tcp: vec![PortMapping::Single(80, 8080)],
                    udp: vec![PortMapping::Range {
                        from: PortRange::new(50..=100),
                        to: PortRange::new(150..=200),
                    }],
                    sctp: vec![],
                })
            });
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_ports_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetPortsParams {
                        instance_id: "00000006".to_string(),
                    },
                )
                .await,
            Ok(GetPortsResponse::Status200_Success(models::InstancePorts {
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
            }))
        );
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_protocol_port_mappings()
            .withf(move |_, id, protocol| id.value == 6 && *protocol == TransportProtocol::Tcp)
            .once()
            .returning(|_, _, _| Some(Vec::new()));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    DeleteProtocolPortsParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp
                    },
                )
                .await,
            Ok(DeleteProtocolPortsResponse::Status200_RemovedAllPublishedPortsOfInstanceWithThisInstance)
        );
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_protocol_port_mappings()
            .withf(move |_, id, protocol| {
                id.value == 0xaaaaaaaa && *protocol == TransportProtocol::Tcp
            })
            .once()
            .returning(|_, _, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    DeleteProtocolPortsParams {
                        instance_id: "aaaaaaaa".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp
                    },
                )
                .await,
            Ok(DeleteProtocolPortsResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_protocol_port_mappings()
            .withf(move |_, id, protocol| {
                id.value == 0xabcdabcd && *protocol == TransportProtocol::Tcp
            })
            .once()
            .returning(|_, _, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetProtocolPortsParams {
                        instance_id: "abcdabcd".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp
                    },
                )
                .await,
            Ok(GetProtocolPortsResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_protocol_port_mappings()
            .withf(move |_, id, protocol| id.value == 6 && *protocol == TransportProtocol::Tcp)
            .once()
            .returning(|_, _, _| Some(vec![PortMapping::Single(80, 8080)]));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetProtocolPortsParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp
                    },
                )
                .await,
            Ok(
                GetProtocolPortsResponse::Status200_PublishedPortsForInstanceWithThisInstance(
                    vec![models::InstancePortMapping::InstancePortMappingSingle(
                        Box::new(models::InstancePortMappingSingle {
                            host_port: 80,
                            container_port: 8080
                        })
                    )]
                )
            )
        );
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_host_port_404_instance() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_port_mapping_range()
            .withf(move |_, id, port, protocol| {
                id.value == 0xffffffff
                    && *protocol == TransportProtocol::Tcp
                    && *port == PortRange::new(80..=80)
            })
            .once()
            .returning(|_, _, _, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    DeletePortRangeParams {
                        instance_id: "ffffffff".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "80".to_string(),
                    },
                )
                .await,
            Ok(DeletePortRangeResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_host_port_404_host() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_port_mapping_range()
            .withf(move |_, id, port, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Tcp
                    && *port == PortRange::new(90..=90)
            })
            .once()
            .returning(|_, _, _, _| Some(false));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    DeletePortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "90".to_string(),
                    },
                )
                .await,
            Ok(DeletePortRangeResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_host_port_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_port_mapping_range()
            .withf(move |_, id, port, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Tcp
                    && *port == PortRange::new(80..=80)
            })
            .once()
            .returning(|_, _, _, _| Some(true));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    DeletePortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "80".to_string(),
                    },
                )
                .await,
            Ok(DeletePortRangeResponse::Status200_Success)
        );
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_host_port_404_instance() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_port_mapping_range()
            .withf(move |_, id, port, protocol| {
                id.value == 0xffffffff
                    && *protocol == TransportProtocol::Tcp
                    && *port == PortRange::new(80..=80)
            })
            .once()
            .returning(|_, _, _, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetPortRangeParams {
                        instance_id: "ffffffff".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "80".to_string(),
                    },
                )
                .await,
            Ok(GetPortRangeResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_host_port_404_host() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_port_mapping_range()
            .withf(move |_, id, port, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Tcp
                    && *port == PortRange::new(90..=90)
            })
            .once()
            .returning(|_, _, _, _| Some(None));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetPortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "90".to_string(),
                    },
                )
                .await,
            Ok(GetPortRangeResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_host_port_200_single() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_port_mapping_range()
            .withf(move |_, id, port, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Tcp
                    && *port == PortRange::new(80..=80)
            })
            .once()
            .returning(|_, _, _, _| Some(Some(PortMapping::Single(80, 8080))));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetPortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "80".to_string(),
                    },
                )
                .await,
            Ok(GetPortRangeResponse::Status200_Success(
                models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                    models::InstancePortMappingSingle {
                        host_port: 80,
                        container_port: 8080,
                    }
                ))
            ))
        );
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_host_port_400_overlap() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_port_mapping()
            .withf(move |_, id, mapping, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Udp
                    && *mapping == PortMapping::Single(80, 20)
            })
            .once()
            .returning(|_, _, _, _| Err(anyhow::anyhow!("TestError")));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutPortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "80".to_string(),
                    },
                    PutPortRangeRequest::I32(Box::new(20)),
                )
                .await,
            Ok(PutPortRangeResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_host_port_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_port_mapping()
            .withf(move |_, id, mapping, protocol| {
                id.value == 0xffffffff
                    && *protocol == TransportProtocol::Udp
                    && *mapping == PortMapping::Single(80, 20)
            })
            .once()
            .returning(|_, _, _, _| Ok(None));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutPortRangeParams {
                        instance_id: "ffffffff".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "80".to_string(),
                    },
                    PutPortRangeRequest::I32(Box::new(20)),
                )
                .await,
            Ok(PutPortRangeResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_host_port_201() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_port_mapping()
            .withf(move |_, id, mapping, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Tcp
                    && *mapping == PortMapping::Single(70, 20)
            })
            .once()
            .returning(|_, _, _, _| Ok(Some(false)));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutPortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "70".to_string(),
                    },
                    PutPortRangeRequest::I32(Box::new(20)),
                )
                .await,
            Ok(PutPortRangeResponse::Status201_TheSpecifiedPortMappingWasCreated)
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_host_port_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_port_mapping()
            .withf(move |_, id, mapping, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Tcp
                    && *mapping == PortMapping::Single(80, 20)
            })
            .once()
            .returning(|_, _, _, _| Ok(Some(true)));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutPortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "80".to_string(),
                    },
                    PutPortRangeRequest::I32(Box::new(20)),
                )
                .await,
            Ok(PutPortRangeResponse::Status200_TheSpecifiedPortMappingWasSet)
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_range_400_range() {
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers::default(),
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    DeletePortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "20-1".to_string(),
                    },
                )
                .await,
            Ok(DeletePortRangeResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_range_404_range() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_port_mapping_range()
            .withf(move |_, id, port, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Udp
                    && *port == PortRange::new(20..=70)
            })
            .once()
            .returning(|_, _, _, _| Some(false));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    DeletePortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "20-70".to_string(),
                    },
                )
                .await,
            Ok(DeletePortRangeResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_range_404_instance() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_port_mapping_range()
            .withf(move |_, id, port, protocol| {
                id.value == 0xaabbccdd
                    && *protocol == TransportProtocol::Udp
                    && *port == PortRange::new(50..=100)
            })
            .once()
            .returning(|_, _, _, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    DeletePortRangeParams {
                        instance_id: "aabbccdd".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "50-100".to_string(),
                    },
                )
                .await,
            Ok(DeletePortRangeResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_ports_transport_protocol_range_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_port_mapping_range()
            .withf(move |_, id, port, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Udp
                    && *port == PortRange::new(50..=100)
            })
            .once()
            .returning(|_, _, _, _| Some(true));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    DeletePortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "50-100".to_string(),
                    },
                )
                .await,
            Ok(DeletePortRangeResponse::Status200_Success)
        );
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_range_400_range() {
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers::default(),
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetPortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "70-4".to_string(),
                    },
                )
                .await,
            Ok(GetPortRangeResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_range_404_range() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_port_mapping_range()
            .withf(move |_, id, port, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Udp
                    && *port == PortRange::new(70..=100)
            })
            .once()
            .returning(|_, _, _, _| Some(None));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetPortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "70-100".to_string(),
                    },
                )
                .await,
            Ok(GetPortRangeResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_range_404_instance() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_port_mapping_range()
            .withf(move |_, id, port, protocol| {
                id.value == 0x12345678
                    && *protocol == TransportProtocol::Udp
                    && *port == PortRange::new(50..=100)
            })
            .once()
            .returning(|_, _, _, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetPortRangeParams {
                        instance_id: "12345678".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "50-100".to_string(),
                    },
                )
                .await,
            Ok(GetPortRangeResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_range_200_range() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_port_mapping_range()
            .withf(move |_, id, port, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Udp
                    && *port == PortRange::new(50..=100)
            })
            .once()
            .returning(|_, _, _, _| {
                Some(Some(PortMapping::Range {
                    from: PortRange::new(50..=100),
                    to: PortRange::new(150..=200),
                }))
            });
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetPortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "50-100".to_string(),
                    },
                )
                .await,
            Ok(GetPortRangeResponse::Status200_Success(
                models::InstancePortMapping::InstancePortMappingRange(Box::new(
                    models::InstancePortMappingRange {
                        host_ports: models::PortRange {
                            start: 50,
                            end: 100,
                        },
                        container_ports: models::PortRange {
                            start: 150,
                            end: 200,
                        },
                    }
                ))
            ))
        );
    }

    #[tokio::test]
    async fn get_instance_config_ports_transport_protocol_range_200_single() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_port_mapping_range()
            .withf(move |_, id, port, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Tcp
                    && *port == PortRange::new(80..=80)
            })
            .once()
            .returning(|_, _, _, _| Some(Some(PortMapping::Single(80, 8080))));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetPortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "80-80".to_string(),
                    },
                )
                .await,
            Ok(GetPortRangeResponse::Status200_Success(
                models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                    models::InstancePortMappingSingle {
                        host_port: 80,
                        container_port: 8080,
                    }
                ))
            ))
        );
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_range_400_host_range() {
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers::default(),
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutPortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                        host_port_range: "70-50".to_string(),
                    },
                    PutPortRangeRequest::PortRange(Box::new(models::PortRange {
                        start: 200,
                        end: 220,
                    })),
                )
                .await,
            Ok(PutPortRangeResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_range_400_container_range() {
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers::default(),
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutPortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                        host_port_range: "70-90".to_string(),
                    },
                    PutPortRangeRequest::PortRange(Box::new(models::PortRange {
                        start: 200,
                        end: 180,
                    })),
                )
                .await,
            Ok(PutPortRangeResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_range_400_range_mismatch() {
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers::default(),
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutPortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                        host_port_range: "70-90".to_string(),
                    },
                    PutPortRangeRequest::PortRange(Box::new(models::PortRange {
                        start: 200,
                        end: 400,
                    }))
                )
                .await,
            Ok(PutPortRangeResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_range_400_overlap() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_port_mapping()
            .withf(move |_, id, mapping, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Tcp
                    && *mapping
                        == PortMapping::Range {
                            from: PortRange::new(70..=90),
                            to: PortRange::new(200..=220),
                        }
            })
            .once()
            .returning(|_, _, _, _| Err(anyhow::anyhow!("TestError")));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutPortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Tcp,
                        host_port_range: "70-90".to_string(),
                    },
                    PutPortRangeRequest::PortRange(Box::new(models::PortRange {
                        start: 200,
                        end: 220,
                    })),
                )
                .await,
            Ok(PutPortRangeResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_range_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_port_mapping()
            .withf(move |_, id, mapping, protocol| {
                id.value == 0xffeeddcc
                    && *protocol == TransportProtocol::Sctp
                    && *mapping
                        == PortMapping::Range {
                            from: PortRange::new(1000..=1100),
                            to: PortRange::new(200..=300),
                        }
            })
            .once()
            .returning(|_, _, _, _| Ok(None));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutPortRangeParams {
                        instance_id: "ffeeddcc".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                        host_port_range: "1000-1100".to_string(),
                    },
                    PutPortRangeRequest::PortRange(Box::new(models::PortRange {
                        start: 200,
                        end: 300,
                    }))
                )
                .await,
            Ok(PutPortRangeResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_range_201() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_port_mapping()
            .withf(move |_, id, mapping, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Sctp
                    && *mapping
                        == PortMapping::Range {
                            from: PortRange::new(1000..=1100),
                            to: PortRange::new(200..=300),
                        }
            })
            .once()
            .returning(|_, _, _, _| Ok(Some(false)));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutPortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                        host_port_range: "1000-1100".to_string(),
                    },
                    PutPortRangeRequest::PortRange(Box::new(models::PortRange {
                        start: 200,
                        end: 300,
                    })),
                )
                .await,
            Ok(PutPortRangeResponse::Status201_TheSpecifiedPortMappingWasCreated)
        );
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_range_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_port_mapping()
            .withf(move |_, id, mapping, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Udp
                    && *mapping
                        == PortMapping::Range {
                            from: PortRange::new(50..=100),
                            to: PortRange::new(200..=250),
                        }
            })
            .once()
            .returning(|_, _, _, _| Ok(Some(true)));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_ports_transport_protocol_host_port_range_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutPortRangeParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                        host_port_range: "50-100".to_string(),
                    },
                    PutPortRangeRequest::PortRange(Box::new(models::PortRange {
                        start: 200,
                        end: 250,
                    })),
                )
                .await,
            Ok(PutPortRangeResponse::Status200_TheSpecifiedPortMappingWasSet)
        );
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_400_overlap() {
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers::default(),
        );
        let port_mappings = vec![
            models::InstancePortMapping::InstancePortMappingRange(Box::new(
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
            models::InstancePortMapping::InstancePortMappingSingle(Box::new(
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
                    PutProtocolPortsParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Sctp,
                    },
                    port_mappings,
                )
                .await,
            Ok(PutProtocolPortsResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_400_port_mapping() {
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers::default(),
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutProtocolPortsParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                    },
                    vec![models::InstancePortMapping::InstancePortMappingRange(
                        Box::new(models::InstancePortMappingRange {
                            host_ports: models::PortRange {
                                start: 2000,
                                end: 1000,
                            },
                            container_ports: models::PortRange {
                                start: 6000,
                                end: 7000,
                            },
                        },)
                    )],
                )
                .await,
            Ok(PutProtocolPortsResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_protocol_port_mappings()
            .withf(move |_, id, mapping, protocol| {
                id.value == 0x77778888 && *protocol == TransportProtocol::Udp && mapping.is_empty()
            })
            .once()
            .returning(|_, _, _, _| false);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_ports_transport_protocol_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutProtocolPortsParams {
                        instance_id: "77778888".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                    },
                    vec![],
                )
                .await,
            Ok(PutProtocolPortsResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_ports_transport_protocol_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_protocol_port_mappings()
            .withf(move |_, id, mapping, protocol| {
                id.value == 6
                    && *protocol == TransportProtocol::Udp
                    && *mapping
                        == vec![
                            PortMapping::Single(100, 20),
                            PortMapping::Range {
                                from: PortRange::new(2000..=3000),
                                to: PortRange::new(6000..=7000),
                            },
                            PortMapping::Single(60, 70),
                        ]
            })
            .once()
            .returning(|_, _, _, _| true);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        let port_mappings = vec![
            models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                models::InstancePortMappingSingle {
                    host_port: 100,
                    container_port: 20,
                },
            )),
            models::InstancePortMapping::InstancePortMappingRange(Box::new(
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
            models::InstancePortMapping::InstancePortMappingSingle(Box::new(
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
                    PutProtocolPortsParams {
                        instance_id: "00000006".to_string(),
                        transport_protocol: models::TransportProtocol::Udp,
                    },
                    port_mappings
                )
                .await,
            Ok(PutProtocolPortsResponse::Status200_PublishedPortsOfInstanceWithThisInstance)
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
        let instance_port_mapping = models::InstancePortMapping::InstancePortMappingRange(
            Box::new(models::InstancePortMappingRange {
                host_ports: models::PortRange { start: 7, end: 10 },
                container_ports: models::PortRange { start: 17, end: 20 },
            }),
        );
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
        let instance_port_mapping = models::InstancePortMapping::InstancePortMappingRange(
            Box::new(models::InstancePortMappingRange {
                host_ports: models::PortRange { start: 70, end: 20 },
                container_ports: models::PortRange { start: 17, end: 20 },
            }),
        );
        assert!(PortMapping::try_from(instance_port_mapping).is_err(),);
    }

    #[test]
    fn try_from_instance_port_mapping_range_container_err() {
        let instance_port_mapping = models::InstancePortMapping::InstancePortMappingRange(
            Box::new(models::InstancePortMappingRange {
                host_ports: models::PortRange { start: 70, end: 80 },
                container_ports: models::PortRange { start: 60, end: 40 },
            }),
        );
        assert!(PortMapping::try_from(instance_port_mapping).is_err(),);
    }

    #[test]
    fn try_from_instance_port_mapping_single_ok() {
        let instance_port_mapping = models::InstancePortMapping::InstancePortMappingSingle(
            Box::new(models::InstancePortMappingSingle {
                host_port: 10,
                container_port: 17,
            }),
        );
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
            models::InstancePortMapping::from(&port_mapping),
            models::InstancePortMapping::InstancePortMappingRange(Box::new(
                models::InstancePortMappingRange {
                    host_ports: models::PortRange { start: 6, end: 9 },
                    container_ports: models::PortRange { start: 11, end: 14 },
                }
            ))
        )
    }

    #[test]
    fn from_port_mapping_single() {
        let port_mapping = PortMapping::Single(100, 1000);
        assert_eq!(
            models::InstancePortMapping::from(&port_mapping),
            models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                models::InstancePortMappingSingle {
                    host_port: 100,
                    container_port: 1000,
                }
            ))
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
                models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                    models::InstancePortMappingSingle {
                        host_port: 100,
                        container_port: 1000,
                    }
                )),
                models::InstancePortMapping::InstancePortMappingSingle(Box::new(
                    models::InstancePortMappingSingle {
                        host_port: 6,
                        container_port: 110,
                    }
                )),
                models::InstancePortMapping::InstancePortMappingRange(Box::new(
                    models::InstancePortMappingRange {
                        host_ports: models::PortRange { start: 10, end: 20 },
                        container_ports: models::PortRange { start: 20, end: 30 },
                    }
                ))
            ]
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

    #[test]
    fn validate_environment_variables_empty() {
        assert!(validate_environment_variables(&[]).is_ok());
    }

    #[test]
    fn validate_environment_variables_ok() {
        assert!(validate_environment_variables(&[
            EnvironmentVariable {
                name: "Variable1".to_string(),
                value: None,
            },
            EnvironmentVariable {
                name: "Variable2".to_string(),
                value: Some("Value".to_string()),
            },
            EnvironmentVariable {
                name: "TEST_VAR".to_string(),
                value: None,
            }
        ])
        .is_ok());
    }

    #[test]
    fn validate_environment_variables_err_single() {
        let errors = validate_environment_variables(&[
            EnvironmentVariable {
                name: "Variable1".to_string(),
                value: None,
            },
            EnvironmentVariable {
                name: "TEST_VAR".to_string(),
                value: Some("Value".to_string()),
            },
            EnvironmentVariable {
                name: "Variable2".to_string(),
                value: Some("Value".to_string()),
            },
            EnvironmentVariable {
                name: "TEST_VAR".to_string(),
                value: None,
            },
        ])
        .err()
        .unwrap();
        assert_eq!(errors.len(), 1, "{errors:?}");
    }

    #[test]
    fn validate_environment_variables_err_multiple() {
        let errors = validate_environment_variables(&[
            EnvironmentVariable {
                name: "Variable1".to_string(),
                value: None,
            },
            EnvironmentVariable {
                name: "TEST_VAR".to_string(),
                value: None,
            },
            EnvironmentVariable {
                name: "TEST_VAR".to_string(),
                value: Some("Value".to_string()),
            },
            EnvironmentVariable {
                name: "Variable2".to_string(),
                value: Some("Value".to_string()),
            },
            EnvironmentVariable {
                name: "TEST_VAR".to_string(),
                value: None,
            },
            EnvironmentVariable {
                name: "Variable1".to_string(),
                value: Some("Value".to_string()),
            },
        ])
        .err()
        .unwrap();
        assert_eq!(errors.len(), 3, "{errors:?}");
    }

    #[test]
    fn from_environment_variable() {
        let name = "TEST_VAR".to_string();
        let value = Some("test-value".to_string());
        assert_eq!(
            models::InstanceEnvironmentVariable::from(EnvironmentVariable {
                name: name.clone(),
                value: value.clone()
            }),
            models::InstanceEnvironmentVariable { name, value }
        );
    }

    #[test]
    fn from_instance_environment_variable() {
        let name = "TEST_VAR".to_string();
        let value = Some("test-value".to_string());
        assert_eq!(
            EnvironmentVariable::from(models::InstanceEnvironmentVariable {
                name: name.clone(),
                value: value.clone()
            }),
            EnvironmentVariable { name, value }
        );
    }

    #[tokio::test]
    async fn get_instance_config_environment_variable_404_instance() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_environment_variable_value()
            .withf(move |_, id, name| id.value == 0x99887766 && name == "variable_name")
            .once()
            .returning(|_, _, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_environment_variable_name_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetEnvironmentVariableParams {
                        instance_id: "99887766".to_string(),
                        variable_name: "variable_name".to_string(),
                    },
                )
                .await,
            Ok(GetEnvironmentVariableResponse::Status404_ResourceNotFound(
                _
            ))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_environment_variable_404_variable() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_environment_variable_value()
            .withf(move |_, id, name| id.value == 6 && name == "variable_name")
            .once()
            .returning(|_, _, _| Some(None));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_environment_variable_name_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetEnvironmentVariableParams {
                        instance_id: "00000006".to_string(),
                        variable_name: "variable_name".to_string(),
                    },
                )
                .await,
            Ok(GetEnvironmentVariableResponse::Status404_ResourceNotFound(
                _
            ))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_environment_variable_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_environment_variable_value()
            .withf(move |_, id, name| id.value == 6 && name == "VAR_1")
            .once()
            .returning(|_, _, _| Some(Some(None)));
        instancius
            .expect_get_instance_config_environment_variable_value()
            .withf(move |_, id, name| id.value == 6 && name == "VAR_2")
            .once()
            .returning(|_, _, _| Some(Some(Some("value".to_string()))));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_environment_variable_name_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetEnvironmentVariableParams {
                        instance_id: "00000006".to_string(),
                        variable_name: "VAR_1".to_string(),
                    },
                )
                .await,
            Ok(GetEnvironmentVariableResponse::Status200_Success(
                models::InstancesInstanceIdConfigEnvironmentVariableNameGet200Response {
                    value: None
                }
            ))
        );
        assert_eq!(
            server
                .instances_instance_id_config_environment_variable_name_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetEnvironmentVariableParams {
                        instance_id: "00000006".to_string(),
                        variable_name: "VAR_2".to_string(),
                    },
                )
                .await,
            Ok(GetEnvironmentVariableResponse::Status200_Success(
                models::InstancesInstanceIdConfigEnvironmentVariableNameGet200Response {
                    value: Some("value".to_string())
                }
            ))
        );
    }

    #[tokio::test]
    async fn delete_instance_config_environment_variable_404_instance() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_environment_variable_value()
            .withf(move |_, id, name| id.value == 0x99887766 && name == "variable_name")
            .once()
            .returning(|_, _, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_environment_variable_name_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    DeleteEnvironmentVariableParams {
                        instance_id: "99887766".to_string(),
                        variable_name: "variable_name".to_string(),
                    },
                )
                .await,
            Ok(DeleteEnvironmentVariableResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_environment_variable_404_variable() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_environment_variable_value()
            .withf(move |_, id, name| id.value == 6 && name == "variable_name")
            .once()
            .returning(|_, _, _| Some(None));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_environment_variable_name_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    DeleteEnvironmentVariableParams {
                        instance_id: "00000006".to_string(),
                        variable_name: "variable_name".to_string(),
                    },
                )
                .await,
            Ok(DeleteEnvironmentVariableResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_environment_variable_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_environment_variable_value()
            .withf(move |_, id, name| id.value == 6 && name == "VAR_1")
            .once()
            .returning(|_, _, _| {
                Some(Some(EnvironmentVariable {
                    name: "VAR_1".to_string(),
                    value: Some("value".to_string()),
                }))
            });
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_environment_variable_name_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    DeleteEnvironmentVariableParams {
                        instance_id: "00000006".to_string(),
                        variable_name: "VAR_1".to_string(),
                    },
                )
                .await,
            Ok(
                DeleteEnvironmentVariableResponse::Status200_EnvironmentVariableOfInstanceWithThisInstance
            )
        );
    }

    #[tokio::test]
    async fn put_instance_config_environment_variable_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_environment_variable_value()
            .withf(move |_, id, var| {
                id.value == 0x12341234
                    && *var
                        == EnvironmentVariable {
                            name: "VAR_3".to_string(),
                            value: Some("new value".to_string()),
                        }
            })
            .once()
            .returning(|_, _, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_environment_variable_name_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutEnvironmentVariableParams {
                        instance_id: "12341234".to_string(),
                        variable_name: "VAR_3".to_string(),
                    },
                    models::InstancesInstanceIdConfigEnvironmentVariableNameGet200Response {
                        value: Some("new value".to_string())
                    }
                )
                .await,
            Ok(PutEnvironmentVariableResponse::Status404_NoInstanceWithThisInstance)
        );
    }

    #[tokio::test]
    async fn put_instance_config_environment_variable_201() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_environment_variable_value()
            .withf(move |_, id, var| {
                id.value == 6
                    && *var
                        == EnvironmentVariable {
                            name: "VAR_3".to_string(),
                            value: Some("new value".to_string()),
                        }
            })
            .once()
            .returning(|_, _, _| Some(None));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_environment_variable_name_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutEnvironmentVariableParams {
                        instance_id: "00000006".to_string(),
                        variable_name: "VAR_3".to_string(),
                    },
                    models::InstancesInstanceIdConfigEnvironmentVariableNameGet200Response {
                        value: Some("new value".to_string())
                    }
                )
                .await,
            Ok(
                PutEnvironmentVariableResponse::Status201_EnvironmentForInstanceWithThisInstanceIdWasCreated
            )
        );
    }

    #[tokio::test]
    async fn put_instance_config_environment_variable_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_environment_variable_value()
            .withf(move |_, id, var| {
                id.value == 6
                    && *var
                        == EnvironmentVariable {
                            name: "VAR_2".to_string(),
                            value: Some("new value".to_string()),
                        }
            })
            .once()
            .returning(|_, _, _| Some(Some("previous_value".to_string())));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_environment_variable_name_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutEnvironmentVariableParams {
                        instance_id: "00000006".to_string(),
                        variable_name: "VAR_2".to_string(),
                    },
                    models::InstancesInstanceIdConfigEnvironmentVariableNameGet200Response {
                        value: Some("new value".to_string())
                    }
                )
                .await,
            Ok(
                PutEnvironmentVariableResponse::Status200_EnvironmentForInstanceWithThisInstanceIdIsSet
            )
        );
    }

    #[tokio::test]
    async fn delete_instance_config_environment_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_environment()
            .withf(move |_, id| id.value == 0x12341234)
            .once()
            .returning(|_, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_environment_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    DeleteEnvironmentParams {
                        instance_id: "12341234".to_string(),
                    },
                )
                .await,
            Ok(DeleteEnvironmentResponse::Status404_NoInstanceWithThisInstance)
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_environment_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_environment()
            .withf(move |_, id| id.value == 6)
            .once()
            .returning(|_, _| Some(Vec::new()));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_environment_delete(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    DeleteEnvironmentParams {
                        instance_id: "00000006".to_string(),
                    },
                )
                .await,
            Ok(DeleteEnvironmentResponse::Status200_EnvironmentOfInstanceWithThisInstance)
        ));
    }

    #[tokio::test]
    async fn get_instance_config_environment_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_environment()
            .withf(move |_, id| id.value == 0x12341234)
            .once()
            .returning(|_, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_environment_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetEnvironmentParams {
                        instance_id: "12341234".to_string(),
                    },
                )
                .await,
            Ok(GetEnvironmentResponse::Status404_NoInstanceWithThisInstance)
        ));
    }

    #[tokio::test]
    async fn get_instance_config_environment_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_environment()
            .withf(move |_, id| id.value == 6)
            .once()
            .returning(|_, _| {
                Some(vec![
                    EnvironmentVariable {
                        name: "VAR_1".to_string(),
                        value: None,
                    },
                    EnvironmentVariable {
                        name: "VAR_2".to_string(),
                        value: Some("value".to_string()),
                    },
                ])
            });
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_environment_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetEnvironmentParams {
                        instance_id: "00000006".to_string(),
                    },
                )
                .await,
            Ok(GetEnvironmentResponse::Status200_Success(
                models::InstanceEnvironment::from(vec![
                    models::InstanceEnvironmentVariable {
                        name: "VAR_1".to_string(),
                        value: None,
                    },
                    models::InstanceEnvironmentVariable {
                        name: "VAR_2".to_string(),
                        value: Some("value".to_string()),
                    }
                ])
            ))
        );
    }

    #[tokio::test]
    async fn put_instance_config_environment_400_duplicate_variable_name() {
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers::default(),
        );
        assert!(matches!(
            server
                .instances_instance_id_config_environment_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutEnvironmentParams {
                        instance_id: "00000001".to_string(),
                    },
                    InstanceEnvironment::from(vec![
                        models::InstanceEnvironmentVariable {
                            name: "VAR_1".to_string(),
                            value: None,
                        },
                        models::InstanceEnvironmentVariable {
                            name: "VAR_1".to_string(),
                            value: Some("value".to_string()),
                        }
                    ]),
                )
                .await,
            Ok(PutEnvironmentResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_environment_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_environment()
            .withf(move |_, id, envs| id.value == 0x78907890 && envs.is_empty())
            .once()
            .returning(|_, _, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_environment_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutEnvironmentParams {
                        instance_id: "78907890".to_string(),
                    },
                    InstanceEnvironment::from(vec![]),
                )
                .await,
            Ok(PutEnvironmentResponse::Status404_NoInstanceWithThisInstance)
        ));
    }

    #[tokio::test]
    async fn put_instance_config_environment_201() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_environment()
            .withf(move |_, id, envs| {
                id.value == 1
                    && *envs
                        == vec![
                            EnvironmentVariable {
                                name: "VAR_1".to_string(),
                                value: None,
                            },
                            EnvironmentVariable {
                                name: "VAR_2".to_string(),
                                value: Some("value".to_string()),
                            },
                        ]
            })
            .once()
            .returning(|_, _, _| Some(Vec::new()));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_environment_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutEnvironmentParams {
                        instance_id: "00000001".to_string(),
                    },
                    InstanceEnvironment::from(vec![
                        models::InstanceEnvironmentVariable {
                            name: "VAR_1".to_string(),
                            value: None,
                        },
                        models::InstanceEnvironmentVariable {
                            name: "VAR_2".to_string(),
                            value: Some("value".to_string()),
                        }
                    ]),
                )
                .await,
            Ok(PutEnvironmentResponse::Status201_EnvironmentForInstanceWithThisInstanceIdWasCreated)
        ));
    }

    #[tokio::test]
    async fn put_instance_config_environment_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_environment()
            .withf(move |_, id, envs| {
                id.value == 6
                    && *envs
                        == vec![
                            EnvironmentVariable {
                                name: "VAR_10".to_string(),
                                value: None,
                            },
                            EnvironmentVariable {
                                name: "VAR_20".to_string(),
                                value: Some("value".to_string()),
                            },
                        ]
            })
            .once()
            .returning(|_, _, _| {
                Some(vec![EnvironmentVariable {
                    name: "previous_var".to_string(),
                    value: None,
                }])
            });
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_environment_put(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    PutEnvironmentParams {
                        instance_id: "00000006".to_string(),
                    },
                    InstanceEnvironment::from(vec![
                        models::InstanceEnvironmentVariable {
                            name: "VAR_10".to_string(),
                            value: None,
                        },
                        models::InstanceEnvironmentVariable {
                            name: "VAR_20".to_string(),
                            value: Some("value".to_string()),
                        }
                    ]),
                )
                .await,
            Ok(PutEnvironmentResponse::Status200_EnvironmentForInstanceWithThisInstanceIdIsSet)
        ));
    }

    #[test]
    fn from_label() {
        assert_eq!(
            models::InstanceLabel::from(Label {
                label: "org.some".to_string(),
                value: Some("value".to_string()),
            }),
            models::InstanceLabel {
                name: "org.some".to_string(),
                value: Some("value".to_string()),
            }
        )
    }

    #[tokio::test]
    async fn get_instance_labels_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_labels()
            .withf(move |_, id| id.value == 0x66229933)
            .once()
            .returning(|_, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_labels_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetLabelsParams {
                        instance_id: "66229933".to_string(),
                    }
                )
                .await,
            Ok(GetLabelsResponse::Status404_NoInstanceWithThisInstance)
        ));
    }

    #[tokio::test]
    async fn get_instance_labels_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_labels()
            .withf(move |_, id| id.value == 1)
            .once()
            .returning(|_, _| {
                Some(vec![
                    Label {
                        label: "tech.flecs".to_string(),
                        value: None,
                    },
                    Label {
                        label: "tech.flecs.some-label".to_string(),
                        value: Some("Some custom label value".to_string()),
                    },
                ])
            });
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_labels_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetLabelsParams {
                        instance_id: "00000001".to_string(),
                    }
                )
                .await,
            Ok(GetLabelsResponse::Status200_Success(vec![
                models::InstanceLabel {
                    name: "tech.flecs".to_string(),
                    value: None,
                },
                models::InstanceLabel {
                    name: "tech.flecs.some-label".to_string(),
                    value: Some("Some custom label value".to_string()),
                }
            ]))
        );
    }

    #[tokio::test]
    async fn get_instance_label_404_instance() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_label_value()
            .withf(move |_, id, name| id.value == 0x12345678 && name == "flecs.tech")
            .once()
            .returning(|_, _, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_labels_label_name_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetLabelParams {
                        instance_id: "12345678".to_string(),
                        label_name: "flecs.tech".to_string(),
                    }
                )
                .await,
            Ok(GetLabelResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_label_404_label() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_label_value()
            .withf(move |_, id, name| id.value == 2 && name == "not.existing.label")
            .once()
            .returning(|_, _, _| Some(None));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_labels_label_name_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetLabelParams {
                        instance_id: "00000002".to_string(),
                        label_name: "not.existing.label".to_string(),
                    }
                )
                .await,
            Ok(GetLabelResponse::Status404_ResourceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_instance_label_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_label_value()
            .withf(move |_, id, name| id.value == 2 && name == "tech.flecs")
            .once()
            .returning(|_, _, _| Some(Some(None)));
        instancius
            .expect_get_instance_label_value()
            .withf(move |_, id, name| id.value == 2 && name == "tech.flecs.some-label")
            .once()
            .returning(|_, _, _| Some(Some(Some("Some custom label value".to_string()))));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_labels_label_name_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetLabelParams {
                        instance_id: "00000002".to_string(),
                        label_name: "tech.flecs".to_string(),
                    }
                )
                .await,
            Ok(GetLabelResponse::Status200_Success(
                models::InstancesInstanceIdConfigLabelsLabelNameGet200Response { value: None }
            ))
        );
        assert_eq!(
            server
                .instances_instance_id_config_labels_label_name_get(
                    Default::default(),
                    Host("host".to_string()),
                    Default::default(),
                    GetLabelParams {
                        instance_id: "00000002".to_string(),
                        label_name: "tech.flecs.some-label".to_string(),
                    }
                )
                .await,
            Ok(GetLabelResponse::Status200_Success(
                models::InstancesInstanceIdConfigLabelsLabelNameGet200Response {
                    value: Some("Some custom label value".to_string())
                }
            ))
        );
    }

    #[test]
    fn instance_config_usb_device_from_some() {
        let usb_path_config = UsbPathConfig {
            dev_num: 20,
            port: "usb12".to_string(),
            bus_num: 10,
        };
        let usb_device = UsbDevice {
            device: "test-dev".to_string(),
            vid: 12,
            pid: 24,
            port: "usb12".to_string(),
            vendor: "Vendor".to_string(),
        };
        assert_eq!(
            instance_config_usb_device_from((usb_path_config, Some(usb_device))),
            models::InstanceConfigUsbDevice {
                port: "usb12".to_string(),
                device_connected: true,
                pid: Some(24),
                vendor: Some("Vendor".to_string()),
                vid: Some(12),
                name: Some("test-dev".to_string()),
            }
        )
    }

    #[test]
    fn instance_config_usb_device_from_none() {
        let usb_path_config = UsbPathConfig {
            dev_num: 20,
            port: "usb12".to_string(),
            bus_num: 10,
        };
        assert_eq!(
            instance_config_usb_device_from((usb_path_config, None)),
            models::InstanceConfigUsbDevice {
                port: "usb12".to_string(),
                device_connected: false,
                pid: None,
                vendor: None,
                vid: None,
                name: None,
            }
        )
    }

    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_delete_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_usb_devices()
            .withf(move |_, id| id.value == 2)
            .once()
            .returning(|_, _| Some(HashMap::new()));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_devices_usb_delete(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbDeletePathParams {
                        instance_id: "00000002".to_string(),
                    }
                )
                .await,
            Ok(InstancesInstanceIdConfigDevicesUsbDeleteResponse::Status200_Success)
        )
    }

    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_delete_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_usb_devices()
            .withf(move |_, id| id.value == 0xaabbccdd)
            .once()
            .returning(|_, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_devices_usb_delete(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbDeletePathParams {
                        instance_id: "aabbccdd".to_string(),
                    }
                )
                .await,
            Ok(InstancesInstanceIdConfigDevicesUsbDeleteResponse::Status404_NoInstanceWithThisInstance)
        )
    }

    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_get_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_devices::<MockUsbDeviceReader>()
            .withf(move |_, id, _| id.value == 6)
            .once()
            .returning(|_, _, _| Ok(Some(vec![])));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_devices_usb_get(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbGetPathParams {
                        instance_id: "00000006".to_string(),
                    }
                )
                .await,
            Ok(InstancesInstanceIdConfigDevicesUsbGetResponse::Status200_Success(vec)) if vec.is_empty()
        ))
    }
    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_get_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_devices::<MockUsbDeviceReader>()
            .withf(move |_, id, _| id.value == 0x1234abcd)
            .once()
            .returning(|_, _, _| Ok(None));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_devices_usb_get(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbGetPathParams {
                        instance_id: "1234abcd".to_string(),
                    }
                )
                .await,
            Ok(InstancesInstanceIdConfigDevicesUsbGetResponse::Status404_NoInstanceWithThisInstance)
        )
    }
    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_get_500() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_devices::<MockUsbDeviceReader>()
            .withf(move |_, id, _| id.value == 6)
            .once()
            .returning(|_, _, _| Err(anyhow::anyhow!("TestError")));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_devices_usb_get(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbGetPathParams {
                        instance_id: "0000006".to_string(),
                    }
                )
                .await,
            Ok(InstancesInstanceIdConfigDevicesUsbGetResponse::Status500_InternalServerError(_))
        ))
    }
    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_port_delete_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_usb_device()
            .withf(move |_, id, port| id.value == 6 && port == "test_port")
            .once()
            .returning(|_, _, _| {
                Some(Some(UsbPathConfig {
                    port: "test_port".to_string(),
                    bus_num: 100,
                    dev_num: 200,
                }))
            });
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_devices_usb_port_delete(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbPortDeletePathParams {
                        instance_id: "00000006".to_string(),
                        port: "test_port".to_string(),
                    }
                )
                .await,
            Ok(InstancesInstanceIdConfigDevicesUsbPortDeleteResponse::Status200_Success)
        ))
    }
    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_port_delete_404_instance() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_usb_device()
            .withf(move |_, id, port| id.value == 0xabcddcba && port == "test_port")
            .once()
            .returning(|_, _, _| None);
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_devices_usb_port_delete(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbPortDeletePathParams {
                        instance_id: "abcddcba".to_string(),
                        port: "test_port".to_string(),
                    }
                )
                .await,
            Ok(
                InstancesInstanceIdConfigDevicesUsbPortDeleteResponse::Status404_ResourceNotFound(
                    _
                )
            )
        ))
    }
    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_port_delete_404_port() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_usb_device()
            .withf(move |_, id, port| id.value == 6 && port == "unknown port")
            .once()
            .returning(|_, _, _| Some(None));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_devices_usb_port_delete(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbPortDeletePathParams {
                        instance_id: "00000006".to_string(),
                        port: "unknown port".to_string(),
                    }
                )
                .await,
            Ok(
                InstancesInstanceIdConfigDevicesUsbPortDeleteResponse::Status404_ResourceNotFound(
                    _
                )
            )
        ))
    }
    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_port_get_200_inactive() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 6 && port == "test_port")
            .once()
            .returning(|_, _, _, _| {
                Ok(GetInstanceUsbDeviceResult::DeviceInactive(UsbPathConfig {
                    port: "test_port".to_string(),
                    bus_num: 10,
                    dev_num: 20,
                }))
            });
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_devices_usb_port_get(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbPortGetPathParams {
                        instance_id: "00000006".to_string(),
                        port: "test_port".to_string(),
                    }
                )
                .await,
            Ok(
                InstancesInstanceIdConfigDevicesUsbPortGetResponse::Status200_Success(
                    models::InstanceConfigUsbDevice {
                        port: "test_port".to_string(),
                        name: None,
                        pid: None,
                        vendor: None,
                        vid: None,
                        device_connected: false,
                    }
                )
            )
        )
    }
    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_port_get_200_active() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 6 && port == "test_port")
            .once()
            .returning(|_, _, _, _| {
                Ok(GetInstanceUsbDeviceResult::DeviceActive(
                    UsbPathConfig {
                        port: "test_port".to_string(),
                        bus_num: 10,
                        dev_num: 20,
                    },
                    UsbDevice {
                        vid: 10,
                        pid: 20,
                        device: "test-dev".to_string(),
                        port: "test_port".to_string(),
                        vendor: "test-vendor".to_string(),
                    },
                ))
            });
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_devices_usb_port_get(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbPortGetPathParams {
                        instance_id: "00000006".to_string(),
                        port: "test_port".to_string(),
                    }
                )
                .await,
            Ok(
                InstancesInstanceIdConfigDevicesUsbPortGetResponse::Status200_Success(
                    models::InstanceConfigUsbDevice {
                        port: "test_port".to_string(),
                        name: Some("test-dev".to_string()),
                        pid: Some(20),
                        vendor: Some("test-vendor".to_string()),
                        vid: Some(10),
                        device_connected: true,
                    }
                )
            )
        )
    }

    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_port_get_404_instance() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 0xaaabbbcc && port == "test_port")
            .once()
            .returning(|_, _, _, _| Ok(GetInstanceUsbDeviceResult::InstanceNotFound));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_devices_usb_port_get(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbPortGetPathParams {
                        instance_id: "aaabbbcc".to_string(),
                        port: "test_port".to_string(),
                    }
                )
                .await,
            Ok(InstancesInstanceIdConfigDevicesUsbPortGetResponse::Status404_ResourceNotFound(_))
        ))
    }
    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_port_get_404_port() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 2 && port == "test_port")
            .once()
            .returning(|_, _, _, _| Ok(GetInstanceUsbDeviceResult::DeviceNotMapped));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_devices_usb_port_get(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbPortGetPathParams {
                        instance_id: "00000002".to_string(),
                        port: "test_port".to_string(),
                    }
                )
                .await,
            Ok(InstancesInstanceIdConfigDevicesUsbPortGetResponse::Status404_ResourceNotFound(_))
        ))
    }
    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_port_get_404_unknown() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 2 && port == "test_port")
            .once()
            .returning(|_, _, _, _| Ok(GetInstanceUsbDeviceResult::UnknownDevice));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_devices_usb_port_get(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbPortGetPathParams {
                        instance_id: "00000002".to_string(),
                        port: "test_port".to_string(),
                    }
                )
                .await,
            Ok(InstancesInstanceIdConfigDevicesUsbPortGetResponse::Status404_ResourceNotFound(_))
        ))
    }
    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_port_get_500() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 6 && port == "test_port")
            .once()
            .returning(|_, _, _, _| Err(anyhow::anyhow!("TestError")));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_devices_usb_port_get(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbPortGetPathParams {
                        instance_id: "00000006".to_string(),
                        port: "test_port".to_string(),
                    }
                )
                .await,
            Ok(
                InstancesInstanceIdConfigDevicesUsbPortGetResponse::Status500_InternalServerError(
                    _
                )
            )
        ))
    }
    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_port_put_404_instance() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 0xaaabbbcc && port == "test_port")
            .once()
            .returning(|_, _, _, _| Ok(PutInstanceUsbDeviceResult::InstanceNotFound));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_devices_usb_port_put(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbPortPutPathParams {
                        instance_id: "aaabbbcc".to_string(),
                        port: "test_port".to_string(),
                    }
                )
                .await,
            Ok(InstancesInstanceIdConfigDevicesUsbPortPutResponse::Status404_ResourceNotFound(_))
        ))
    }
    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_port_put_404_device() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 3 && port == "test_port")
            .once()
            .returning(|_, _, _, _| Ok(PutInstanceUsbDeviceResult::DeviceNotFound));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_devices_usb_port_put(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbPortPutPathParams {
                        instance_id: "00000003".to_string(),
                        port: "test_port".to_string(),
                    }
                )
                .await,
            Ok(InstancesInstanceIdConfigDevicesUsbPortPutResponse::Status404_ResourceNotFound(_))
        ))
    }
    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_port_put_201() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 3 && port == "test_port")
            .once()
            .returning(|_, _, _, _| Ok(PutInstanceUsbDeviceResult::DeviceMappingCreated));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_devices_usb_port_put(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbPortPutPathParams {
                        instance_id: "00000003".to_string(),
                        port: "test_port".to_string(),
                    }
                )
                .await,
            Ok(InstancesInstanceIdConfigDevicesUsbPortPutResponse::Status201_UsbDeviceWasPassedThrough)
        )
    }
    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_port_put_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 6 && port == "test_port")
            .once()
            .returning(|_, _, _, _| {
                Ok(PutInstanceUsbDeviceResult::DeviceMappingUpdated(
                    UsbPathConfig {
                        port: "test_port".to_string(),
                        bus_num: 121,
                        dev_num: 919,
                    },
                ))
            });
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert_eq!(
            server
                .instances_instance_id_config_devices_usb_port_put(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbPortPutPathParams {
                        instance_id: "00000006".to_string(),
                        port: "test_port".to_string(),
                    }
                )
                .await,
            Ok(InstancesInstanceIdConfigDevicesUsbPortPutResponse::Status200_AlreadyPassedThrough)
        )
    }
    #[tokio::test]
    async fn instances_instance_id_config_devices_usb_port_put_500() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_usb_device::<MockUsbDeviceReader>()
            .withf(move |_, id, port, _| id.value == 6 && port == "test_port")
            .once()
            .returning(|_, _, _, _| Err(anyhow::anyhow!("TestError")));
        let server = ServerImpl::test_instance(
            crate::vault::tests::create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockSorcerers {
                instancius: Arc::new(instancius),
                ..MockSorcerers::default()
            },
        );
        assert!(matches!(
            server
                .instances_instance_id_config_devices_usb_port_put(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    InstancesInstanceIdConfigDevicesUsbPortPutPathParams {
                        instance_id: "00000006".to_string(),
                        port: "test_port".to_string(),
                    }
                )
                .await,
            Ok(
                InstancesInstanceIdConfigDevicesUsbPortPutResponse::Status500_InternalServerError(
                    _
                )
            )
        ))
    }
}
