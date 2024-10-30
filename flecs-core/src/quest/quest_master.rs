use crate::quest::{QuestId, SyncQuest};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
pub enum DeleteQuestError {
    Unknown,
    StillRunning,
}

#[derive(Default)]
pub struct QuestMaster {
    quests: HashMap<QuestId, SyncQuest>,
}

impl QuestMaster {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quest::{Quest, State};
    use std::sync::Arc;
    use tokio::sync::Mutex;
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
