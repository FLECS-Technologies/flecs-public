use crate::lore::ProviderLoreRef;
use crate::vault::pouch::Pouch;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const PROVIDERS_FILE_NAME: &str = "providers.json";

pub type ProviderId = crate::jeweler::gem::instance::InstanceId;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CoreProviders {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<ProviderId>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Gems {
    pub core_providers: CoreProviders,
    pub default_providers: HashMap<String, ProviderId>,
}

pub struct ProviderPouch {
    lore: ProviderLoreRef,
    providers: Gems,
}

impl Pouch for ProviderPouch {
    type Gems = Gems;

    fn gems(&self) -> &Self::Gems {
        &self.providers
    }

    fn gems_mut(&mut self) -> &mut Self::Gems {
        &mut self.providers
    }
}

impl ProviderPouch {
    fn base_path(&self) -> &Path {
        &self.lore.as_ref().as_ref().base_path
    }

    fn providers_file_path(&self) -> PathBuf {
        self.base_path().join(PROVIDERS_FILE_NAME)
    }

    pub(in super::super) fn close(&mut self) -> crate::vault::Result<()> {
        let path = self.providers_file_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.providers)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub(in super::super) fn open(&mut self) -> crate::vault::Result<()> {
        self.providers = Gems::default();
        self.providers = serde_json::from_reader(std::fs::File::open(self.providers_file_path())?)?;
        Ok(())
    }
}

impl ProviderPouch {
    pub fn new(lore: ProviderLoreRef) -> Self {
        Self {
            lore,
            providers: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::lore;
    use crate::relic::var::test::MockVarReader;
    use std::sync::Arc;
    use testdir::testdir;

    pub fn test_provider_pouch() -> ProviderPouch {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        ProviderPouch {
            lore,
            providers: Default::default(),
        }
    }
}
