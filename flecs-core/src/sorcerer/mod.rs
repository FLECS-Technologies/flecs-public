pub mod appraiser;
pub mod authmancer;
pub mod instancius;
pub mod licenso;
pub mod magequester;
pub mod manifesto;
mod spell;
pub mod systemus;

pub use super::{Error, Result};
use crate::sorcerer::appraiser::{AppRaiser, AppraiserImpl};
use std::sync::Arc;

pub type Sorcerers = SorcerersTemplate<AppraiserImpl>;
pub trait Sorcerer: Send + Sync {}
impl Default for Sorcerers {
    fn default() -> Self {
        Self {
            app_raiser: Default::default(),
        }
    }
}

pub struct SorcerersTemplate<A: AppRaiser + ?Sized> {
    pub app_raiser: Arc<A>,
}

impl<A: AppRaiser + ?Sized> Clone for SorcerersTemplate<A> {
    fn clone(&self) -> Self {
        Self {
            app_raiser: self.app_raiser.clone(),
        }
    }
}

#[cfg(test)]
pub type MockSorcerers = SorcerersTemplate<crate::sorcerer::appraiser::MockAppRaiser>;

#[cfg(test)]
impl Default for MockSorcerers {
    fn default() -> Self {
        Self {
            app_raiser: Arc::new(crate::sorcerer::appraiser::MockAppRaiser::default()),
        }
    }
}
