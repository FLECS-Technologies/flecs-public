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
use crate::sorcerer::authmancer::{Authmancer, AuthmancerImpl};
use crate::sorcerer::instancius::{Instancius, InstanciusImpl};
use std::sync::Arc;

pub type Sorcerers = SorcerersTemplate<AppraiserImpl, AuthmancerImpl, InstanciusImpl>;
pub trait Sorcerer: Send + Sync {}
impl Default for Sorcerers {
    fn default() -> Self {
        Self {
            app_raiser: Default::default(),
            authmancer: Default::default(),
            instancius: Default::default(),
        }
    }
}

pub struct SorcerersTemplate<
    APP: AppRaiser + ?Sized,
    AUTH: Authmancer + ?Sized,
    I: Instancius + ?Sized,
> {
    pub app_raiser: Arc<APP>,
    pub authmancer: Arc<AUTH>,
    pub instancius: Arc<I>,
}

impl<APP: AppRaiser + ?Sized, AUTH: Authmancer + ?Sized, I: Instancius + ?Sized> Clone
    for SorcerersTemplate<APP, AUTH, I>
{
    fn clone(&self) -> Self {
        Self {
            app_raiser: self.app_raiser.clone(),
            authmancer: self.authmancer.clone(),
            instancius: self.instancius.clone(),
        }
    }
}

#[cfg(test)]
pub type MockSorcerers = SorcerersTemplate<
    crate::sorcerer::appraiser::MockAppRaiser,
    crate::sorcerer::authmancer::MockAuthmancer,
    crate::sorcerer::instancius::MockInstancius,
>;

#[cfg(test)]
impl Default for MockSorcerers {
    fn default() -> Self {
        Self {
            app_raiser: Arc::new(crate::sorcerer::appraiser::MockAppRaiser::default()),
            authmancer: Arc::new(crate::sorcerer::authmancer::MockAuthmancer::default()),
            instancius: Arc::new(crate::sorcerer::instancius::MockInstancius::default()),
        }
    }
}
