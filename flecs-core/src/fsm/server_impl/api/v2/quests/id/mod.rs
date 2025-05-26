use crate::enchantment::quest_master::{DeleteQuestError, QuestMaster};
use crate::quest::QuestId;
use crate::sorcerer::mage_quester::MageQuester;
use flecsd_axum_server::apis::quests::{
    QuestsIdDeleteResponse as DeleteResponse, QuestsIdGetResponse as GetResponse,
};
use flecsd_axum_server::models::{
    QuestsIdDeletePathParams as DeletePathParams, QuestsIdGetPathParams as GetPathParams,
};
use std::sync::Arc;

pub async fn get<M: MageQuester>(
    mage_quester: Arc<M>,
    quest_master: QuestMaster,
    path_params: GetPathParams,
) -> GetResponse {
    match mage_quester
        .get_quest_model(quest_master, QuestId(path_params.id as u64))
        .await
    {
        Some(quest) => GetResponse::Status200_Success(quest),
        None => GetResponse::Status404_QuestNotFound,
    }
}

pub async fn delete<M: MageQuester>(
    mage_quester: Arc<M>,
    quest_master: QuestMaster,
    path_params: DeletePathParams,
) -> DeleteResponse {
    match mage_quester
        .delete_quest(quest_master, QuestId(path_params.id as u64))
        .await
    {
        Ok(_) => DeleteResponse::Status200_Success,
        Err(DeleteQuestError::StillRunning) => {
            DeleteResponse::Status400_UnfinishedQuestsCanNotBeDeleted
        }
        Err(DeleteQuestError::Unknown) => DeleteResponse::Status404_QuestNotFound,
    }
}
