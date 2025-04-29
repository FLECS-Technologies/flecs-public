use crate::jeweler::gem::instance::compose::ComposeInstanceDeserializable;
use crate::jeweler::gem::instance::docker::DockerInstanceDeserializable;
use crate::jeweler::gem::instance::docker::config::{
    InstanceConfig, InstancePortMapping, UsbPathConfig,
};
use crate::jeweler::gem::instance::status::InstanceStatus;
use crate::jeweler::gem::manifest::single::{EnvironmentVariable, PortMapping};
use crate::jeweler::network::NetworkId;
use crate::relic::device::usb::{UsbDevice, UsbDeviceReader};
use crate::vault::pouch::AppKey;
use crate::vault::pouch::deployment::DeploymentId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct Deployment(Vec<Instance>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub instance_id: String,
    pub instance_name: String,
    pub app_key: AppKey,
    #[serde(rename = "status")]
    pub _status: String,
    pub desired: String,
    pub networks: Vec<Network>,
    #[serde(rename = "startupOptions")]
    pub _startup_options: Vec<u64>,
    pub usb_devices: Vec<Device>,
    #[serde(skip_serializing_if = "HashSet::is_empty", default)]
    pub environment: HashSet<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Network {
    pub ip_address: String,
    #[serde(rename = "macAddress")]
    pub _mac_address: String,
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub device: String,
    pub pid: u16,
    pub port: String,
    pub vendor: String,
    pub vid: u16,
}

fn migrate_environment<'a, I: Iterator<Item = &'a String>>(
    env: I,
) -> Result<Vec<EnvironmentVariable>, anyhow::Error> {
    env.map(|s| EnvironmentVariable::from_str(s.as_str()))
        .collect()
}

fn migrate_ports<'a, I: Iterator<Item = &'a String>>(
    ports: I,
) -> Result<InstancePortMapping, anyhow::Error> {
    let ports: Result<Vec<_>, _> = ports.map(|s| PortMapping::from_str(s.as_str())).collect();
    let ports = ports?;
    Ok(InstancePortMapping {
        tcp: ports.clone(),
        udp: ports,
        sctp: Vec::new(),
    })
}

impl From<Device> for UsbDevice {
    fn from(value: Device) -> Self {
        Self {
            vid: value.vid,
            pid: value.pid,
            port: value.port,
            device: value.device,
            vendor: value.vendor,
        }
    }
}

fn migrate_devices<I: Iterator<Item = Device>, U: UsbDeviceReader>(
    devices: I,
    usb_reader: &U,
) -> Result<HashMap<String, UsbPathConfig>, anyhow::Error> {
    devices
        .map(
            |device| match UsbPathConfig::try_from((&UsbDevice::from(device), usb_reader)) {
                Ok(device) => Ok((device.port.clone(), device)),
                Err(e) => Err(e),
            },
        )
        .collect()
}

pub fn migrate_docker_instance<U: UsbDeviceReader>(
    value: Instance,
    usb_device_reader: &U,
    default_deployment_id: &DeploymentId,
) -> Result<DockerInstanceDeserializable, anyhow::Error> {
    let id = FromStr::from_str(&value.instance_id)?;
    let connected_networks: Result<HashMap<NetworkId, IpAddr>, _> = value
        .networks
        .into_iter()
        .map(|s| match IpAddr::from_str(&s.ip_address) {
            Ok(address) => Ok((s.network, address)),
            Err(e) => Err(e),
        })
        .collect();
    let connected_networks = connected_networks?;
    let config = InstanceConfig {
        volume_mounts: Default::default(),
        environment_variables: migrate_environment(value.environment.iter())?,
        port_mapping: migrate_ports(value.ports.iter())?,
        connected_networks,
        usb_devices: migrate_devices(value.usb_devices.into_iter(), usb_device_reader)?,
        mapped_editor_ports: Default::default(),
    };
    Ok(DockerInstanceDeserializable {
        hostname: format!("flecs-{id}"),
        name: value.instance_name,
        id,
        config,
        desired: InstanceStatus::from(value.desired.as_str()),
        app_key: value.app_key,
        deployment_id: default_deployment_id.clone(),
    })
}

pub fn migrate_docker_deployment<U: UsbDeviceReader>(
    docker_deployment: Deployment,
    usb_device_reader: &U,
    default_deployment_id: &DeploymentId,
) -> Result<Vec<DockerInstanceDeserializable>, anyhow::Error> {
    docker_deployment
        .0
        .into_iter()
        .map(|instance| migrate_docker_instance(instance, usb_device_reader, default_deployment_id))
        .collect()
}

pub fn migrate_compose_instance(
    value: Instance,
    default_deployment_id: &DeploymentId,
) -> Result<ComposeInstanceDeserializable, anyhow::Error> {
    let id = FromStr::from_str(&value.instance_id)?;
    Ok(ComposeInstanceDeserializable {
        name: value.instance_name,
        id,
        desired: InstanceStatus::from(value.desired.as_str()),
        app_key: value.app_key,
        deployment_id: default_deployment_id.clone(),
    })
}

pub fn migrate_compose_deployment(
    compose_deployment: Deployment,
    default_deployment_id: &DeploymentId,
) -> Result<Vec<ComposeInstanceDeserializable>, anyhow::Error> {
    compose_deployment
        .0
        .into_iter()
        .map(|instance| migrate_compose_instance(instance, default_deployment_id))
        .collect()
}
