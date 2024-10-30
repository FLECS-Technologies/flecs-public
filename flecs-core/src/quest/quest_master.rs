pub use super::{Error, Result};
use crate::quest::{finish_quest, Quest, QuestId, State, SyncQuest};
use futures::future::BoxFuture;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::future::Future;
use tokio::sync::mpsc::{channel, error::TrySendError, Sender};

#[derive(Debug, Eq, PartialEq)]
pub enum DeleteQuestError {
    Unknown,
    StillRunning,
}

pub struct QuestMaster {
    quests: HashMap<QuestId, SyncQuest>,
    schedule_channel: Sender<(SyncQuest, BoxFuture<'static, Result<()>>)>,
}

impl Default for QuestMaster {
    fn default() -> Self {
        Self::new()
    }
}

impl QuestMaster {
    pub fn new() -> Self {
        let (tx, mut rx) = channel::<(SyncQuest, BoxFuture<'static, Result<()>>)>(1000);

        tokio::spawn(async move {
            while let Some((quest, future)) = rx.recv().await {
                quest.lock().await.state = State::Ongoing;
                match tokio::spawn(future).await {
                    Ok(result) => {
                        if let Err(e) = finish_quest(&quest, result).await {
                            eprintln!("Quest with id {} failed: {e}", quest.lock().await.id.0);
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Quest with id {} caused a panic: {e}",
                            quest.lock().await.id.0
                        );
                        quest.lock().await.state = State::Failed;
                    }
                }
            }
        });
        Self {
            quests: HashMap::new(),
            schedule_channel: tx,
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

    pub async fn schedule_quest<F, Fut>(
        &mut self,
        description: String,
        f: F,
    ) -> Result<(QuestId, SyncQuest)>
    where
        F: FnOnce(SyncQuest) -> Fut,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let quest = Quest::new_synced(description.clone());
        let quest_id = quest.lock().await.id;

        match self
            .schedule_channel
            .try_send((quest.clone(), Box::pin(f(quest.clone()))))
        {
            Ok(()) => {
                self.quests.insert(quest_id, quest.clone());
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quest::{Quest, State};
    use ntest::timeout;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Mutex;
    use tokio::time::sleep;

    fn create_test_quest(id: u64) -> SyncQuest {
        Arc::new(Mutex::new(Quest {
            id: QuestId(id),
            description: "Test Quest".to_string(),
            detail: None,
            sub_quests: vec![],
            progress: None,
            state: State::Ongoing,
        }))
    }
    #[tokio::test]
    async fn test_query_quest() {
        let mut master = QuestMaster::default();

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
        let mut master = QuestMaster::default();

        let (quest_id, _) = master
            .schedule_quest("Test Quest Description".to_string(), test_quest_fn)
            .await
            .unwrap();
        assert!(master.quests.contains_key(&quest_id));
    }

    #[tokio::test]
    #[timeout(10)]
    async fn test_schedule_quest_start() {
        let mut master = QuestMaster::default();
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
    #[timeout(20)]
    async fn test_schedule_quest_success() {
        let mut master = QuestMaster::default();
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
    #[timeout(20)]
    async fn test_schedule_quest_failure() {
        let mut master = QuestMaster::default();
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
        let mut master = QuestMaster::default();

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
        let mut master = QuestMaster::default();
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

        assert!(master.quests.get(&QuestId(1)).is_none());

        // Delete missing quest
        match master.delete_quest(QuestId(1)).await {
            Err(DeleteQuestError::Unknown) => {}
            _ => panic!("Expected error {:?}", DeleteQuestError::Unknown),
        }
    }
}