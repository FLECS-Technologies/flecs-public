pub mod path_prefix;

use crate::jeweler::gem::instance::InstanceId;
use crate::sorcerer::instancius::{InstanceEditorPathPrefixError, Instancius};
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::InstancesInstanceIdConfigEditorsPortGetResponse as GetResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::InstancesInstanceIdConfigEditorsPortGetPathParams as GetPathParams;
use std::str::FromStr;
use std::sync::Arc;

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .get_instance_editor(vault, instance_id, path_params.port as u16)
        .await
    {
        Ok(editor) => GetResponse::Status200_Success(editor),
        Err(e @ InstanceEditorPathPrefixError::InstanceNotFound(_))
        | Err(e @ InstanceEditorPathPrefixError::EditorNotFound(..)) => {
            GetResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
                additional_info: Some(e.to_string()),
            })
        }
        Err(e) => {
            GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}
