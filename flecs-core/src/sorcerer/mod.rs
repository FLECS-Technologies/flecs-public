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
use crate::sorcerer::systemus::{Systemus, SystemusImpl};
use std::sync::Arc;

pub type Sorcerers = SorcerersTemplate<
    AppraiserImpl,
    AuthmancerImpl,
    InstanciusImpl,
    LicensoImpl,
    MageQuesterImpl,
    ManifestoImpl,
    SystemusImpl,
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
            systemus: Default::default(),
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
    SYS: Systemus + ?Sized,
> {
    pub app_raiser: Arc<APP>,
    pub authmancer: Arc<AUTH>,
    pub instancius: Arc<I>,
    pub licenso: Arc<L>,
    pub mage_quester: Arc<Q>,
    pub manifesto: Arc<M>,
    pub systemus: Arc<SYS>,
}

impl<
        APP: AppRaiser + ?Sized,
        AUTH: Authmancer + ?Sized,
        I: Instancius + ?Sized,
        L: Licenso + ?Sized,
        Q: MageQuester + ?Sized,
        M: Manifesto + ?Sized,
        SYS: Systemus + ?Sized,
    > Clone for SorcerersTemplate<APP, AUTH, I, L, Q, M, SYS>
{
    fn clone(&self) -> Self {
        Self {
            app_raiser: self.app_raiser.clone(),
            authmancer: self.authmancer.clone(),
            instancius: self.instancius.clone(),
            licenso: self.licenso.clone(),
            mage_quester: self.mage_quester.clone(),
            manifesto: self.manifesto.clone(),
            systemus: self.systemus.clone(),
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
    crate::sorcerer::systemus::MockSystemus,
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
            systemus: Arc::new(crate::sorcerer::systemus::MockSystemus::default()),
        }
    }
}
