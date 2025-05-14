use crate::enchantment::floxy::Floxy;
use crate::fsm::server_impl::ServerImpl;
use crate::jeweler::gem::instance::InstanceId;
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
    InstancesInstanceIdConfigHostnameGetResponse, InstancesInstanceIdConfigHostnamePutResponse,
    InstancesInstanceIdConfigLabelsGetResponse as GetLabelsResponse,
    InstancesInstanceIdConfigLabelsLabelNameGetResponse as GetLabelResponse,
    InstancesInstanceIdConfigMountsBindContainerPathGetResponse,
    InstancesInstanceIdConfigMountsBindGetResponse, InstancesInstanceIdConfigMountsGetResponse,
    InstancesInstanceIdConfigMountsVolumesGetResponse,
    InstancesInstanceIdConfigMountsVolumesVolumeNameGetResponse,
    InstancesInstanceIdConfigNetworksGetResponse,
    InstancesInstanceIdConfigNetworksNetworkIdDeleteResponse,
    InstancesInstanceIdConfigNetworksNetworkIdGetResponse,
    InstancesInstanceIdConfigNetworksPostResponse,
    InstancesInstanceIdConfigPortsDeleteResponse as DeletePortsResponse,
    InstancesInstanceIdConfigPortsGetResponse as GetPortsResponse,
    InstancesInstanceIdConfigPortsTransportProtocolDeleteResponse as DeleteProtocolPortsResponse,
    InstancesInstanceIdConfigPortsTransportProtocolGetResponse as GetProtocolPortsResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeleteResponse as DeletePortRangeResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetResponse as GetPortRangeResponse,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutResponse as PutPortRangeResponse,
    InstancesInstanceIdConfigPortsTransportProtocolPutResponse as PutProtocolPortsResponse,
    InstancesInstanceIdDeleteResponse, InstancesInstanceIdEditorPortGetResponse,
    InstancesInstanceIdGetResponse, InstancesInstanceIdLogsGetResponse,
    InstancesInstanceIdPatchResponse, InstancesInstanceIdStartPostResponse,
    InstancesInstanceIdStopPostResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    InstanceEnvironment, InstancesCreatePostRequest, InstancesGetQueryParams,
    InstancesInstanceIdConfigDevicesUsbDeletePathParams,
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
    InstancesInstanceIdConfigHostnameGetPathParams, InstancesInstanceIdConfigHostnamePutPathParams,
    InstancesInstanceIdConfigHostnamePutRequest,
    InstancesInstanceIdConfigLabelsGetPathParams as GetLabelsParams,
    InstancesInstanceIdConfigLabelsLabelNameGetPathParams as GetLabelParams,
    InstancesInstanceIdConfigMountsBindContainerPathGetPathParams,
    InstancesInstanceIdConfigMountsBindGetPathParams, InstancesInstanceIdConfigMountsGetPathParams,
    InstancesInstanceIdConfigMountsVolumesGetPathParams,
    InstancesInstanceIdConfigMountsVolumesVolumeNameGetPathParams,
    InstancesInstanceIdConfigNetworksGetPathParams,
    InstancesInstanceIdConfigNetworksNetworkIdDeletePathParams,
    InstancesInstanceIdConfigNetworksNetworkIdGetPathParams,
    InstancesInstanceIdConfigNetworksPostPathParams, InstancesInstanceIdConfigNetworksPostRequest,
    InstancesInstanceIdConfigPortsDeletePathParams as DeletePortsParams,
    InstancesInstanceIdConfigPortsGetPathParams as GetPortsParams,
    InstancesInstanceIdConfigPortsTransportProtocolDeletePathParams as DeleteProtocolPortsParams,
    InstancesInstanceIdConfigPortsTransportProtocolGetPathParams as GetProtocolPortsParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeletePathParams as DeletePortRangeParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetPathParams as GetPortRangeParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutPathParams as PutPortRangeParams,
    InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutRequest as PutPortRangeRequest,
    InstancesInstanceIdConfigPortsTransportProtocolPutPathParams as PutProtocolPortsParams,
    InstancesInstanceIdDeletePathParams, InstancesInstanceIdEditorPortGetPathParams,
    InstancesInstanceIdGetPathParams, InstancesInstanceIdLogsGetPathParams,
    InstancesInstanceIdPatchPathParams, InstancesInstanceIdPatchRequest,
    InstancesInstanceIdStartPostPathParams, InstancesInstanceIdStopPostPathParams,
};
use http::Method;
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
    D: Deploymento,
    E: Exportius,
    IMP: Importius,
    F: Floxy + 'static,
    T: UsbDeviceReader + 'static,
    NET: NetworkAdapterReader + 'static,
    NetDev: NetDeviceReader,
> Instances for ServerImpl<APP, AUTH, I, L, Q, M, SYS, D, E, IMP, F, T, NET, NetDev>
{
    async fn instances_create_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: InstancesCreatePostRequest,
    ) -> Result<InstancesCreatePostResponse, ()> {
        super::api::v2::instances::create::post(
            self.vault.clone(),
            self.sorcerers.app_raiser.clone(),
            self.sorcerers.instancius.clone(),
            self.enchantments.quest_master.clone(),
            body,
        )
        .await
    }

    async fn instances_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        query_params: InstancesGetQueryParams,
    ) -> Result<InstancesGetResponse, ()> {
        Ok(super::api::v2::instances::get(
            self.vault.clone(),
            self.sorcerers.instancius.clone(),
            query_params,
        )
        .await)
    }

    async fn instances_instance_id_config_devices_usb_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigDevicesUsbDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigDevicesUsbDeleteResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::devices::usb::delete(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_devices_usb_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigDevicesUsbGetPathParams,
    ) -> Result<InstancesInstanceIdConfigDevicesUsbGetResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::devices::usb::get(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                self.usb_reader.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_devices_usb_port_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigDevicesUsbPortDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigDevicesUsbPortDeleteResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::devices::usb::port::delete(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_devices_usb_port_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigDevicesUsbPortGetPathParams,
    ) -> Result<InstancesInstanceIdConfigDevicesUsbPortGetResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::devices::usb::port::get(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                self.usb_reader.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_devices_usb_port_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigDevicesUsbPortPutPathParams,
    ) -> Result<InstancesInstanceIdConfigDevicesUsbPortPutResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::devices::usb::port::put(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                self.usb_reader.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_environment_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: DeleteEnvironmentParams,
    ) -> Result<DeleteEnvironmentResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::environment::delete(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_environment_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: GetEnvironmentParams,
    ) -> Result<GetEnvironmentResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::environment::get(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_environment_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: PutEnvironmentParams,
        body: InstanceEnvironment,
    ) -> Result<PutEnvironmentResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::environment::put(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
                body,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_environment_variable_name_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: DeleteEnvironmentVariableParams,
    ) -> Result<DeleteEnvironmentVariableResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::environment::variable_name::delete(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_environment_variable_name_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: GetEnvironmentVariableParams,
    ) -> Result<GetEnvironmentVariableResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::environment::variable_name::get(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_environment_variable_name_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: PutEnvironmentVariableParams,
        body: PutEnvironmentVariableRequest,
    ) -> Result<PutEnvironmentVariableResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::environment::variable_name::put(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
                body,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_hostname_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigHostnameGetPathParams,
    ) -> Result<InstancesInstanceIdConfigHostnameGetResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::hostname::get(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_hostname_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigHostnamePutPathParams,
        body: InstancesInstanceIdConfigHostnamePutRequest,
    ) -> Result<InstancesInstanceIdConfigHostnamePutResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::hostname::put(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
                body,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_labels_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: GetLabelsParams,
    ) -> Result<GetLabelsResponse, ()> {
        Ok(super::api::v2::instances::instance_id::config::labels::get(
            self.vault.clone(),
            self.sorcerers.instancius.clone(),
            path_params,
        )
        .await)
    }

    async fn instances_instance_id_config_labels_label_name_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: GetLabelParams,
    ) -> Result<GetLabelResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::labels::label_name::get(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_mounts_bind_container_path_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigMountsBindContainerPathGetPathParams,
    ) -> Result<InstancesInstanceIdConfigMountsBindContainerPathGetResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::mounts::bind::container_path::get(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_mounts_bind_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigMountsBindGetPathParams,
    ) -> Result<InstancesInstanceIdConfigMountsBindGetResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::mounts::bind::get(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_mounts_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigMountsGetPathParams,
    ) -> Result<InstancesInstanceIdConfigMountsGetResponse, ()> {
        Ok(super::api::v2::instances::instance_id::config::mounts::get(
            self.vault.clone(),
            self.sorcerers.instancius.clone(),
            path_params,
        )
        .await)
    }

    async fn instances_instance_id_config_mounts_volumes_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigMountsVolumesGetPathParams,
    ) -> Result<InstancesInstanceIdConfigMountsVolumesGetResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::mounts::volumes::get(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_mounts_volumes_volume_name_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigMountsVolumesVolumeNameGetPathParams,
    ) -> Result<InstancesInstanceIdConfigMountsVolumesVolumeNameGetResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::mounts::volumes::volume_name::get(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_networks_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigNetworksGetPathParams,
    ) -> Result<InstancesInstanceIdConfigNetworksGetResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::networks::get(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_networks_network_id_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigNetworksNetworkIdDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigNetworksNetworkIdDeleteResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::networks::network_id::delete(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_networks_network_id_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigNetworksNetworkIdGetPathParams,
    ) -> Result<InstancesInstanceIdConfigNetworksNetworkIdGetResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::networks::network_id::get(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_networks_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdConfigNetworksPostPathParams,
        body: InstancesInstanceIdConfigNetworksPostRequest,
    ) -> Result<InstancesInstanceIdConfigNetworksPostResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::networks::post(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
                body,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_ports_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: DeletePortsParams,
    ) -> Result<DeletePortsResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::ports::delete(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_ports_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: GetPortsParams,
    ) -> Result<GetPortsResponse, ()> {
        Ok(super::api::v2::instances::instance_id::config::ports::get(
            self.vault.clone(),
            self.sorcerers.instancius.clone(),
            path_params,
        )
        .await)
    }

    async fn instances_instance_id_config_ports_transport_protocol_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: DeleteProtocolPortsParams,
    ) -> Result<DeleteProtocolPortsResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::ports::transport_protocol::delete(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_ports_transport_protocol_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: GetProtocolPortsParams,
    ) -> Result<GetProtocolPortsResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::ports::transport_protocol::get(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_config_ports_transport_protocol_host_port_range_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: DeletePortRangeParams,
    ) -> Result<DeletePortRangeResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::ports::transport_protocol::host_port_range::delete(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params
            )
            .await
        )
    }

    async fn instances_instance_id_config_ports_transport_protocol_host_port_range_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: GetPortRangeParams,
    ) -> Result<GetPortRangeResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::ports::transport_protocol::host_port_range::get(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params
            )
            .await
        )
    }

    async fn instances_instance_id_config_ports_transport_protocol_host_port_range_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: PutPortRangeParams,
        body: PutPortRangeRequest,
    ) -> Result<PutPortRangeResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::ports::transport_protocol::host_port_range::put(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params, body
            )
            .await
        )
    }

    async fn instances_instance_id_config_ports_transport_protocol_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: PutProtocolPortsParams,
        body: Vec<models::InstancePortMapping>,
    ) -> Result<PutProtocolPortsResponse, ()> {
        Ok(
            super::api::v2::instances::instance_id::config::ports::transport_protocol::put(
                self.vault.clone(),
                self.sorcerers.instancius.clone(),
                path_params,
                body,
            )
            .await,
        )
    }

    async fn instances_instance_id_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdDeletePathParams,
    ) -> Result<InstancesInstanceIdDeleteResponse, ()> {
        Ok(super::api::v2::instances::instance_id::delete(
            self.vault.clone(),
            self.sorcerers.instancius.clone(),
            self.enchantments.floxy.clone(),
            self.enchantments.quest_master.clone(),
            path_params,
        )
        .await)
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
        super::api::v2::instances::instance_id::editor::port::get(
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
        Ok(super::api::v2::instances::instance_id::get(
            self.vault.clone(),
            self.sorcerers.instancius.clone(),
            path_params,
        )
        .await)
    }

    async fn instances_instance_id_logs_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdLogsGetPathParams,
    ) -> Result<InstancesInstanceIdLogsGetResponse, ()> {
        Ok(super::api::v2::instances::instance_id::logs::get(
            self.vault.clone(),
            self.sorcerers.instancius.clone(),
            path_params,
        )
        .await)
    }

    async fn instances_instance_id_patch(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdPatchPathParams,
        body: InstancesInstanceIdPatchRequest,
    ) -> Result<InstancesInstanceIdPatchResponse, ()> {
        Ok(super::api::v2::instances::instance_id::patch(
            self.vault.clone(),
            self.sorcerers.instancius.clone(),
            self.enchantments.floxy.clone(),
            self.enchantments.quest_master.clone(),
            path_params,
            body,
        )
        .await)
    }

    async fn instances_instance_id_start_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdStartPostPathParams,
    ) -> Result<InstancesInstanceIdStartPostResponse, ()> {
        super::api::v2::instances::instance_id::start::post(
            self.vault.clone(),
            self.sorcerers.instancius.clone(),
            self.enchantments.floxy.clone(),
            self.enchantments.quest_master.clone(),
            path_params,
        )
        .await
    }

    async fn instances_instance_id_stop_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdStopPostPathParams,
    ) -> Result<InstancesInstanceIdStopPostResponse, ()> {
        super::api::v2::instances::instance_id::stop::post(
            self.vault.clone(),
            self.sorcerers.instancius.clone(),
            self.enchantments.floxy.clone(),
            self.enchantments.quest_master.clone(),
            path_params,
        )
        .await
    }
}
