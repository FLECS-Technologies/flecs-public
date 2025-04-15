use crate::quest::{Quest, QuestId, QuestResult, State, SyncQuest, finish_quest};
use anyhow::Result;
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{Receiver, Sender, channel, error::TrySendError};
use tokio::sync::oneshot::error::RecvError;
use tokio::task::JoinError;
use tracing::{debug, error, info, warn};

#[derive(Debug, Eq, PartialEq)]
pub enum DeleteQuestError {
    Unknown,
    StillRunning,
}

pub enum ControlSignal {
    ShutdownWith {
        quest: SyncQuest,
        future: BoxFuture<'static, Result<QuestResult>>,
        result_sender: tokio::sync::oneshot::Sender<Result<Result<QuestResult>, JoinError>>,
    },
}

impl Display for ControlSignal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::ShutdownWith { .. } => "ShutdownWith",
            }
        )
    }
}

type ScheduledQuest = (
    SyncQuest,
    BoxFuture<'static, Result<QuestResult>>,
    std::time::Instant,
);

type ScheduledControlSignal = (ControlSignal, std::time::Instant);
pub type QuestMaster = Arc<tokio::sync::Mutex<QuestMasterInner>>;

pub struct QuestMasterInner {
    quests: HashMap<QuestId, SyncQuest>,
    schedule_channel: Sender<ScheduledQuest>,
    control_channel: Sender<ScheduledControlSignal>,
}

impl Default for QuestMasterInner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ShutdownError {
    #[error("Failed to send shutdown request: {0}")]
    SendError(#[from] SendError<ScheduledControlSignal>),
    #[error("Failed to receive shutdown result: {0}")]
    ReceiveError(#[from] RecvError),
    #[error("Failed to execute shutdown: {0}")]
    JoinError(#[from] JoinError),
}

impl QuestMasterInner {
    pub fn new() -> Self {
        let (quest_sender, mut quest_receiver) = channel::<(
            SyncQuest,
            BoxFuture<'static, Result<QuestResult>>,
            std::time::Instant,
        )>(1000);
        let (control_sender, mut control_receiver) = channel::<ScheduledControlSignal>(100);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    control_message = control_receiver.recv() => {
                        let Some((signal, time)) = control_message else {
                            panic!("Channel for control signals was shutdown.");
                        };
                        if Self::handle_control_signal(signal, time, &mut quest_receiver).await {
                            break
                        }
                    },
                    scheduled_quest = quest_receiver.recv() => {
                        if let Some((quest, future, scheduled_time)) = scheduled_quest {
                            _ = Self::process_quest(quest, future, scheduled_time).await;
                        }
                    }
                }
            }
            info!("QuestMaster stopped processing quests and control signals.");
        });
        Self {
            quests: HashMap::new(),
            schedule_channel: quest_sender,
            control_channel: control_sender,
        }
    }

    /// Returns true if the scheduler loop should be stopped
    async fn handle_control_signal(
        signal: ControlSignal,
        time: std::time::Instant,
        quest_receiver: &mut Receiver<ScheduledQuest>,
    ) -> bool {
        info!(
            "Received signal {signal} after {:?}",
            std::time::Instant::now() - time
        );
        match signal {
            ControlSignal::ShutdownWith {
                quest,
                future,
                result_sender,
            } => {
                quest_receiver.close();
                let result = Self::process_quest(quest, future, time).await;
                if result_sender.send(result).is_err() {
                    error!("Failed to send result of shutdown back.")
                };
                true
            }
        }
    }

    pub async fn shutdown_with<F, Fut>(
        &mut self,
        f: F,
    ) -> Result<Result<QuestResult>, ShutdownError>
    where
        F: FnOnce(SyncQuest) -> Fut + Send + 'static,
        Fut: Future<Output = Result<QuestResult>> + Send + 'static,
    {
        let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
        let quest = Quest::new_synced("Shutting down QuestMaster".to_string());
        self.control_channel
            .send((
                ControlSignal::ShutdownWith {
                    future: Box::pin(f(quest.clone())),
                    result_sender,
                    quest,
                },
                std::time::Instant::now(),
            ))
            .await?;
        Ok(result_receiver.await??)
    }

    async fn process_quest(
        quest: SyncQuest,
        future: BoxFuture<'static, Result<QuestResult>>,
        scheduled_time: std::time::Instant,
    ) -> Result<Result<QuestResult>, JoinError> {
        let start = {
            let mut quest = quest.lock().await;
            quest.state = State::Ongoing;
            info!(
                "Quest '{}' with id {} started. It waited for {:#?} in queue.",
                quest.id.0,
                quest.description,
                std::time::Instant::now() - scheduled_time
            );
            std::time::Instant::now()
        };

        match tokio::spawn(future).await {
            Ok(result) => match finish_quest(&quest, result).await {
                Err(e) => {
                    let quest = quest.lock().await;
                    warn!(
                        "Quest '{}' with id {} failed after {:#?}: {e}",
                        quest.description,
                        quest.id.0,
                        std::time::Instant::now() - start
                    );
                    Ok(Err(e))
                }
                Ok(result) => {
                    let mut quest = quest.lock().await;
                    if quest.result == QuestResult::None {
                        quest.result = result.clone();
                    }
                    info!(
                        "Quest '{}' with id {} succeeded after {:#?}.",
                        quest.description,
                        quest.id.0,
                        std::time::Instant::now() - start
                    );
                    Ok(Ok(result))
                }
            },
            Err(e) => {
                let mut quest = quest.lock().await;
                error!(
                    "Quest '{}' with id {} caused a panic after {:#?}: {e}",
                    quest.description,
                    quest.id.0,
                    std::time::Instant::now() - start
                );
                quest.state = State::Failed;
                Err(e)
            }
        }
    }

    pub fn query_quest(&self, quest_id: QuestId) -> Option<SyncQuest> {
        self.quests.get(&quest_id).map(Clone::clone)
    }

    pub fn get_quests(&self) -> Vec<SyncQuest> {
        self.quests.values().cloned().collect()
    }

    pub async fn delete_quest(&mut self, quest_id: QuestId) -> Result<SyncQuest, DeleteQuestError> {
        if let Entry::Occupied(quest) = self.quests.entry(quest_id) {
            if quest.get().lock().await.state.is_finished() {
                Ok(quest.remove())
            } else {
                Err(DeleteQuestError::StillRunning)
            }
        } else {
            Err(DeleteQuestError::Unknown)
        }
    }

    pub async fn schedule_quest_with_result<F, Fut>(
        &mut self,
        description: String,
        f: F,
    ) -> Result<(QuestId, SyncQuest)>
    where
        F: FnOnce(SyncQuest) -> Fut,
        Fut: Future<Output = Result<QuestResult>> + Send + 'static,
    {
        let quest = Quest::new_synced(description.clone());
        let quest_id = quest.lock().await.id;

        match self.schedule_channel.try_send((
            quest.clone(),
            Box::pin(f(quest.clone())),
            std::time::Instant::now(),
        )) {
            Ok(()) => {
                self.quests.insert(quest_id, quest.clone());
                debug!("Quest '{description}' scheduled with id {}", quest_id.0);
                Ok((quest_id, quest))
            }
            Err(TrySendError::Full(_)) => anyhow::bail!(
                "Could not schedule quest {}, ({}), scheduler is full",
                quest_id.0,
                description
            ),
            Err(TrySendError::Closed(_)) => anyhow::bail!(
                "Could not schedule quest {}, ({}), scheduler was shutdown",
                quest_id.0,
                description
            ),
        }
    }

    pub async fn schedule_quest<F, Fut>(
        &mut self,
        description: String,
        f: F,
    ) -> Result<(QuestId, SyncQuest)>
    where
        F: FnOnce(SyncQuest) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.schedule_quest_with_result(description, |quest| async move {
            f(quest).await?;
            Ok(QuestResult::None)
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quest::{State, create_test_quest};
    use crate::vault::pouch::instance::InstanceId;
    use ntest::timeout;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_query_quest() {
        let mut master = QuestMasterInner::default();

        // Add quests to master
        master.quests.insert(QuestId(1), create_test_quest(1));
        master.quests.insert(QuestId(2), create_test_quest(2));
        master.quests.insert(QuestId(3), create_test_quest(3));
        master.quests.insert(QuestId(5), create_test_quest(5));

        // Query existing quests
        assert_eq!(master.query_quest(QuestId(1)).unwrap().lock().await.id.0, 1);
        assert_eq!(master.query_quest(QuestId(2)).unwrap().lock().await.id.0, 2);
        assert_eq!(master.query_quest(QuestId(3)).unwrap().lock().await.id.0, 3);
        assert_eq!(master.query_quest(QuestId(5)).unwrap().lock().await.id.0, 5);

        // Query non-existent quest
        assert!(master.query_quest(QuestId(4)).is_none());
        assert!(master.query_quest(QuestId(0)).is_none());
    }

    async fn test_quest_fn(quest: SyncQuest) -> Result<()> {
        quest.lock().await.state = State::Ongoing;
        quest.lock().await.detail = Some("Test quest fn done".to_string());
        quest.lock().await.state = State::Success;
        Ok(())
    }

    #[tokio::test]
    async fn test_schedule_quest() {
        let mut master = QuestMasterInner::default();

        let (quest_id, _) = master
            .schedule_quest("Test Quest Description".to_string(), test_quest_fn)
            .await
            .unwrap();
        assert!(master.quests.contains_key(&quest_id));
    }

    #[tokio::test]
    #[timeout(10000)]
    async fn test_schedule_quest_start() {
        let mut master = QuestMasterInner::default();
        let (tx, rx) = tokio::sync::oneshot::channel::<u64>();

        let _ = master
            .schedule_quest("Test Quest Description".to_string(), |_| async move {
                tx.send(1234).unwrap();
                Ok(())
            })
            .await
            .unwrap();
        assert_eq!(rx.await.unwrap(), 1234);
    }

    #[tokio::test]
    #[timeout(10000)]
    async fn test_schedule_quest_success() {
        let mut master = QuestMasterInner::default();
        let (tx, rx) = tokio::sync::oneshot::channel::<u64>();

        let (_, quest) = master
            .schedule_quest("Test Quest Description".to_string(), |_| async move {
                tx.send(1234).unwrap();
                Ok(())
            })
            .await
            .unwrap();
        assert_eq!(rx.await.unwrap(), 1234);
        sleep(Duration::from_millis(10)).await;
        assert_eq!(quest.lock().await.state, State::Success);
    }

    #[tokio::test]
    #[timeout(10000)]
    async fn test_schedule_quest_with_result_success() {
        let mut master = QuestMasterInner::default();
        let (tx, rx) = tokio::sync::oneshot::channel::<u64>();

        let (_, quest) = master
            .schedule_quest_with_result("Test Quest Description".to_string(), |_| async move {
                tx.send(1234).unwrap();
                Ok(QuestResult::ExportId("TestExportId".to_string()))
            })
            .await
            .unwrap();
        assert_eq!(rx.await.unwrap(), 1234);
        sleep(Duration::from_millis(10)).await;
        assert_eq!(quest.lock().await.state, State::Success);
        assert_eq!(
            quest.lock().await.result,
            QuestResult::ExportId("TestExportId".to_string())
        );
    }

    #[tokio::test]
    #[timeout(10000)]
    async fn test_schedule_quest_with_result_success_no_overwrite() {
        let mut master = QuestMasterInner::default();
        let (tx, rx) = tokio::sync::oneshot::channel::<u64>();

        let (_, quest) = master
            .schedule_quest_with_result("Test Quest Description".to_string(), |quest| async move {
                tx.send(1234).unwrap();
                quest.lock().await.result = QuestResult::ExportId("RealExportId".to_string());
                Ok(QuestResult::ExportId("FakeExportId".to_string()))
            })
            .await
            .unwrap();
        assert_eq!(rx.await.unwrap(), 1234);
        sleep(Duration::from_millis(10)).await;
        assert_eq!(quest.lock().await.state, State::Success);
        assert_eq!(
            quest.lock().await.result,
            QuestResult::ExportId("RealExportId".to_string())
        );
    }

    #[tokio::test]
    #[timeout(10000)]
    async fn test_schedule_quest_failure() {
        let mut master = QuestMasterInner::default();
        let (tx, rx) = tokio::sync::oneshot::channel::<u64>();

        let (_, quest) = master
            .schedule_quest("Test Quest Description".to_string(), |_| async move {
                tx.send(1234).unwrap();
                anyhow::bail!("")
            })
            .await
            .unwrap();
        assert_eq!(rx.await.unwrap(), 1234);
        sleep(Duration::from_millis(10)).await;
        assert_eq!(quest.lock().await.state, State::Failed);
    }

    #[tokio::test]
    async fn test_get_quests() {
        let mut master = QuestMasterInner::default();

        // Empty case
        assert!(master.get_quests().is_empty());

        // With quests
        let quest1 = create_test_quest(1);
        let quest2 = create_test_quest(2);
        master.quests.insert(QuestId(1), quest1.clone());
        master.quests.insert(QuestId(2), quest2.clone());

        let quests = master.get_quests();
        assert_eq!(quests.len(), 2);
        let (mut one, mut two) = (false, false);
        for quest in quests {
            match quest.lock().await.id.0 {
                1 => one = true,
                2 => two = true,
                _ => panic!(),
            }
        }
        assert!(one);
        assert!(two);
    }

    #[tokio::test]
    async fn test_delete_quest() {
        let mut master = QuestMasterInner::default();
        let quest = create_test_quest(1);

        // Add quest to master
        master.quests.insert(QuestId(1), quest.clone());

        // Delete ongoing quest
        match master.delete_quest(QuestId(1)).await {
            Err(DeleteQuestError::StillRunning) => {}
            _ => panic!("Expected error {:?}", DeleteQuestError::StillRunning),
        }

        master.quests.get(&QuestId(1)).unwrap().lock().await.state = State::Success;

        // Delete finished quest
        let deleted = master.delete_quest(QuestId(1)).await.unwrap();
        assert_eq!(deleted.lock().await.id.0, 1);

        assert!(!master.quests.contains_key(&QuestId(1)));

        // Delete missing quest
        match master.delete_quest(QuestId(1)).await {
            Err(DeleteQuestError::Unknown) => {}
            _ => panic!("Expected error {:?}", DeleteQuestError::Unknown),
        }
    }

    #[tokio::test]
    #[timeout(10000)]
    async fn handle_control_signal_shutdown_with() {
        let quest = create_test_quest(1);
        let (_quest_sender, mut quest_receiver) = channel::<(
            SyncQuest,
            BoxFuture<'static, Result<QuestResult>>,
            std::time::Instant,
        )>(1);
        const EXPECTED_INSTANCE: InstanceId = InstanceId::new(200);
        let (expected_result_sender, expected_result_receiver) = tokio::sync::oneshot::channel();
        let f = |_quest| async {
            expected_result_sender.send(20).unwrap();
            Ok(QuestResult::InstanceId(EXPECTED_INSTANCE))
        };
        let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
        let signal = ControlSignal::ShutdownWith {
            future: Box::pin(f(quest.clone())),
            result_sender,
            quest,
        };
        assert!(
            QuestMasterInner::handle_control_signal(
                signal,
                std::time::Instant::now(),
                &mut quest_receiver
            )
            .await
        );
        assert!(quest_receiver.is_closed());
        assert_eq!(expected_result_receiver.await, Ok(20));
        assert!(matches!(
            result_receiver.await,
            Ok(Ok(Ok(QuestResult::InstanceId(EXPECTED_INSTANCE))))
        ));
    }

    #[tokio::test]
    #[timeout(10000)]
    async fn shutdown_with_ok() {
        let mut quest_master = QuestMasterInner::default();
        const EXPECTED_STRING: &str = "TestString";
        let (expected_result_sender, expected_result_receiver) = tokio::sync::oneshot::channel();
        let f = |_quest| async {
            expected_result_sender.send(20).unwrap();
            Ok(QuestResult::ExportId(EXPECTED_STRING.to_string()))
        };
        assert_eq!(
            quest_master.shutdown_with(f).await.unwrap().unwrap(),
            QuestResult::ExportId(EXPECTED_STRING.to_string())
        );
        assert_eq!(expected_result_receiver.await, Ok(20));
        let f = |_quest| async { panic!("Closure should not be called") };
        assert!(quest_master.shutdown_with(f).await.is_err());
    }

    #[tokio::test]
    #[timeout(10000)]
    async fn shutdown_with_err_last_quest() {
        let mut quest_master = QuestMasterInner::default();
        let (expected_result_sender, expected_result_receiver) = tokio::sync::oneshot::channel();
        let f = |_quest| async {
            expected_result_sender.send(20).unwrap();
            Err(anyhow::anyhow!("TestError"))
        };
        assert!(quest_master.shutdown_with(f).await.unwrap().is_err());
        assert_eq!(expected_result_receiver.await, Ok(20));
    }
}
