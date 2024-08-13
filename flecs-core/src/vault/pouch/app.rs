use crate::vault::pouch::{AppKey, Pouch, VaultPouch};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub enum AppStatus {
    #[serde(rename = "not installed")]
    NotInstalled,
    #[serde(rename = "manifest downloaded")]
    ManifestDownloaded,
    #[serde(rename = "token acquired")]
    TokenAcquired,
    #[serde(rename = "image downloaded")]
    ImageDownloaded,
    #[serde(rename = "installed")]
    Installed,
    #[serde(rename = "removed")]
    Removed,
    #[serde(rename = "purged")]
    Purged,
    #[serde(rename = "orphaned")]
    Orphaned,
}

// TODO: Implement version handling
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct App {
    app_key: AppKey,
    status: AppStatus,
    desired: AppStatus,
    installed_size: i64,
}

pub struct AppPouch {
    path: PathBuf,
    apps: HashMap<AppKey, App>,
}

impl Pouch for AppPouch {
    type Gems = HashMap<AppKey, App>;

    fn gems(&self) -> &Self::Gems {
        &self.apps
    }

    fn gems_mut(&mut self) -> &mut Self::Gems {
        &mut self.apps
    }
}

impl VaultPouch for AppPouch {
    fn close(&mut self) -> crate::vault::Result<()> {
        let content: Vec<_> = self.apps.values().collect();
        let content = serde_json::to_string(&content)?;
        fs::write(self.path.join("apps.json"), content)?;
        Ok(())
    }

    fn open(&mut self) -> crate::vault::Result<()> {
        let session_file = fs::read_to_string(self.path.join("apps.json"))?;
        let apps: Vec<App> = serde_json::from_str(&session_file)?;
        self.apps = apps
            .into_iter()
            .map(|app| (app.app_key.clone(), app))
            .collect();
        Ok(())
    }
}

impl AppPouch {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            apps: HashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO: Test AppPouch
}
