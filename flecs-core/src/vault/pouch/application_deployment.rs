use crate::lore::MargoLoreRef;
use crate::vault::Error;
use crate::vault::pouch::Pouch;
use margo_types::application_deployment::ApplicationDeployment;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

const APPLICATION_DEPLOYMENT_FILE_ENDING: &str = "yaml";
pub type Id = String;
pub type Gems = HashMap<Id, ApplicationDeployment>;

pub struct ApplicationDeploymentPouch {
    lore: MargoLoreRef,
    application_deployments: Gems,
}

impl Pouch for ApplicationDeploymentPouch {
    type Gems = Gems;

    fn gems(&self) -> &Self::Gems {
        &self.application_deployments
    }

    fn gems_mut(&mut self) -> &mut Self::Gems {
        &mut self.application_deployments
    }
}

impl ApplicationDeploymentPouch {
    fn base_path(&self) -> &Path {
        &self.lore.as_ref().as_ref().base_path
    }

    pub(in super::super) fn close(&mut self) -> crate::vault::Result<()> {
        let mut errors: Vec<String> = Vec::new();
        let base_path = self.base_path().to_path_buf();
        fs::create_dir_all(&base_path)?;
        for application_deployment in self.application_deployments.values() {
            if let Err(e) = Self::write_application_deployment(&base_path, application_deployment) {
                errors.push(e.to_string())
            }
        }
        Error::from_error_list((), errors)
    }

    pub(in super::super) fn open(&mut self) -> crate::vault::Result<()> {
        let path = self
            .base_path()
            .join(format!("**/*.{APPLICATION_DEPLOYMENT_FILE_ENDING}"));
        let path = path.to_string_lossy();
        self.application_deployments.clear();
        for entry in glob::glob(path.as_ref())?.flatten() {
            match Self::read_application_deployment(entry.as_path()) {
                Err(e) => {
                    warn!("Could not read application deployment from {entry:?}: {e}");
                }
                Ok(application_deployment) => {
                    self.application_deployments.insert(
                        application_deployment.metadata.annotations.id.clone(),
                        application_deployment,
                    );
                    debug!("Successful read application deployment from {entry:?}");
                }
            }
        }
        Ok(())
    }
}

impl ApplicationDeploymentPouch {
    pub fn new(lore: MargoLoreRef) -> Self {
        Self {
            lore,
            application_deployments: HashMap::default(),
        }
    }

    fn read_application_deployment(path: &Path) -> crate::vault::Result<ApplicationDeployment> {
        let file = std::fs::File::open(path)?;
        let application_deployment: ApplicationDeployment = serde_norway::from_reader(file)?;
        Ok(application_deployment)
    }

    fn write_application_deployment(
        base_path: &Path,
        application_deployment: &ApplicationDeployment,
    ) -> crate::vault::Result<()> {
        let path = base_path.join(format!(
            "{}.{APPLICATION_DEPLOYMENT_FILE_ENDING}",
            application_deployment.metadata.annotations.id
        ));
        let file = std::fs::File::create(path)?;
        serde_norway::to_writer(file, application_deployment)?;
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::lore;
    use crate::relic::var::test::MockVarReader;
    use crate::vault::pouch::application_deployment::ApplicationDeploymentPouch;
    use std::sync::Arc;
    use testdir::testdir;

    pub fn test_application_deployment_pouch() -> ApplicationDeploymentPouch {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        ApplicationDeploymentPouch {
            lore,
            application_deployments: Default::default(),
        }
    }
}
