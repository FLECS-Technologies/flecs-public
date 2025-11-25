pub mod config;
pub mod depends;
pub mod editor;
pub mod logs;
pub mod provides;
pub mod start;
pub mod stop;
use crate::enchantment::floxy::Floxy;
use crate::enchantment::quest_master::QuestMaster;
use crate::jeweler::gem::instance::InstanceId;
use crate::lore::InstanceLoreRef;
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

pub async fn delete<I: Instancius + 'static>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    floxy: Arc<dyn Floxy>,
    quest_master: QuestMaster,
    path_params: DeletePathParams,
) -> DeleteResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    if !instancius
        .does_instance_exist(vault.clone(), instance_id)
        .await
    {
        return DeleteResponse::Status404_NoInstanceWithThisInstance;
    }
    match quest_master
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
    {
        Ok((id, _)) => DeleteResponse::Status202_Accepted(models::JobMeta::new(id.0 as i32)),
        Err(e) => DeleteResponse::Status500_InternalServerError(models::AdditionalInfo::new(
            e.to_string(),
        )),
    }
}

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(path_params.instance_id.as_str()).unwrap();
    match instancius.get_instance_detailed(vault, instance_id).await {
        Ok(Some(details)) => GetResponse::Status200_Success(details),
        Ok(None) => GetResponse::Status404_NoInstanceWithThisInstance,
        Err(e) => {
            GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

pub async fn patch<I: Instancius + 'static>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    floxy: Arc<dyn Floxy>,
    quest_master: QuestMaster,
    lore: InstanceLoreRef,
    path_params: PatchPathParams,
    request: PatchRequest,
) -> PatchResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    if !instancius
        .does_instance_exist(vault.clone(), instance_id)
        .await
    {
        return PatchResponse::Status404_NoInstanceWithThisInstance;
    }
    match quest_master
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
                        lore.as_ref().as_ref().base_path.clone(),
                    )
                    .await?;
                Ok(())
            },
        )
        .await
    {
        Ok((id, _)) => PatchResponse::Status202_Accepted(models::JobMeta::new(id.0 as i32)),
        Err(e) => {
            PatchResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}
