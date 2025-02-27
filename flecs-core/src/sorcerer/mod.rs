pub mod appraiser;
pub mod authmancer;
pub mod instancius;
pub mod licenso;
pub mod mage_quester;
pub mod manifesto;
mod spell;
pub mod systemus;

pub use super::{Error, Result};
use crate::sorcerer::appraiser::{AppRaiser, AppraiserImpl};
use crate::sorcerer::authmancer::{Authmancer, AuthmancerImpl};
use crate::sorcerer::instancius::{Instancius, InstanciusImpl};
use crate::sorcerer::licenso::{Licenso, LicensoImpl};
use crate::sorcerer::mage_quester::{MageQuester, MageQuesterImpl};
use crate::sorcerer::manifesto::{Manifesto, ManifestoImpl};
use std::sync::Arc;

pub type Sorcerers = SorcerersTemplate<
    AppraiserImpl,
    AuthmancerImpl,
    InstanciusImpl,
    LicensoImpl,
    MageQuesterImpl,
    ManifestoImpl,
>;
pub trait Sorcerer: Send + Sync {}
impl Default for Sorcerers {
    fn default() -> Self {
        Self {
            app_raiser: Default::default(),
            authmancer: Default::default(),
            instancius: Default::default(),
            licenso: Default::default(),
            mage_quester: Default::default(),
            manifesto: Default::default(),
        }
    }
}

pub struct SorcerersTemplate<
    APP: AppRaiser + ?Sized,
    AUTH: Authmancer + ?Sized,
    I: Instancius + ?Sized,
    L: Licenso + ?Sized,
    Q: MageQuester + ?Sized,
    M: Manifesto + ?Sized,
> {
    pub app_raiser: Arc<APP>,
    pub authmancer: Arc<AUTH>,
    pub instancius: Arc<I>,
    pub licenso: Arc<L>,
    pub mage_quester: Arc<Q>,
    pub manifesto: Arc<M>,
}

impl<
        APP: AppRaiser + ?Sized,
        AUTH: Authmancer + ?Sized,
        I: Instancius + ?Sized,
        L: Licenso + ?Sized,
        Q: MageQuester + ?Sized,
        M: Manifesto + ?Sized,
    > Clone for SorcerersTemplate<APP, AUTH, I, L, Q, M>
{
    fn clone(&self) -> Self {
        Self {
            app_raiser: self.app_raiser.clone(),
            authmancer: self.authmancer.clone(),
            instancius: self.instancius.clone(),
            licenso: self.licenso.clone(),
            mage_quester: self.mage_quester.clone(),
            manifesto: self.manifesto.clone(),
        }
    }
}

#[cfg(test)]
pub type MockSorcerers = SorcerersTemplate<
    crate::sorcerer::appraiser::MockAppRaiser,
    crate::sorcerer::authmancer::MockAuthmancer,
    crate::sorcerer::instancius::MockInstancius,
    crate::sorcerer::licenso::MockLicenso,
    crate::sorcerer::mage_quester::MockMageQuester,
    crate::sorcerer::manifesto::MockManifesto,
>;

#[cfg(test)]
impl Default for MockSorcerers {
    fn default() -> Self {
        Self {
            app_raiser: Arc::new(crate::sorcerer::appraiser::MockAppRaiser::default()),
            authmancer: Arc::new(crate::sorcerer::authmancer::MockAuthmancer::default()),
            instancius: Arc::new(crate::sorcerer::instancius::MockInstancius::default()),
            licenso: Arc::new(crate::sorcerer::licenso::MockLicenso::default()),
            mage_quester: Arc::new(crate::sorcerer::mage_quester::MockMageQuester::default()),
            manifesto: Arc::new(crate::sorcerer::manifesto::MockManifesto::default()),
        }
    }
}
