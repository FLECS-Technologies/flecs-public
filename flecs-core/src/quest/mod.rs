pub use super::Result;
use futures_util::StreamExt;
use std::fmt::{Display, Formatter};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

pub mod quest_master;

pub enum AwaitResult<T> {
    Value(T),
    JoinHandle(JoinHandle<T>),
}

impl<T> AwaitResult<T> {
    pub async fn try_access_value(&mut self) -> std::result::Result<&T, tokio::task::JoinError> {
        match self {
            Self::Value(v) => Ok(v),
            Self::JoinHandle(j) => {
                *self = Self::Value(j.await?);
                match self {
                    Self::Value(v) => Ok(v),
                    _ => panic!(),
                }
            }
        }
    }

    pub fn new_synced_join_handle(handle: JoinHandle<T>) -> SyncAwaitResult<T> {
        Arc::new(Mutex::new(Self::JoinHandle(handle)))
    }

    pub fn new_synced_value(value: T) -> SyncAwaitResult<T> {
        Arc::new(Mutex::new(Self::Value(value)))
    }
}

pub type SyncAwaitResult<T> = Arc<Mutex<AwaitResult<T>>>;
#[repr(transparent)]
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct QuestId(pub u64);
pub type SyncQuest = Arc<Mutex<Quest>>;
pub type QuestResult<T> = (SyncQuest, JoinHandle<T>);

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

pub struct Quest {
    pub id: QuestId,
    pub description: String,
    pub detail: Option<String>,
    pub sub_quests: Vec<SyncQuest>,
    pub progress: Option<Progress>,
    pub state: State,
}

pub struct Progress {
    pub current: u64,
    pub total: u64,
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

    pub async fn sub_quest_progress(&self) -> Progress {
        let current = futures::stream::iter(self.sub_quests.iter())
            .filter(|sub_quest| async { sub_quest.lock().await.state.is_finished() })
            .count()
            .await;
        Progress {
            current: current as u64,
            total: self.sub_quests.len() as u64,
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
            } else if let Some(Progress { current, total }) = &quest.progress {
                format!(
                    "{indent}{}: {}{} {current}/{total}\n",
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
