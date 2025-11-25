use crate::enchantment::floxy::Floxy;
use crate::jeweler::gem::instance::InstanceId;
use crate::sorcerer::instancius::{InstanceEditorPathPrefixError, Instancius};
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::{
    InstancesInstanceIdConfigEditorsPortPathPrefixDeleteResponse as DeleteResponse,
    InstancesInstanceIdConfigEditorsPortPathPrefixPutResponse as PutResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    InstancesInstanceIdConfigEditorsPortPathPrefixDeletePathParams as DeletePathParams,
    InstancesInstanceIdConfigEditorsPortPathPrefixPutPathParams as PutPathParams,
    InstancesInstanceIdConfigEditorsPortPathPrefixPutRequest as PutRequest,
};
use std::str::FromStr;
use std::sync::Arc;

pub async fn put<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    floxy: Arc<dyn Floxy>,
    path_params: PutPathParams,
    request: PutRequest,
) -> PutResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .put_instance_editor_path_prefix(
            vault,
            floxy,
            instance_id,
            path_params.port as u16,
            request.path_prefix,
        )
        .await
    {
        Ok(_) => PutResponse::Status200_PathPrefixOfEditorWasChanged,
        Err(e @ InstanceEditorPathPrefixError::InstanceNotFound(..))
        | Err(e @ InstanceEditorPathPrefixError::EditorNotFound(..)) => {
            PutResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
                additional_info: Some(e.to_string()),
            })
        }
        Err(e @ InstanceEditorPathPrefixError::NotSupported(..)) => {
            PutResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Err(e) => {
            PutResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

pub async fn delete<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    floxy: Arc<dyn Floxy>,
    path_params: DeletePathParams,
) -> DeleteResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .delete_instance_editor_path_prefix(vault, floxy, instance_id, path_params.port as u16)
        .await
    {
        Ok(_) => DeleteResponse::Status200_PathPrefixOfEditorWasRemoved,
        Err(e @ InstanceEditorPathPrefixError::InstanceNotFound(..))
        | Err(e @ InstanceEditorPathPrefixError::EditorNotFound(..)) => {
            DeleteResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
                additional_info: Some(e.to_string()),
            })
        }
        Err(e @ InstanceEditorPathPrefixError::NotSupported(..)) => {
            DeleteResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Err(e) => DeleteResponse::Status500_InternalServerError(models::AdditionalInfo::new(
            e.to_string(),
        )),
    }
}
