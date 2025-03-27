use crate::enchantment::quest_master::{DeleteQuestError, QuestMaster};
use crate::sorcerer::mage_quester::MageQuester;
use flecsd_axum_server::apis::jobs::{
    JobsJobIdDeleteResponse as DeleteResponse, JobsJobIdGetResponse as GetResponse,
};
use flecsd_axum_server::models::{
    JobsJobIdDeletePathParams as DeletePathParams, JobsJobIdGetPathParams as GetPathParams,
};
use std::sync::Arc;

pub async fn delete<M: MageQuester>(
    mage_quester: Arc<M>,
    quest_master: QuestMaster,
    path_params: DeletePathParams,
) -> DeleteResponse {
    match mage_quester
        .delete_job(quest_master, path_params.job_id as u64)
        .await
    {
        Ok(_) => DeleteResponse::Status200_Success,
        Err(DeleteQuestError::StillRunning) => DeleteResponse::Status400_JobNotFinished(format!(
            "Not removing unfinished job {}",
            path_params.job_id
        )),
        Err(DeleteQuestError::Unknown) => DeleteResponse::Status404_NotFound,
    }
}

pub async fn get<M: MageQuester>(
    mage_quester: Arc<M>,
    quest_master: QuestMaster,
    path_params: GetPathParams,
) -> GetResponse {
    match mage_quester
        .get_job(quest_master, path_params.job_id as u64)
        .await
    {
        Some(job) => GetResponse::Status200_Success(job),
        None => GetResponse::Status404_NotFound,
    }
}
