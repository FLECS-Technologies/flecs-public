use crate::vault::pouch::{AppKey, Pouch, VaultPouch};
use crate::vault::Error;
use flecs_app_manifest::AppManifest;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub struct ManifestPouch {
    path: PathBuf,
    manifests: HashMap<AppKey, AppManifest>,
}

impl Pouch for ManifestPouch {
    type Gems = HashMap<AppKey, AppManifest>;

    fn gems(&self) -> &Self::Gems {
        &self.manifests
    }

    fn gems_mut(&mut self) -> &mut Self::Gems {
        &mut self.manifests
    }
}

impl VaultPouch for ManifestPouch {
    fn close(&mut self) -> crate::vault::Result<()> {
        let mut errors: Vec<String> = Vec::new();
        for (key, manifest) in &self.manifests {
            let path = self.path.join(key.name.as_str()).join(key.version.as_str());
            if let Err(e) = fs::create_dir_all(&path) {
                errors.push(e.to_string());
                break;
            }
            match serde_json::to_string(&manifest.original) {
                Err(e) => errors.push(e.to_string()),
                Ok(content) => {
                    if let Err(e) = fs::write(path.join("manifest.json"), content) {
                        errors.push(e.to_string())
                    }
                }
            }
        }
        match errors.len() {
            0 => Ok(()),
            1 => Err(Error::Single(errors.into_iter().next().unwrap())),
            _ => Err(Error::Multiple(errors)),
        }
    }

    fn open(&mut self) -> crate::vault::Result<()> {
        let path = self.path.join("*/*/manifest.json");
        let path = path.to_str().ok_or(Error::Single(String::from("")))?;
        self.manifests.clear();
        for entry in glob::glob(path)?.flatten() {
            match Self::read_manifest(entry.as_path()) {
                Err(e) => {
                    eprintln!("Could not read manifest from {entry:?}: {e}");
                }
                Ok(manifest) => {
                    self.manifests.insert(
                        AppKey {
                            name: (*manifest.manifest.app).clone(),
                            version: manifest.manifest.version.clone(),
                        },
                        manifest,
                    );
                    eprintln!("Successful read manifest from {entry:?}");
                }
            }
        }
        Ok(())
    }
}

impl ManifestPouch {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            manifests: HashMap::default(),
        }
    }
    fn read_manifest(path: &Path) -> crate::vault::Result<AppManifest> {
        let content = fs::read_to_string(path)?;
        let manifest = flecs_app_manifest::AppManifestVersion::from_str(&content)?;
        let manifest = flecs_app_manifest::AppManifest::try_from(manifest)?;
        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    // TODO: Test ManifestPouch
}
