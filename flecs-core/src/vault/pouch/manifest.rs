use crate::jeweler::gem::manifest::AppManifest;
use crate::vault::Error;
use crate::vault::pouch::{AppKey, Pouch};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::{debug, warn};

const MANIFEST_FILE_NAME: &str = "manifest.json";

pub type Gems = HashMap<AppKey, AppManifest>;

pub struct ManifestPouch {
    path: PathBuf,
    manifests: Gems,
    existing_manifest_keys: HashSet<AppKey>,
}

impl Pouch for ManifestPouch {
    type Gems = Gems;

    fn gems(&self) -> &Self::Gems {
        &self.manifests
    }

    fn gems_mut(&mut self) -> &mut Self::Gems {
        &mut self.manifests
    }
}

impl ManifestPouch {
    pub(in super::super) fn close(&mut self) -> crate::vault::Result<()> {
        let mut errors: Vec<String> = Vec::new();
        fs::create_dir_all(&self.path)?;
        for (key, manifest) in &self.manifests {
            let path = self.path.join(key.name.as_str()).join(key.version.as_str());
            if let Err(e) = fs::create_dir_all(&path) {
                errors.push(e.to_string());
                break;
            }
            match serde_json::to_string_pretty(manifest) {
                Err(e) => errors.push(e.to_string()),
                Ok(content) => {
                    if let Err(e) = fs::write(path.join(MANIFEST_FILE_NAME), content) {
                        errors.push(e.to_string())
                    }
                }
            }
        }
        let mut existing_manifest_keys = self.manifests.keys().cloned().collect();
        std::mem::swap(
            &mut existing_manifest_keys,
            &mut self.existing_manifest_keys,
        );
        for key in existing_manifest_keys {
            if !self.existing_manifest_keys.contains(&key) {
                let path = self.path.join(key.name.as_str()).join(key.version.as_str());
                if let Err(e) = fs::remove_dir_all(&path) {
                    errors.push(e.to_string());
                }
            }
        }
        match errors.len() {
            0 => Ok(()),
            1 => Err(Error::Single(errors.into_iter().next().unwrap())),
            _ => Err(Error::Multiple(errors)),
        }
    }

    pub(in super::super) fn open(&mut self) -> crate::vault::Result<()> {
        let path = self.path.join(format!("*/*/{MANIFEST_FILE_NAME}"));
        let path = path.to_str().ok_or(Error::Single(String::from("")))?;
        self.manifests.clear();
        for entry in glob::glob(path)?.flatten() {
            match Self::read_manifest(entry.as_path()) {
                Err(e) => {
                    warn!("Could not read manifest from {entry:?}: {e}");
                }
                Ok(manifest) => {
                    self.existing_manifest_keys.insert(manifest.key().clone());
                    self.manifests.insert(manifest.key().clone(), manifest);
                    debug!("Successful read manifest from {entry:?}");
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
            existing_manifest_keys: HashSet::default(),
        }
    }
    fn read_manifest(path: &Path) -> crate::vault::Result<AppManifest> {
        let content = fs::read_to_string(path)?;
        let manifest = flecs_app_manifest::AppManifestVersion::from_str(&content)?;
        let manifest = flecs_app_manifest::AppManifest::try_from(manifest)?;
        let manifest = AppManifest::try_from(manifest).map_err(|e| Error::Single(e.to_string()))?;
        Ok(manifest)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::tests::prepare_test_path;
    use flecs_app_manifest::AppManifestVersion;
    use serde_json::Value;
    use std::fs;
    use std::io::Write;
    use testdir::testdir;

    fn manifest_from_json(json: &Value) -> AppManifest {
        let manifest = AppManifestVersion::from_str(&serde_json::to_string(json).unwrap()).unwrap();
        let manifest = flecs_app_manifest::AppManifest::try_from(manifest).unwrap();
        manifest.try_into().unwrap()
    }

    pub fn test_manifests() -> Vec<AppManifest> {
        vec![
            min_app_1_1_0_manifest(),
            min_app_1_0_0_manifest(),
            min_app_1_1_4_manifest(),
            min_app_2_4_5_manifest(),
            single_instance_app_manifest(),
            multi_instance_app_manifest(),
            label_manifest(),
            editor_manifest(),
            network_manifest(),
            mount_manifest(),
        ]
    }

    pub fn test_manifest_pouch() -> ManifestPouch {
        let manifests = HashMap::from_iter(
            test_manifests()
                .into_iter()
                .map(|manifest| (manifest.key().clone(), manifest)),
        );
        ManifestPouch {
            path: testdir!().join("manifests"),
            existing_manifest_keys: HashSet::from_iter(manifests.keys().cloned()),
            manifests,
        }
    }

    pub fn single_instance_app_manifest() -> AppManifest {
        let json = serde_json::json!({
            "_schemaVersion": "3.0.0",
            "app": "tech.flecs.single-instance",
            "version": "1.0.0",
            "image": "flecs.azurecr.io/tech.flecs.single-instance",
            "multiInstance": false
        });
        manifest_from_json(&json)
    }

    pub fn multi_instance_app_manifest() -> AppManifest {
        let json = serde_json::json!({
            "_schemaVersion": "3.0.0",
            "app": "tech.flecs.multi-instance",
            "version": "1.0.0",
            "image": "flecs.azurecr.io/tech.flecs.multi-instance",
            "multiInstance": true
        });
        manifest_from_json(&json)
    }

    pub fn min_app_1_0_0_manifest() -> AppManifest {
        let json = serde_json::json!({
            "_schemaVersion": "3.0.0",
            "app": "tech.flecs.min-app",
            "version": "1.0.0",
            "image": "flecs.azurecr.io/tech.flecs.min-app"
        });
        manifest_from_json(&json)
    }

    pub fn min_app_1_1_0_manifest() -> AppManifest {
        let json = serde_json::json!({
            "_schemaVersion": "3.0.0",
            "app": "tech.flecs.min-app",
            "version": "1.1.0",
            "image": "flecs.azurecr.io/tech.flecs.min-app"
        });
        manifest_from_json(&json)
    }

    pub fn min_app_1_1_4_manifest() -> AppManifest {
        let json = serde_json::json!({
            "_schemaVersion": "3.0.0",
            "app": "tech.flecs.min-app",
            "version": "1.1.4",
            "image": "flecs.azurecr.io/tech.flecs.min-app"
        });
        manifest_from_json(&json)
    }

    pub fn min_app_2_4_5_manifest() -> AppManifest {
        let json = serde_json::json!({
            "_schemaVersion": "3.0.0",
            "app": "tech.flecs.min-app",
            "version": "2.4.5",
            "image": "flecs.azurecr.io/tech.flecs.min-app"
        });
        manifest_from_json(&json)
    }

    pub fn label_manifest() -> AppManifest {
        let json = serde_json::json!({
            "_schemaVersion": "3.0.0",
            "app": "tech.flecs.label-app",
            "version": "7.6.2",
            "image": "flecs.azurecr.io/tech.flecs.label-app",
            "labels": [
                "tech.flecs",
                "tech.flecs.some-label=Some custom label value"
            ]
        });
        manifest_from_json(&json)
    }

    pub fn editor_manifest() -> AppManifest {
        let json = serde_json::json!({
            "_schemaVersion": "3.0.0",
            "app": "tech.flecs.editor-app",
            "version": "5.2.1",
            "image": "flecs.azurecr.io/tech.flecs.editor-app",
            "editors": [
                {
                   "name": "editor 1",
                   "port": 1234,
                   "supportsReverseProxy": false
                },
                {
                   "name": "editor 2",
                   "port": 5678,
                   "supportsReverseProxy": true
                },
                {
                   "name": "editor 3",
                   "port": 3000,
                   "supportsReverseProxy": false
                },
            ]
        });
        manifest_from_json(&json)
    }

    pub fn network_manifest() -> AppManifest {
        let json = serde_json::json!({
            "_schemaVersion": "3.0.0",
            "app": "tech.flecs.network-app",
            "version": "1.2.12",
            "image": "flecs.azurecr.io/tech.flecs.network-app"
        });
        manifest_from_json(&json)
    }

    pub fn no_manifest() -> AppManifest {
        let json = serde_json::json!({
            "_schemaVersion": "3.0.0",
            "app": "tech.flecs.no-manifest",
            "version": "1.0.0",
            "image": "flecs.azurecr.io/tech.flecs.no-manifest"
        });
        manifest_from_json(&json)
    }

    pub fn mount_manifest() -> AppManifest {
        let json = serde_json::json!({
            "_schemaVersion": "3.0.0",
            "app": "tech.flecs.mount",
            "version": "0.4.0",
            "image": "flecs.azurecr.io/tech.flecs.mount",
            "volumes": [
                "/etc/config:/etc/config",
                "/log/app-logs:/log"
            ]
        });
        manifest_from_json(&json)
    }

    pub fn create_test_manifest(app_name: &str, app_version: &str) -> AppManifest {
        let manifest = AppManifestVersion::from_str(
            &serde_json::to_string(&create_test_json_v3(app_name, app_version)).unwrap(),
        )
        .unwrap();
        let manifest = flecs_app_manifest::AppManifest::try_from(manifest).unwrap();
        manifest.try_into().unwrap()
    }
    fn create_test_json_v3(app_name: &str, app_version: &str) -> Value {
        serde_json::json!({
            "_schemaVersion": "3.0.0",
            "app": app_name,
            "version": app_version,
            "revision": "0",
            "image": format!("flecs.azurecr.io/{app_name}"),
            "multiInstance": false,
            "editors": [
                {
                   "name": "editor 1",
                   "port": 1234,
                   "supportsReverseProxy": false
                },
                {
                   "name": "editor 2",
                   "port": 5678,
                   "supportsReverseProxy": true
                },
                {
                   "name": "editor 3",
                   "port": 3000,
                   "supportsReverseProxy": false
                },
            ],
            "args": [
                "--launch-arg1",
                "--launch-arg2=some_value"
            ],
            "capabilities": [
                "DOCKER",
                "NET_RAW"
            ],
            "conffiles": [
                "default.conf:/etc/my-app/default.conf",
                "default.conf:/etc/my-app/default.conf:rw",
                "default.conf:/etc/my-app/default.conf:ro"
            ],
            "devices": [
                "/dev/net/tun"
            ],
            "env": [
                "MY_ENV=value",
                "tech.flecs.some-app_value=any"
            ],
            "interactive": false,
            "ports": [
                "8001:8001",
                "5000",
                "5001-5008:6001-6008",
                "6001-6008"
            ],
            "volumes": [
                "my-app-etc:/etc/my-app",
                "/etc/my.app:/etc/my-app"
            ],
            "labels": [
                "tech.flecs",
                "tech.flecs.some-label=Some custom label value"
            ]
        })
    }

    // TODO: Test delete of manifest files on close
    #[test]
    fn close_manifest_pouch() {
        let path = prepare_test_path(module_path!(), "close_pouch");

        let (name, version) = ("mample".to_string(), "2.1.3".to_string());
        let manifest_path1 = path.join(&name).join(&version).join(MANIFEST_FILE_NAME);
        let json1 = create_test_json_v3(&name, &version);
        let manifest1 = create_test_manifest(&name, &version);
        let manifest_key1 = AppKey {
            name: name.clone(),
            version: version.clone(),
        };
        let (name, version) = ("tamble".to_string(), "10.23.1".to_string());
        let manifest_path2 = path.join(&name).join(&version).join(MANIFEST_FILE_NAME);
        let json2 = create_test_json_v3(&name, &version);
        let manifest2 = create_test_manifest(&name, &version);

        let manifests = HashMap::from([
            (manifest1.key().clone(), manifest1),
            (manifest2.key().clone(), manifest2),
        ]);
        let existing_manifest_keys = manifests.keys().cloned().collect();
        let mut manifest_pouch = ManifestPouch {
            manifests,
            path,
            existing_manifest_keys,
        };
        manifest_pouch.close().unwrap();
        let file = fs::File::open(manifest_path1.clone()).unwrap();
        let content: Value = serde_json::from_reader(file).unwrap();
        assert_eq!(content, json1);
        let file = fs::File::open(manifest_path2.clone()).unwrap();
        let content: Value = serde_json::from_reader(file).unwrap();
        assert_eq!(content, json2);
        manifest_pouch.manifests.remove(&manifest_key1).unwrap();
        assert_eq!(manifest_pouch.manifests.len(), 1);
        assert!(!manifest_pouch.manifests.contains_key(&manifest_key1));
        manifest_pouch.close().unwrap();
        let file = fs::File::open(manifest_path2).unwrap();
        let content: Value = serde_json::from_reader(file).unwrap();
        assert_eq!(content, json2);
        assert!(!manifest_path1.try_exists().unwrap());
    }

    #[test]
    fn open_manifest_pouch() {
        let path = prepare_test_path(module_path!(), "open_pouch");

        let (name, version) = ("mample".to_string(), "2.1.3".to_string());
        let manifest_path1 = path.join(&name).join(&version).join(MANIFEST_FILE_NAME);
        let json1 = create_test_json_v3(&name, &version);
        let manifest1 = create_test_manifest(&name, &version);

        let (name, version) = ("tamble".to_string(), "10.23.1".to_string());
        let manifest_path2 = path.join(&name).join(&version).join(MANIFEST_FILE_NAME);
        let json2 = create_test_json_v3(&name, &version);
        let manifest2 = create_test_manifest(&name, &version);

        let manifests = HashMap::from([
            (manifest1.key().clone(), manifest1),
            (manifest2.key().clone(), manifest2),
        ]);
        let mut manifest_pouch = ManifestPouch {
            manifests: HashMap::default(),
            path,
            existing_manifest_keys: HashSet::new(),
        };
        fs::create_dir_all(manifest_path1.parent().unwrap()).unwrap();
        fs::File::create(manifest_path1)
            .unwrap()
            .write_all(serde_json::to_string_pretty(&json1).unwrap().as_bytes())
            .unwrap();
        fs::create_dir_all(manifest_path2.parent().unwrap()).unwrap();
        fs::File::create(manifest_path2)
            .unwrap()
            .write_all(serde_json::to_string_pretty(&json2).unwrap().as_bytes())
            .unwrap();
        manifest_pouch.open().unwrap();
        assert_eq!(manifests, manifest_pouch.manifests);
    }

    #[test]
    fn open_manifest_pouch_error() {
        let path = prepare_test_path(module_path!(), "open_pouch_error");

        let manifest_path1 = path.join("mample").join("2.1.3").join(MANIFEST_FILE_NAME);

        let (name, version) = ("tamble".to_string(), "10.23.1".to_string());
        let manifest_path2 = path.join(&name).join(&version).join(MANIFEST_FILE_NAME);
        let json = create_test_json_v3(&name, &version);
        let manifest = create_test_manifest(&name, &version);

        let manifests = HashMap::from([(manifest.key().clone(), manifest)]);
        let mut manifest_pouch = ManifestPouch {
            manifests: HashMap::default(),
            path,
            existing_manifest_keys: HashSet::new(),
        };
        fs::create_dir_all(manifest_path1.parent().unwrap()).unwrap();
        fs::File::create(manifest_path1)
            .unwrap()
            .write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes())
            .unwrap();
        fs::create_dir_all(manifest_path2.parent().unwrap()).unwrap();
        fs::File::create(manifest_path2)
            .unwrap()
            .write_all(b"random invalid data")
            .unwrap();
        matches!(manifest_pouch.open(), Err(Error::Single(_)));
        assert_eq!(manifests, manifest_pouch.manifests);
    }

    #[test]
    fn open_manifest_pouch_errors() {
        let path = prepare_test_path(module_path!(), "open_pouch_errors");

        let manifest_path1 = path.join("mample").join("2.1.3").join(MANIFEST_FILE_NAME);
        let manifest_path2 = path.join("tamble").join("10.23.1").join(MANIFEST_FILE_NAME);

        let mut manifest_pouch = ManifestPouch {
            manifests: HashMap::default(),
            path,
            existing_manifest_keys: HashSet::default(),
        };
        fs::create_dir_all(manifest_path1.parent().unwrap()).unwrap();
        fs::File::create(manifest_path1)
            .unwrap()
            .write_all(b"random invalid data")
            .unwrap();
        fs::create_dir_all(manifest_path2.parent().unwrap()).unwrap();
        fs::File::create(manifest_path2)
            .unwrap()
            .write_all(b"random invalid data")
            .unwrap();
        matches!(manifest_pouch.open(), Err(Error::Multiple(_)));
        assert!(manifest_pouch.manifests.is_empty());
    }

    #[test]
    fn manifest_gems() {
        let path = prepare_test_path(module_path!(), "gems");

        let manifest1 = create_test_manifest("mample", "2.1.3");
        let manifest2 = create_test_manifest("tamble", "10.23.1");

        let gems = HashMap::from([
            (manifest1.key().clone(), manifest1),
            (manifest2.key().clone(), manifest2),
        ]);
        let mut manifest_pouch = ManifestPouch {
            manifests: gems.clone(),
            path: path.parent().unwrap().to_path_buf(),
            existing_manifest_keys: gems.keys().cloned().collect(),
        };
        assert_eq!(&gems, manifest_pouch.gems_mut());
        assert_eq!(&gems, manifest_pouch.gems());
    }
}
