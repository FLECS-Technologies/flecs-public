pub mod config;
pub mod editor;
pub mod logs;
pub mod start;
pub mod stop;
use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::enchantment::quest_master::QuestMaster;
use crate::jeweler::gem::instance::InstanceId;
use crate::sorcerer::instancius::Instancius;
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::{
    InstancesInstanceIdDeleteResponse as DeleteResponse,
    InstancesInstanceIdGetResponse as GetResponse,
    InstancesInstanceIdPatchResponse as PatchResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    InstancesInstanceIdDeletePathParams as DeletePathParams,
    InstancesInstanceIdGetPathParams as GetPathParams,
    InstancesInstanceIdPatchPathParams as PatchPathParams,
    InstancesInstanceIdPatchRequest as PatchRequest,
};
use std::str::FromStr;
use std::sync::Arc;

pub async fn delete<I: Instancius + 'static, F: Floxy + 'static>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    floxy: Arc<F>,
    quest_master: QuestMaster,
    path_params: DeletePathParams,
) -> Result<DeleteResponse, ()> {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    if !instancius
        .does_instance_exist(vault.clone(), instance_id)
        .await
    {
        return Ok(DeleteResponse::Status404_NoInstanceWithThisInstance);
    }
    let floxy = FloxyOperation::new_arc(floxy);
    let quest_id = quest_master
        .lock()
        .await
        .schedule_quest(
            format!("Delete instance {instance_id}"),
            move |quest| async move {
                instancius
                    .delete_instance(quest, vault, floxy, instance_id)
                    .await
            },
        )
        .await
        // TODO: Add 500 Response to API
        .map_err(|_| ())?
        .0;
    Ok(DeleteResponse::Status202_Accepted(models::JobMeta::new(
        quest_id.0 as i32,
    )))
}

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = match InstanceId::from_str(path_params.instance_id.as_str()) {
        Err(e) => {
            return GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(
                format!("Failed to parse instance id: {e}"),
            ));
        }
        Ok(instance_id) => instance_id,
    };
    match instancius.get_instance_detailed(vault, instance_id).await {
        Ok(Some(details)) => GetResponse::Status200_Success(details),
        Ok(None) => GetResponse::Status404_NoInstanceWithThisInstance,
        Err(e) => {
            GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

pub async fn patch<I: Instancius + 'static, F: Floxy + 'static>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    floxy: Arc<F>,
    quest_master: QuestMaster,
    path_params: PatchPathParams,
    request: PatchRequest,
) -> Result<PatchResponse, ()> {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    if !instancius
        .does_instance_exist(vault.clone(), instance_id)
        .await
    {
        return Ok(PatchResponse::Status404_NoInstanceWithThisInstance);
    }
    let floxy = FloxyOperation::new_arc(floxy);
    let quest_id = quest_master
        .lock()
        .await
        .schedule_quest(
            format!("Update instance {instance_id} to {}", request.to),
            move |quest| async move {
                instancius
                    .update_instance(
                        quest,
                        vault,
                        floxy,
                        instance_id,
                        request.to,
                        crate::lore::base_path().join("instances"),
                    )
                    .await?;
                Ok(())
            },
        )
        .await
        // TODO: Add 500 Response to API
        .map_err(|_| ())?
        .0;
    Ok(PatchResponse::Status202_Accepted(models::JobMeta::new(
        quest_id.0 as i32,
    )))
}
