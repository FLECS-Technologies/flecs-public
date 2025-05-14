use crate::jeweler::gem::instance::InstanceId;
use crate::sorcerer::instancius::{InstanceConfigHostnameError, Instancius};
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::{
    InstancesInstanceIdConfigHostnameGetResponse as GetResponse,
    InstancesInstanceIdConfigHostnamePutResponse as PutResponse,
};
use flecsd_axum_server::models::{
    InstancesInstanceIdConfigHostnameGetPathParams as GetPathParams,
    InstancesInstanceIdConfigHostnamePutPathParams as PutPathParams,
    InstancesInstanceIdConfigHostnamePutRequest as PutRequest,
};
use std::str::FromStr;
use std::sync::Arc;

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius.get_instance_hostname(vault, instance_id).await {
        Ok(hostname) => GetResponse::Status200_Success(hostname),
        Err(InstanceConfigHostnameError::Unsupported(_)) => {
            GetResponse::Status400_InstanceDoesNotSupportHostnames
        }
        Err(InstanceConfigHostnameError::InstanceNotFound(_)) => {
            GetResponse::Status404_NoInstanceWithThisInstance
        }
    }
}

pub async fn put<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: PutPathParams,
    request: PutRequest,
) -> PutResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .put_instance_hostname(vault, instance_id, request.hostname)
        .await
    {
        Ok(_) => PutResponse::Status200_Success,
        Err(InstanceConfigHostnameError::Unsupported(_)) => {
            PutResponse::Status400_InstanceDoesNotSupportHostnames
        }
        Err(InstanceConfigHostnameError::InstanceNotFound(_)) => {
            PutResponse::Status404_NoInstanceWithThisInstance
        }
    }
}
