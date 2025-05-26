use crate::enchantment::quest_master::{DeleteQuestError, QuestMaster};
use crate::quest::{Quest, QuestId, QuestResult, State, SyncQuest};
use crate::sorcerer::Sorcerer;
use crate::sorcerer::mage_quester::MageQuester;
use async_trait::async_trait;
use flecsd_axum_server::models;
use flecsd_axum_server::models::Job;
use futures_util::{StreamExt, stream};

#[derive(Default)]
pub struct MageQuesterImpl {}

impl Sorcerer for MageQuesterImpl {}

#[async_trait]
impl MageQuester for MageQuesterImpl {
    async fn get_job(&self, quest_master: QuestMaster, id: u64) -> Option<Job> {
        let quest = quest_master.lock().await.query_quest(QuestId(id));
        match quest {
            None => None,
            Some(quest) => Some(job_from_quest(quest).await),
        }
    }

    async fn get_jobs(&self, quest_master: QuestMaster) -> Vec<Job> {
        let quests = quest_master.lock().await.get_quests();
        futures::stream::iter(quests.into_iter())
            .then(|quest| async move { job_from_quest(quest).await })
            .collect::<Vec<models::Job>>()
            .await
    }

    async fn delete_job(
        &self,
        quest_master: QuestMaster,
        id: u64,
    ) -> Result<SyncQuest, DeleteQuestError> {
        self.delete_quest(quest_master, QuestId(id)).await
    }

    async fn delete_quest(
        &self,
        quest_master: QuestMaster,
        id: QuestId,
    ) -> Result<SyncQuest, DeleteQuestError> {
        quest_master.lock().await.delete_quest(id).await
    }

    async fn get_quest_model(
        &self,
        quest_master: QuestMaster,
        id: QuestId,
    ) -> Option<models::Quest> {
        let quest = quest_master.lock().await.query_quest(id);
        match quest {
            None => None,
            Some(quest) => Some(Quest::create_model(quest).await),
        }
    }

    async fn get_quest_models(&self, quest_master: QuestMaster) -> Vec<models::Quest> {
        let quests = quest_master.lock().await.get_quests();
        let mut models: Vec<_> = stream::iter(quests)
            .then(Quest::create_model)
            .collect()
            .await;
        models.sort_by(|l, r| l.id.cmp(&r.id));
        models
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
        result: match quest.state {
            State::Failed => models::JobResult {
                code: -1,
                message: quest.detail.clone().unwrap_or_default(),
            },
            _ => models::JobResult { code: 0, message },
        },
    }
}
