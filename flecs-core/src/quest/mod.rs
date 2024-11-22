pub use super::{Error, Result};
use futures_util::future::BoxFuture;
use futures_util::StreamExt;
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::debug;

pub mod quest_master;

pub enum AwaitResult<T> {
    Result(Result<T>),
    JoinHandle(JoinHandle<Result<T>>),
}

impl<T> AwaitResult<T> {
    pub async fn try_access_value(&mut self) -> Result<&T> {
        match self {
            Self::Result(Ok(v)) => Ok(v),
            Self::Result(Err(e)) => Err(anyhow::anyhow!("{}", e)),
            Self::JoinHandle(j) => {
                *self = Self::Result(j.await?);
                match self {
                    Self::Result(Ok(v)) => Ok(v),
                    Self::Result(Err(e)) => Err(anyhow::anyhow!("{}", e)),
                    _ => panic!(),
                }
            }
        }
    }

    pub fn new_synced_join_handle(handle: JoinHandle<Result<T>>) -> SyncAwaitResult<T> {
        Arc::new(Mutex::new(Self::JoinHandle(handle)))
    }

    pub fn new_synced_value(value: T) -> SyncAwaitResult<T> {
        Arc::new(Mutex::new(Self::Result(Ok(value))))
    }
}

pub type SyncAwaitResult<T> = Arc<Mutex<AwaitResult<T>>>;
#[repr(transparent)]
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct QuestId(pub u64);
pub type SyncQuest = Arc<Mutex<Quest>>;
pub type QuestResult<T> = (Arc<Mutex<Quest>>, JoinHandle<Result<T>>);

fn get_quest_id() -> QuestId {
    static ID: OnceLock<AtomicU64> = OnceLock::new();
    QuestId(
        ID.get_or_init(|| AtomicU64::new(0))
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
    )
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

async fn finish_quest<T>(quest: &SyncQuest, result: Result<T>) -> Result<T> {
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
}

#[derive(Debug, Eq, PartialEq, Default, Clone)]
pub struct Progress {
    pub current: u64,
    pub total: Option<u64>,
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
        }
    }

    pub fn new_synced(description: String) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new(description)))
    }

    pub async fn create_sub_quest<F, Fut, T>(
        &mut self,
        description: String,
        f: F,
    ) -> (QuestId, SyncQuest, BoxFuture<'static, Result<T>>)
    where
        F: FnOnce(SyncQuest) -> Fut,
        Fut: Future<Output = Result<T>> + Send + 'static,
        T: Send + Sync + 'static,
    {
        let quest = Quest::new_synced(description);
        let quest_id = quest.lock().await.id;
        let f = Box::pin(f(quest.clone()));
        self.sub_quests.push(quest.clone());
        let return_quest = quest.clone();
        let result = Box::pin(Self::process_sub_quest(quest, f));
        (quest_id, return_quest, result)
    }

    pub async fn spawn_sub_quest<'a, F, Fut, T>(
        &'a mut self,
        description: String,
        f: F,
    ) -> (QuestId, SyncQuest, JoinHandle<Result<T>>)
    where
        F: FnOnce(SyncQuest) -> Fut,
        Fut: Future<Output = Result<T>> + Send + 'static,
        T: Send + Sync + 'a + 'static,
    {
        let quest = Quest::new_synced(description);
        let quest_id = quest.lock().await.id;
        let f = Box::pin(f(quest.clone()));
        self.sub_quests.push(quest.clone());
        let return_quest = quest.clone();
        let result = tokio::spawn(Self::process_sub_quest(quest, f));
        (quest_id, return_quest, result)
    }

    async fn process_sub_quest<Fut, T>(quest: SyncQuest, f: Pin<Box<Fut>>) -> Result<T>
    where
        Fut: Future<Output = Result<T>> + Send,
        T: Send + Sync,
    {
        let start = {
            let mut quest = quest.lock().await;
            quest.state = State::Ongoing;
            debug!(
                "Started sub-quest '{}' with id {}",
                quest.description.as_str(),
                quest.id.0,
            );
            std::time::Instant::now()
        };
        let result = finish_quest(&quest, f.await).await;
        {
            let quest = quest.lock().await;
            if result.is_ok() {
                debug!(
                    "Sub-quest '{}' with id {} succeeded after {:#?}.",
                    quest.description,
                    quest.id.0,
                    std::time::Instant::now() - start
                )
            } else {
                debug!(
                    "Sub-quest '{}' with id {} failed after {:#?}.",
                    quest.description,
                    quest.id.0,
                    std::time::Instant::now() - start
                )
            }
        }
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

    pub fn fail_with_error(&mut self, error: &Error) {
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
