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
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigEnvironmentGetResponse {
    /// Success
    Status200_Success(models::InstanceEnvironment),
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
pub enum InstancesInstanceIdConfigGetResponse {
    /// Success
    Status200_Success(models::InstanceConfig),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigPortsDeleteResponse {
    /// Exposed ports of instance with this instance_id was deleted
    Status200_ExposedPortsOfInstanceWithThisInstance,
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigPortsGetResponse {
    /// Success
    Status200_Success(models::InstancePorts),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdConfigPortsPutResponse {
    /// Exposed ports for instance with this instance id is set
    Status200_ExposedPortsForInstanceWithThisInstanceIdIsSet,
    /// Exposed ports for instance with this instance id was created
    Status201_ExposedPortsForInstanceWithThisInstanceIdWasCreated,
    /// Malformed request
    Status400_MalformedRequest(models::AdditionalInfo),
    /// No instance with this instance_id found
    Status404_NoInstanceWithThisInstance,
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
    ) -> Result<InstancesCreatePostResponse, String>;

    /// Query all instances of one or all Apps.
    ///
    /// InstancesGet - GET /v2/instances
    async fn instances_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        query_params: models::InstancesGetQueryParams,
    ) -> Result<InstancesGetResponse, String>;

    /// Delete environment of an instance.
    ///
    /// InstancesInstanceIdConfigEnvironmentDelete - DELETE /v2/instances/{instance_id}/config/environment
    async fn instances_instance_id_config_environment_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigEnvironmentDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigEnvironmentDeleteResponse, String>;

    /// Retrieve environment of an instance.
    ///
    /// InstancesInstanceIdConfigEnvironmentGet - GET /v2/instances/{instance_id}/config/environment
    async fn instances_instance_id_config_environment_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigEnvironmentGetPathParams,
    ) -> Result<InstancesInstanceIdConfigEnvironmentGetResponse, String>;

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
    ) -> Result<InstancesInstanceIdConfigEnvironmentPutResponse, String>;

    /// Get configuration of an Instance.
    ///
    /// InstancesInstanceIdConfigGet - GET /v2/instances/{instance_id}/config
    async fn instances_instance_id_config_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigGetPathParams,
    ) -> Result<InstancesInstanceIdConfigGetResponse, String>;

    /// Delete exposed ports of an instance.
    ///
    /// InstancesInstanceIdConfigPortsDelete - DELETE /v2/instances/{instance_id}/config/ports
    async fn instances_instance_id_config_ports_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigPortsDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigPortsDeleteResponse, String>;

    /// Retrieve exposed ports of an instance.
    ///
    /// InstancesInstanceIdConfigPortsGet - GET /v2/instances/{instance_id}/config/ports
    async fn instances_instance_id_config_ports_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigPortsGetPathParams,
    ) -> Result<InstancesInstanceIdConfigPortsGetResponse, String>;

    /// Modify or create exposed ports of an instance.
    ///
    /// InstancesInstanceIdConfigPortsPut - PUT /v2/instances/{instance_id}/config/ports
    async fn instances_instance_id_config_ports_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdConfigPortsPutPathParams,
        body: models::InstancePorts,
    ) -> Result<InstancesInstanceIdConfigPortsPutResponse, String>;

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
    ) -> Result<InstancesInstanceIdConfigPostResponse, String>;

    /// Delete a single instance.
    ///
    /// InstancesInstanceIdDelete - DELETE /v2/instances/{instance_id}
    async fn instances_instance_id_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdDeletePathParams,
    ) -> Result<InstancesInstanceIdDeleteResponse, String>;

    /// Access an editor of an app.
    ///
    /// InstancesInstanceIdEditorPortGet - GET /v2/instances/{instance_id}/editor/{port}
    async fn instances_instance_id_editor_port_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdEditorPortGetPathParams,
    ) -> Result<InstancesInstanceIdEditorPortGetResponse, String>;

    /// Obtain details of an App instance.
    ///
    /// InstancesInstanceIdGet - GET /v2/instances/{instance_id}
    async fn instances_instance_id_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdGetPathParams,
    ) -> Result<InstancesInstanceIdGetResponse, String>;

    /// Retrieve logs of an Instance.
    ///
    /// InstancesInstanceIdLogsGet - GET /v2/instances/{instance_id}/logs
    async fn instances_instance_id_logs_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdLogsGetPathParams,
    ) -> Result<InstancesInstanceIdLogsGetResponse, String>;

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
    ) -> Result<InstancesInstanceIdPatchResponse, String>;

    /// Start an App instance.
    ///
    /// InstancesInstanceIdStartPost - POST /v2/instances/{instance_id}/start
    async fn instances_instance_id_start_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdStartPostPathParams,
    ) -> Result<InstancesInstanceIdStartPostResponse, String>;

    /// Stop an App instance.
    ///
    /// InstancesInstanceIdStopPost - POST /v2/instances/{instance_id}/stop
    async fn instances_instance_id_stop_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdStopPostPathParams,
    ) -> Result<InstancesInstanceIdStopPostResponse, String>;
}
