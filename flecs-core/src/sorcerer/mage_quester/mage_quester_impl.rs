use crate::enchantment::quest_master::{DeleteQuestError, QuestMaster};
use crate::quest::{QuestId, QuestResult, State, SyncQuest};
use crate::sorcerer::mage_quester::MageQuester;
use crate::sorcerer::Sorcerer;
use async_trait::async_trait;
use flecsd_axum_server::models;
use flecsd_axum_server::models::Job;
use futures_util::StreamExt;

#[derive(Default)]
pub struct MageQuesterImpl {}

impl Sorcerer for MageQuesterImpl {}

#[async_trait]
impl MageQuester for MageQuesterImpl {
    async fn get_job(&self, quest_master: QuestMaster, id: u64) -> Option<Job> {
        match quest_master.lock().await.query_quest(QuestId(id)) {
            None => None,
            Some(quest) => Some(job_from_quest(quest).await),
        }
    }

    async fn get_jobs(&self, quest_master: QuestMaster) -> Vec<Job> {
        let quest_master = quest_master.lock().await;
        futures::stream::iter(quest_master.get_quests().into_iter())
            .then(|quest| async move { job_from_quest(quest).await })
            .collect::<Vec<models::Job>>()
            .await
    }

    async fn delete_job(
        &self,
        quest_master: QuestMaster,
        id: u64,
    ) -> Result<SyncQuest, DeleteQuestError> {
        quest_master.lock().await.delete_quest(QuestId(id)).await
    }
}

// TODO: Rework job and quest api
async fn job_from_quest(quest: SyncQuest) -> models::Job {
    let quest = quest.lock().await;
    let sub_quest_progress = quest.sub_quest_progress().await;
    let (units_total, units_done) = if let Some(progress) = &quest.progress {
        (
            progress.total.unwrap_or_default() as i32,
            progress.current as i32,
        )
    } else {
        (0, 0)
    };
    let message = match &quest.result {
        QuestResult::None => String::new(),
        QuestResult::InstanceId(id) => id.to_string(),
        QuestResult::ExportId(id) => id.clone(),
    };
    models::Job {
        id: quest.id.0 as u32,
        status: quest.state.into(),
        description: quest.description.clone(),
        num_steps: sub_quest_progress.total.unwrap_or_default() as i32,
        current_step: models::JobStep {
            description: if let Some(detail) = &quest.detail {
                detail.clone()
            } else {
                String::new()
            },
            num: sub_quest_progress.current as i32,
            unit: 0,
            units_total,
            units_done,
            rate: 0,
        },
        result: models::JobResult {
            code: match quest.state {
                State::Failed => -1,
                _ => 0,
            },
            message,
        },
    }
}
