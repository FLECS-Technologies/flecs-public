mod mage_quester_impl;
use crate::quest::quest_master::DeleteQuestError;
use crate::quest::{State, SyncQuest};
use crate::sorcerer::Sorcerer;
use async_trait::async_trait;
use flecsd_axum_server::models;
pub use mage_quester_impl::MageQuesterImpl;
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait MageQuester: Sorcerer {
    async fn get_job(&self, id: u64) -> Option<models::Job>;
    async fn get_jobs(&self) -> Vec<models::Job>;
    async fn delete_job(&self, id: u64) -> Result<SyncQuest, DeleteQuestError>;
}

#[cfg(test)]
impl Sorcerer for MockMageQuester {}

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
