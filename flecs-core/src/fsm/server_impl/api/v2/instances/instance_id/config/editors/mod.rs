pub mod port;

use crate::jeweler::gem::instance::InstanceId;
use crate::sorcerer::instancius::{InstanceEditorPathPrefixError, Instancius};
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::InstancesInstanceIdConfigEditorsGetResponse as GetResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::InstancesInstanceIdConfigEditorsGetPathParams as GetPathParams;
use std::str::FromStr;
use std::sync::Arc;

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius.get_instance_editors(vault, instance_id).await {
        Ok(editors) => GetResponse::Status200_Success(models::InstanceEditors::from(editors)),
        Err(InstanceEditorPathPrefixError::InstanceNotFound(_)) => {
            GetResponse::Status404_NoInstanceWithThisInstance
        }
        Err(e) => {
            GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}
