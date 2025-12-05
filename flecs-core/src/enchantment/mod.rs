use crate::enchantment::quest_master::QuestMaster;
use std::fmt::Display;

pub mod quest_master;

pub trait Enchantment: Send + Sync + Display {}

pub struct Enchantments {
    pub quest_master: QuestMaster,
}

impl Clone for Enchantments {
    fn clone(&self) -> Self {
        Self {
            quest_master: self.quest_master.clone(),
        }
    }
}

#[cfg(test)]
impl Enchantments {
    pub fn test_instance() -> Enchantments {
        Self {
            quest_master: Default::default(),
        }
    }
}
