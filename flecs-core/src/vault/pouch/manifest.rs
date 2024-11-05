use crate::vault::pouch::{AppKey, Pouch, VaultPouch};
use crate::vault::Error;
use flecs_app_manifest::AppManifest;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

const MANIFEST_FILE_NAME: &str = "manifest.json";

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
            match serde_json::to_string_pretty(manifest) {
                Err(e) => errors.push(e.to_string()),
                Ok(content) => {
                    if let Err(e) = fs::write(path.join(MANIFEST_FILE_NAME), content) {
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
        let path = self.path.join(format!("*/*/{MANIFEST_FILE_NAME}"));
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
                    println!("Successful read manifest from {entry:?}");
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
    use super::*;
    use flecs_app_manifest::AppManifestVersion;
    use serde_json::Value;
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    const TEST_PATH: &str = "/tmp/flecs-tests/manifest-pouch/";

    fn prepare_path(path: &Path) {
        println!("Preparing {:?}", path);
        let _ = fs::remove_dir_all(path);
        assert!(!path.try_exists().unwrap());
        fs::create_dir_all(path).unwrap();
        assert!(path.try_exists().unwrap());
    }

    fn create_test_manifest(app_name: &str, app_version: &str) -> AppManifest {
        let manifest = AppManifestVersion::from_str(
            &serde_json::to_string(&create_test_json_v3(app_name, app_version)).unwrap(),
        )
        .unwrap();
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
                }
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

    #[test]
    fn close_manifest_pouch() {
        let path = Path::new(TEST_PATH).join("close_pouch");
        prepare_path(&path);

        let (name, version) = ("mample".to_string(), "2.1.3".to_string());
        let manifest_path1 = path.join(&name).join(&version).join(MANIFEST_FILE_NAME);
        let json1 = create_test_json_v3(&name, &version);
        let manifest1 = create_test_manifest(&name, &version);

        let (name, version) = ("tamble".to_string(), "10.23.1".to_string());
        let manifest_path2 = path.join(&name).join(&version).join(MANIFEST_FILE_NAME);
        let json2 = create_test_json_v3(&name, &version);
        let manifest2 = create_test_manifest(&name, &version);

        let manifests = HashMap::from([
            (
                AppKey {
                    name: manifest1.manifest.app.to_string(),
                    version: manifest1.manifest.version.clone(),
                },
                manifest1,
            ),
            (
                AppKey {
                    name: manifest2.manifest.app.to_string(),
                    version: manifest2.manifest.version.clone(),
                },
                manifest2,
            ),
        ]);
        let mut manifest_pouch = ManifestPouch { manifests, path };
        manifest_pouch.close().unwrap();
        let file = fs::File::open(manifest_path1).unwrap();
        let content: Value = serde_json::from_reader(file).unwrap();
        assert_eq!(content, json1);
        let file = fs::File::open(manifest_path2).unwrap();
        let content: Value = serde_json::from_reader(file).unwrap();
        assert_eq!(content, json2);
    }

    #[test]
    fn open_manifest_pouch() {
        let path = Path::new(TEST_PATH).join("open_pouch");
        prepare_path(&path);

        let (name, version) = ("mample".to_string(), "2.1.3".to_string());
        let manifest_path1 = path.join(&name).join(&version).join(MANIFEST_FILE_NAME);
        let json1 = create_test_json_v3(&name, &version);
        let manifest1 = create_test_manifest(&name, &version);

        let (name, version) = ("tamble".to_string(), "10.23.1".to_string());
        let manifest_path2 = path.join(&name).join(&version).join(MANIFEST_FILE_NAME);
        let json2 = create_test_json_v3(&name, &version);
        let manifest2 = create_test_manifest(&name, &version);

        let manifests = HashMap::from([
            (
                AppKey {
                    name: manifest1.manifest.app.to_string(),
                    version: manifest1.manifest.version.clone(),
                },
                manifest1,
            ),
            (
                AppKey {
                    name: manifest2.manifest.app.to_string(),
                    version: manifest2.manifest.version.clone(),
                },
                manifest2,
            ),
        ]);
        let mut manifest_pouch = ManifestPouch {
            manifests: HashMap::default(),
            path,
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
        let path = Path::new(TEST_PATH).join("open_pouch_error");
        prepare_path(&path);

        let manifest_path1 = path.join("mample").join("2.1.3").join(MANIFEST_FILE_NAME);

        let (name, version) = ("tamble".to_string(), "10.23.1".to_string());
        let manifest_path2 = path.join(&name).join(&version).join(MANIFEST_FILE_NAME);
        let json = create_test_json_v3(&name, &version);
        let manifest = create_test_manifest(&name, &version);

        let manifests = HashMap::from([(
            AppKey {
                name: manifest.manifest.app.to_string(),
                version: manifest.manifest.version.clone(),
            },
            manifest,
        )]);
        let mut manifest_pouch = ManifestPouch {
            manifests: HashMap::default(),
            path,
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
        let path = Path::new(TEST_PATH).join("open_pouch_errors");
        prepare_path(&path);

        let manifest_path1 = path.join("mample").join("2.1.3").join(MANIFEST_FILE_NAME);
        let manifest_path2 = path.join("tamble").join("10.23.1").join(MANIFEST_FILE_NAME);

        let mut manifest_pouch = ManifestPouch {
            manifests: HashMap::default(),
            path,
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
        let path = Path::new(TEST_PATH).join("close_pouch");
        prepare_path(&path);

        let manifest1 = create_test_manifest("mample", "2.1.3");
        let manifest2 = create_test_manifest("tamble", "10.23.1");

        let gems = HashMap::from([
            (
                AppKey {
                    name: manifest1.manifest.app.to_string(),
                    version: manifest1.manifest.version.clone(),
                },
                manifest1,
            ),
            (
                AppKey {
                    name: manifest2.manifest.app.to_string(),
                    version: manifest2.manifest.version.clone(),
                },
                manifest2,
            ),
        ]);
        let mut manifest_pouch = ManifestPouch {
            manifests: gems.clone(),
            path: PathBuf::from(TEST_PATH),
        };
        assert_eq!(&gems, manifest_pouch.gems_mut());
        assert_eq!(&gems, manifest_pouch.gems());
    }
}
