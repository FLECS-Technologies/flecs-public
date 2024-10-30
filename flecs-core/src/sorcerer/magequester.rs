use crate::lore;
use crate::quest::quest_master::DeleteQuestError;
use crate::quest::{QuestId, State, SyncQuest};
use flecsd_axum_server::models;
use futures::stream::StreamExt;

pub async fn get_job(id: u64) -> Option<models::Job> {
    match lore::quest::default()
        .await
        .lock()
        .await
        .query_quest(QuestId(id))
    {
        None => None,
        Some(quest) => Some(job_from_quest(quest).await),
    }
}

pub async fn get_jobs() -> Vec<models::Job> {
    let questmaster = lore::quest::default().await;
    let questmaster = questmaster.lock().await;
    futures::stream::iter(questmaster.get_quests().into_iter())
        .then(|quest| async move { job_from_quest(quest).await })
        .collect::<Vec<models::Job>>()
        .await
}

pub async fn delete_job(id: u64) -> Result<SyncQuest, DeleteQuestError> {
    lore::quest::default()
        .await
        .lock()
        .await
        .delete_quest(QuestId(id))
        .await
}

// TODO: Rework job and quest api
async fn job_from_quest(quest: SyncQuest) -> models::Job {
    let quest = quest.lock().await;
    let sub_quest_progress = quest.sub_quest_progress().await;
    let (units_total, units_done) = if let Some(progress) = &quest.progress {
        (progress.total as i32, progress.current as i32)
    } else {
        (0, 0)
    };

    models::Job {
        id: quest.id.0 as u32,
        status: quest.state.into(),
        description: quest.description.clone(),
        num_steps: sub_quest_progress.total as i32,
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
            message: String::new(),
        },
    }
}

// TODO: Rework job and quest api
impl From<State> for models::JobStatus {
    fn from(value: State) -> Self {
        match value {
            State::Failing => Self::Running,
            State::Ongoing => Self::Running,
            State::Pending => Self::Pending,
            State::Failed => Self::Failed,
            State::Success => Self::Successful,
            State::Skipped => Self::Unknown,
        }
    }
}
