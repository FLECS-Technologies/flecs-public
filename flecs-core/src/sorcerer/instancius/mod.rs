mod instancius_impl;
pub use super::Result;
use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::jeweler::gem;
use crate::jeweler::gem::instance::docker::config::{
    InstancePortMapping, TransportProtocol, UsbPathConfig,
};
use crate::jeweler::gem::instance::{InstanceId, Logs};
use crate::jeweler::gem::manifest::single::{
    BindMount, EnvironmentVariable, Label, PortMapping, PortRange, VolumeMount,
};
use crate::jeweler::network::NetworkId;
use crate::jeweler::volume::VolumeId;
use crate::quest::SyncQuest;
use crate::relic::device::usb::{UsbDevice, UsbDeviceReader};
use crate::sorcerer::Sorcerer;
pub use crate::sorcerer::spell::instance::DisconnectInstanceError;
pub use crate::sorcerer::spell::instance::QueryInstanceConfigError;
use crate::sorcerer::spell::instance::UpdateInstanceError;
use crate::vault::Vault;
use crate::vault::pouch::AppKey;
use anyhow::Error;
use async_trait::async_trait;
pub use instancius_impl::InstanciusImpl;
#[cfg(test)]
use mockall::automock;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::num::NonZeroU16;
use std::path::PathBuf;
use std::sync::Arc;

pub type UsbDevices = Vec<(UsbPathConfig, Option<UsbDevice>)>;

#[derive(Eq, PartialEq, Debug)]
pub enum GetInstanceUsbDeviceResult {
    UnknownDevice,
    DeviceNotMapped,
    InstanceNotFound,
    DeviceInactive(UsbPathConfig),
    DeviceActive(UsbPathConfig, UsbDevice),
    NotSupported,
}
#[derive(Eq, PartialEq, Debug)]
pub enum PutInstanceUsbDeviceResult {
    DeviceNotFound,
    InstanceNotFound,
    DeviceMappingCreated,
    DeviceMappingUpdated(UsbPathConfig),
    NotSupported,
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
    NotSupported,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum GetInstanceConfigBindMountError {
    #[error("Instance not found {0}")]
    InstanceNotFound(InstanceId),
    #[error("No bind mound with container path {0} found")]
    BindMountNotFound(PathBuf),
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum GetInstanceConfigVolumeMountError {
    #[error("Instance not found {0}")]
    InstanceNotFound(InstanceId),
    #[error("Instance {0} does not support volume mounts")]
    NotSupported(InstanceId),
    #[error("No volume with name {0} found")]
    VolumeMountNotFound(String),
}

impl From<QueryInstanceConfigError> for GetInstanceConfigVolumeMountError {
    fn from(value: QueryInstanceConfigError) -> Self {
        match value {
            QueryInstanceConfigError::NotFound(id) => Self::InstanceNotFound(id),
            QueryInstanceConfigError::NotSupported(id) => Self::NotSupported(id),
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ConnectInstanceConfigNetworkError {
    #[error("Instance not found: {0}")]
    InstanceNotFound(InstanceId),
    #[error("No free ip address available")]
    NoFreeAddress,
    #[error("Given address '{address}' is not part of network '{network}'")]
    AddressOutOfRange { address: IpAddr, network: NetworkId },
    #[error("Given network '{network}' is invalid: {reason}")]
    InvalidNetwork { network: NetworkId, reason: String },
    #[error("Network not found: {0}")]
    NetworkNotFound(NetworkId),
    #[error("Instance already connected to network '{0}'")]
    NetworkAlreadyConnected(NetworkId),
    #[error("Instance {0} does not support connecting to networks")]
    Unsupported(InstanceId),
    #[error("Failed to connect instance: {0}")]
    Other(String),
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum InstanceConfigHostnameError {
    #[error("Instance not found: {0}")]
    InstanceNotFound(InstanceId),
    #[error("Instance {0} does not support hostnames")]
    Unsupported(InstanceId),
}

impl From<anyhow::Error> for ConnectInstanceConfigNetworkError {
    fn from(value: Error) -> Self {
        Self::Other(value.to_string())
    }
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

    async fn halt_all_instances<F: Floxy + 'static>(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
    ) -> Result<()>;

    async fn start_all_instances_as_desired<F: Floxy + 'static>(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
    ) -> Result<()>;

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
    ) -> Result<gem::instance::docker::config::InstanceConfig, QueryInstanceConfigError>;

    async fn get_instance_hostname(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Result<String, InstanceConfigHostnameError>;

    async fn put_instance_hostname(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        hostname: String,
    ) -> Result<String, InstanceConfigHostnameError>;

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
    ) -> Result<HashMap<String, UsbPathConfig>, QueryInstanceConfigError>;

    async fn delete_instance_usb_device(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        port: String,
    ) -> std::result::Result<Option<UsbPathConfig>, QueryInstanceConfigError>;

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
    ) -> std::result::Result<Option<PortMapping>, QueryInstanceConfigError>;

    async fn get_instance_config_port_mappings(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> std::result::Result<InstancePortMapping, QueryInstanceConfigError>;

    async fn get_instance_config_protocol_port_mappings(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        transport_protocol: TransportProtocol,
    ) -> std::result::Result<Vec<PortMapping>, QueryInstanceConfigError>;

    async fn delete_instance_config_protocol_port_mappings(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        transport_protocol: TransportProtocol,
    ) -> std::result::Result<Vec<PortMapping>, QueryInstanceConfigError>;

    async fn delete_instance_config_port_mapping(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        host_port: u16,
        transport_protocol: TransportProtocol,
    ) -> std::result::Result<bool, QueryInstanceConfigError>;

    async fn delete_instance_config_port_mapping_range(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        host_port_range: PortRange,
        transport_protocol: TransportProtocol,
    ) -> Result<bool, QueryInstanceConfigError>;

    async fn get_instance_config_port_mapping_range(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        host_port_range: PortRange,
        transport_protocol: TransportProtocol,
    ) -> std::result::Result<Option<PortMapping>, QueryInstanceConfigError>;

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
    ) -> std::result::Result<(), QueryInstanceConfigError>;

    async fn delete_instance_config_port_mappings(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> std::result::Result<(), QueryInstanceConfigError>;

    async fn get_instance_config_environment_variable_value(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        variable_name: String,
    ) -> std::result::Result<Option<Option<String>>, QueryInstanceConfigError>;

    async fn put_instance_config_environment_variable_value(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        environment_variable: EnvironmentVariable,
    ) -> std::result::Result<Option<String>, QueryInstanceConfigError>;

    async fn delete_instance_config_environment_variable_value(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        variable_name: String,
    ) -> std::result::Result<Option<EnvironmentVariable>, QueryInstanceConfigError>;

    async fn get_instance_config_environment(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> std::result::Result<Vec<EnvironmentVariable>, QueryInstanceConfigError>;

    async fn put_instance_config_environment(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        environment: Vec<EnvironmentVariable>,
    ) -> std::result::Result<Vec<EnvironmentVariable>, QueryInstanceConfigError>;

    async fn delete_instance_config_environment(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> std::result::Result<Vec<EnvironmentVariable>, QueryInstanceConfigError>;

    async fn get_instance_config_networks(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> std::result::Result<HashMap<String, IpAddr>, QueryInstanceConfigError>;

    async fn get_instance_config_network(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        network_id: NetworkId,
    ) -> GetInstanceConfigNetworkResult;

    async fn get_instance_config_mounts(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Option<(Vec<VolumeMount>, Vec<BindMount>)>;

    async fn get_instance_config_volume_mounts(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> std::result::Result<Vec<VolumeMount>, QueryInstanceConfigError>;

    async fn get_instance_config_volume_mount(
        &self,
        vault: Arc<Vault>,
        instance_id: InstanceId,
        volume_id: VolumeId,
    ) -> Result<VolumeMount, GetInstanceConfigVolumeMountError>;

    async fn get_instance_config_bind_mounts(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Option<Vec<BindMount>>;

    async fn get_instance_config_bind_mount(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        container_path: PathBuf,
    ) -> Result<BindMount, GetInstanceConfigBindMountError>;

    async fn disconnect_instance_from_network(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        network_id: NetworkId,
    ) -> Result<IpAddr, DisconnectInstanceError>;

    async fn connect_instance_to_network(
        &self,
        vault: Arc<Vault>,
        network_id: NetworkId,
        id: InstanceId,
        address: Option<Ipv4Addr>,
    ) -> Result<IpAddr, ConnectInstanceConfigNetworkError>;

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

    async fn update_instance<F: Floxy + 'static>(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        instance_id: InstanceId,
        new_version: String,
        base_path: PathBuf,
    ) -> Result<(), UpdateInstanceError>;
}

#[cfg(test)]
impl Sorcerer for MockInstancius {}
