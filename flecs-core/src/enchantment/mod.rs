use crate::enchantment::floxy::Floxy;
use crate::enchantment::quest_master::QuestMaster;
use std::fmt::Display;
use std::sync::Arc;

pub mod floxy;
pub mod quest_master;

pub trait Enchantment: Send + Sync + Display {}

pub struct Enchantments {
    pub floxy: Arc<dyn Floxy>,
    pub quest_master: QuestMaster,
}

impl Clone for Enchantments {
    fn clone(&self) -> Self {
        Self {
            floxy: self.floxy.clone(),
            quest_master: self.quest_master.clone(),
        }
    }
}

#[cfg(test)]
impl Enchantments {
    pub fn test_instance() -> Enchantments {
        Self {
            floxy: Arc::new(floxy::MockFloxy::new()),
            quest_master: Default::default(),
        }
    }
}
