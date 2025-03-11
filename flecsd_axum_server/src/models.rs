#![allow(unused_qualifications)]

use http::HeaderValue;
use validator::Validate;

#[cfg(feature = "server")]
use crate::header;
use crate::{models, types::*};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AppsAppDeletePathParams {
    pub app: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AppsAppDeleteQueryParams {
    #[serde(rename = "version")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AppsAppGetPathParams {
    pub app: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AppsAppGetQueryParams {
    #[serde(rename = "version")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FlunderBrowseGetQueryParams {
    #[serde(rename = "q")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesGetQueryParams {
    #[serde(rename = "app")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    #[serde(rename = "version")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigDevicesUsbDeletePathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGDEVICESUSBDELETEPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGDEVICESUSBDELETEPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigDevicesUsbGetPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGDEVICESUSBGETPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGDEVICESUSBGETPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigDevicesUsbPortDeletePathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGDEVICESUSBPORTDELETEPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGDEVICESUSBPORTDELETEPATHPARAMS_PORT),
                    )]
    pub port: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGDEVICESUSBPORTDELETEPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}
lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGDEVICESUSBPORTDELETEPATHPARAMS_PORT: regex::Regex = regex::Regex::new("^usb[1-9][0-9]*|[1-9][0-9]*-[1-9][0-9]*(?:\\.[1-9][0-9]*)*$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigDevicesUsbPortGetPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGDEVICESUSBPORTGETPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGDEVICESUSBPORTGETPATHPARAMS_PORT),
                    )]
    pub port: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGDEVICESUSBPORTGETPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}
lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGDEVICESUSBPORTGETPATHPARAMS_PORT: regex::Regex = regex::Regex::new("^usb[1-9][0-9]*|[1-9][0-9]*-[1-9][0-9]*(?:\\.[1-9][0-9]*)*$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigDevicesUsbPortPutPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGDEVICESUSBPORTPUTPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGDEVICESUSBPORTPUTPATHPARAMS_PORT),
                    )]
    pub port: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGDEVICESUSBPORTPUTPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}
lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGDEVICESUSBPORTPUTPATHPARAMS_PORT: regex::Regex = regex::Regex::new("^usb[1-9][0-9]*|[1-9][0-9]*-[1-9][0-9]*(?:\\.[1-9][0-9]*)*$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigEnvironmentDeletePathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTDELETEPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTDELETEPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigEnvironmentGetPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTGETPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTGETPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigEnvironmentPutPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTPUTPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTPUTPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigEnvironmentVariableNameDeletePathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTVARIABLENAMEDELETEPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTVARIABLENAMEDELETEPATHPARAMS_VARIABLE_NAME),
                    )]
    pub variable_name: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTVARIABLENAMEDELETEPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}
lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTVARIABLENAMEDELETEPATHPARAMS_VARIABLE_NAME: regex::Regex = regex::Regex::new("^[a-zA-Z]+(?:[a-zA-Z0-9_\\-\\.])*$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigEnvironmentVariableNameGetPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTVARIABLENAMEGETPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTVARIABLENAMEGETPATHPARAMS_VARIABLE_NAME),
                    )]
    pub variable_name: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTVARIABLENAMEGETPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}
lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTVARIABLENAMEGETPATHPARAMS_VARIABLE_NAME: regex::Regex = regex::Regex::new("^[a-zA-Z]+(?:[a-zA-Z0-9_\\-\\.])*$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigEnvironmentVariableNamePutPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTVARIABLENAMEPUTPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTVARIABLENAMEPUTPATHPARAMS_VARIABLE_NAME),
                    )]
    pub variable_name: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTVARIABLENAMEPUTPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}
lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGENVIRONMENTVARIABLENAMEPUTPATHPARAMS_VARIABLE_NAME: regex::Regex = regex::Regex::new("^[a-zA-Z]+(?:[a-zA-Z0-9_\\-\\.])*$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigGetPathParams {
    pub instance_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigLabelsGetPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGLABELSGETPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGLABELSGETPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigLabelsLabelNameGetPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGLABELSLABELNAMEGETPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGLABELSLABELNAMEGETPATHPARAMS_LABEL_NAME),
                    )]
    pub label_name: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGLABELSLABELNAMEGETPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}
lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGLABELSLABELNAMEGETPATHPARAMS_LABEL_NAME: regex::Regex = regex::Regex::new("^[a-z](?:(?:[\\-\\.]?[a-zA-Z0-9])*[\\-\\.]?[a-z])?$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigPortsDeletePathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGPORTSDELETEPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGPORTSDELETEPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigPortsGetPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGPORTSGETPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGPORTSGETPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigPortsTransportProtocolDeletePathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLDELETEPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
    pub transport_protocol: models::TransportProtocol,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLDELETEPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigPortsTransportProtocolGetPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLGETPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
    pub transport_protocol: models::TransportProtocol,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLGETPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeletePathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLHOSTPORTRANGEDELETEPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
    pub transport_protocol: models::TransportProtocol,
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLHOSTPORTRANGEDELETEPATHPARAMS_HOST_PORT_RANGE),
                    )]
    pub host_port_range: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLHOSTPORTRANGEDELETEPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}
lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLHOSTPORTRANGEDELETEPATHPARAMS_HOST_PORT_RANGE: regex::Regex = regex::Regex::new("^[0-9]+(?:-[0-9]+)?$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLHOSTPORTRANGEGETPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
    pub transport_protocol: models::TransportProtocol,
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLHOSTPORTRANGEGETPATHPARAMS_HOST_PORT_RANGE),
                    )]
    pub host_port_range: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLHOSTPORTRANGEGETPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}
lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLHOSTPORTRANGEGETPATHPARAMS_HOST_PORT_RANGE: regex::Regex = regex::Regex::new("^[0-9]+(?:-[0-9]+)?$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLHOSTPORTRANGEPUTPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
    pub transport_protocol: models::TransportProtocol,
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLHOSTPORTRANGEPUTPATHPARAMS_HOST_PORT_RANGE),
                    )]
    pub host_port_range: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLHOSTPORTRANGEPUTPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}
lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLHOSTPORTRANGEPUTPATHPARAMS_HOST_PORT_RANGE: regex::Regex = regex::Regex::new("^[0-9]+(?:-[0-9]+)?$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigPortsTransportProtocolPutPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLPUTPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
    pub transport_protocol: models::TransportProtocol,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDCONFIGPORTSTRANSPORTPROTOCOLPUTPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigPostPathParams {
    pub instance_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdDeletePathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDDELETEPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDDELETEPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdEditorPortGetPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDEDITORPORTGETPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
    #[validate(range(min = 1i32, max = 65535i32))]
    pub port: i32,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDEDITORPORTGETPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdGetPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDGETPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDGETPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdLogsGetPathParams {
    pub instance_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdPatchPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDPATCHPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDPATCHPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdStartPostPathParams {
    #[validate(
                          regex(path = *RE_INSTANCESINSTANCEIDSTARTPOSTPATHPARAMS_INSTANCE_ID),
                    )]
    pub instance_id: String,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDSTARTPOSTPATHPARAMS_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdStopPostPathParams {
    pub instance_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct JobsJobIdDeletePathParams {
    pub job_id: i32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct JobsJobIdGetPathParams {
    pub job_id: i32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SystemDevicesUsbPortGetPathParams {
    #[validate(
                          regex(path = *RE_SYSTEMDEVICESUSBPORTGETPATHPARAMS_PORT),
                    )]
    pub port: String,
}

lazy_static::lazy_static! {
    static ref RE_SYSTEMDEVICESUSBPORTGETPATHPARAMS_PORT: regex::Regex = regex::Regex::new("^usb[1-9][0-9]*|[1-9][0-9]*-[1-9][0-9]*(?:\\.[1-9][0-9]*)*$").unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SystemNetworkAdaptersNetworkAdapterIdGetPathParams {
    pub network_adapter_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SystemNetworksNetworkIdDhcpIpv4PostPathParams {
    pub network_id: String,
}

/// Additional info
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AdditionalInfo {
    #[serde(rename = "additionalInfo")]
    pub additional_info: String,
}

impl AdditionalInfo {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(additional_info: String) -> AdditionalInfo {
        AdditionalInfo { additional_info }
    }
}

/// Converts the AdditionalInfo value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for AdditionalInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("additionalInfo".to_string()),
            Some(self.additional_info.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AdditionalInfo value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AdditionalInfo {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub additional_info: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AdditionalInfo".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "additionalInfo" => intermediate_rep.additional_info.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AdditionalInfo".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AdditionalInfo {
            additional_info: intermediate_rep
                .additional_info
                .into_iter()
                .next()
                .ok_or_else(|| "additionalInfo missing in AdditionalInfo".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AdditionalInfo> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<AdditionalInfo>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AdditionalInfo>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AdditionalInfo - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<AdditionalInfo> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AdditionalInfo as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AdditionalInfo - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AppEditor {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "port")]
    #[validate(range(min = 1u16, max = 65535u16))]
    pub port: u16,

    #[serde(rename = "supportsReverseProxy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_reverse_proxy: Option<bool>,
}

impl AppEditor {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(name: String, port: u16) -> AppEditor {
        AppEditor {
            name,
            port,
            supports_reverse_proxy: Some(true),
        }
    }
}

/// Converts the AppEditor value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for AppEditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("name".to_string()),
            Some(self.name.to_string()),
            Some("port".to_string()),
            Some(self.port.to_string()),
            self.supports_reverse_proxy
                .as_ref()
                .map(|supports_reverse_proxy| {
                    [
                        "supportsReverseProxy".to_string(),
                        supports_reverse_proxy.to_string(),
                    ]
                    .join(",")
                }),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AppEditor value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AppEditor {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub name: Vec<String>,
            pub port: Vec<u16>,
            pub supports_reverse_proxy: Vec<bool>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AppEditor".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "name" => intermediate_rep.name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "port" => intermediate_rep.port.push(
                        <u16 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "supportsReverseProxy" => intermediate_rep.supports_reverse_proxy.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AppEditor".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AppEditor {
            name: intermediate_rep
                .name
                .into_iter()
                .next()
                .ok_or_else(|| "name missing in AppEditor".to_string())?,
            port: intermediate_rep
                .port
                .into_iter()
                .next()
                .ok_or_else(|| "port missing in AppEditor".to_string())?,
            supports_reverse_proxy: intermediate_rep.supports_reverse_proxy.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AppEditor> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<AppEditor>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AppEditor>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AppEditor - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<AppEditor> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AppEditor as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AppEditor - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Instance of an App
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AppInstance {
    #[serde(rename = "instanceId")]
    #[validate(
            regex(path = *RE_APPINSTANCE_INSTANCE_ID),
        )]
    pub instance_id: String,

    /// Instance name
    #[serde(rename = "instanceName")]
    pub instance_name: String,

    #[serde(rename = "appKey")]
    pub app_key: models::AppKey,

    #[serde(rename = "status")]
    pub status: models::InstanceStatus,

    #[serde(rename = "desired")]
    pub desired: models::InstanceStatus,

    #[serde(rename = "editors")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editors: Option<models::InstanceEditors>,
}

lazy_static::lazy_static! {
    static ref RE_APPINSTANCE_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

impl AppInstance {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        instance_id: String,
        instance_name: String,
        app_key: models::AppKey,
        status: models::InstanceStatus,
        desired: models::InstanceStatus,
    ) -> AppInstance {
        AppInstance {
            instance_id,
            instance_name,
            app_key,
            status,
            desired,
            editors: None,
        }
    }
}

/// Converts the AppInstance value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for AppInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("instanceId".to_string()),
            Some(self.instance_id.to_string()),
            Some("instanceName".to_string()),
            Some(self.instance_name.to_string()),
            // Skipping appKey in query parameter serialization

            // Skipping status in query parameter serialization

            // Skipping desired in query parameter serialization

            // Skipping editors in query parameter serialization
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AppInstance value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AppInstance {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub instance_id: Vec<String>,
            pub instance_name: Vec<String>,
            pub app_key: Vec<models::AppKey>,
            pub status: Vec<models::InstanceStatus>,
            pub desired: Vec<models::InstanceStatus>,
            pub editors: Vec<models::InstanceEditors>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AppInstance".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "instanceId" => intermediate_rep.instance_id.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "instanceName" => intermediate_rep.instance_name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "appKey" => intermediate_rep.app_key.push(
                        <models::AppKey as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep.status.push(
                        <models::InstanceStatus as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "desired" => intermediate_rep.desired.push(
                        <models::InstanceStatus as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "editors" => intermediate_rep.editors.push(
                        <models::InstanceEditors as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AppInstance".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AppInstance {
            instance_id: intermediate_rep
                .instance_id
                .into_iter()
                .next()
                .ok_or_else(|| "instanceId missing in AppInstance".to_string())?,
            instance_name: intermediate_rep
                .instance_name
                .into_iter()
                .next()
                .ok_or_else(|| "instanceName missing in AppInstance".to_string())?,
            app_key: intermediate_rep
                .app_key
                .into_iter()
                .next()
                .ok_or_else(|| "appKey missing in AppInstance".to_string())?,
            status: intermediate_rep
                .status
                .into_iter()
                .next()
                .ok_or_else(|| "status missing in AppInstance".to_string())?,
            desired: intermediate_rep
                .desired
                .into_iter()
                .next()
                .ok_or_else(|| "desired missing in AppInstance".to_string())?,
            editors: intermediate_rep.editors.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AppInstance> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<AppInstance>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AppInstance>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AppInstance - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<AppInstance> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AppInstance as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AppInstance - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AppKey {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "version")]
    pub version: String,
}

impl AppKey {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(name: String, version: String) -> AppKey {
        AppKey { name, version }
    }
}

/// Converts the AppKey value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for AppKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("name".to_string()),
            Some(self.name.to_string()),
            Some("version".to_string()),
            Some(self.version.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AppKey value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AppKey {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub name: Vec<String>,
            pub version: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AppKey".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "name" => intermediate_rep.name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "version" => intermediate_rep.version.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AppKey".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AppKey {
            name: intermediate_rep
                .name
                .into_iter()
                .next()
                .ok_or_else(|| "name missing in AppKey".to_string())?,
            version: intermediate_rep
                .version
                .into_iter()
                .next()
                .ok_or_else(|| "version missing in AppKey".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AppKey> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<AppKey>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AppKey>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AppKey - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<AppKey> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AppKey as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AppKey - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum AppStatus {
    #[serde(rename = "not installed")]
    NotInstalled,
    #[serde(rename = "manifest downloaded")]
    ManifestDownloaded,
    #[serde(rename = "token acquired")]
    TokenAcquired,
    #[serde(rename = "image downloaded")]
    ImageDownloaded,
    #[serde(rename = "installed")]
    Installed,
    #[serde(rename = "removed")]
    Removed,
    #[serde(rename = "purged")]
    Purged,
    #[serde(rename = "orphaned")]
    Orphaned,
    #[serde(rename = "unknown")]
    Unknown,
}

impl std::fmt::Display for AppStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            AppStatus::NotInstalled => write!(f, "not installed"),
            AppStatus::ManifestDownloaded => write!(f, "manifest downloaded"),
            AppStatus::TokenAcquired => write!(f, "token acquired"),
            AppStatus::ImageDownloaded => write!(f, "image downloaded"),
            AppStatus::Installed => write!(f, "installed"),
            AppStatus::Removed => write!(f, "removed"),
            AppStatus::Purged => write!(f, "purged"),
            AppStatus::Orphaned => write!(f, "orphaned"),
            AppStatus::Unknown => write!(f, "unknown"),
        }
    }
}

impl std::str::FromStr for AppStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "not installed" => std::result::Result::Ok(AppStatus::NotInstalled),
            "manifest downloaded" => std::result::Result::Ok(AppStatus::ManifestDownloaded),
            "token acquired" => std::result::Result::Ok(AppStatus::TokenAcquired),
            "image downloaded" => std::result::Result::Ok(AppStatus::ImageDownloaded),
            "installed" => std::result::Result::Ok(AppStatus::Installed),
            "removed" => std::result::Result::Ok(AppStatus::Removed),
            "purged" => std::result::Result::Ok(AppStatus::Purged),
            "orphaned" => std::result::Result::Ok(AppStatus::Orphaned),
            "unknown" => std::result::Result::Ok(AppStatus::Unknown),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AppsInstallPostRequest {
    #[serde(rename = "appKey")]
    pub app_key: models::AppKey,
}

impl AppsInstallPostRequest {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(app_key: models::AppKey) -> AppsInstallPostRequest {
        AppsInstallPostRequest { app_key }
    }
}

/// Converts the AppsInstallPostRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for AppsInstallPostRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping appKey in query parameter serialization

        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AppsInstallPostRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AppsInstallPostRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub app_key: Vec<models::AppKey>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AppsInstallPostRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "appKey" => intermediate_rep.app_key.push(
                        <models::AppKey as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AppsInstallPostRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AppsInstallPostRequest {
            app_key: intermediate_rep
                .app_key
                .into_iter()
                .next()
                .ok_or_else(|| "appKey missing in AppsInstallPostRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AppsInstallPostRequest> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<AppsInstallPostRequest>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AppsInstallPostRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AppsInstallPostRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<AppsInstallPostRequest> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AppsInstallPostRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AppsInstallPostRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AppsSideloadPostRequest {
    #[serde(rename = "manifest")]
    pub manifest: String,
}

impl AppsSideloadPostRequest {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(manifest: String) -> AppsSideloadPostRequest {
        AppsSideloadPostRequest { manifest }
    }
}

/// Converts the AppsSideloadPostRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for AppsSideloadPostRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("manifest".to_string()),
            Some(self.manifest.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AppsSideloadPostRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AppsSideloadPostRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub manifest: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AppsSideloadPostRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "manifest" => intermediate_rep.manifest.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AppsSideloadPostRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AppsSideloadPostRequest {
            manifest: intermediate_rep
                .manifest
                .into_iter()
                .next()
                .ok_or_else(|| "manifest missing in AppsSideloadPostRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AppsSideloadPostRequest> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<AppsSideloadPostRequest>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AppsSideloadPostRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AppsSideloadPostRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<AppsSideloadPostRequest> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AppsSideloadPostRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AppsSideloadPostRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AuthResponseData {
    #[serde(rename = "user")]
    pub user: models::User,

    #[serde(rename = "jwt")]
    pub jwt: models::Jwt,

    #[serde(rename = "feature_flags")]
    pub feature_flags: models::FeatureFlags,
}

impl AuthResponseData {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        user: models::User,
        jwt: models::Jwt,
        feature_flags: models::FeatureFlags,
    ) -> AuthResponseData {
        AuthResponseData {
            user,
            jwt,
            feature_flags,
        }
    }
}

/// Converts the AuthResponseData value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for AuthResponseData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping user in query parameter serialization

            // Skipping jwt in query parameter serialization

            // Skipping feature_flags in query parameter serialization

        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AuthResponseData value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AuthResponseData {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub user: Vec<models::User>,
            pub jwt: Vec<models::Jwt>,
            pub feature_flags: Vec<models::FeatureFlags>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AuthResponseData".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "user" => intermediate_rep.user.push(
                        <models::User as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "jwt" => intermediate_rep.jwt.push(
                        <models::Jwt as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "feature_flags" => intermediate_rep.feature_flags.push(
                        <models::FeatureFlags as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AuthResponseData".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AuthResponseData {
            user: intermediate_rep
                .user
                .into_iter()
                .next()
                .ok_or_else(|| "user missing in AuthResponseData".to_string())?,
            jwt: intermediate_rep
                .jwt
                .into_iter()
                .next()
                .ok_or_else(|| "jwt missing in AuthResponseData".to_string())?,
            feature_flags: intermediate_rep
                .feature_flags
                .into_iter()
                .next()
                .ok_or_else(|| "feature_flags missing in AuthResponseData".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AuthResponseData> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<AuthResponseData>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AuthResponseData>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AuthResponseData - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<AuthResponseData> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AuthResponseData as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AuthResponseData - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct DeviceLicenseActivationStatusGet200Response {
    #[serde(rename = "isValid")]
    pub is_valid: bool,
}

impl DeviceLicenseActivationStatusGet200Response {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(is_valid: bool) -> DeviceLicenseActivationStatusGet200Response {
        DeviceLicenseActivationStatusGet200Response { is_valid }
    }
}

/// Converts the DeviceLicenseActivationStatusGet200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for DeviceLicenseActivationStatusGet200Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> =
            vec![Some("isValid".to_string()), Some(self.is_valid.to_string())];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a DeviceLicenseActivationStatusGet200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for DeviceLicenseActivationStatusGet200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub is_valid: Vec<bool>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val =
                match string_iter.next() {
                    Some(x) => x,
                    None => return std::result::Result::Err(
                        "Missing value while parsing DeviceLicenseActivationStatusGet200Response"
                            .to_string(),
                    ),
                };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "isValid" => intermediate_rep.is_valid.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => return std::result::Result::Err(
                        "Unexpected key while parsing DeviceLicenseActivationStatusGet200Response"
                            .to_string(),
                    ),
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(DeviceLicenseActivationStatusGet200Response {
            is_valid: intermediate_rep
                .is_valid
                .into_iter()
                .next()
                .ok_or_else(|| {
                    "isValid missing in DeviceLicenseActivationStatusGet200Response".to_string()
                })?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<DeviceLicenseActivationStatusGet200Response> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<DeviceLicenseActivationStatusGet200Response>>
    for HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<DeviceLicenseActivationStatusGet200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for DeviceLicenseActivationStatusGet200Response - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue>
    for header::IntoHeaderValue<DeviceLicenseActivationStatusGet200Response>
{
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <DeviceLicenseActivationStatusGet200Response as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into DeviceLicenseActivationStatusGet200Response - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct DeviceLicenseInfoGet200Response {
    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(rename = "license")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    #[serde(rename = "sessionId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<models::SessionId>,
}

impl DeviceLicenseInfoGet200Response {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(r#type: String) -> DeviceLicenseInfoGet200Response {
        DeviceLicenseInfoGet200Response {
            r#type,
            license: None,
            session_id: None,
        }
    }
}

/// Converts the DeviceLicenseInfoGet200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for DeviceLicenseInfoGet200Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("type".to_string()),
            Some(self.r#type.to_string()),
            self.license
                .as_ref()
                .map(|license| ["license".to_string(), license.to_string()].join(",")),
            // Skipping sessionId in query parameter serialization
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a DeviceLicenseInfoGet200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for DeviceLicenseInfoGet200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub r#type: Vec<String>,
            pub license: Vec<String>,
            pub session_id: Vec<models::SessionId>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing DeviceLicenseInfoGet200Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "type" => intermediate_rep.r#type.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "license" => intermediate_rep.license.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "sessionId" => intermediate_rep.session_id.push(
                        <models::SessionId as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing DeviceLicenseInfoGet200Response"
                                .to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(DeviceLicenseInfoGet200Response {
            r#type: intermediate_rep
                .r#type
                .into_iter()
                .next()
                .ok_or_else(|| "type missing in DeviceLicenseInfoGet200Response".to_string())?,
            license: intermediate_rep.license.into_iter().next(),
            session_id: intermediate_rep.session_id.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<DeviceLicenseInfoGet200Response> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<DeviceLicenseInfoGet200Response>>
    for HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<DeviceLicenseInfoGet200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for DeviceLicenseInfoGet200Response - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue>
    for header::IntoHeaderValue<DeviceLicenseInfoGet200Response>
{
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <DeviceLicenseInfoGet200Response as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into DeviceLicenseInfoGet200Response - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Devices {
    #[serde(rename = "usb")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usb: Option<Vec<models::UsbDevice>>,
}

impl Devices {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new() -> Devices {
        Devices { usb: None }
    }
}

/// Converts the Devices value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Devices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping usb in query parameter serialization

        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Devices value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Devices {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub usb: Vec<Vec<models::UsbDevice>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Devices".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "usb" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Devices"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Devices".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Devices {
            usb: intermediate_rep.usb.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Devices> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Devices>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Devices>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Devices - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Devices> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Devices as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Devices - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Device Onboarding Service Manifest
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Dosschema {
    #[serde(rename = "_schemaVersion")]
    #[validate(length(min = 1))]
    pub _schema_version: String,

    #[serde(rename = "time")]
    #[validate(length(min = 1))]
    pub time: String,

    #[serde(rename = "apps")]
    #[validate(length(min = 1))]
    pub apps: Vec<models::DosschemaAppsInner>,
}

impl Dosschema {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        _schema_version: String,
        time: String,
        apps: Vec<models::DosschemaAppsInner>,
    ) -> Dosschema {
        Dosschema {
            _schema_version,
            time,
            apps,
        }
    }
}

/// Converts the Dosschema value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Dosschema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("_schemaVersion".to_string()),
            Some(self._schema_version.to_string()),
            Some("time".to_string()),
            Some(self.time.to_string()),
            // Skipping apps in query parameter serialization
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Dosschema value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Dosschema {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub _schema_version: Vec<String>,
            pub time: Vec<String>,
            pub apps: Vec<Vec<models::DosschemaAppsInner>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Dosschema".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "_schemaVersion" => intermediate_rep._schema_version.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "time" => intermediate_rep.time.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    "apps" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Dosschema"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Dosschema".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Dosschema {
            _schema_version: intermediate_rep
                ._schema_version
                .into_iter()
                .next()
                .ok_or_else(|| "_schemaVersion missing in Dosschema".to_string())?,
            time: intermediate_rep
                .time
                .into_iter()
                .next()
                .ok_or_else(|| "time missing in Dosschema".to_string())?,
            apps: intermediate_rep
                .apps
                .into_iter()
                .next()
                .ok_or_else(|| "apps missing in Dosschema".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Dosschema> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Dosschema>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Dosschema>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Dosschema - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Dosschema> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Dosschema as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Dosschema - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct DosschemaAppsInner {
    #[serde(rename = "name")]
    #[validate(length(min = 1))]
    pub name: String,

    #[serde(rename = "version")]
    #[validate(length(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl DosschemaAppsInner {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(name: String) -> DosschemaAppsInner {
        DosschemaAppsInner {
            name,
            version: None,
        }
    }
}

/// Converts the DosschemaAppsInner value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for DosschemaAppsInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("name".to_string()),
            Some(self.name.to_string()),
            self.version
                .as_ref()
                .map(|version| ["version".to_string(), version.to_string()].join(",")),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a DosschemaAppsInner value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for DosschemaAppsInner {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub name: Vec<String>,
            pub version: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing DosschemaAppsInner".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "name" => intermediate_rep.name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "version" => intermediate_rep.version.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing DosschemaAppsInner".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(DosschemaAppsInner {
            name: intermediate_rep
                .name
                .into_iter()
                .next()
                .ok_or_else(|| "name missing in DosschemaAppsInner".to_string())?,
            version: intermediate_rep.version.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<DosschemaAppsInner> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<DosschemaAppsInner>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<DosschemaAppsInner>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for DosschemaAppsInner - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<DosschemaAppsInner> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <DosschemaAppsInner as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into DosschemaAppsInner - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FeatureFlags {
    #[serde(rename = "isVendor")]
    pub is_vendor: bool,

    #[serde(rename = "isWhitelabeled")]
    pub is_whitelabeled: bool,
}

impl FeatureFlags {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(is_vendor: bool, is_whitelabeled: bool) -> FeatureFlags {
        FeatureFlags {
            is_vendor,
            is_whitelabeled,
        }
    }
}

/// Converts the FeatureFlags value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for FeatureFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("isVendor".to_string()),
            Some(self.is_vendor.to_string()),
            Some("isWhitelabeled".to_string()),
            Some(self.is_whitelabeled.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a FeatureFlags value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for FeatureFlags {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub is_vendor: Vec<bool>,
            pub is_whitelabeled: Vec<bool>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing FeatureFlags".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "isVendor" => intermediate_rep.is_vendor.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "isWhitelabeled" => intermediate_rep.is_whitelabeled.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing FeatureFlags".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(FeatureFlags {
            is_vendor: intermediate_rep
                .is_vendor
                .into_iter()
                .next()
                .ok_or_else(|| "isVendor missing in FeatureFlags".to_string())?,
            is_whitelabeled: intermediate_rep
                .is_whitelabeled
                .into_iter()
                .next()
                .ok_or_else(|| "isWhitelabeled missing in FeatureFlags".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<FeatureFlags> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<FeatureFlags>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<FeatureFlags>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for FeatureFlags - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<FeatureFlags> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <FeatureFlags as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into FeatureFlags - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FlunderBrowseGet200Response {
    #[serde(rename = "data")]
    pub data: Vec<models::FlunderBrowseGet200ResponseDataInner>,

    #[serde(rename = "additionalInfo")]
    pub additional_info: String,
}

impl FlunderBrowseGet200Response {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        data: Vec<models::FlunderBrowseGet200ResponseDataInner>,
        additional_info: String,
    ) -> FlunderBrowseGet200Response {
        FlunderBrowseGet200Response {
            data,
            additional_info,
        }
    }
}

/// Converts the FlunderBrowseGet200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for FlunderBrowseGet200Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping data in query parameter serialization
            Some("additionalInfo".to_string()),
            Some(self.additional_info.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a FlunderBrowseGet200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for FlunderBrowseGet200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub data: Vec<Vec<models::FlunderBrowseGet200ResponseDataInner>>,
            pub additional_info: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing FlunderBrowseGet200Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "data" => return std::result::Result::Err("Parsing a container in this style is not supported in FlunderBrowseGet200Response".to_string()),
                    #[allow(clippy::redundant_clone)]
                    "additionalInfo" => intermediate_rep.additional_info.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing FlunderBrowseGet200Response".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(FlunderBrowseGet200Response {
            data: intermediate_rep
                .data
                .into_iter()
                .next()
                .ok_or_else(|| "data missing in FlunderBrowseGet200Response".to_string())?,
            additional_info: intermediate_rep
                .additional_info
                .into_iter()
                .next()
                .ok_or_else(|| {
                    "additionalInfo missing in FlunderBrowseGet200Response".to_string()
                })?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<FlunderBrowseGet200Response> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<FlunderBrowseGet200Response>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<FlunderBrowseGet200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for FlunderBrowseGet200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<FlunderBrowseGet200Response> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <FlunderBrowseGet200Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into FlunderBrowseGet200Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FlunderBrowseGet200ResponseDataInner {
    #[serde(rename = "key")]
    pub key: String,

    #[serde(rename = "value")]
    pub value: String,

    #[serde(rename = "encoding")]
    pub encoding: String,

    #[serde(rename = "timestamp")]
    pub timestamp: String,
}

impl FlunderBrowseGet200ResponseDataInner {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        key: String,
        value: String,
        encoding: String,
        timestamp: String,
    ) -> FlunderBrowseGet200ResponseDataInner {
        FlunderBrowseGet200ResponseDataInner {
            key,
            value,
            encoding,
            timestamp,
        }
    }
}

/// Converts the FlunderBrowseGet200ResponseDataInner value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for FlunderBrowseGet200ResponseDataInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("key".to_string()),
            Some(self.key.to_string()),
            Some("value".to_string()),
            Some(self.value.to_string()),
            Some("encoding".to_string()),
            Some(self.encoding.to_string()),
            Some("timestamp".to_string()),
            Some(self.timestamp.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a FlunderBrowseGet200ResponseDataInner value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for FlunderBrowseGet200ResponseDataInner {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub key: Vec<String>,
            pub value: Vec<String>,
            pub encoding: Vec<String>,
            pub timestamp: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing FlunderBrowseGet200ResponseDataInner"
                            .to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "key" => intermediate_rep.key.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "value" => intermediate_rep.value.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "encoding" => intermediate_rep.encoding.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "timestamp" => intermediate_rep.timestamp.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing FlunderBrowseGet200ResponseDataInner"
                                .to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(FlunderBrowseGet200ResponseDataInner {
            key: intermediate_rep
                .key
                .into_iter()
                .next()
                .ok_or_else(|| "key missing in FlunderBrowseGet200ResponseDataInner".to_string())?,
            value: intermediate_rep.value.into_iter().next().ok_or_else(|| {
                "value missing in FlunderBrowseGet200ResponseDataInner".to_string()
            })?,
            encoding: intermediate_rep
                .encoding
                .into_iter()
                .next()
                .ok_or_else(|| {
                    "encoding missing in FlunderBrowseGet200ResponseDataInner".to_string()
                })?,
            timestamp: intermediate_rep
                .timestamp
                .into_iter()
                .next()
                .ok_or_else(|| {
                    "timestamp missing in FlunderBrowseGet200ResponseDataInner".to_string()
                })?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<FlunderBrowseGet200ResponseDataInner> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<FlunderBrowseGet200ResponseDataInner>>
    for HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<FlunderBrowseGet200ResponseDataInner>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for FlunderBrowseGet200ResponseDataInner - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue>
    for header::IntoHeaderValue<FlunderBrowseGet200ResponseDataInner>
{
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <FlunderBrowseGet200ResponseDataInner as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into FlunderBrowseGet200ResponseDataInner - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Id(String);

impl validator::Validate for Id {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for Id {
    fn from(x: String) -> Self {
        Id(x)
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::str::FromStr for Id {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(Id(x.to_string()))
    }
}

impl std::convert::From<Id> for String {
    fn from(x: Id) -> Self {
        x.0
    }
}

impl std::ops::Deref for Id {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for Id {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstalledApp {
    #[serde(rename = "appKey")]
    pub app_key: models::AppKey,

    #[serde(rename = "status")]
    pub status: models::AppStatus,

    #[serde(rename = "desired")]
    pub desired: models::AppStatus,

    #[serde(rename = "installedSize")]
    pub installed_size: i32,

    #[serde(rename = "multiInstance")]
    pub multi_instance: bool,

    #[serde(rename = "editors")]
    pub editors: Vec<models::AppEditor>,
}

impl InstalledApp {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        app_key: models::AppKey,
        status: models::AppStatus,
        desired: models::AppStatus,
        installed_size: i32,
        multi_instance: bool,
        editors: Vec<models::AppEditor>,
    ) -> InstalledApp {
        InstalledApp {
            app_key,
            status,
            desired,
            installed_size,
            multi_instance,
            editors,
        }
    }
}

/// Converts the InstalledApp value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstalledApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping appKey in query parameter serialization

            // Skipping status in query parameter serialization

            // Skipping desired in query parameter serialization
            Some("installedSize".to_string()),
            Some(self.installed_size.to_string()),
            Some("multiInstance".to_string()),
            Some(self.multi_instance.to_string()),
            // Skipping editors in query parameter serialization
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstalledApp value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstalledApp {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub app_key: Vec<models::AppKey>,
            pub status: Vec<models::AppStatus>,
            pub desired: Vec<models::AppStatus>,
            pub installed_size: Vec<i32>,
            pub multi_instance: Vec<bool>,
            pub editors: Vec<Vec<models::AppEditor>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstalledApp".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "appKey" => intermediate_rep.app_key.push(
                        <models::AppKey as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep.status.push(
                        <models::AppStatus as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "desired" => intermediate_rep.desired.push(
                        <models::AppStatus as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "installedSize" => intermediate_rep.installed_size.push(
                        <i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "multiInstance" => intermediate_rep.multi_instance.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    "editors" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in InstalledApp"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstalledApp".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstalledApp {
            app_key: intermediate_rep
                .app_key
                .into_iter()
                .next()
                .ok_or_else(|| "appKey missing in InstalledApp".to_string())?,
            status: intermediate_rep
                .status
                .into_iter()
                .next()
                .ok_or_else(|| "status missing in InstalledApp".to_string())?,
            desired: intermediate_rep
                .desired
                .into_iter()
                .next()
                .ok_or_else(|| "desired missing in InstalledApp".to_string())?,
            installed_size: intermediate_rep
                .installed_size
                .into_iter()
                .next()
                .ok_or_else(|| "installedSize missing in InstalledApp".to_string())?,
            multi_instance: intermediate_rep
                .multi_instance
                .into_iter()
                .next()
                .ok_or_else(|| "multiInstance missing in InstalledApp".to_string())?,
            editors: intermediate_rep
                .editors
                .into_iter()
                .next()
                .ok_or_else(|| "editors missing in InstalledApp".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstalledApp> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstalledApp>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstalledApp>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstalledApp - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstalledApp> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstalledApp as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstalledApp - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceConfig {
    #[serde(rename = "networkAdapters")]
    pub network_adapters: Vec<models::InstanceConfigNetworkAdapter>,

    #[serde(rename = "devices")]
    pub devices: models::InstanceConfigDevices,
}

impl InstanceConfig {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        network_adapters: Vec<models::InstanceConfigNetworkAdapter>,
        devices: models::InstanceConfigDevices,
    ) -> InstanceConfig {
        InstanceConfig {
            network_adapters,
            devices,
        }
    }
}

/// Converts the InstanceConfig value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstanceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping networkAdapters in query parameter serialization

            // Skipping devices in query parameter serialization

        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstanceConfig value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstanceConfig {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub network_adapters: Vec<Vec<models::InstanceConfigNetworkAdapter>>,
            pub devices: Vec<models::InstanceConfigDevices>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstanceConfig".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "networkAdapters" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in InstanceConfig"
                                .to_string(),
                        )
                    }
                    #[allow(clippy::redundant_clone)]
                    "devices" => intermediate_rep.devices.push(
                        <models::InstanceConfigDevices as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstanceConfig".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstanceConfig {
            network_adapters: intermediate_rep
                .network_adapters
                .into_iter()
                .next()
                .ok_or_else(|| "networkAdapters missing in InstanceConfig".to_string())?,
            devices: intermediate_rep
                .devices
                .into_iter()
                .next()
                .ok_or_else(|| "devices missing in InstanceConfig".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstanceConfig> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstanceConfig>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstanceConfig>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstanceConfig - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstanceConfig> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstanceConfig as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstanceConfig - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceConfigDevices {
    #[serde(rename = "usb")]
    pub usb: Vec<models::InstanceConfigUsbDevice>,
}

impl InstanceConfigDevices {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(usb: Vec<models::InstanceConfigUsbDevice>) -> InstanceConfigDevices {
        InstanceConfigDevices { usb }
    }
}

/// Converts the InstanceConfigDevices value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstanceConfigDevices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping usb in query parameter serialization

        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstanceConfigDevices value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstanceConfigDevices {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub usb: Vec<Vec<models::InstanceConfigUsbDevice>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstanceConfigDevices".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "usb" => return std::result::Result::Err("Parsing a container in this style is not supported in InstanceConfigDevices".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing InstanceConfigDevices".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstanceConfigDevices {
            usb: intermediate_rep
                .usb
                .into_iter()
                .next()
                .ok_or_else(|| "usb missing in InstanceConfigDevices".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstanceConfigDevices> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstanceConfigDevices>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstanceConfigDevices>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstanceConfigDevices - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstanceConfigDevices> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstanceConfigDevices as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstanceConfigDevices - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceConfigNetworkAdapter {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "active")]
    pub active: bool,

    #[serde(rename = "connected")]
    pub connected: bool,

    #[serde(rename = "ipAddress")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    #[serde(rename = "subnetMask")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_mask: Option<String>,

    #[serde(rename = "gateway")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,
}

impl InstanceConfigNetworkAdapter {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(name: String, active: bool, connected: bool) -> InstanceConfigNetworkAdapter {
        InstanceConfigNetworkAdapter {
            name,
            active,
            connected,
            ip_address: None,
            subnet_mask: None,
            gateway: None,
        }
    }
}

/// Converts the InstanceConfigNetworkAdapter value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstanceConfigNetworkAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("name".to_string()),
            Some(self.name.to_string()),
            Some("active".to_string()),
            Some(self.active.to_string()),
            Some("connected".to_string()),
            Some(self.connected.to_string()),
            self.ip_address
                .as_ref()
                .map(|ip_address| ["ipAddress".to_string(), ip_address.to_string()].join(",")),
            self.subnet_mask
                .as_ref()
                .map(|subnet_mask| ["subnetMask".to_string(), subnet_mask.to_string()].join(",")),
            self.gateway
                .as_ref()
                .map(|gateway| ["gateway".to_string(), gateway.to_string()].join(",")),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstanceConfigNetworkAdapter value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstanceConfigNetworkAdapter {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub name: Vec<String>,
            pub active: Vec<bool>,
            pub connected: Vec<bool>,
            pub ip_address: Vec<String>,
            pub subnet_mask: Vec<String>,
            pub gateway: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstanceConfigNetworkAdapter".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "name" => intermediate_rep.name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "active" => intermediate_rep.active.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "connected" => intermediate_rep.connected.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "ipAddress" => intermediate_rep.ip_address.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "subnetMask" => intermediate_rep.subnet_mask.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "gateway" => intermediate_rep.gateway.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstanceConfigNetworkAdapter".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstanceConfigNetworkAdapter {
            name: intermediate_rep
                .name
                .into_iter()
                .next()
                .ok_or_else(|| "name missing in InstanceConfigNetworkAdapter".to_string())?,
            active: intermediate_rep
                .active
                .into_iter()
                .next()
                .ok_or_else(|| "active missing in InstanceConfigNetworkAdapter".to_string())?,
            connected: intermediate_rep
                .connected
                .into_iter()
                .next()
                .ok_or_else(|| "connected missing in InstanceConfigNetworkAdapter".to_string())?,
            ip_address: intermediate_rep.ip_address.into_iter().next(),
            subnet_mask: intermediate_rep.subnet_mask.into_iter().next(),
            gateway: intermediate_rep.gateway.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstanceConfigNetworkAdapter> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstanceConfigNetworkAdapter>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstanceConfigNetworkAdapter>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstanceConfigNetworkAdapter - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstanceConfigNetworkAdapter> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <InstanceConfigNetworkAdapter as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into InstanceConfigNetworkAdapter - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceConfigUsbDevice {
    #[serde(rename = "port")]
    #[validate(
            regex(path = *RE_INSTANCECONFIGUSBDEVICE_PORT),
        )]
    pub port: String,

    #[serde(rename = "name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "pid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<i32>,

    #[serde(rename = "vendor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,

    #[serde(rename = "vid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vid: Option<i32>,

    #[serde(rename = "device_connected")]
    pub device_connected: bool,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCECONFIGUSBDEVICE_PORT: regex::Regex = regex::Regex::new("^usb[1-9][0-9]*|[1-9][0-9]*-[1-9][0-9]*(?:\\.[1-9][0-9]*)*$").unwrap();
}

impl InstanceConfigUsbDevice {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(port: String, device_connected: bool) -> InstanceConfigUsbDevice {
        InstanceConfigUsbDevice {
            port,
            name: None,
            pid: None,
            vendor: None,
            vid: None,
            device_connected,
        }
    }
}

/// Converts the InstanceConfigUsbDevice value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstanceConfigUsbDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("port".to_string()),
            Some(self.port.to_string()),
            self.name
                .as_ref()
                .map(|name| ["name".to_string(), name.to_string()].join(",")),
            self.pid
                .as_ref()
                .map(|pid| ["pid".to_string(), pid.to_string()].join(",")),
            self.vendor
                .as_ref()
                .map(|vendor| ["vendor".to_string(), vendor.to_string()].join(",")),
            self.vid
                .as_ref()
                .map(|vid| ["vid".to_string(), vid.to_string()].join(",")),
            Some("device_connected".to_string()),
            Some(self.device_connected.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstanceConfigUsbDevice value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstanceConfigUsbDevice {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub port: Vec<String>,
            pub name: Vec<String>,
            pub pid: Vec<i32>,
            pub vendor: Vec<String>,
            pub vid: Vec<i32>,
            pub device_connected: Vec<bool>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstanceConfigUsbDevice".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "port" => intermediate_rep.port.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "name" => intermediate_rep.name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "pid" => intermediate_rep.pid.push(
                        <i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "vendor" => intermediate_rep.vendor.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "vid" => intermediate_rep.vid.push(
                        <i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "device_connected" => intermediate_rep.device_connected.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstanceConfigUsbDevice".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstanceConfigUsbDevice {
            port: intermediate_rep
                .port
                .into_iter()
                .next()
                .ok_or_else(|| "port missing in InstanceConfigUsbDevice".to_string())?,
            name: intermediate_rep.name.into_iter().next(),
            pid: intermediate_rep.pid.into_iter().next(),
            vendor: intermediate_rep.vendor.into_iter().next(),
            vid: intermediate_rep.vid.into_iter().next(),
            device_connected: intermediate_rep
                .device_connected
                .into_iter()
                .next()
                .ok_or_else(|| "device_connected missing in InstanceConfigUsbDevice".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstanceConfigUsbDevice> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstanceConfigUsbDevice>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstanceConfigUsbDevice>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstanceConfigUsbDevice - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstanceConfigUsbDevice> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstanceConfigUsbDevice as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstanceConfigUsbDevice - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceDetailConfigFile {
    #[serde(rename = "container")]
    pub container: String,

    #[serde(rename = "host")]
    pub host: String,
}

impl InstanceDetailConfigFile {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(container: String, host: String) -> InstanceDetailConfigFile {
        InstanceDetailConfigFile { container, host }
    }
}

/// Converts the InstanceDetailConfigFile value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstanceDetailConfigFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("container".to_string()),
            Some(self.container.to_string()),
            Some("host".to_string()),
            Some(self.host.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstanceDetailConfigFile value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstanceDetailConfigFile {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub container: Vec<String>,
            pub host: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstanceDetailConfigFile".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "container" => intermediate_rep.container.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "host" => intermediate_rep.host.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstanceDetailConfigFile".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstanceDetailConfigFile {
            container: intermediate_rep
                .container
                .into_iter()
                .next()
                .ok_or_else(|| "container missing in InstanceDetailConfigFile".to_string())?,
            host: intermediate_rep
                .host
                .into_iter()
                .next()
                .ok_or_else(|| "host missing in InstanceDetailConfigFile".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstanceDetailConfigFile> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstanceDetailConfigFile>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstanceDetailConfigFile>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstanceDetailConfigFile - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstanceDetailConfigFile> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstanceDetailConfigFile as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstanceDetailConfigFile - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceDetailConfigFiles(Vec<InstanceDetailConfigFile>);

impl validator::Validate for InstanceDetailConfigFiles {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<Vec<InstanceDetailConfigFile>> for InstanceDetailConfigFiles {
    fn from(x: Vec<InstanceDetailConfigFile>) -> Self {
        InstanceDetailConfigFiles(x)
    }
}

impl std::convert::From<InstanceDetailConfigFiles> for Vec<InstanceDetailConfigFile> {
    fn from(x: InstanceDetailConfigFiles) -> Self {
        x.0
    }
}

impl std::iter::FromIterator<InstanceDetailConfigFile> for InstanceDetailConfigFiles {
    fn from_iter<U: IntoIterator<Item = InstanceDetailConfigFile>>(u: U) -> Self {
        InstanceDetailConfigFiles(Vec::<InstanceDetailConfigFile>::from_iter(u))
    }
}

impl std::iter::IntoIterator for InstanceDetailConfigFiles {
    type Item = InstanceDetailConfigFile;
    type IntoIter = std::vec::IntoIter<InstanceDetailConfigFile>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> std::iter::IntoIterator for &'a InstanceDetailConfigFiles {
    type Item = &'a InstanceDetailConfigFile;
    type IntoIter = std::slice::Iter<'a, InstanceDetailConfigFile>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> std::iter::IntoIterator for &'a mut InstanceDetailConfigFiles {
    type Item = &'a mut InstanceDetailConfigFile;
    type IntoIter = std::slice::IterMut<'a, InstanceDetailConfigFile>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl std::ops::Deref for InstanceDetailConfigFiles {
    type Target = Vec<InstanceDetailConfigFile>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for InstanceDetailConfigFiles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Converts the InstanceDetailConfigFiles value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstanceDetailConfigFiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstanceDetailConfigFiles value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstanceDetailConfigFiles {
    type Err = <InstanceDetailConfigFile as std::str::FromStr>::Err;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut items = vec![];
        for item in s.split(',') {
            items.push(item.parse()?);
        }
        std::result::Result::Ok(InstanceDetailConfigFiles(items))
    }
}

// Methods for converting between header::IntoHeaderValue<InstanceDetailConfigFiles> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstanceDetailConfigFiles>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstanceDetailConfigFiles>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstanceDetailConfigFiles - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstanceDetailConfigFiles> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstanceDetailConfigFiles as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstanceDetailConfigFiles - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Hostname of an instance
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceDetailHostname(String);

impl validator::Validate for InstanceDetailHostname {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for InstanceDetailHostname {
    fn from(x: String) -> Self {
        InstanceDetailHostname(x)
    }
}

impl std::fmt::Display for InstanceDetailHostname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::str::FromStr for InstanceDetailHostname {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(InstanceDetailHostname(x.to_string()))
    }
}

impl std::convert::From<InstanceDetailHostname> for String {
    fn from(x: InstanceDetailHostname) -> Self {
        x.0
    }
}

impl std::ops::Deref for InstanceDetailHostname {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for InstanceDetailHostname {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

/// IP address of an instance
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceDetailIpAddress(String);

impl validator::Validate for InstanceDetailIpAddress {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for InstanceDetailIpAddress {
    fn from(x: String) -> Self {
        InstanceDetailIpAddress(x)
    }
}

impl std::fmt::Display for InstanceDetailIpAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::str::FromStr for InstanceDetailIpAddress {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(InstanceDetailIpAddress(x.to_string()))
    }
}

impl std::convert::From<InstanceDetailIpAddress> for String {
    fn from(x: InstanceDetailIpAddress) -> Self {
        x.0
    }
}

impl std::ops::Deref for InstanceDetailIpAddress {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for InstanceDetailIpAddress {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

/// Bind mounts of an instance
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceDetailMounts {
    #[serde(rename = "mounts")]
    pub mounts: Vec<models::InstanceDetailMountsMountsInner>,
}

impl InstanceDetailMounts {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(mounts: Vec<models::InstanceDetailMountsMountsInner>) -> InstanceDetailMounts {
        InstanceDetailMounts { mounts }
    }
}

/// Converts the InstanceDetailMounts value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstanceDetailMounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping mounts in query parameter serialization

        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstanceDetailMounts value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstanceDetailMounts {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub mounts: Vec<Vec<models::InstanceDetailMountsMountsInner>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstanceDetailMounts".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "mounts" => return std::result::Result::Err("Parsing a container in this style is not supported in InstanceDetailMounts".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing InstanceDetailMounts".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstanceDetailMounts {
            mounts: intermediate_rep
                .mounts
                .into_iter()
                .next()
                .ok_or_else(|| "mounts missing in InstanceDetailMounts".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstanceDetailMounts> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstanceDetailMounts>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstanceDetailMounts>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstanceDetailMounts - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstanceDetailMounts> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstanceDetailMounts as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstanceDetailMounts - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceDetailMountsMountsInner {
    #[serde(rename = "container")]
    pub container: String,

    #[serde(rename = "host")]
    pub host: String,
}

impl InstanceDetailMountsMountsInner {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(container: String, host: String) -> InstanceDetailMountsMountsInner {
        InstanceDetailMountsMountsInner { container, host }
    }
}

/// Converts the InstanceDetailMountsMountsInner value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstanceDetailMountsMountsInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("container".to_string()),
            Some(self.container.to_string()),
            Some("host".to_string()),
            Some(self.host.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstanceDetailMountsMountsInner value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstanceDetailMountsMountsInner {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub container: Vec<String>,
            pub host: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstanceDetailMountsMountsInner".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "container" => intermediate_rep.container.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "host" => intermediate_rep.host.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstanceDetailMountsMountsInner"
                                .to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstanceDetailMountsMountsInner {
            container: intermediate_rep
                .container
                .into_iter()
                .next()
                .ok_or_else(|| {
                    "container missing in InstanceDetailMountsMountsInner".to_string()
                })?,
            host: intermediate_rep
                .host
                .into_iter()
                .next()
                .ok_or_else(|| "host missing in InstanceDetailMountsMountsInner".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstanceDetailMountsMountsInner> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstanceDetailMountsMountsInner>>
    for HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstanceDetailMountsMountsInner>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for InstanceDetailMountsMountsInner - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue>
    for header::IntoHeaderValue<InstanceDetailMountsMountsInner>
{
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <InstanceDetailMountsMountsInner as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into InstanceDetailMountsMountsInner - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceDetailPort {
    #[serde(rename = "container")]
    pub container: String,

    #[serde(rename = "host")]
    pub host: String,
}

impl InstanceDetailPort {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(container: String, host: String) -> InstanceDetailPort {
        InstanceDetailPort { container, host }
    }
}

/// Converts the InstanceDetailPort value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstanceDetailPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("container".to_string()),
            Some(self.container.to_string()),
            Some("host".to_string()),
            Some(self.host.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstanceDetailPort value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstanceDetailPort {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub container: Vec<String>,
            pub host: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstanceDetailPort".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "container" => intermediate_rep.container.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "host" => intermediate_rep.host.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstanceDetailPort".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstanceDetailPort {
            container: intermediate_rep
                .container
                .into_iter()
                .next()
                .ok_or_else(|| "container missing in InstanceDetailPort".to_string())?,
            host: intermediate_rep
                .host
                .into_iter()
                .next()
                .ok_or_else(|| "host missing in InstanceDetailPort".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstanceDetailPort> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstanceDetailPort>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstanceDetailPort>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstanceDetailPort - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstanceDetailPort> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstanceDetailPort as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstanceDetailPort - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceDetailVolume {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "path")]
    pub path: String,
}

impl InstanceDetailVolume {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(name: String, path: String) -> InstanceDetailVolume {
        InstanceDetailVolume { name, path }
    }
}

/// Converts the InstanceDetailVolume value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstanceDetailVolume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("name".to_string()),
            Some(self.name.to_string()),
            Some("path".to_string()),
            Some(self.path.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstanceDetailVolume value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstanceDetailVolume {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub name: Vec<String>,
            pub path: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstanceDetailVolume".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "name" => intermediate_rep.name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "path" => intermediate_rep.path.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstanceDetailVolume".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstanceDetailVolume {
            name: intermediate_rep
                .name
                .into_iter()
                .next()
                .ok_or_else(|| "name missing in InstanceDetailVolume".to_string())?,
            path: intermediate_rep
                .path
                .into_iter()
                .next()
                .ok_or_else(|| "path missing in InstanceDetailVolume".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstanceDetailVolume> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstanceDetailVolume>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstanceDetailVolume>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstanceDetailVolume - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstanceDetailVolume> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstanceDetailVolume as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstanceDetailVolume - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceEditor {
    /// Descriptive name of the editor
    #[serde(rename = "name")]
    pub name: String,

    /// Link to the editor of an instance
    #[serde(rename = "url")]
    pub url: String,
}

impl InstanceEditor {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(name: String, url: String) -> InstanceEditor {
        InstanceEditor { name, url }
    }
}

/// Converts the InstanceEditor value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstanceEditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("name".to_string()),
            Some(self.name.to_string()),
            Some("url".to_string()),
            Some(self.url.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstanceEditor value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstanceEditor {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub name: Vec<String>,
            pub url: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstanceEditor".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "name" => intermediate_rep.name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "url" => intermediate_rep.url.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstanceEditor".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstanceEditor {
            name: intermediate_rep
                .name
                .into_iter()
                .next()
                .ok_or_else(|| "name missing in InstanceEditor".to_string())?,
            url: intermediate_rep
                .url
                .into_iter()
                .next()
                .ok_or_else(|| "url missing in InstanceEditor".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstanceEditor> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstanceEditor>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstanceEditor>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstanceEditor - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstanceEditor> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstanceEditor as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstanceEditor - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceEditors(Vec<InstanceEditor>);

impl validator::Validate for InstanceEditors {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<Vec<InstanceEditor>> for InstanceEditors {
    fn from(x: Vec<InstanceEditor>) -> Self {
        InstanceEditors(x)
    }
}

impl std::convert::From<InstanceEditors> for Vec<InstanceEditor> {
    fn from(x: InstanceEditors) -> Self {
        x.0
    }
}

impl std::iter::FromIterator<InstanceEditor> for InstanceEditors {
    fn from_iter<U: IntoIterator<Item = InstanceEditor>>(u: U) -> Self {
        InstanceEditors(Vec::<InstanceEditor>::from_iter(u))
    }
}

impl std::iter::IntoIterator for InstanceEditors {
    type Item = InstanceEditor;
    type IntoIter = std::vec::IntoIter<InstanceEditor>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> std::iter::IntoIterator for &'a InstanceEditors {
    type Item = &'a InstanceEditor;
    type IntoIter = std::slice::Iter<'a, InstanceEditor>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> std::iter::IntoIterator for &'a mut InstanceEditors {
    type Item = &'a mut InstanceEditor;
    type IntoIter = std::slice::IterMut<'a, InstanceEditor>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl std::ops::Deref for InstanceEditors {
    type Target = Vec<InstanceEditor>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for InstanceEditors {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Converts the InstanceEditors value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstanceEditors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstanceEditors value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstanceEditors {
    type Err = <InstanceEditor as std::str::FromStr>::Err;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut items = vec![];
        for item in s.split(',') {
            items.push(item.parse()?);
        }
        std::result::Result::Ok(InstanceEditors(items))
    }
}

// Methods for converting between header::IntoHeaderValue<InstanceEditors> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstanceEditors>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstanceEditors>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstanceEditors - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstanceEditors> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstanceEditors as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstanceEditors - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceEnvironment(Vec<InstanceEnvironmentVariable>);

impl validator::Validate for InstanceEnvironment {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<Vec<InstanceEnvironmentVariable>> for InstanceEnvironment {
    fn from(x: Vec<InstanceEnvironmentVariable>) -> Self {
        InstanceEnvironment(x)
    }
}

impl std::convert::From<InstanceEnvironment> for Vec<InstanceEnvironmentVariable> {
    fn from(x: InstanceEnvironment) -> Self {
        x.0
    }
}

impl std::iter::FromIterator<InstanceEnvironmentVariable> for InstanceEnvironment {
    fn from_iter<U: IntoIterator<Item = InstanceEnvironmentVariable>>(u: U) -> Self {
        InstanceEnvironment(Vec::<InstanceEnvironmentVariable>::from_iter(u))
    }
}

impl std::iter::IntoIterator for InstanceEnvironment {
    type Item = InstanceEnvironmentVariable;
    type IntoIter = std::vec::IntoIter<InstanceEnvironmentVariable>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> std::iter::IntoIterator for &'a InstanceEnvironment {
    type Item = &'a InstanceEnvironmentVariable;
    type IntoIter = std::slice::Iter<'a, InstanceEnvironmentVariable>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> std::iter::IntoIterator for &'a mut InstanceEnvironment {
    type Item = &'a mut InstanceEnvironmentVariable;
    type IntoIter = std::slice::IterMut<'a, InstanceEnvironmentVariable>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl std::ops::Deref for InstanceEnvironment {
    type Target = Vec<InstanceEnvironmentVariable>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for InstanceEnvironment {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Converts the InstanceEnvironment value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstanceEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstanceEnvironment value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstanceEnvironment {
    type Err = <InstanceEnvironmentVariable as std::str::FromStr>::Err;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut items = vec![];
        for item in s.split(',') {
            items.push(item.parse()?);
        }
        std::result::Result::Ok(InstanceEnvironment(items))
    }
}

// Methods for converting between header::IntoHeaderValue<InstanceEnvironment> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstanceEnvironment>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstanceEnvironment>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstanceEnvironment - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstanceEnvironment> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstanceEnvironment as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstanceEnvironment - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceEnvironmentVariable {
    #[serde(rename = "name")]
    #[validate(
            regex(path = *RE_INSTANCEENVIRONMENTVARIABLE_NAME),
        )]
    pub name: String,

    #[serde(rename = "value")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCEENVIRONMENTVARIABLE_NAME: regex::Regex = regex::Regex::new("^[a-zA-Z]+(?:[a-zA-Z0-9_\\-\\.])*$").unwrap();
}

impl InstanceEnvironmentVariable {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(name: String) -> InstanceEnvironmentVariable {
        InstanceEnvironmentVariable { name, value: None }
    }
}

/// Converts the InstanceEnvironmentVariable value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstanceEnvironmentVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("name".to_string()),
            Some(self.name.to_string()),
            self.value
                .as_ref()
                .map(|value| ["value".to_string(), value.to_string()].join(",")),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstanceEnvironmentVariable value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstanceEnvironmentVariable {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub name: Vec<String>,
            pub value: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstanceEnvironmentVariable".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "name" => intermediate_rep.name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "value" => intermediate_rep.value.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstanceEnvironmentVariable".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstanceEnvironmentVariable {
            name: intermediate_rep
                .name
                .into_iter()
                .next()
                .ok_or_else(|| "name missing in InstanceEnvironmentVariable".to_string())?,
            value: intermediate_rep.value.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstanceEnvironmentVariable> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstanceEnvironmentVariable>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstanceEnvironmentVariable>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstanceEnvironmentVariable - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstanceEnvironmentVariable> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstanceEnvironmentVariable as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstanceEnvironmentVariable - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceEnvironmentVariableName(String);

impl validator::Validate for InstanceEnvironmentVariableName {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for InstanceEnvironmentVariableName {
    fn from(x: String) -> Self {
        InstanceEnvironmentVariableName(x)
    }
}

impl std::fmt::Display for InstanceEnvironmentVariableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::str::FromStr for InstanceEnvironmentVariableName {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(InstanceEnvironmentVariableName(x.to_string()))
    }
}

impl std::convert::From<InstanceEnvironmentVariableName> for String {
    fn from(x: InstanceEnvironmentVariableName) -> Self {
        x.0
    }
}

impl std::ops::Deref for InstanceEnvironmentVariableName {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for InstanceEnvironmentVariableName {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceEnvironmentVariableValue(String);

impl validator::Validate for InstanceEnvironmentVariableValue {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for InstanceEnvironmentVariableValue {
    fn from(x: String) -> Self {
        InstanceEnvironmentVariableValue(x)
    }
}

impl std::fmt::Display for InstanceEnvironmentVariableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::str::FromStr for InstanceEnvironmentVariableValue {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(InstanceEnvironmentVariableValue(x.to_string()))
    }
}

impl std::convert::From<InstanceEnvironmentVariableValue> for String {
    fn from(x: InstanceEnvironmentVariableValue) -> Self {
        x.0
    }
}

impl std::ops::Deref for InstanceEnvironmentVariableValue {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for InstanceEnvironmentVariableValue {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceId(String);

impl validator::Validate for InstanceId {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for InstanceId {
    fn from(x: String) -> Self {
        InstanceId(x)
    }
}

impl std::fmt::Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::str::FromStr for InstanceId {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(InstanceId(x.to_string()))
    }
}

impl std::convert::From<InstanceId> for String {
    fn from(x: InstanceId) -> Self {
        x.0
    }
}

impl std::ops::Deref for InstanceId {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for InstanceId {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceLabel {
    #[serde(rename = "name")]
    #[validate(
            regex(path = *RE_INSTANCELABEL_NAME),
        )]
    pub name: String,

    #[serde(rename = "value")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCELABEL_NAME: regex::Regex = regex::Regex::new("^[a-z](?:(?:[\\-\\.]?[a-zA-Z0-9])*[\\-\\.]?[a-z])?$").unwrap();
}

impl InstanceLabel {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(name: String) -> InstanceLabel {
        InstanceLabel { name, value: None }
    }
}

/// Converts the InstanceLabel value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstanceLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("name".to_string()),
            Some(self.name.to_string()),
            self.value
                .as_ref()
                .map(|value| ["value".to_string(), value.to_string()].join(",")),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstanceLabel value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstanceLabel {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub name: Vec<String>,
            pub value: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstanceLabel".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "name" => intermediate_rep.name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "value" => intermediate_rep.value.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstanceLabel".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstanceLabel {
            name: intermediate_rep
                .name
                .into_iter()
                .next()
                .ok_or_else(|| "name missing in InstanceLabel".to_string())?,
            value: intermediate_rep.value.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstanceLabel> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstanceLabel>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstanceLabel>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstanceLabel - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstanceLabel> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstanceLabel as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstanceLabel - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceLabelName(String);

impl validator::Validate for InstanceLabelName {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for InstanceLabelName {
    fn from(x: String) -> Self {
        InstanceLabelName(x)
    }
}

impl std::fmt::Display for InstanceLabelName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::str::FromStr for InstanceLabelName {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(InstanceLabelName(x.to_string()))
    }
}

impl std::convert::From<InstanceLabelName> for String {
    fn from(x: InstanceLabelName) -> Self {
        x.0
    }
}

impl std::ops::Deref for InstanceLabelName {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for InstanceLabelName {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceLabelValue(String);

impl validator::Validate for InstanceLabelValue {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for InstanceLabelValue {
    fn from(x: String) -> Self {
        InstanceLabelValue(x)
    }
}

impl std::fmt::Display for InstanceLabelValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::str::FromStr for InstanceLabelValue {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(InstanceLabelValue(x.to_string()))
    }
}

impl std::convert::From<InstanceLabelValue> for String {
    fn from(x: InstanceLabelValue) -> Self {
        x.0
    }
}

impl std::ops::Deref for InstanceLabelValue {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for InstanceLabelValue {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

/// Instance name
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstanceName(String);

impl validator::Validate for InstanceName {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for InstanceName {
    fn from(x: String) -> Self {
        InstanceName(x)
    }
}

impl std::fmt::Display for InstanceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::str::FromStr for InstanceName {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(InstanceName(x.to_string()))
    }
}

impl std::convert::From<InstanceName> for String {
    fn from(x: InstanceName) -> Self {
        x.0
    }
}

impl std::ops::Deref for InstanceName {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for InstanceName {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[allow(non_camel_case_types)]
pub enum InstancePortMapping {
    InstancePortMappingRange(Box<models::InstancePortMappingRange>),
    InstancePortMappingSingle(Box<models::InstancePortMappingSingle>),
}

impl validator::Validate for InstancePortMapping {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        match self {
            Self::InstancePortMappingRange(x) => x.validate(),
            Self::InstancePortMappingSingle(x) => x.validate(),
        }
    }
}

impl From<models::InstancePortMappingRange> for InstancePortMapping {
    fn from(value: models::InstancePortMappingRange) -> Self {
        Self::InstancePortMappingRange(Box::new(value))
    }
}
impl From<models::InstancePortMappingSingle> for InstancePortMapping {
    fn from(value: models::InstancePortMappingSingle) -> Self {
        Self::InstancePortMappingSingle(Box::new(value))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstancePortMapping value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstancePortMapping {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancePortMappingRange {
    #[serde(rename = "host_ports")]
    pub host_ports: models::PortRange,

    #[serde(rename = "container_ports")]
    pub container_ports: models::PortRange,
}

impl InstancePortMappingRange {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        host_ports: models::PortRange,
        container_ports: models::PortRange,
    ) -> InstancePortMappingRange {
        InstancePortMappingRange {
            host_ports,
            container_ports,
        }
    }
}

/// Converts the InstancePortMappingRange value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstancePortMappingRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping host_ports in query parameter serialization

            // Skipping container_ports in query parameter serialization

        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstancePortMappingRange value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstancePortMappingRange {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub host_ports: Vec<models::PortRange>,
            pub container_ports: Vec<models::PortRange>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstancePortMappingRange".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "host_ports" => intermediate_rep.host_ports.push(
                        <models::PortRange as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "container_ports" => intermediate_rep.container_ports.push(
                        <models::PortRange as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstancePortMappingRange".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstancePortMappingRange {
            host_ports: intermediate_rep
                .host_ports
                .into_iter()
                .next()
                .ok_or_else(|| "host_ports missing in InstancePortMappingRange".to_string())?,
            container_ports: intermediate_rep
                .container_ports
                .into_iter()
                .next()
                .ok_or_else(|| "container_ports missing in InstancePortMappingRange".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstancePortMappingRange> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstancePortMappingRange>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstancePortMappingRange>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstancePortMappingRange - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstancePortMappingRange> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstancePortMappingRange as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstancePortMappingRange - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancePortMappingSingle {
    #[serde(rename = "host_port")]
    #[validate(range(min = 1u16, max = 65535u16))]
    pub host_port: u16,

    #[serde(rename = "container_port")]
    #[validate(range(min = 1u16, max = 65535u16))]
    pub container_port: u16,
}

impl InstancePortMappingSingle {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(host_port: u16, container_port: u16) -> InstancePortMappingSingle {
        InstancePortMappingSingle {
            host_port,
            container_port,
        }
    }
}

/// Converts the InstancePortMappingSingle value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstancePortMappingSingle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("host_port".to_string()),
            Some(self.host_port.to_string()),
            Some("container_port".to_string()),
            Some(self.container_port.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstancePortMappingSingle value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstancePortMappingSingle {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub host_port: Vec<u16>,
            pub container_port: Vec<u16>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstancePortMappingSingle".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "host_port" => intermediate_rep.host_port.push(
                        <u16 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "container_port" => intermediate_rep.container_port.push(
                        <u16 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstancePortMappingSingle".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstancePortMappingSingle {
            host_port: intermediate_rep
                .host_port
                .into_iter()
                .next()
                .ok_or_else(|| "host_port missing in InstancePortMappingSingle".to_string())?,
            container_port: intermediate_rep
                .container_port
                .into_iter()
                .next()
                .ok_or_else(|| "container_port missing in InstancePortMappingSingle".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstancePortMappingSingle> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstancePortMappingSingle>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstancePortMappingSingle>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstancePortMappingSingle - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstancePortMappingSingle> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstancePortMappingSingle as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstancePortMappingSingle - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancePorts {
    #[serde(rename = "tcp")]
    pub tcp: Vec<models::InstancePortMapping>,

    #[serde(rename = "udp")]
    pub udp: Vec<models::InstancePortMapping>,

    #[serde(rename = "sctp")]
    pub sctp: Vec<models::InstancePortMapping>,
}

impl InstancePorts {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        tcp: Vec<models::InstancePortMapping>,
        udp: Vec<models::InstancePortMapping>,
        sctp: Vec<models::InstancePortMapping>,
    ) -> InstancePorts {
        InstancePorts { tcp, udp, sctp }
    }
}

/// Converts the InstancePorts value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstancePorts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping tcp in query parameter serialization

            // Skipping udp in query parameter serialization

            // Skipping sctp in query parameter serialization

        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstancePorts value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstancePorts {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub tcp: Vec<Vec<models::InstancePortMapping>>,
            pub udp: Vec<Vec<models::InstancePortMapping>>,
            pub sctp: Vec<Vec<models::InstancePortMapping>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstancePorts".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "tcp" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in InstancePorts"
                                .to_string(),
                        )
                    }
                    "udp" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in InstancePorts"
                                .to_string(),
                        )
                    }
                    "sctp" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in InstancePorts"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstancePorts".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstancePorts {
            tcp: intermediate_rep
                .tcp
                .into_iter()
                .next()
                .ok_or_else(|| "tcp missing in InstancePorts".to_string())?,
            udp: intermediate_rep
                .udp
                .into_iter()
                .next()
                .ok_or_else(|| "udp missing in InstancePorts".to_string())?,
            sctp: intermediate_rep
                .sctp
                .into_iter()
                .next()
                .ok_or_else(|| "sctp missing in InstancePorts".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstancePorts> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstancePorts>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstancePorts>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstancePorts - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstancePorts> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstancePorts as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstancePorts - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum InstanceStatus {
    #[serde(rename = "not created")]
    NotCreated,
    #[serde(rename = "requested")]
    Requested,
    #[serde(rename = "resources ready")]
    ResourcesReady,
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "orphaned")]
    Orphaned,
    #[serde(rename = "unknown")]
    Unknown,
}

impl std::fmt::Display for InstanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            InstanceStatus::NotCreated => write!(f, "not created"),
            InstanceStatus::Requested => write!(f, "requested"),
            InstanceStatus::ResourcesReady => write!(f, "resources ready"),
            InstanceStatus::Created => write!(f, "created"),
            InstanceStatus::Stopped => write!(f, "stopped"),
            InstanceStatus::Running => write!(f, "running"),
            InstanceStatus::Orphaned => write!(f, "orphaned"),
            InstanceStatus::Unknown => write!(f, "unknown"),
        }
    }
}

impl std::str::FromStr for InstanceStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "not created" => std::result::Result::Ok(InstanceStatus::NotCreated),
            "requested" => std::result::Result::Ok(InstanceStatus::Requested),
            "resources ready" => std::result::Result::Ok(InstanceStatus::ResourcesReady),
            "created" => std::result::Result::Ok(InstanceStatus::Created),
            "stopped" => std::result::Result::Ok(InstanceStatus::Stopped),
            "running" => std::result::Result::Ok(InstanceStatus::Running),
            "orphaned" => std::result::Result::Ok(InstanceStatus::Orphaned),
            "unknown" => std::result::Result::Ok(InstanceStatus::Unknown),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesCreatePostRequest {
    #[serde(rename = "appKey")]
    pub app_key: models::AppKey,

    /// Instance name
    #[serde(rename = "instanceName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_name: Option<String>,
}

impl InstancesCreatePostRequest {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(app_key: models::AppKey) -> InstancesCreatePostRequest {
        InstancesCreatePostRequest {
            app_key,
            instance_name: None,
        }
    }
}

/// Converts the InstancesCreatePostRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstancesCreatePostRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping appKey in query parameter serialization
            self.instance_name.as_ref().map(|instance_name| {
                ["instanceName".to_string(), instance_name.to_string()].join(",")
            }),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstancesCreatePostRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstancesCreatePostRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub app_key: Vec<models::AppKey>,
            pub instance_name: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstancesCreatePostRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "appKey" => intermediate_rep.app_key.push(
                        <models::AppKey as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "instanceName" => intermediate_rep.instance_name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstancesCreatePostRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstancesCreatePostRequest {
            app_key: intermediate_rep
                .app_key
                .into_iter()
                .next()
                .ok_or_else(|| "appKey missing in InstancesCreatePostRequest".to_string())?,
            instance_name: intermediate_rep.instance_name.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstancesCreatePostRequest> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstancesCreatePostRequest>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstancesCreatePostRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InstancesCreatePostRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstancesCreatePostRequest> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InstancesCreatePostRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InstancesCreatePostRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigEnvironmentVariableNameGet200Response {
    #[serde(rename = "value")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl InstancesInstanceIdConfigEnvironmentVariableNameGet200Response {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new() -> InstancesInstanceIdConfigEnvironmentVariableNameGet200Response {
        InstancesInstanceIdConfigEnvironmentVariableNameGet200Response { value: None }
    }
}

/// Converts the InstancesInstanceIdConfigEnvironmentVariableNameGet200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstancesInstanceIdConfigEnvironmentVariableNameGet200Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![self
            .value
            .as_ref()
            .map(|value| ["value".to_string(), value.to_string()].join(","))];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstancesInstanceIdConfigEnvironmentVariableNameGet200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstancesInstanceIdConfigEnvironmentVariableNameGet200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub value: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing InstancesInstanceIdConfigEnvironmentVariableNameGet200Response".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "value" => intermediate_rep.value.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing InstancesInstanceIdConfigEnvironmentVariableNameGet200Response".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(
            InstancesInstanceIdConfigEnvironmentVariableNameGet200Response {
                value: intermediate_rep.value.into_iter().next(),
            },
        )
    }
}

// Methods for converting between header::IntoHeaderValue<InstancesInstanceIdConfigEnvironmentVariableNameGet200Response> and HeaderValue

#[cfg(feature = "server")]
impl
    std::convert::TryFrom<
        header::IntoHeaderValue<InstancesInstanceIdConfigEnvironmentVariableNameGet200Response>,
    > for HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<
            InstancesInstanceIdConfigEnvironmentVariableNameGet200Response,
        >,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for InstancesInstanceIdConfigEnvironmentVariableNameGet200Response - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue>
    for header::IntoHeaderValue<InstancesInstanceIdConfigEnvironmentVariableNameGet200Response>
{
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <InstancesInstanceIdConfigEnvironmentVariableNameGet200Response as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into InstancesInstanceIdConfigEnvironmentVariableNameGet200Response - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdConfigLabelsLabelNameGet200Response {
    #[serde(rename = "value")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl InstancesInstanceIdConfigLabelsLabelNameGet200Response {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new() -> InstancesInstanceIdConfigLabelsLabelNameGet200Response {
        InstancesInstanceIdConfigLabelsLabelNameGet200Response { value: None }
    }
}

/// Converts the InstancesInstanceIdConfigLabelsLabelNameGet200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstancesInstanceIdConfigLabelsLabelNameGet200Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![self
            .value
            .as_ref()
            .map(|value| ["value".to_string(), value.to_string()].join(","))];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstancesInstanceIdConfigLabelsLabelNameGet200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstancesInstanceIdConfigLabelsLabelNameGet200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub value: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing InstancesInstanceIdConfigLabelsLabelNameGet200Response".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "value" => intermediate_rep.value.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing InstancesInstanceIdConfigLabelsLabelNameGet200Response".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstancesInstanceIdConfigLabelsLabelNameGet200Response {
            value: intermediate_rep.value.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstancesInstanceIdConfigLabelsLabelNameGet200Response> and HeaderValue

#[cfg(feature = "server")]
impl
    std::convert::TryFrom<
        header::IntoHeaderValue<InstancesInstanceIdConfigLabelsLabelNameGet200Response>,
    > for HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstancesInstanceIdConfigLabelsLabelNameGet200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for InstancesInstanceIdConfigLabelsLabelNameGet200Response - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue>
    for header::IntoHeaderValue<InstancesInstanceIdConfigLabelsLabelNameGet200Response>
{
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <InstancesInstanceIdConfigLabelsLabelNameGet200Response as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into InstancesInstanceIdConfigLabelsLabelNameGet200Response - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[allow(non_camel_case_types)]
pub enum InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutRequest {
    PortRange(Box<models::PortRange>),
    I32(Box<i32>),
}

impl validator::Validate
    for InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutRequest
{
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        match self {
            Self::PortRange(x) => x.validate(),
            Self::I32(_) => std::result::Result::Ok(()),
        }
    }
}

impl From<models::PortRange>
    for InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutRequest
{
    fn from(value: models::PortRange) -> Self {
        Self::PortRange(Box::new(value))
    }
}
impl From<i32> for InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutRequest {
    fn from(value: i32) -> Self {
        Self::I32(Box::new(value))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutRequest {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdGet200Response {
    #[serde(rename = "instanceId")]
    #[validate(
            regex(path = *RE_INSTANCESINSTANCEIDGET200RESPONSE_INSTANCE_ID),
        )]
    pub instance_id: String,

    /// Instance name
    #[serde(rename = "instanceName")]
    pub instance_name: String,

    #[serde(rename = "appKey")]
    pub app_key: models::AppKey,

    #[serde(rename = "status")]
    pub status: models::InstanceStatus,

    #[serde(rename = "desired")]
    pub desired: models::InstanceStatus,

    #[serde(rename = "configFiles")]
    pub config_files: models::InstanceDetailConfigFiles,

    /// Hostname of an instance
    #[serde(rename = "hostname")]
    pub hostname: String,

    /// IP address of an instance
    #[serde(rename = "ipAddress")]
    pub ip_address: String,

    /// Allocated network ports of an instance
    #[serde(rename = "ports")]
    pub ports: Vec<models::InstanceDetailPort>,

    /// Automatic volumes of an instance
    #[serde(rename = "volumes")]
    pub volumes: Vec<models::InstanceDetailVolume>,

    #[serde(rename = "editors")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editors: Option<models::InstanceEditors>,
}

lazy_static::lazy_static! {
    static ref RE_INSTANCESINSTANCEIDGET200RESPONSE_INSTANCE_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}$").unwrap();
}

impl InstancesInstanceIdGet200Response {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        instance_id: String,
        instance_name: String,
        app_key: models::AppKey,
        status: models::InstanceStatus,
        desired: models::InstanceStatus,
        config_files: models::InstanceDetailConfigFiles,
        hostname: String,
        ip_address: String,
        ports: Vec<models::InstanceDetailPort>,
        volumes: Vec<models::InstanceDetailVolume>,
    ) -> InstancesInstanceIdGet200Response {
        InstancesInstanceIdGet200Response {
            instance_id,
            instance_name,
            app_key,
            status,
            desired,
            config_files,
            hostname,
            ip_address,
            ports,
            volumes,
            editors: None,
        }
    }
}

/// Converts the InstancesInstanceIdGet200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstancesInstanceIdGet200Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("instanceId".to_string()),
            Some(self.instance_id.to_string()),
            Some("instanceName".to_string()),
            Some(self.instance_name.to_string()),
            // Skipping appKey in query parameter serialization

            // Skipping status in query parameter serialization

            // Skipping desired in query parameter serialization

            // Skipping configFiles in query parameter serialization
            Some("hostname".to_string()),
            Some(self.hostname.to_string()),
            Some("ipAddress".to_string()),
            Some(self.ip_address.to_string()),
            // Skipping ports in query parameter serialization

            // Skipping volumes in query parameter serialization

            // Skipping editors in query parameter serialization
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstancesInstanceIdGet200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstancesInstanceIdGet200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub instance_id: Vec<String>,
            pub instance_name: Vec<String>,
            pub app_key: Vec<models::AppKey>,
            pub status: Vec<models::InstanceStatus>,
            pub desired: Vec<models::InstanceStatus>,
            pub config_files: Vec<models::InstanceDetailConfigFiles>,
            pub hostname: Vec<String>,
            pub ip_address: Vec<String>,
            pub ports: Vec<Vec<models::InstanceDetailPort>>,
            pub volumes: Vec<Vec<models::InstanceDetailVolume>>,
            pub editors: Vec<models::InstanceEditors>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstancesInstanceIdGet200Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "instanceId" => intermediate_rep.instance_id.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "instanceName" => intermediate_rep.instance_name.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "appKey" => intermediate_rep.app_key.push(<models::AppKey as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep.status.push(<models::InstanceStatus as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "desired" => intermediate_rep.desired.push(<models::InstanceStatus as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "configFiles" => intermediate_rep.config_files.push(<models::InstanceDetailConfigFiles as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "hostname" => intermediate_rep.hostname.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "ipAddress" => intermediate_rep.ip_address.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "ports" => return std::result::Result::Err("Parsing a container in this style is not supported in InstancesInstanceIdGet200Response".to_string()),
                    "volumes" => return std::result::Result::Err("Parsing a container in this style is not supported in InstancesInstanceIdGet200Response".to_string()),
                    #[allow(clippy::redundant_clone)]
                    "editors" => intermediate_rep.editors.push(<models::InstanceEditors as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing InstancesInstanceIdGet200Response".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstancesInstanceIdGet200Response {
            instance_id: intermediate_rep
                .instance_id
                .into_iter()
                .next()
                .ok_or_else(|| {
                    "instanceId missing in InstancesInstanceIdGet200Response".to_string()
                })?,
            instance_name: intermediate_rep
                .instance_name
                .into_iter()
                .next()
                .ok_or_else(|| {
                    "instanceName missing in InstancesInstanceIdGet200Response".to_string()
                })?,
            app_key: intermediate_rep
                .app_key
                .into_iter()
                .next()
                .ok_or_else(|| "appKey missing in InstancesInstanceIdGet200Response".to_string())?,
            status: intermediate_rep
                .status
                .into_iter()
                .next()
                .ok_or_else(|| "status missing in InstancesInstanceIdGet200Response".to_string())?,
            desired: intermediate_rep.desired.into_iter().next().ok_or_else(|| {
                "desired missing in InstancesInstanceIdGet200Response".to_string()
            })?,
            config_files: intermediate_rep
                .config_files
                .into_iter()
                .next()
                .ok_or_else(|| {
                    "configFiles missing in InstancesInstanceIdGet200Response".to_string()
                })?,
            hostname: intermediate_rep
                .hostname
                .into_iter()
                .next()
                .ok_or_else(|| {
                    "hostname missing in InstancesInstanceIdGet200Response".to_string()
                })?,
            ip_address: intermediate_rep
                .ip_address
                .into_iter()
                .next()
                .ok_or_else(|| {
                    "ipAddress missing in InstancesInstanceIdGet200Response".to_string()
                })?,
            ports: intermediate_rep
                .ports
                .into_iter()
                .next()
                .ok_or_else(|| "ports missing in InstancesInstanceIdGet200Response".to_string())?,
            volumes: intermediate_rep.volumes.into_iter().next().ok_or_else(|| {
                "volumes missing in InstancesInstanceIdGet200Response".to_string()
            })?,
            editors: intermediate_rep.editors.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstancesInstanceIdGet200Response> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstancesInstanceIdGet200Response>>
    for HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstancesInstanceIdGet200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for InstancesInstanceIdGet200Response - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue>
    for header::IntoHeaderValue<InstancesInstanceIdGet200Response>
{
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <InstancesInstanceIdGet200Response as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into InstancesInstanceIdGet200Response - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdLogsGet200Response {
    #[serde(rename = "stdout")]
    pub stdout: String,

    #[serde(rename = "stderr")]
    pub stderr: String,
}

impl InstancesInstanceIdLogsGet200Response {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(stdout: String, stderr: String) -> InstancesInstanceIdLogsGet200Response {
        InstancesInstanceIdLogsGet200Response { stdout, stderr }
    }
}

/// Converts the InstancesInstanceIdLogsGet200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstancesInstanceIdLogsGet200Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("stdout".to_string()),
            Some(self.stdout.to_string()),
            Some("stderr".to_string()),
            Some(self.stderr.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstancesInstanceIdLogsGet200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstancesInstanceIdLogsGet200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub stdout: Vec<String>,
            pub stderr: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstancesInstanceIdLogsGet200Response"
                            .to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "stdout" => intermediate_rep.stdout.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "stderr" => intermediate_rep.stderr.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstancesInstanceIdLogsGet200Response"
                                .to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstancesInstanceIdLogsGet200Response {
            stdout: intermediate_rep.stdout.into_iter().next().ok_or_else(|| {
                "stdout missing in InstancesInstanceIdLogsGet200Response".to_string()
            })?,
            stderr: intermediate_rep.stderr.into_iter().next().ok_or_else(|| {
                "stderr missing in InstancesInstanceIdLogsGet200Response".to_string()
            })?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstancesInstanceIdLogsGet200Response> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstancesInstanceIdLogsGet200Response>>
    for HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstancesInstanceIdLogsGet200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for InstancesInstanceIdLogsGet200Response - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue>
    for header::IntoHeaderValue<InstancesInstanceIdLogsGet200Response>
{
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <InstancesInstanceIdLogsGet200Response as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into InstancesInstanceIdLogsGet200Response - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstancesInstanceIdPatchRequest {
    #[serde(rename = "to")]
    pub to: String,
}

impl InstancesInstanceIdPatchRequest {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(to: String) -> InstancesInstanceIdPatchRequest {
        InstancesInstanceIdPatchRequest { to }
    }
}

/// Converts the InstancesInstanceIdPatchRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstancesInstanceIdPatchRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![Some("to".to_string()), Some(self.to.to_string())];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstancesInstanceIdPatchRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstancesInstanceIdPatchRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub to: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InstancesInstanceIdPatchRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "to" => intermediate_rep.to.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InstancesInstanceIdPatchRequest"
                                .to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstancesInstanceIdPatchRequest {
            to: intermediate_rep
                .to
                .into_iter()
                .next()
                .ok_or_else(|| "to missing in InstancesInstanceIdPatchRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstancesInstanceIdPatchRequest> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstancesInstanceIdPatchRequest>>
    for HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InstancesInstanceIdPatchRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for InstancesInstanceIdPatchRequest - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue>
    for header::IntoHeaderValue<InstancesInstanceIdPatchRequest>
{
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <InstancesInstanceIdPatchRequest as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into InstancesInstanceIdPatchRequest - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Ipv4Address(String);

impl validator::Validate for Ipv4Address {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for Ipv4Address {
    fn from(x: String) -> Self {
        Ipv4Address(x)
    }
}

impl std::fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::str::FromStr for Ipv4Address {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(Ipv4Address(x.to_string()))
    }
}

impl std::convert::From<Ipv4Address> for String {
    fn from(x: Ipv4Address) -> Self {
        x.0
    }
}

impl std::ops::Deref for Ipv4Address {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for Ipv4Address {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Ipv4Network {
    #[serde(rename = "address")]
    pub address: String,

    #[serde(rename = "netmask")]
    pub netmask: String,
}

impl Ipv4Network {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(address: String, netmask: String) -> Ipv4Network {
        Ipv4Network { address, netmask }
    }
}

/// Converts the Ipv4Network value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Ipv4Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("address".to_string()),
            Some(self.address.to_string()),
            Some("netmask".to_string()),
            Some(self.netmask.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Ipv4Network value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Ipv4Network {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub address: Vec<String>,
            pub netmask: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Ipv4Network".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "address" => intermediate_rep.address.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "netmask" => intermediate_rep.netmask.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Ipv4Network".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Ipv4Network {
            address: intermediate_rep
                .address
                .into_iter()
                .next()
                .ok_or_else(|| "address missing in Ipv4Network".to_string())?,
            netmask: intermediate_rep
                .netmask
                .into_iter()
                .next()
                .ok_or_else(|| "netmask missing in Ipv4Network".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Ipv4Network> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Ipv4Network>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Ipv4Network>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Ipv4Network - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Ipv4Network> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Ipv4Network as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Ipv4Network - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Ipv6Address(String);

impl validator::Validate for Ipv6Address {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for Ipv6Address {
    fn from(x: String) -> Self {
        Ipv6Address(x)
    }
}

impl std::fmt::Display for Ipv6Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::str::FromStr for Ipv6Address {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(Ipv6Address(x.to_string()))
    }
}

impl std::convert::From<Ipv6Address> for String {
    fn from(x: Ipv6Address) -> Self {
        x.0
    }
}

impl std::ops::Deref for Ipv6Address {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for Ipv6Address {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Ipv6Network {
    #[serde(rename = "address")]
    pub address: String,

    #[serde(rename = "prefix_len")]
    #[validate(range(min = 0u8, max = 128u8))]
    pub prefix_len: u8,
}

impl Ipv6Network {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(address: String, prefix_len: u8) -> Ipv6Network {
        Ipv6Network {
            address,
            prefix_len,
        }
    }
}

/// Converts the Ipv6Network value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Ipv6Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("address".to_string()),
            Some(self.address.to_string()),
            Some("prefix_len".to_string()),
            Some(self.prefix_len.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Ipv6Network value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Ipv6Network {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub address: Vec<String>,
            pub prefix_len: Vec<u8>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Ipv6Network".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "address" => intermediate_rep.address.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "prefix_len" => intermediate_rep
                        .prefix_len
                        .push(<u8 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Ipv6Network".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Ipv6Network {
            address: intermediate_rep
                .address
                .into_iter()
                .next()
                .ok_or_else(|| "address missing in Ipv6Network".to_string())?,
            prefix_len: intermediate_rep
                .prefix_len
                .into_iter()
                .next()
                .ok_or_else(|| "prefix_len missing in Ipv6Network".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Ipv6Network> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Ipv6Network>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Ipv6Network>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Ipv6Network - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Ipv6Network> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Ipv6Network as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Ipv6Network - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Job {
    #[serde(rename = "id")]
    #[validate(range(min = 1u32, max = 4294967295u32))]
    pub id: u32,

    #[serde(rename = "status")]
    pub status: models::JobStatus,

    #[serde(rename = "description")]
    pub description: String,

    #[serde(rename = "numSteps")]
    pub num_steps: i32,

    #[serde(rename = "currentStep")]
    pub current_step: models::JobStep,

    #[serde(rename = "result")]
    pub result: models::JobResult,
}

impl Job {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        id: u32,
        status: models::JobStatus,
        description: String,
        num_steps: i32,
        current_step: models::JobStep,
        result: models::JobResult,
    ) -> Job {
        Job {
            id,
            status,
            description,
            num_steps,
            current_step,
            result,
        }
    }
}

/// Converts the Job value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("id".to_string()),
            Some(self.id.to_string()),
            // Skipping status in query parameter serialization
            Some("description".to_string()),
            Some(self.description.to_string()),
            Some("numSteps".to_string()),
            Some(self.num_steps.to_string()),
            // Skipping currentStep in query parameter serialization

            // Skipping result in query parameter serialization
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Job value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Job {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub id: Vec<u32>,
            pub status: Vec<models::JobStatus>,
            pub description: Vec<String>,
            pub num_steps: Vec<i32>,
            pub current_step: Vec<models::JobStep>,
            pub result: Vec<models::JobResult>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err("Missing value while parsing Job".to_string())
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "id" => intermediate_rep.id.push(
                        <u32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep.status.push(
                        <models::JobStatus as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "description" => intermediate_rep.description.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "numSteps" => intermediate_rep.num_steps.push(
                        <i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "currentStep" => intermediate_rep.current_step.push(
                        <models::JobStep as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "result" => intermediate_rep.result.push(
                        <models::JobResult as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Job".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Job {
            id: intermediate_rep
                .id
                .into_iter()
                .next()
                .ok_or_else(|| "id missing in Job".to_string())?,
            status: intermediate_rep
                .status
                .into_iter()
                .next()
                .ok_or_else(|| "status missing in Job".to_string())?,
            description: intermediate_rep
                .description
                .into_iter()
                .next()
                .ok_or_else(|| "description missing in Job".to_string())?,
            num_steps: intermediate_rep
                .num_steps
                .into_iter()
                .next()
                .ok_or_else(|| "numSteps missing in Job".to_string())?,
            current_step: intermediate_rep
                .current_step
                .into_iter()
                .next()
                .ok_or_else(|| "currentStep missing in Job".to_string())?,
            result: intermediate_rep
                .result
                .into_iter()
                .next()
                .ok_or_else(|| "result missing in Job".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Job> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Job>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Job>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Job - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Job> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <Job as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => {
                    std::result::Result::Ok(header::IntoHeaderValue(value))
                }
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into Job - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct JobId(i32);

impl validator::Validate for JobId {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<i32> for JobId {
    fn from(x: i32) -> Self {
        JobId(x)
    }
}

impl std::convert::From<JobId> for i32 {
    fn from(x: JobId) -> Self {
        x.0
    }
}

impl std::ops::Deref for JobId {
    type Target = i32;
    fn deref(&self) -> &i32 {
        &self.0
    }
}

impl std::ops::DerefMut for JobId {
    fn deref_mut(&mut self) -> &mut i32 {
        &mut self.0
    }
}

/// Job metadata for accepted requests
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct JobMeta {
    #[serde(rename = "jobId")]
    pub job_id: i32,
}

impl JobMeta {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(job_id: i32) -> JobMeta {
        JobMeta { job_id }
    }
}

/// Converts the JobMeta value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for JobMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> =
            vec![Some("jobId".to_string()), Some(self.job_id.to_string())];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a JobMeta value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for JobMeta {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub job_id: Vec<i32>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing JobMeta".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "jobId" => intermediate_rep.job_id.push(
                        <i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing JobMeta".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(JobMeta {
            job_id: intermediate_rep
                .job_id
                .into_iter()
                .next()
                .ok_or_else(|| "jobId missing in JobMeta".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<JobMeta> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<JobMeta>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<JobMeta>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for JobMeta - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<JobMeta> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <JobMeta as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into JobMeta - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct JobResult {
    #[serde(rename = "code")]
    pub code: i32,

    #[serde(rename = "message")]
    pub message: String,
}

impl JobResult {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(code: i32, message: String) -> JobResult {
        JobResult { code, message }
    }
}

/// Converts the JobResult value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for JobResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("code".to_string()),
            Some(self.code.to_string()),
            Some("message".to_string()),
            Some(self.message.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a JobResult value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for JobResult {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub code: Vec<i32>,
            pub message: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing JobResult".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "code" => intermediate_rep.code.push(
                        <i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "message" => intermediate_rep.message.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing JobResult".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(JobResult {
            code: intermediate_rep
                .code
                .into_iter()
                .next()
                .ok_or_else(|| "code missing in JobResult".to_string())?,
            message: intermediate_rep
                .message
                .into_iter()
                .next()
                .ok_or_else(|| "message missing in JobResult".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<JobResult> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<JobResult>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<JobResult>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for JobResult - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<JobResult> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <JobResult as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into JobResult - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum JobStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "queued")]
    Queued,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "successful")]
    Successful,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "unknown")]
    Unknown,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            JobStatus::Pending => write!(f, "pending"),
            JobStatus::Queued => write!(f, "queued"),
            JobStatus::Running => write!(f, "running"),
            JobStatus::Cancelled => write!(f, "cancelled"),
            JobStatus::Successful => write!(f, "successful"),
            JobStatus::Failed => write!(f, "failed"),
            JobStatus::Unknown => write!(f, "unknown"),
        }
    }
}

impl std::str::FromStr for JobStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "pending" => std::result::Result::Ok(JobStatus::Pending),
            "queued" => std::result::Result::Ok(JobStatus::Queued),
            "running" => std::result::Result::Ok(JobStatus::Running),
            "cancelled" => std::result::Result::Ok(JobStatus::Cancelled),
            "successful" => std::result::Result::Ok(JobStatus::Successful),
            "failed" => std::result::Result::Ok(JobStatus::Failed),
            "unknown" => std::result::Result::Ok(JobStatus::Unknown),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct JobStep {
    #[serde(rename = "description")]
    pub description: String,

    #[serde(rename = "num")]
    pub num: i32,

    #[serde(rename = "unit")]
    pub unit: i32,

    #[serde(rename = "unitsTotal")]
    pub units_total: i32,

    #[serde(rename = "unitsDone")]
    pub units_done: i32,

    #[serde(rename = "rate")]
    pub rate: i32,
}

impl JobStep {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        description: String,
        num: i32,
        unit: i32,
        units_total: i32,
        units_done: i32,
        rate: i32,
    ) -> JobStep {
        JobStep {
            description,
            num,
            unit,
            units_total,
            units_done,
            rate,
        }
    }
}

/// Converts the JobStep value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for JobStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("description".to_string()),
            Some(self.description.to_string()),
            Some("num".to_string()),
            Some(self.num.to_string()),
            Some("unit".to_string()),
            Some(self.unit.to_string()),
            Some("unitsTotal".to_string()),
            Some(self.units_total.to_string()),
            Some("unitsDone".to_string()),
            Some(self.units_done.to_string()),
            Some("rate".to_string()),
            Some(self.rate.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a JobStep value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for JobStep {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub description: Vec<String>,
            pub num: Vec<i32>,
            pub unit: Vec<i32>,
            pub units_total: Vec<i32>,
            pub units_done: Vec<i32>,
            pub rate: Vec<i32>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing JobStep".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "description" => intermediate_rep.description.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "num" => intermediate_rep.num.push(
                        <i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "unit" => intermediate_rep.unit.push(
                        <i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "unitsTotal" => intermediate_rep.units_total.push(
                        <i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "unitsDone" => intermediate_rep.units_done.push(
                        <i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "rate" => intermediate_rep.rate.push(
                        <i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing JobStep".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(JobStep {
            description: intermediate_rep
                .description
                .into_iter()
                .next()
                .ok_or_else(|| "description missing in JobStep".to_string())?,
            num: intermediate_rep
                .num
                .into_iter()
                .next()
                .ok_or_else(|| "num missing in JobStep".to_string())?,
            unit: intermediate_rep
                .unit
                .into_iter()
                .next()
                .ok_or_else(|| "unit missing in JobStep".to_string())?,
            units_total: intermediate_rep
                .units_total
                .into_iter()
                .next()
                .ok_or_else(|| "unitsTotal missing in JobStep".to_string())?,
            units_done: intermediate_rep
                .units_done
                .into_iter()
                .next()
                .ok_or_else(|| "unitsDone missing in JobStep".to_string())?,
            rate: intermediate_rep
                .rate
                .into_iter()
                .next()
                .ok_or_else(|| "rate missing in JobStep".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<JobStep> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<JobStep>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<JobStep>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for JobStep - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<JobStep> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <JobStep as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into JobStep - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Jwt {
    #[serde(rename = "token")]
    pub token: String,

    #[serde(rename = "token_expires")]
    #[validate(range(min = 0u32))]
    pub token_expires: u32,
}

impl Jwt {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(token: String, token_expires: u32) -> Jwt {
        Jwt {
            token,
            token_expires,
        }
    }
}

/// Converts the Jwt value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Jwt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("token".to_string()),
            Some(self.token.to_string()),
            Some("token_expires".to_string()),
            Some(self.token_expires.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Jwt value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Jwt {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub token: Vec<String>,
            pub token_expires: Vec<u32>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err("Missing value while parsing Jwt".to_string())
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "token" => intermediate_rep.token.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "token_expires" => intermediate_rep.token_expires.push(
                        <u32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Jwt".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Jwt {
            token: intermediate_rep
                .token
                .into_iter()
                .next()
                .ok_or_else(|| "token missing in Jwt".to_string())?,
            token_expires: intermediate_rep
                .token_expires
                .into_iter()
                .next()
                .ok_or_else(|| "token_expires missing in Jwt".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Jwt> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Jwt>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Jwt>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Jwt - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Jwt> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <Jwt as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => {
                    std::result::Result::Ok(header::IntoHeaderValue(value))
                }
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into Jwt - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// License key for App installation
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct LicenseKey(String);

impl validator::Validate for LicenseKey {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for LicenseKey {
    fn from(x: String) -> Self {
        LicenseKey(x)
    }
}

impl std::fmt::Display for LicenseKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::str::FromStr for LicenseKey {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(LicenseKey(x.to_string()))
    }
}

impl std::convert::From<LicenseKey> for String {
    fn from(x: LicenseKey) -> Self {
        x.0
    }
}

impl std::ops::Deref for LicenseKey {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for LicenseKey {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MacAddress(String);

impl validator::Validate for MacAddress {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for MacAddress {
    fn from(x: String) -> Self {
        MacAddress(x)
    }
}

impl std::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::str::FromStr for MacAddress {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(MacAddress(x.to_string()))
    }
}

impl std::convert::From<MacAddress> for String {
    fn from(x: MacAddress) -> Self {
        x.0
    }
}

impl std::ops::Deref for MacAddress {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for MacAddress {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[allow(non_camel_case_types)]
pub enum Network {
    Ipv4Network(Box<models::Ipv4Network>),
    Ipv6Network(Box<models::Ipv6Network>),
}

impl validator::Validate for Network {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        match self {
            Self::Ipv4Network(x) => x.validate(),
            Self::Ipv6Network(x) => x.validate(),
        }
    }
}

impl From<models::Ipv4Network> for Network {
    fn from(value: models::Ipv4Network) -> Self {
        Self::Ipv4Network(Box::new(value))
    }
}
impl From<models::Ipv6Network> for Network {
    fn from(value: models::Ipv6Network) -> Self {
        Self::Ipv6Network(Box::new(value))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Network value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Network {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NetworkAdapter {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "ipv4_addresses")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4_addresses: Option<Vec<models::Ipv4Address>>,

    #[serde(rename = "ipv6_addresses")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_addresses: Option<Vec<models::Ipv6Address>>,

    #[serde(rename = "networks")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networks: Option<Vec<models::Network>>,

    #[serde(rename = "gateway")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,

    #[serde(rename = "mac_address")]
    #[validate(
            regex(path = *RE_NETWORKADAPTER_MAC_ADDRESS),
        )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,

    #[serde(rename = "net_type")]
    pub net_type: models::NetworkType,

    #[serde(rename = "is_connected")]
    pub is_connected: bool,
}

lazy_static::lazy_static! {
    static ref RE_NETWORKADAPTER_MAC_ADDRESS: regex::Regex = regex::Regex::new("^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$").unwrap();
}

impl NetworkAdapter {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(name: String, net_type: models::NetworkType, is_connected: bool) -> NetworkAdapter {
        NetworkAdapter {
            name,
            ipv4_addresses: None,
            ipv6_addresses: None,
            networks: None,
            gateway: None,
            mac_address: None,
            net_type,
            is_connected,
        }
    }
}

/// Converts the NetworkAdapter value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for NetworkAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("name".to_string()),
            Some(self.name.to_string()),
            self.ipv4_addresses.as_ref().map(|ipv4_addresses| {
                [
                    "ipv4_addresses".to_string(),
                    ipv4_addresses
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                ]
                .join(",")
            }),
            self.ipv6_addresses.as_ref().map(|ipv6_addresses| {
                [
                    "ipv6_addresses".to_string(),
                    ipv6_addresses
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                ]
                .join(",")
            }),
            // Skipping networks in query parameter serialization
            self.gateway
                .as_ref()
                .map(|gateway| ["gateway".to_string(), gateway.to_string()].join(",")),
            self.mac_address
                .as_ref()
                .map(|mac_address| ["mac_address".to_string(), mac_address.to_string()].join(",")),
            // Skipping net_type in query parameter serialization
            Some("is_connected".to_string()),
            Some(self.is_connected.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkAdapter value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkAdapter {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub name: Vec<String>,
            pub ipv4_addresses: Vec<Vec<models::Ipv4Address>>,
            pub ipv6_addresses: Vec<Vec<models::Ipv6Address>>,
            pub networks: Vec<Vec<models::Network>>,
            pub gateway: Vec<String>,
            pub mac_address: Vec<String>,
            pub net_type: Vec<models::NetworkType>,
            pub is_connected: Vec<bool>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing NetworkAdapter".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "name" => intermediate_rep.name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    "ipv4_addresses" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in NetworkAdapter"
                                .to_string(),
                        )
                    }
                    "ipv6_addresses" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in NetworkAdapter"
                                .to_string(),
                        )
                    }
                    "networks" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in NetworkAdapter"
                                .to_string(),
                        )
                    }
                    #[allow(clippy::redundant_clone)]
                    "gateway" => intermediate_rep.gateway.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "mac_address" => intermediate_rep.mac_address.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "net_type" => intermediate_rep.net_type.push(
                        <models::NetworkType as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "is_connected" => intermediate_rep.is_connected.push(
                        <bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing NetworkAdapter".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkAdapter {
            name: intermediate_rep
                .name
                .into_iter()
                .next()
                .ok_or_else(|| "name missing in NetworkAdapter".to_string())?,
            ipv4_addresses: intermediate_rep.ipv4_addresses.into_iter().next(),
            ipv6_addresses: intermediate_rep.ipv6_addresses.into_iter().next(),
            networks: intermediate_rep.networks.into_iter().next(),
            gateway: intermediate_rep.gateway.into_iter().next(),
            mac_address: intermediate_rep.mac_address.into_iter().next(),
            net_type: intermediate_rep
                .net_type
                .into_iter()
                .next()
                .ok_or_else(|| "net_type missing in NetworkAdapter".to_string())?,
            is_connected: intermediate_rep
                .is_connected
                .into_iter()
                .next()
                .ok_or_else(|| "is_connected missing in NetworkAdapter".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkAdapter> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<NetworkAdapter>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<NetworkAdapter>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for NetworkAdapter - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<NetworkAdapter> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <NetworkAdapter as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into NetworkAdapter - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum NetworkType {
    #[serde(rename = "Unknown")]
    Unknown,
    #[serde(rename = "Wired")]
    Wired,
    #[serde(rename = "Wireless")]
    Wireless,
    #[serde(rename = "Local")]
    Local,
    #[serde(rename = "Bridge")]
    Bridge,
    #[serde(rename = "Virtual")]
    Virtual,
}

impl std::fmt::Display for NetworkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            NetworkType::Unknown => write!(f, "Unknown"),
            NetworkType::Wired => write!(f, "Wired"),
            NetworkType::Wireless => write!(f, "Wireless"),
            NetworkType::Local => write!(f, "Local"),
            NetworkType::Bridge => write!(f, "Bridge"),
            NetworkType::Virtual => write!(f, "Virtual"),
        }
    }
}

impl std::str::FromStr for NetworkType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Unknown" => std::result::Result::Ok(NetworkType::Unknown),
            "Wired" => std::result::Result::Ok(NetworkType::Wired),
            "Wireless" => std::result::Result::Ok(NetworkType::Wireless),
            "Local" => std::result::Result::Ok(NetworkType::Local),
            "Bridge" => std::result::Result::Ok(NetworkType::Bridge),
            "Virtual" => std::result::Result::Ok(NetworkType::Virtual),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

/// Additional info
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct OptionalAdditionalInfo {
    #[serde(rename = "additionalInfo")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_info: Option<String>,
}

impl OptionalAdditionalInfo {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new() -> OptionalAdditionalInfo {
        OptionalAdditionalInfo {
            additional_info: None,
        }
    }
}

/// Converts the OptionalAdditionalInfo value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for OptionalAdditionalInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> =
            vec![self.additional_info.as_ref().map(|additional_info| {
                ["additionalInfo".to_string(), additional_info.to_string()].join(",")
            })];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a OptionalAdditionalInfo value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for OptionalAdditionalInfo {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub additional_info: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing OptionalAdditionalInfo".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "additionalInfo" => intermediate_rep.additional_info.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing OptionalAdditionalInfo".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(OptionalAdditionalInfo {
            additional_info: intermediate_rep.additional_info.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<OptionalAdditionalInfo> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<OptionalAdditionalInfo>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<OptionalAdditionalInfo>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for OptionalAdditionalInfo - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<OptionalAdditionalInfo> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <OptionalAdditionalInfo as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into OptionalAdditionalInfo - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Port(i32);

impl validator::Validate for Port {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<i32> for Port {
    fn from(x: i32) -> Self {
        Port(x)
    }
}

impl std::convert::From<Port> for i32 {
    fn from(x: Port) -> Self {
        x.0
    }
}

impl std::ops::Deref for Port {
    type Target = i32;
    fn deref(&self) -> &i32 {
        &self.0
    }
}

impl std::ops::DerefMut for Port {
    fn deref_mut(&mut self) -> &mut i32 {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct PortPathParameter(String);

impl validator::Validate for PortPathParameter {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for PortPathParameter {
    fn from(x: String) -> Self {
        PortPathParameter(x)
    }
}

impl std::fmt::Display for PortPathParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::str::FromStr for PortPathParameter {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(PortPathParameter(x.to_string()))
    }
}

impl std::convert::From<PortPathParameter> for String {
    fn from(x: PortPathParameter) -> Self {
        x.0
    }
}

impl std::ops::Deref for PortPathParameter {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for PortPathParameter {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct PortRange {
    #[serde(rename = "start")]
    #[validate(range(min = 1u16, max = 65535u16))]
    pub start: u16,

    #[serde(rename = "end")]
    #[validate(range(min = 1u16, max = 65535u16))]
    pub end: u16,
}

impl PortRange {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(start: u16, end: u16) -> PortRange {
        PortRange { start, end }
    }
}

/// Converts the PortRange value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for PortRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("start".to_string()),
            Some(self.start.to_string()),
            Some("end".to_string()),
            Some(self.end.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a PortRange value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for PortRange {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub start: Vec<u16>,
            pub end: Vec<u16>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing PortRange".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "start" => intermediate_rep.start.push(
                        <u16 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "end" => intermediate_rep.end.push(
                        <u16 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing PortRange".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(PortRange {
            start: intermediate_rep
                .start
                .into_iter()
                .next()
                .ok_or_else(|| "start missing in PortRange".to_string())?,
            end: intermediate_rep
                .end
                .into_iter()
                .next()
                .ok_or_else(|| "end missing in PortRange".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<PortRange> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<PortRange>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<PortRange>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for PortRange - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<PortRange> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <PortRange as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into PortRange - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SessionId {
    #[serde(rename = "id")]
    #[validate(
            regex(path = *RE_SESSIONID_ID),
        )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "timestamp")]
    #[validate(range(min = 0u64, max = 9223372036854775807u64))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
}

lazy_static::lazy_static! {
    static ref RE_SESSIONID_ID: regex::Regex = regex::Regex::new("^[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}$").unwrap();
}

impl SessionId {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new() -> SessionId {
        SessionId {
            id: None,
            timestamp: None,
        }
    }
}

/// Converts the SessionId value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            self.id
                .as_ref()
                .map(|id| ["id".to_string(), id.to_string()].join(",")),
            self.timestamp
                .as_ref()
                .map(|timestamp| ["timestamp".to_string(), timestamp.to_string()].join(",")),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SessionId value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SessionId {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub id: Vec<String>,
            pub timestamp: Vec<u64>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing SessionId".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "id" => intermediate_rep.id.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "timestamp" => intermediate_rep.timestamp.push(
                        <u64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing SessionId".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SessionId {
            id: intermediate_rep.id.into_iter().next(),
            timestamp: intermediate_rep.timestamp.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SessionId> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<SessionId>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<SessionId>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for SessionId - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<SessionId> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <SessionId as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into SessionId - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SystemDistro {
    #[serde(rename = "codename")]
    pub codename: String,

    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "version")]
    pub version: String,
}

impl SystemDistro {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(codename: String, id: String, name: String, version: String) -> SystemDistro {
        SystemDistro {
            codename,
            id,
            name,
            version,
        }
    }
}

/// Converts the SystemDistro value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for SystemDistro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("codename".to_string()),
            Some(self.codename.to_string()),
            Some("id".to_string()),
            Some(self.id.to_string()),
            Some("name".to_string()),
            Some(self.name.to_string()),
            Some("version".to_string()),
            Some(self.version.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SystemDistro value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SystemDistro {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub codename: Vec<String>,
            pub id: Vec<String>,
            pub name: Vec<String>,
            pub version: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing SystemDistro".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "codename" => intermediate_rep.codename.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "id" => intermediate_rep.id.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "name" => intermediate_rep.name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "version" => intermediate_rep.version.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing SystemDistro".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SystemDistro {
            codename: intermediate_rep
                .codename
                .into_iter()
                .next()
                .ok_or_else(|| "codename missing in SystemDistro".to_string())?,
            id: intermediate_rep
                .id
                .into_iter()
                .next()
                .ok_or_else(|| "id missing in SystemDistro".to_string())?,
            name: intermediate_rep
                .name
                .into_iter()
                .next()
                .ok_or_else(|| "name missing in SystemDistro".to_string())?,
            version: intermediate_rep
                .version
                .into_iter()
                .next()
                .ok_or_else(|| "version missing in SystemDistro".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SystemDistro> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<SystemDistro>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<SystemDistro>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for SystemDistro - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<SystemDistro> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <SystemDistro as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into SystemDistro - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SystemInfo {
    #[serde(rename = "arch")]
    pub arch: String,

    #[serde(rename = "distro")]
    pub distro: models::SystemDistro,

    #[serde(rename = "kernel")]
    pub kernel: models::SystemKernel,

    #[serde(rename = "platform")]
    pub platform: String,
}

impl SystemInfo {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        arch: String,
        distro: models::SystemDistro,
        kernel: models::SystemKernel,
        platform: String,
    ) -> SystemInfo {
        SystemInfo {
            arch,
            distro,
            kernel,
            platform,
        }
    }
}

/// Converts the SystemInfo value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for SystemInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("arch".to_string()),
            Some(self.arch.to_string()),
            // Skipping distro in query parameter serialization

            // Skipping kernel in query parameter serialization
            Some("platform".to_string()),
            Some(self.platform.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SystemInfo value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SystemInfo {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub arch: Vec<String>,
            pub distro: Vec<models::SystemDistro>,
            pub kernel: Vec<models::SystemKernel>,
            pub platform: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing SystemInfo".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "arch" => intermediate_rep.arch.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "distro" => intermediate_rep.distro.push(
                        <models::SystemDistro as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "kernel" => intermediate_rep.kernel.push(
                        <models::SystemKernel as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "platform" => intermediate_rep.platform.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing SystemInfo".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SystemInfo {
            arch: intermediate_rep
                .arch
                .into_iter()
                .next()
                .ok_or_else(|| "arch missing in SystemInfo".to_string())?,
            distro: intermediate_rep
                .distro
                .into_iter()
                .next()
                .ok_or_else(|| "distro missing in SystemInfo".to_string())?,
            kernel: intermediate_rep
                .kernel
                .into_iter()
                .next()
                .ok_or_else(|| "kernel missing in SystemInfo".to_string())?,
            platform: intermediate_rep
                .platform
                .into_iter()
                .next()
                .ok_or_else(|| "platform missing in SystemInfo".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SystemInfo> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<SystemInfo>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<SystemInfo>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for SystemInfo - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<SystemInfo> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <SystemInfo as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into SystemInfo - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SystemKernel {
    #[serde(rename = "build")]
    pub build: String,

    #[serde(rename = "machine")]
    pub machine: String,

    #[serde(rename = "version")]
    pub version: String,
}

impl SystemKernel {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(build: String, machine: String, version: String) -> SystemKernel {
        SystemKernel {
            build,
            machine,
            version,
        }
    }
}

/// Converts the SystemKernel value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for SystemKernel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("build".to_string()),
            Some(self.build.to_string()),
            Some("machine".to_string()),
            Some(self.machine.to_string()),
            Some("version".to_string()),
            Some(self.version.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SystemKernel value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SystemKernel {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub build: Vec<String>,
            pub machine: Vec<String>,
            pub version: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing SystemKernel".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "build" => intermediate_rep.build.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "machine" => intermediate_rep.machine.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "version" => intermediate_rep.version.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing SystemKernel".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SystemKernel {
            build: intermediate_rep
                .build
                .into_iter()
                .next()
                .ok_or_else(|| "build missing in SystemKernel".to_string())?,
            machine: intermediate_rep
                .machine
                .into_iter()
                .next()
                .ok_or_else(|| "machine missing in SystemKernel".to_string())?,
            version: intermediate_rep
                .version
                .into_iter()
                .next()
                .ok_or_else(|| "version missing in SystemKernel".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SystemKernel> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<SystemKernel>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<SystemKernel>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for SystemKernel - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<SystemKernel> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <SystemKernel as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into SystemKernel - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SystemNetworksNetworkIdDhcpIpv4Post200Response {
    #[serde(rename = "ipv4_address")]
    pub ipv4_address: String,
}

impl SystemNetworksNetworkIdDhcpIpv4Post200Response {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(ipv4_address: String) -> SystemNetworksNetworkIdDhcpIpv4Post200Response {
        SystemNetworksNetworkIdDhcpIpv4Post200Response { ipv4_address }
    }
}

/// Converts the SystemNetworksNetworkIdDhcpIpv4Post200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for SystemNetworksNetworkIdDhcpIpv4Post200Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("ipv4_address".to_string()),
            Some(self.ipv4_address.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SystemNetworksNetworkIdDhcpIpv4Post200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SystemNetworksNetworkIdDhcpIpv4Post200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub ipv4_address: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err(
                    "Missing value while parsing SystemNetworksNetworkIdDhcpIpv4Post200Response"
                        .to_string(),
                ),
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "ipv4_address" => intermediate_rep.ipv4_address.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing SystemNetworksNetworkIdDhcpIpv4Post200Response".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SystemNetworksNetworkIdDhcpIpv4Post200Response {
            ipv4_address: intermediate_rep
                .ipv4_address
                .into_iter()
                .next()
                .ok_or_else(|| {
                    "ipv4_address missing in SystemNetworksNetworkIdDhcpIpv4Post200Response"
                        .to_string()
                })?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SystemNetworksNetworkIdDhcpIpv4Post200Response> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<SystemNetworksNetworkIdDhcpIpv4Post200Response>>
    for HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<SystemNetworksNetworkIdDhcpIpv4Post200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for SystemNetworksNetworkIdDhcpIpv4Post200Response - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue>
    for header::IntoHeaderValue<SystemNetworksNetworkIdDhcpIpv4Post200Response>
{
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <SystemNetworksNetworkIdDhcpIpv4Post200Response as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into SystemNetworksNetworkIdDhcpIpv4Post200Response - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SystemVersionGet200Response {
    #[serde(rename = "api")]
    pub api: String,

    #[serde(rename = "core")]
    pub core: String,
}

impl SystemVersionGet200Response {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(api: String, core: String) -> SystemVersionGet200Response {
        SystemVersionGet200Response { api, core }
    }
}

/// Converts the SystemVersionGet200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for SystemVersionGet200Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("api".to_string()),
            Some(self.api.to_string()),
            Some("core".to_string()),
            Some(self.core.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SystemVersionGet200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SystemVersionGet200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub api: Vec<String>,
            pub core: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing SystemVersionGet200Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "api" => intermediate_rep.api.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "core" => intermediate_rep.core.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing SystemVersionGet200Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SystemVersionGet200Response {
            api: intermediate_rep
                .api
                .into_iter()
                .next()
                .ok_or_else(|| "api missing in SystemVersionGet200Response".to_string())?,
            core: intermediate_rep
                .core
                .into_iter()
                .next()
                .ok_or_else(|| "core missing in SystemVersionGet200Response".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SystemVersionGet200Response> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<SystemVersionGet200Response>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<SystemVersionGet200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for SystemVersionGet200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<SystemVersionGet200Response> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <SystemVersionGet200Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into SystemVersionGet200Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum TransportProtocol {
    #[serde(rename = "tcp")]
    Tcp,
    #[serde(rename = "udp")]
    Udp,
    #[serde(rename = "sctp")]
    Sctp,
}

impl std::fmt::Display for TransportProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            TransportProtocol::Tcp => write!(f, "tcp"),
            TransportProtocol::Udp => write!(f, "udp"),
            TransportProtocol::Sctp => write!(f, "sctp"),
        }
    }
}

impl std::str::FromStr for TransportProtocol {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "tcp" => std::result::Result::Ok(TransportProtocol::Tcp),
            "udp" => std::result::Result::Ok(TransportProtocol::Udp),
            "sctp" => std::result::Result::Ok(TransportProtocol::Sctp),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct UsbDevice {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "pid")]
    pub pid: i32,

    #[serde(rename = "port")]
    #[validate(
            regex(path = *RE_USBDEVICE_PORT),
        )]
    pub port: String,

    #[serde(rename = "vendor")]
    pub vendor: String,

    #[serde(rename = "vid")]
    pub vid: i32,
}

lazy_static::lazy_static! {
    static ref RE_USBDEVICE_PORT: regex::Regex = regex::Regex::new("^usb[1-9][0-9]*|[1-9][0-9]*-[1-9][0-9]*(?:\\.[1-9][0-9]*)*$").unwrap();
}

impl UsbDevice {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(name: String, pid: i32, port: String, vendor: String, vid: i32) -> UsbDevice {
        UsbDevice {
            name,
            pid,
            port,
            vendor,
            vid,
        }
    }
}

/// Converts the UsbDevice value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for UsbDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("name".to_string()),
            Some(self.name.to_string()),
            Some("pid".to_string()),
            Some(self.pid.to_string()),
            Some("port".to_string()),
            Some(self.port.to_string()),
            Some("vendor".to_string()),
            Some(self.vendor.to_string()),
            Some("vid".to_string()),
            Some(self.vid.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a UsbDevice value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for UsbDevice {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub name: Vec<String>,
            pub pid: Vec<i32>,
            pub port: Vec<String>,
            pub vendor: Vec<String>,
            pub vid: Vec<i32>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing UsbDevice".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "name" => intermediate_rep.name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "pid" => intermediate_rep.pid.push(
                        <i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "port" => intermediate_rep.port.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "vendor" => intermediate_rep.vendor.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "vid" => intermediate_rep.vid.push(
                        <i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing UsbDevice".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(UsbDevice {
            name: intermediate_rep
                .name
                .into_iter()
                .next()
                .ok_or_else(|| "name missing in UsbDevice".to_string())?,
            pid: intermediate_rep
                .pid
                .into_iter()
                .next()
                .ok_or_else(|| "pid missing in UsbDevice".to_string())?,
            port: intermediate_rep
                .port
                .into_iter()
                .next()
                .ok_or_else(|| "port missing in UsbDevice".to_string())?,
            vendor: intermediate_rep
                .vendor
                .into_iter()
                .next()
                .ok_or_else(|| "vendor missing in UsbDevice".to_string())?,
            vid: intermediate_rep
                .vid
                .into_iter()
                .next()
                .ok_or_else(|| "vid missing in UsbDevice".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<UsbDevice> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<UsbDevice>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<UsbDevice>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for UsbDevice - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<UsbDevice> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <UsbDevice as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into UsbDevice - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct UsbPort(String);

impl validator::Validate for UsbPort {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for UsbPort {
    fn from(x: String) -> Self {
        UsbPort(x)
    }
}

impl std::fmt::Display for UsbPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::str::FromStr for UsbPort {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(UsbPort(x.to_string()))
    }
}

impl std::convert::From<UsbPort> for String {
    fn from(x: UsbPort) -> Self {
        x.0
    }
}

impl std::ops::Deref for UsbPort {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for UsbPort {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct User {
    #[serde(rename = "ID")]
    #[validate(range(min = 0u32))]
    pub id: u32,

    #[serde(rename = "user_email")]
    pub user_email: String,

    #[serde(rename = "user_login")]
    pub user_login: String,

    #[serde(rename = "display_name")]
    pub display_name: String,
}

impl User {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(id: u32, user_email: String, user_login: String, display_name: String) -> User {
        User {
            id,
            user_email,
            user_login,
            display_name,
        }
    }
}

/// Converts the User value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("ID".to_string()),
            Some(self.id.to_string()),
            Some("user_email".to_string()),
            Some(self.user_email.to_string()),
            Some("user_login".to_string()),
            Some(self.user_login.to_string()),
            Some("display_name".to_string()),
            Some(self.display_name.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a User value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for User {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub id: Vec<u32>,
            pub user_email: Vec<String>,
            pub user_login: Vec<String>,
            pub display_name: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err("Missing value while parsing User".to_string())
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "ID" => intermediate_rep.id.push(
                        <u32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "user_email" => intermediate_rep.user_email.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "user_login" => intermediate_rep.user_login.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "display_name" => intermediate_rep.display_name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing User".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(User {
            id: intermediate_rep
                .id
                .into_iter()
                .next()
                .ok_or_else(|| "ID missing in User".to_string())?,
            user_email: intermediate_rep
                .user_email
                .into_iter()
                .next()
                .ok_or_else(|| "user_email missing in User".to_string())?,
            user_login: intermediate_rep
                .user_login
                .into_iter()
                .next()
                .ok_or_else(|| "user_login missing in User".to_string())?,
            display_name: intermediate_rep
                .display_name
                .into_iter()
                .next()
                .ok_or_else(|| "display_name missing in User".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<User> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<User>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<User>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for User - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<User> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <User as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => {
                    std::result::Result::Ok(header::IntoHeaderValue(value))
                }
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into User - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}
