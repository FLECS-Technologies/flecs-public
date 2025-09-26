pub use super::{Error, Result};
use crate::vault::pouch::instance::InstanceId;
use futures_util::StreamExt;
use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{debug, warn};
use utoipa::ToSchema;

#[repr(transparent)]
#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug, Deserialize, Serialize, ToSchema)]
#[serde(transparent)]
pub struct QuestId(pub u64);

pub type SyncQuest = Arc<Mutex<Quest>>;

fn get_quest_id() -> QuestId {
    static ID: OnceLock<AtomicU64> = OnceLock::new();
    QuestId(
        ID.get_or_init(|| AtomicU64::new(0))
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
    )
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum QuestResult {
    None,
    InstanceId(InstanceId),
    ExportId(String),
}

impl QuestResult {
    fn to_model_string(&self) -> Option<String> {
        match self {
            Self::None => None,
            Self::InstanceId(id) => Some(id.to_string()),
            Self::ExportId(id) => Some(id.clone()),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum State {
    Failing,
    Ongoing,
    Pending,
    Failed,
    Success,
    Skipped,
}

impl From<State> for flecsd_axum_server::models::QuestState {
    fn from(value: State) -> Self {
        match value {
            State::Failing => Self::Failing,
            State::Ongoing => Self::Ongoing,
            State::Pending => Self::Pending,
            State::Failed => Self::Failed,
            State::Success => Self::Success,
            State::Skipped => Self::Skipped,
        }
    }
}

impl State {
    pub fn is_finished(&self) -> bool {
        matches!(self, State::Failed | State::Success | State::Skipped)
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Failing => {
                write!(f, "Failing")
            }
            State::Ongoing => {
                write!(f, "Ongoing")
            }
            State::Failed => {
                write!(f, "Failed")
            }
            State::Success => {
                write!(f, "Success")
            }
            State::Skipped => {
                write!(f, "Skipped")
            }
            State::Pending => {
                write!(f, "Pending")
            }
        }
    }
}

pub(crate) async fn finish_quest<T, E>(quest: &SyncQuest, result: Result<T, E>) -> Result<T, E>
where
    E: Display,
{
    match result {
        Ok(ok) => {
            let mut quest = quest.lock().await;
            if !quest.state.is_finished() {
                quest.state = State::Success
            }
            Ok(ok)
        }
        Err(e) => {
            quest.lock().await.fail_with_error(&e);
            Err(e)
        }
    }
}

pub struct Quest {
    pub id: QuestId,
    pub description: String,
    pub detail: Option<String>,
    sub_quests: Vec<SyncQuest>,
    pub progress: Option<Progress>,
    pub state: State,
    pub result: QuestResult,
}

#[derive(Debug, Eq, PartialEq, Default, Clone)]
pub struct Progress {
    pub current: u64,
    pub total: Option<u64>,
}

impl From<&Progress> for flecsd_axum_server::models::QuestProgress {
    fn from(value: &Progress) -> Self {
        Self {
            current: value.current,
            total: value.total,
        }
    }
}

impl Quest {
    fn new(description: String) -> Self {
        Self {
            id: get_quest_id(),
            state: State::Pending,
            description,
            sub_quests: Vec::new(),
            progress: None,
            detail: None,
            result: QuestResult::None,
        }
    }

    pub fn new_synced(description: impl Into<String>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new(description.into())))
    }

    pub async fn create_sub_quest<F, Fut, T, E>(
        &mut self,
        description: impl Into<String>,
        f: F,
    ) -> (QuestId, SyncQuest, BoxFuture<'static, Result<T, E>>)
    where
        F: FnOnce(SyncQuest) -> Fut,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
        T: Send + Sync + 'static,
        E: Display + std::marker::Send + 'static,
    {
        let quest = Quest::new_synced(description);
        let quest_id = quest.lock().await.id;
        let f = Box::pin(f(quest.clone()));
        self.sub_quests.push(quest.clone());
        let return_quest = quest.clone();
        let result = Box::pin(Self::process_sub_quest(quest, f));
        (quest_id, return_quest, result)
    }

    pub async fn create_infallible_sub_quest<F, Fut, T>(
        &mut self,
        description: impl Into<String>,
        f: F,
    ) -> (QuestId, SyncQuest, BoxFuture<'static, T>)
    where
        F: FnOnce(SyncQuest) -> Fut,
        Fut: Future<Output = T> + Send + 'static,
        T: Send + Sync + 'static,
    {
        let quest = Quest::new_synced(description);
        let quest_id = quest.lock().await.id;
        let f = Box::pin(f(quest.clone()));
        self.sub_quests.push(quest.clone());
        let return_quest = quest.clone();
        let result = Box::pin(Self::process_infallible_sub_quest(quest, f));
        (quest_id, return_quest, result)
    }

    pub async fn spawn_sub_quest<'a, F, Fut, T, E>(
        &'a mut self,
        description: impl Into<String>,
        f: F,
    ) -> (QuestId, SyncQuest, JoinHandle<Result<T, E>>)
    where
        F: FnOnce(SyncQuest) -> Fut,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
        T: Send + Sync + 'a + 'static,
        E: Display + Send + 'static,
    {
        let quest = Quest::new_synced(description);
        let quest_id = quest.lock().await.id;
        let f = Box::pin(f(quest.clone()));
        self.sub_quests.push(quest.clone());
        let return_quest = quest.clone();
        let result = tokio::spawn(Self::process_sub_quest(quest, f));
        (quest_id, return_quest, result)
    }

    async fn start_quest(quest: &SyncQuest) -> std::time::Instant {
        let mut quest = quest.lock().await;
        quest.state = State::Ongoing;
        debug!(
            "Started sub-quest '{}' with id {}",
            quest.description.as_str(),
            quest.id.0,
        );
        std::time::Instant::now()
    }

    async fn process_sub_quest<Fut, T, E>(quest: SyncQuest, f: Pin<Box<Fut>>) -> Result<T, E>
    where
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send + Sync,
        E: Display,
    {
        let start = Self::start_quest(&quest).await;
        let result = finish_quest(&quest, f.await).await;
        {
            let quest = quest.lock().await;
            match &result {
                Ok(_) => debug!(
                    "Sub-quest '{}' with id {} succeeded after {:#?}.",
                    quest.description,
                    quest.id.0,
                    std::time::Instant::now() - start
                ),
                Err(e) => warn!(
                    "Sub-quest '{}' with id {} failed after {:#?}: {e}",
                    quest.description,
                    quest.id.0,
                    std::time::Instant::now() - start
                ),
            }
        }
        result
    }

    async fn process_infallible_sub_quest<Fut, T>(quest: SyncQuest, f: Pin<Box<Fut>>) -> T
    where
        Fut: Future<Output = T> + Send,
        T: Send + Sync,
    {
        let start = Self::start_quest(&quest).await;
        let result = f.await;

        let mut quest = quest.lock().await;
        if !quest.state.is_finished() {
            quest.state = State::Success;
        }
        debug!(
            "Sub-quest '{}' with id {} finished after {:#?} with state {}.",
            quest.description,
            quest.id.0,
            std::time::Instant::now() - start,
            quest.state
        );

        result
    }

    pub async fn sub_quest_progress(&self) -> Progress {
        let current = futures::stream::iter(self.sub_quests.iter())
            .filter(|sub_quest| async { sub_quest.lock().await.state.is_finished() })
            .count()
            .await;
        Progress {
            current: current as u64,
            total: Some(self.sub_quests.len() as u64),
        }
    }

    pub fn fail_with_error<E: Display>(&mut self, error: &E) {
        self.state = State::Failed;
        self.detail = Some(error.to_string());
    }

    pub fn update<T: StatusUpdate>(&mut self, update: &T) {
        match update.state() {
            None => {}
            Some(state) => self.state = state,
        }
        self.detail = update.details();
        self.progress = update.progress();
    }

    pub fn add_progress(&mut self, new_progress: u64) {
        match &mut self.progress {
            Some(progress) => progress.current += new_progress,
            None => {
                self.progress = Some(Progress {
                    current: new_progress,
                    total: None,
                })
            }
        }
    }
}

impl Quest {
    pub async fn create_model(s: Arc<Mutex<Self>>) -> flecsd_axum_server::models::Quest {
        let mut stack = Vec::new();
        let mut quests: HashMap<QuestId, (flecsd_axum_server::models::Quest, HashSet<u64>)> =
            HashMap::new();
        let mut quest_mapping: HashMap<u64, QuestId> = HashMap::new();
        let mut leaf_quests = Vec::new();
        stack.push((s, 0));

        while let Some((quest, depth)) = stack.pop() {
            let quest = quest.lock().await;
            let mut sub_quests = HashSet::new();
            let (progress, leaf_quest) = if !quest.sub_quests.is_empty() {
                let mut current = 0;
                let total = Some(quest.sub_quests.len() as u64);
                for sub_quest in quest.sub_quests.iter() {
                    {
                        let sub_quest = sub_quest.lock().await;
                        if sub_quest.state.is_finished() {
                            current += 1;
                        }
                        sub_quests.insert(sub_quest.id.0);
                        quest_mapping.insert(sub_quest.id.0, quest.id);
                    }
                    stack.push((sub_quest.clone(), depth + 1));
                }
                (Some(Progress { current, total }), false)
            } else {
                (quest.progress.clone(), true)
            };

            let quest_model = flecsd_axum_server::models::Quest {
                subquests: None,
                detail: quest.detail.clone(),
                id: quest.id.0,
                result: quest.result.to_model_string(),
                description: quest.description.clone(),
                progress: progress
                    .as_ref()
                    .map(flecsd_axum_server::models::QuestProgress::from),
                state: flecsd_axum_server::models::QuestState::from(quest.state),
            };
            if leaf_quest {
                leaf_quests.push(quest_model)
            } else {
                quests.insert(quest.id, (quest_model, sub_quests));
            }
        }
        let mut stack = leaf_quests;
        while let Some(quest) = stack.pop() {
            match quest_mapping.get(&quest.id) {
                None => return quest,
                Some(parent_id) => match quests.entry(*parent_id) {
                    Entry::Vacant(_) => {}
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().1.remove(&quest.id);
                        entry
                            .get_mut()
                            .0
                            .subquests
                            .get_or_insert_default()
                            .push(quest);
                        if entry.get_mut().1.is_empty() {
                            stack.push(entry.remove().0);
                        }
                    }
                },
            }
        }
        unreachable!()
    }

    pub async fn fmt(s: Arc<Mutex<Self>>) -> String {
        let mut stack = Vec::new();
        let mut result = String::new();
        stack.push((s, 0));

        while let Some((quest, depth)) = stack.pop() {
            let indent = "  ".repeat(depth);
            let quest = quest.lock().await;
            let details = if let Some(details) = &quest.detail {
                format!(" ({details})")
            } else {
                "".to_string()
            };
            let s = if !quest.sub_quests.is_empty() {
                let mut current = 0;
                let total = quest.sub_quests.len();
                for sub_quest in quest.sub_quests.iter().rev() {
                    if sub_quest.lock().await.state.is_finished() {
                        current += 1;
                    }
                    stack.push((sub_quest.clone(), depth + 1));
                }

                format!(
                    "{indent}{}: {}{} {current}/{total}\n",
                    quest.description, quest.state, details
                )
            } else if let Some(Progress {
                current,
                total: Some(total),
            }) = &quest.progress
            {
                format!(
                    "{indent}{}: {}{} {current}/{total}\n",
                    quest.description, quest.state, details
                )
            } else if let Some(Progress { current, .. }) = &quest.progress {
                format!(
                    "{indent}{}: {}{} {current}\n",
                    quest.description, quest.state, details
                )
            } else {
                format!(
                    "{indent}{}: {}{}\n",
                    quest.description, quest.state, details
                )
            };
            result.push_str(&s);
        }
        result
    }
}

pub trait StatusUpdate {
    fn progress(&self) -> Option<Progress>;
    fn details(&self) -> Option<String>;
    fn state(&self) -> Option<State>;
}

#[cfg(test)]
pub fn create_test_quest(id: u64) -> SyncQuest {
    Arc::new(Mutex::new(Quest {
        id: QuestId(id),
        description: "Test Quest".to_string(),
        detail: None,
        sub_quests: vec![],
        progress: None,
        state: State::Ongoing,
        result: QuestResult::None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestStatusUpdate {
        state: Option<State>,
        details: Option<String>,
        progress: Option<Progress>,
    }

    impl StatusUpdate for TestStatusUpdate {
        fn progress(&self) -> Option<Progress> {
            self.progress.clone()
        }

        fn details(&self) -> Option<String> {
            self.details.clone()
        }

        fn state(&self) -> Option<State> {
            self.state
        }
    }

    #[tokio::test]
    async fn create_sub_quest_ok() {
        let quest = Quest::new_synced("TestQuest #1".to_string());
        let (id, sub_quest, result) = quest
            .lock()
            .await
            .create_sub_quest("TestSubQuest".to_string(), |_quest| async {
                Ok::<bool, anyhow::Error>(true)
            })
            .await;
        assert_eq!(quest.lock().await.sub_quests.len(), 1);
        assert_eq!(quest.lock().await.sub_quests[0].lock().await.id, id);
        assert_eq!(sub_quest.lock().await.id, id);
        assert_eq!(sub_quest.lock().await.state, State::Pending);
        assert!(result.await.unwrap());
        assert_eq!(sub_quest.lock().await.state, State::Success);
    }

    #[tokio::test]
    async fn create_infallible_sub_quest_ok() {
        let quest = Quest::new_synced("TestQuest #1".to_string());
        let (id, sub_quest, result) = quest
            .lock()
            .await
            .create_infallible_sub_quest("TestSubQuest".to_string(), |_quest| async { true })
            .await;
        assert_eq!(quest.lock().await.sub_quests.len(), 1);
        assert_eq!(quest.lock().await.sub_quests[0].lock().await.id, id);
        assert_eq!(sub_quest.lock().await.id, id);
        assert_eq!(sub_quest.lock().await.state, State::Pending);
        assert!(result.await);
        assert_eq!(sub_quest.lock().await.state, State::Success);
    }

    #[tokio::test]
    async fn create_sub_quest_err() {
        let quest = Quest::new_synced("TestQuest #1".to_string());
        let (id, sub_quest, result) = quest
            .lock()
            .await
            .create_sub_quest("TestSubQuest".to_string(), |_quest| async {
                Err::<bool, anyhow::Error>(anyhow::anyhow!("TestError"))
            })
            .await;
        assert_eq!(quest.lock().await.sub_quests.len(), 1);
        assert_eq!(quest.lock().await.sub_quests[0].lock().await.id, id);
        assert_eq!(sub_quest.lock().await.id, id);
        assert_eq!(sub_quest.lock().await.state, State::Pending);
        assert!(result.await.is_err());
        assert_eq!(sub_quest.lock().await.state, State::Failed);
    }

    #[tokio::test]
    async fn spawn_sub_quest_ok() {
        let quest = Quest::new_synced("TestQuest #1".to_string());
        let (id, sub_quest, result) = quest
            .lock()
            .await
            .spawn_sub_quest("TestSubQuest".to_string(), |_quest| async {
                Ok::<bool, anyhow::Error>(true)
            })
            .await;
        assert_eq!(quest.lock().await.sub_quests.len(), 1);
        assert_eq!(quest.lock().await.sub_quests[0].lock().await.id, id);
        assert_eq!(sub_quest.lock().await.id, id);
        assert!(result.await.unwrap().unwrap());
        assert_eq!(sub_quest.lock().await.state, State::Success);
    }

    #[tokio::test]
    async fn spawn_sub_quest_err() {
        let quest = Quest::new_synced("TestQuest #1".to_string());
        let (id, sub_quest, result) = quest
            .lock()
            .await
            .spawn_sub_quest("TestSubQuest".to_string(), |_quest| async {
                Err::<bool, anyhow::Error>(anyhow::anyhow!("TestError"))
            })
            .await;
        assert_eq!(quest.lock().await.sub_quests.len(), 1);
        assert_eq!(quest.lock().await.sub_quests[0].lock().await.id, id);
        assert_eq!(sub_quest.lock().await.id, id);
        assert!(result.await.unwrap().is_err());
        assert_eq!(sub_quest.lock().await.state, State::Failed);
    }

    #[tokio::test]
    async fn spawn_sub_quest_panic() {
        let quest = Quest::new_synced("TestQuest #1".to_string());
        let (id, sub_quest, result): (
            QuestId,
            Arc<Mutex<Quest>>,
            JoinHandle<std::result::Result<bool, anyhow::Error>>,
        ) = quest
            .lock()
            .await
            .spawn_sub_quest("TestSubQuest".to_string(), |_quest| async { panic!() })
            .await;
        assert_eq!(quest.lock().await.sub_quests.len(), 1);
        assert_eq!(quest.lock().await.sub_quests[0].lock().await.id, id);
        assert_eq!(sub_quest.lock().await.id, id);
        assert!(result.await.is_err());
    }

    #[tokio::test]
    async fn process_infallible_sub_quest_ok() {
        let quest = Quest::new_synced("TestQuest #1".to_string());
        let f = || async { true };
        let f = Box::pin(f());
        assert!(Quest::process_infallible_sub_quest(quest.clone(), f).await);
        assert_eq!(quest.lock().await.state, State::Success);
    }

    #[tokio::test]
    async fn process_infallible_sub_quest_state_kept() {
        let quest = Quest::new_synced("TestQuest #1".to_string());
        let f = |quest: SyncQuest| async move {
            quest.lock().await.state = State::Failed;
            true
        };
        let f = Box::pin(f(quest.clone()));
        assert!(Quest::process_infallible_sub_quest(quest.clone(), f).await);
        assert_eq!(quest.lock().await.state, State::Failed);
    }

    #[tokio::test]
    async fn process_sub_quest_ok() {
        let quest = Quest::new_synced("TestQuest #1".to_string());
        let f = || async { Ok::<bool, anyhow::Error>(true) };
        let f = Box::pin(f());
        assert!(Quest::process_sub_quest(quest.clone(), f).await.unwrap());
        assert_eq!(quest.lock().await.state, State::Success);
    }

    #[tokio::test]
    async fn process_sub_quest_err() {
        let quest = Quest::new_synced("TestQuest #1".to_string());
        let f = || async { Err::<bool, anyhow::Error>(anyhow::anyhow!("TestError")) };
        let f = Box::pin(f());
        assert!(Quest::process_sub_quest(quest.clone(), f).await.is_err());
        assert_eq!(quest.lock().await.state, State::Failed);
    }

    #[tokio::test]
    async fn test_finish_quest() {
        let quest = Quest::new_synced("TestQuest #1".to_string());
        assert_eq!(quest.lock().await.state, State::Pending);
        Quest::start_quest(&quest).await;
        assert_eq!(quest.lock().await.state, State::Ongoing);
    }

    #[tokio::test]
    async fn start_quest_ok() {
        let quest = Quest::new_synced("TestQuest #1".to_string());
        let result: Result<bool> = Ok(true);
        assert!(finish_quest(&quest, result).await.unwrap());
        assert_eq!(quest.lock().await.state, State::Success);
    }

    #[tokio::test]
    async fn start_quest_err() {
        let quest = Quest::new_synced("TestQuest #1".to_string());
        let result: Result<bool> = Err(anyhow::anyhow!("TestError"));
        assert!(finish_quest(&quest, result).await.is_err());
        assert_eq!(quest.lock().await.state, State::Failed);
    }

    #[tokio::test]
    async fn start_quest_state_kept() {
        let quest = Quest::new_synced("TestQuest #1".to_string());
        quest.lock().await.state = State::Failed;
        let result: Result<bool> = Ok(true);
        assert!(finish_quest(&quest, result).await.unwrap());
        assert_eq!(quest.lock().await.state, State::Failed);
    }

    #[tokio::test]
    async fn test_fail() {
        let mut quest = Quest::new("TestQuest #1".to_string());
        assert_eq!(quest.state, State::Pending);
        quest.fail_with_error(&anyhow::anyhow!(""));
        assert_eq!(quest.state, State::Failed);
        assert!(quest.detail.is_some());
    }

    #[tokio::test]
    async fn test_status_update() {
        let mut quest = Quest::new("TestQuest #1".to_string());
        assert_eq!(quest.state, State::Pending);
        assert_eq!(quest.progress, None);
        assert_eq!(quest.detail, None);
        let empty_update = TestStatusUpdate {
            state: None,
            details: None,
            progress: None,
        };
        quest.update(&empty_update);
        assert_eq!(quest.state, State::Pending);
        assert_eq!(quest.progress, None);
        assert_eq!(quest.detail, None);
        let update = TestStatusUpdate {
            state: Some(State::Failing),
            details: Some("Details".to_string()),
            progress: Some(Progress {
                total: Some(1000),
                current: 100,
            }),
        };
        quest.update(&update);
        assert_eq!(quest.state, State::Failing);
        assert_eq!(quest.progress, update.progress);
        assert_eq!(quest.detail, update.details);
        quest.update(&empty_update);
        assert_eq!(quest.state, State::Failing);
        assert_eq!(quest.progress, None);
        assert_eq!(quest.detail, None);
    }

    #[tokio::test]
    async fn test_add_progress() {
        let mut quest = Quest::new("TestQuest #1".to_string());
        assert_eq!(quest.progress, None);
        quest.add_progress(100);
        assert_eq!(
            quest.progress,
            Some(Progress {
                total: None,
                current: 100
            })
        );
        quest.progress = Some(Progress {
            total: Some(10000),
            current: 500,
        });
        quest.add_progress(100);
        assert_eq!(
            quest.progress,
            Some(Progress {
                total: Some(10000),
                current: 600,
            })
        );
    }

    #[tokio::test]
    async fn test_subquest_progress() {
        let mut quest = Quest::new("TestQuest #1".to_string());
        assert_eq!(
            quest.sub_quest_progress().await,
            Progress {
                current: 0,
                total: Some(0)
            }
        );
        quest
            .sub_quests
            .push(Quest::new_synced("TestSubquest #1".to_string()));
        assert_eq!(
            quest.sub_quest_progress().await,
            Progress {
                current: 0,
                total: Some(1)
            }
        );
        quest
            .sub_quests
            .push(Quest::new_synced("TestSubquest #2".to_string()));
        assert_eq!(
            quest.sub_quest_progress().await,
            Progress {
                current: 0,
                total: Some(2)
            }
        );
        quest.sub_quests[0].lock().await.state = State::Failed;
        assert_eq!(
            quest.sub_quest_progress().await,
            Progress {
                current: 1,
                total: Some(2)
            }
        );
        quest.sub_quests[1].lock().await.state = State::Success;
        assert_eq!(
            quest.sub_quest_progress().await,
            Progress {
                current: 2,
                total: Some(2)
            }
        );
    }
}
