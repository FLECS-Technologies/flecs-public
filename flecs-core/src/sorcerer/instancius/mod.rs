mod instancius_impl;
pub use super::Result;
use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::jeweler::gem;
use crate::jeweler::gem::instance::{InstanceId, TransportProtocol, UsbPathConfig};
use crate::jeweler::gem::manifest::{EnvironmentVariable, Label, PortMapping, PortRange};
use crate::jeweler::instance::Logs;
use crate::jeweler::network::NetworkId;
use crate::quest::SyncQuest;
use crate::relic::device::usb::{UsbDevice, UsbDeviceReader};
pub use crate::sorcerer::spell::instance::DisconnectInstanceError;
use crate::sorcerer::Sorcerer;
use crate::vault::pouch::AppKey;
use crate::vault::Vault;
use async_trait::async_trait;
pub use instancius_impl::InstanciusImpl;
#[cfg(test)]
use mockall::automock;
use std::collections::HashMap;
use std::net::IpAddr;
use std::num::NonZeroU16;
use std::sync::Arc;

pub type UsbDevices = Vec<(UsbPathConfig, Option<UsbDevice>)>;

#[derive(Eq, PartialEq, Debug)]
pub enum GetInstanceUsbDeviceResult {
    UnknownDevice,
    DeviceNotMapped,
    InstanceNotFound,
    DeviceInactive(UsbPathConfig),
    DeviceActive(UsbPathConfig, UsbDevice),
}
#[derive(Eq, PartialEq, Debug)]
pub enum PutInstanceUsbDeviceResult {
    DeviceNotFound,
    InstanceNotFound,
    DeviceMappingCreated,
    DeviceMappingUpdated(UsbPathConfig),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RedirectEditorRequestResult {
    InstanceNotFound,
    UnknownPort,
    EditorSupportsReverseProxy,
    InstanceNotRunning,
    InstanceNotConnectedToNetwork,
    Redirected(u16),
}

#[derive(Debug, PartialEq, Eq)]
pub enum GetInstanceConfigNetworkResult {
    InstanceNotFound,
    UnknownNetwork,
    Network { name: String, address: IpAddr },
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Instancius: Sorcerer {
    async fn start_instance<F: Floxy + 'static>(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        instance_id: InstanceId,
    ) -> Result<()>;

    async fn stop_instance<F: Floxy + 'static>(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        instance_id: InstanceId,
    ) -> Result<()>;

    async fn get_instance(
        &self,
        vault: Arc<Vault>,
        instance_id: InstanceId,
    ) -> Result<Option<flecsd_axum_server::models::AppInstance>>;

    async fn get_instance_detailed(
        &self,
        vault: Arc<Vault>,
        instance_id: InstanceId,
    ) -> Result<Option<flecsd_axum_server::models::InstancesInstanceIdGet200Response>>;

    async fn get_instances_filtered(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        app_name: Option<String>,
        app_version: Option<String>,
    ) -> Vec<flecsd_axum_server::models::AppInstance>;

    async fn get_all_instances(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
    ) -> Vec<flecsd_axum_server::models::AppInstance>;

    async fn create_instance(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        app_key: AppKey,
        name: String,
    ) -> Result<InstanceId>;

    async fn does_instance_exist(&self, vault: Arc<Vault>, id: InstanceId) -> bool;

    async fn get_instance_config(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Option<gem::instance::config::InstanceConfig>;

    async fn get_instance_usb_devices<U: UsbDeviceReader + 'static>(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        usb_reader: Arc<U>,
    ) -> Result<Option<UsbDevices>>;

    async fn delete_instance_usb_devices(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Option<HashMap<String, UsbPathConfig>>;

    async fn delete_instance_usb_device(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        port: String,
    ) -> Option<Option<UsbPathConfig>>;

    async fn get_instance_usb_device<U: UsbDeviceReader + 'static>(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        port: String,
        usb_reader: Arc<U>,
    ) -> Result<GetInstanceUsbDeviceResult>;

    async fn put_instance_usb_device<U: UsbDeviceReader + 'static>(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        port: String,
        usb_reader: Arc<U>,
    ) -> Result<PutInstanceUsbDeviceResult>;

    async fn get_instance_config_port_mapping(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        host_port: u16,
        transport_protocol: TransportProtocol,
    ) -> Option<Option<PortMapping>>;

    async fn get_instance_config_port_mappings(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Option<gem::instance::config::InstancePortMapping>;

    async fn get_instance_config_protocol_port_mappings(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        transport_protocol: TransportProtocol,
    ) -> Option<Vec<PortMapping>>;

    async fn delete_instance_config_protocol_port_mappings(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        transport_protocol: TransportProtocol,
    ) -> Option<Vec<PortMapping>>;

    async fn delete_instance_config_port_mapping(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        host_port: u16,
        transport_protocol: TransportProtocol,
    ) -> Option<bool>;

    async fn delete_instance_config_port_mapping_range(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        host_port_range: PortRange,
        transport_protocol: TransportProtocol,
    ) -> Option<bool>;

    async fn get_instance_config_port_mapping_range(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        host_port_range: PortRange,
        transport_protocol: TransportProtocol,
    ) -> Option<Option<PortMapping>>;

    async fn put_instance_config_port_mapping(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        port_mapping: PortMapping,
        transport_protocol: TransportProtocol,
    ) -> Result<Option<bool>>;

    async fn put_instance_config_protocol_port_mappings(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        port_mapping: Vec<PortMapping>,
        transport_protocol: TransportProtocol,
    ) -> bool;

    async fn delete_instance_config_port_mappings(&self, vault: Arc<Vault>, id: InstanceId)
        -> bool;

    async fn get_instance_config_environment_variable_value(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        variable_name: String,
    ) -> Option<Option<Option<String>>>;

    async fn put_instance_config_environment_variable_value(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        environment_variable: EnvironmentVariable,
    ) -> Option<Option<String>>;

    async fn delete_instance_config_environment_variable_value(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        variable_name: String,
    ) -> Option<Option<EnvironmentVariable>>;

    async fn get_instance_config_environment(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Option<Vec<EnvironmentVariable>>;

    async fn put_instance_config_environment(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        environment: Vec<EnvironmentVariable>,
    ) -> Option<Vec<EnvironmentVariable>>;

    async fn delete_instance_config_environment(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Option<Vec<EnvironmentVariable>>;

    async fn get_instance_config_networks(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Option<HashMap<String, IpAddr>>;

    async fn get_instance_config_network(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        network_id: NetworkId,
    ) -> GetInstanceConfigNetworkResult;

    async fn disconnect_instance_from_network(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        network_id: NetworkId,
    ) -> Result<IpAddr, DisconnectInstanceError>;

    async fn delete_instance<F: Floxy + 'static>(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        id: InstanceId,
    ) -> Result<()>;

    async fn get_instance_logs(&self, vault: Arc<Vault>, id: InstanceId) -> Result<Logs>;

    async fn get_instance_labels(&self, vault: Arc<Vault>, id: InstanceId) -> Option<Vec<Label>>;

    async fn get_instance_label_value(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        label_name: String,
    ) -> Option<Option<Option<String>>>;

    async fn redirect_editor_request<F: Floxy + 'static>(
        &self,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        instance_id: InstanceId,
        port: NonZeroU16,
    ) -> Result<RedirectEditorRequestResult>;
}

#[cfg(test)]
impl Sorcerer for MockInstancius {}
