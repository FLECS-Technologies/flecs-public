pub mod appraiser;
pub mod authmancer;
pub mod deploymento;
pub mod exportius;
pub mod importius;
pub mod instancius;
pub mod licenso;
pub mod mage_quester;
pub mod manifesto;
pub mod providius;
mod spell;
pub mod systemus;

pub use super::{Error, Result};
use crate::sorcerer::appraiser::{AppRaiser, AppraiserImpl};
use crate::sorcerer::authmancer::{Authmancer, AuthmancerImpl};
use crate::sorcerer::deploymento::{Deploymento, DeploymentoImpl};
use crate::sorcerer::exportius::{Exportius, ExportiusImpl};
use crate::sorcerer::importius::{Importius, ImportiusImpl};
use crate::sorcerer::instancius::{Instancius, InstanciusImpl};
use crate::sorcerer::licenso::{Licenso, LicensoImpl};
use crate::sorcerer::mage_quester::{MageQuester, MageQuesterImpl};
use crate::sorcerer::manifesto::{Manifesto, ManifestoImpl};
use crate::sorcerer::providius::Providius;
use crate::sorcerer::providius::providius_impl::ProvidiusImpl;
use crate::sorcerer::systemus::{Systemus, SystemusImpl};
use std::sync::Arc;

pub type FlecsSorcerers = Sorcerers<
    AppraiserImpl,
    AuthmancerImpl,
    InstanciusImpl,
    LicensoImpl,
    MageQuesterImpl,
    ManifestoImpl,
    SystemusImpl,
    DeploymentoImpl,
    ExportiusImpl,
    ImportiusImpl,
>;
pub trait Sorcerer: Send + Sync {}
impl Default for FlecsSorcerers {
    fn default() -> Self {
        Self {
            app_raiser: Default::default(),
            authmancer: Default::default(),
            instancius: Default::default(),
            licenso: Default::default(),
            mage_quester: Default::default(),
            manifesto: Default::default(),
            systemus: Default::default(),
            deploymento: Default::default(),
            exportius: Default::default(),
            importius: Default::default(),
            providius: Arc::new(ProvidiusImpl),
        }
    }
}

pub struct Sorcerers<
    APP: AppRaiser + ?Sized,
    AUTH: Authmancer + ?Sized,
    I: Instancius + ?Sized,
    L: Licenso + ?Sized,
    Q: MageQuester + ?Sized,
    M: Manifesto + ?Sized,
    SYS: Systemus + ?Sized,
    D: Deploymento + ?Sized,
    E: Exportius + ?Sized,
    IMP: Importius + ?Sized,
> {
    pub app_raiser: Arc<APP>,
    pub authmancer: Arc<AUTH>,
    pub instancius: Arc<I>,
    pub licenso: Arc<L>,
    pub mage_quester: Arc<Q>,
    pub manifesto: Arc<M>,
    pub systemus: Arc<SYS>,
    pub deploymento: Arc<D>,
    pub exportius: Arc<E>,
    pub importius: Arc<IMP>,
    pub providius: Arc<dyn Providius>,
}

impl<
    APP: AppRaiser + ?Sized,
    AUTH: Authmancer + ?Sized,
    I: Instancius + ?Sized,
    L: Licenso + ?Sized,
    Q: MageQuester + ?Sized,
    M: Manifesto + ?Sized,
    SYS: Systemus + ?Sized,
    D: Deploymento + ?Sized,
    E: Exportius + ?Sized,
    IMP: Importius + ?Sized,
> Clone for Sorcerers<APP, AUTH, I, L, Q, M, SYS, D, E, IMP>
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
            deploymento: self.deploymento.clone(),
            exportius: self.exportius.clone(),
            importius: self.importius.clone(),
            providius: self.providius.clone(),
        }
    }
}

#[cfg(test)]
pub type MockSorcerers = Sorcerers<
    crate::sorcerer::appraiser::MockAppRaiser,
    crate::sorcerer::authmancer::MockAuthmancer,
    crate::sorcerer::instancius::MockInstancius,
    crate::sorcerer::licenso::MockLicenso,
    crate::sorcerer::mage_quester::MockMageQuester,
    crate::sorcerer::manifesto::MockManifesto,
    crate::sorcerer::systemus::MockSystemus,
    crate::sorcerer::deploymento::MockDeploymento,
    crate::sorcerer::exportius::MockExportius,
    crate::sorcerer::importius::MockImportius,
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
            deploymento: Arc::new(crate::sorcerer::deploymento::MockDeploymento::default()),
            exportius: Arc::new(crate::sorcerer::exportius::MockExportius::default()),
            importius: Arc::new(crate::sorcerer::importius::MockImportius::default()),
            providius: Arc::new(crate::sorcerer::providius::MockProvidius::default()),
        }
    }
}
