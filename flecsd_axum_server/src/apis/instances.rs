use async_trait::async_trait;
use axum::extract::*;
use axum_extra::extract::{CookieJar, Multipart};
use bytes::Bytes;
use http::Method;
use serde::{Deserialize, Serialize};

use crate::{models, types::*};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesCreatePostResponse {
    /// Accepted
    Status202_Accepted(models::JobMeta),
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesGetResponse {
    /// Success
    Status200_Success(Vec<models::AppInstance>),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigEnvironmentDeleteResponse {
    /// Environment of instance with this instance_id was deleted
    Status200_EnvironmentOfInstanceWithThisInstance,
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigEnvironmentGetResponse {
    /// Success
    Status200_Success(models::InstanceEnvironment),
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigEnvironmentPutResponse {
    /// Environment for instance with this instance id is set
    Status200_EnvironmentForInstanceWithThisInstanceIdIsSet,
    /// Environment for instance with this instance id was created
    Status201_EnvironmentForInstanceWithThisInstanceIdWasCreated,
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigEnvironmentVariableNameDeleteResponse {
    /// Environment variable of instance with this instance_id was deleted
    Status200_EnvironmentVariableOfInstanceWithThisInstance,
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// Resource not found
    Status404_ResourceNotFound(models::OptionalAdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigEnvironmentVariableNameGetResponse {
    /// Success
    Status200_Success(models::InstancesInstanceIdConfigEnvironmentVariableNameGet200Response),
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// Resource not found
    Status404_ResourceNotFound(models::OptionalAdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigEnvironmentVariableNamePutResponse {
    /// Environment for instance with this instance id is set
    Status200_EnvironmentForInstanceWithThisInstanceIdIsSet,
    /// Environment for instance with this instance id was created
    Status201_EnvironmentForInstanceWithThisInstanceIdWasCreated,
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigGetResponse {
    /// Success
    Status200_Success(models::InstanceConfig),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigLabelsGetResponse {
    /// Success
    Status200_Success(Vec<models::InstanceLabel>),
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigLabelsLabelNameGetResponse {
    /// Success
    Status200_Success(models::InstancesInstanceIdConfigLabelsLabelNameGet200Response),
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// Resource not found
    Status404_ResourceNotFound(models::OptionalAdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigPortsDeleteResponse {
    /// Exposed ports of instance with this instance_id was deleted
    Status200_ExposedPortsOfInstanceWithThisInstance,
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigPortsGetResponse {
    /// Success
    Status200_Success(models::InstancePorts),
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigPortsTransportProtocolDeleteResponse {
    /// Removed all published ports of instance with this instance_id for the given transport_protocol
    Status200_RemovedAllPublishedPortsOfInstanceWithThisInstance,
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// Resource not found
    Status404_ResourceNotFound(models::OptionalAdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigPortsTransportProtocolGetResponse {
    /// Published ports for instance with this instance_id for the given transport_protocol
    Status200_PublishedPortsForInstanceWithThisInstance(Vec<models::InstancePortMapping>),
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// Resource not found
    Status404_ResourceNotFound(models::OptionalAdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeleteResponse {
    /// Success
    Status200_Success,
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// Resource not found
    Status404_ResourceNotFound(models::OptionalAdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetResponse {
    /// Success
    Status200_Success(models::InstancePortMapping),
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// Resource not found
    Status404_ResourceNotFound(models::OptionalAdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutResponse {
    /// The specified port mapping was set, the previous mapping of the host port range was overwritten
    Status200_TheSpecifiedPortMappingWasSet,
    /// The specified port mapping was created
    Status201_TheSpecifiedPortMappingWasCreated,
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// Resource not found
    Status404_ResourceNotFound(models::OptionalAdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigPortsTransportProtocolPutResponse {
    /// Published ports of instance with this instance_id for the given transport_protocol was set
    Status200_PublishedPortsOfInstanceWithThisInstance,
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// Resource not found
    Status404_ResourceNotFound(models::OptionalAdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigPostResponse {
    /// Success
    Status200_Success(models::InstanceConfig),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdDeleteResponse {
    /// Accepted
    Status202_Accepted(models::JobMeta),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdEditorPortGetResponse {
    /// Found
    Status302_Found { location: String },
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// Instance id or port not found
    Status404_InstanceIdOrPortNotFound(models::AdditionalInfo),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdGetResponse {
    /// Success
    Status200_Success(models::InstancesInstanceIdGet200Response),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdLogsGetResponse {
    /// Success
    Status200_Success(models::InstancesInstanceIdLogsGet200Response),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdPatchResponse {
    /// Accepted
    Status202_Accepted(models::JobMeta),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdStartPostResponse {
    /// Accepted
    Status202_Accepted(models::JobMeta),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdStopPostResponse {
    /// Accepted
    Status202_Accepted(models::JobMeta),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

/// Instances
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Instances {
    /// Create new instance of an installed App.
    ///
    /// InstancesCreatePost - POST /v2/instances/create
    async fn instances_create_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        body: models::InstancesCreatePostRequest,
    ) -> Result<InstancesCreatePostResponse, ()>;

    /// Query all instances of one or all Apps.
    ///
    /// InstancesGet - GET /v2/instances
    async fn instances_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        query_params: models::InstancesGetQueryParams,
    ) -> Result<InstancesGetResponse, ()>;

    /// Delete environment of an instance.
    ///
    /// InstancesInstanceIdConfigEnvironmentDelete - DELETE /v2/instances/{instance_id}/config/environment
    async fn instances_instance_id_config_environment_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigEnvironmentDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigEnvironmentDeleteResponse, ()>;

    /// Retrieve environment of an instance.
    ///
    /// InstancesInstanceIdConfigEnvironmentGet - GET /v2/instances/{instance_id}/config/environment
    async fn instances_instance_id_config_environment_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigEnvironmentGetPathParams,
    ) -> Result<InstancesInstanceIdConfigEnvironmentGetResponse, ()>;

    /// Modify or create environment of an instance.
    ///
    /// InstancesInstanceIdConfigEnvironmentPut - PUT /v2/instances/{instance_id}/config/environment
    async fn instances_instance_id_config_environment_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigEnvironmentPutPathParams,
        body: models::InstanceEnvironment,
    ) -> Result<InstancesInstanceIdConfigEnvironmentPutResponse, ()>;

    /// Remove an environment variable of an instance.
    ///
    /// InstancesInstanceIdConfigEnvironmentVariableNameDelete - DELETE /v2/instances/{instance_id}/config/environment/{variable_name}
    async fn instances_instance_id_config_environment_variable_name_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigEnvironmentVariableNameDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigEnvironmentVariableNameDeleteResponse, ()>;

    /// Retrieve the value of an environment variable.
    ///
    /// InstancesInstanceIdConfigEnvironmentVariableNameGet - GET /v2/instances/{instance_id}/config/environment/{variable_name}
    async fn instances_instance_id_config_environment_variable_name_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigEnvironmentVariableNameGetPathParams,
    ) -> Result<InstancesInstanceIdConfigEnvironmentVariableNameGetResponse, ()>;

    /// Set the value of an environment variable of an instance.
    ///
    /// InstancesInstanceIdConfigEnvironmentVariableNamePut - PUT /v2/instances/{instance_id}/config/environment/{variable_name}
    async fn instances_instance_id_config_environment_variable_name_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigEnvironmentVariableNamePutPathParams,
        body: models::InstancesInstanceIdConfigEnvironmentVariableNameGet200Response,
    ) -> Result<InstancesInstanceIdConfigEnvironmentVariableNamePutResponse, ()>;

    /// Get configuration of an Instance.
    ///
    /// InstancesInstanceIdConfigGet - GET /v2/instances/{instance_id}/config
    async fn instances_instance_id_config_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigGetPathParams,
    ) -> Result<InstancesInstanceIdConfigGetResponse, ()>;

    /// Retrieve labels of an instance.
    ///
    /// InstancesInstanceIdConfigLabelsGet - GET /v2/instances/{instance_id}/config/labels
    async fn instances_instance_id_config_labels_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigLabelsGetPathParams,
    ) -> Result<InstancesInstanceIdConfigLabelsGetResponse, ()>;

    /// Retrieve value of a specific label of an instance.
    ///
    /// InstancesInstanceIdConfigLabelsLabelNameGet - GET /v2/instances/{instance_id}/config/labels/{label_name}
    async fn instances_instance_id_config_labels_label_name_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigLabelsLabelNameGetPathParams,
    ) -> Result<InstancesInstanceIdConfigLabelsLabelNameGetResponse, ()>;

    /// Delete exposed ports of an instance.
    ///
    /// InstancesInstanceIdConfigPortsDelete - DELETE /v2/instances/{instance_id}/config/ports
    async fn instances_instance_id_config_ports_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigPortsDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigPortsDeleteResponse, ()>;

    /// Retrieve exposed ports of an instance.
    ///
    /// InstancesInstanceIdConfigPortsGet - GET /v2/instances/{instance_id}/config/ports
    async fn instances_instance_id_config_ports_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigPortsGetPathParams,
    ) -> Result<InstancesInstanceIdConfigPortsGetResponse, ()>;

    /// Remove all published ports of an instance for the given transport_protocol.
    ///
    /// InstancesInstanceIdConfigPortsTransportProtocolDelete - DELETE /v2/instances/{instance_id}/config/ports/{transport_protocol}
    async fn instances_instance_id_config_ports_transport_protocol_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigPortsTransportProtocolDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigPortsTransportProtocolDeleteResponse, ()>;

    /// Get published ports of an instance for the given transport_protocol.
    ///
    /// InstancesInstanceIdConfigPortsTransportProtocolGet - GET /v2/instances/{instance_id}/config/ports/{transport_protocol}
    async fn instances_instance_id_config_ports_transport_protocol_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigPortsTransportProtocolGetPathParams,
    ) -> Result<InstancesInstanceIdConfigPortsTransportProtocolGetResponse, ()>;

    /// Remove instance port range that is mapped to the given host port range.
    ///
    /// InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDelete - DELETE /v2/instances/{instance_id}/config/ports/{transport_protocol}/{host_port_range}
    async fn instances_instance_id_config_ports_transport_protocol_host_port_range_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeleteResponse, ()>;

    /// Retrieve instance port range that is mapped to the given host port range.
    ///
    /// InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGet - GET /v2/instances/{instance_id}/config/ports/{transport_protocol}/{host_port_range}
    async fn instances_instance_id_config_ports_transport_protocol_host_port_range_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetPathParams,
    ) -> Result<InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetResponse, ()>;

    /// Set instance port range that is mapped to the given host port range.
    ///
    /// InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePut - PUT /v2/instances/{instance_id}/config/ports/{transport_protocol}/{host_port_range}
    async fn instances_instance_id_config_ports_transport_protocol_host_port_range_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutPathParams,
        body: models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutRequest,
    ) -> Result<InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutResponse, ()>;

    /// Update or create published ports of an instance for the given transport protocol.
    ///
    /// InstancesInstanceIdConfigPortsTransportProtocolPut - PUT /v2/instances/{instance_id}/config/ports/{transport_protocol}
    async fn instances_instance_id_config_ports_transport_protocol_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigPortsTransportProtocolPutPathParams,
        body: Vec<models::InstancePortMapping>,
    ) -> Result<InstancesInstanceIdConfigPortsTransportProtocolPutResponse, ()>;

    /// Update configuration of an Instance.
    ///
    /// InstancesInstanceIdConfigPost - POST /v2/instances/{instance_id}/config
    async fn instances_instance_id_config_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigPostPathParams,
        body: models::InstanceConfig,
    ) -> Result<InstancesInstanceIdConfigPostResponse, ()>;

    /// Delete a single instance.
    ///
    /// InstancesInstanceIdDelete - DELETE /v2/instances/{instance_id}
    async fn instances_instance_id_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdDeletePathParams,
    ) -> Result<InstancesInstanceIdDeleteResponse, ()>;

    /// Access an editor of an app.
    ///
    /// InstancesInstanceIdEditorPortGet - GET /v2/instances/{instance_id}/editor/{port}
    async fn instances_instance_id_editor_port_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdEditorPortGetPathParams,
    ) -> Result<InstancesInstanceIdEditorPortGetResponse, ()>;

    /// Obtain details of an App instance.
    ///
    /// InstancesInstanceIdGet - GET /v2/instances/{instance_id}
    async fn instances_instance_id_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdGetPathParams,
    ) -> Result<InstancesInstanceIdGetResponse, ()>;

    /// Retrieve logs of an Instance.
    ///
    /// InstancesInstanceIdLogsGet - GET /v2/instances/{instance_id}/logs
    async fn instances_instance_id_logs_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdLogsGetPathParams,
    ) -> Result<InstancesInstanceIdLogsGetResponse, ()>;

    /// Update or downgrade Instance to another App version.
    ///
    /// InstancesInstanceIdPatch - PATCH /v2/instances/{instance_id}
    async fn instances_instance_id_patch(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdPatchPathParams,
        body: models::InstancesInstanceIdPatchRequest,
    ) -> Result<InstancesInstanceIdPatchResponse, ()>;

    /// Start an App instance.
    ///
    /// InstancesInstanceIdStartPost - POST /v2/instances/{instance_id}/start
    async fn instances_instance_id_start_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdStartPostPathParams,
    ) -> Result<InstancesInstanceIdStartPostResponse, ()>;

    /// Stop an App instance.
    ///
    /// InstancesInstanceIdStopPost - POST /v2/instances/{instance_id}/stop
    async fn instances_instance_id_stop_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdStopPostPathParams,
    ) -> Result<InstancesInstanceIdStopPostResponse, ()>;
}
