use crate::enchantment::floxy::Floxy;
use std::fmt::Display;
use std::sync::Arc;

pub mod floxy;

pub trait Enchantment: Send + Sync + Display {}

pub struct Enchantments<F: Floxy> {
    pub floxy: Arc<F>,
}

#[cfg(test)]
impl Enchantments<floxy::MockFloxy> {
    pub fn test_instance() -> Enchantments<floxy::MockFloxy> {
        Self {
            floxy: Arc::new(floxy::MockFloxy::new()),
        }
    }
}
