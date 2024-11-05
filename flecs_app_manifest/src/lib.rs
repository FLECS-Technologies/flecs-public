use generated::manifest_2_0_0;
use generated::manifest_3_0_0;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::str::FromStr;

pub mod conversion;
pub mod generated;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(tag = "_schemaVersion")]
pub enum AppManifestVersion {
    #[serde(rename = "2.0.0")]
    V2_0_0(manifest_2_0_0::FlecsAppManifest),
    #[serde(rename = "3.0.0")]
    V3_0_0(manifest_3_0_0::FlecsAppManifest),
}

impl FromStr for AppManifestVersion {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("_schemaVersion") {
            serde_json::from_str(s)
        } else {
            Ok(Self::V2_0_0(serde_json::from_str(s)?))
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub struct AppManifest {
    #[serde(skip_serializing)]
    manifest: manifest_3_0_0::FlecsAppManifest,
    #[serde(flatten)]
    original: AppManifestVersion,
}

impl Deref for AppManifest {
    type Target = manifest_3_0_0::FlecsAppManifest;

    fn deref(&self) -> &Self::Target {
        &self.manifest
    }
}

impl TryFrom<AppManifestVersion> for AppManifest {
    type Error = manifest_3_0_0::error::ConversionError;

    fn try_from(value: AppManifestVersion) -> Result<Self, Self::Error> {
        Ok(AppManifest {
            manifest: match &value {
                AppManifestVersion::V2_0_0(val) => manifest_3_0_0::FlecsAppManifest::try_from(val)?,
                AppManifestVersion::V3_0_0(val) => val.clone(),
            },
            original: value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_MANIFEST_NO_SCHEMA_VERSION: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "multiInstance": false,
    "editor": ":1234",
    "args": [
        "--adminspace-permissions=rw",
        "--rest-http-port=8000"
    ],
    "env": [
        "RUST_LOG=debug"
    ],
    "interactive": false,
    "ports": [
        "7447:7447"
    ],
    "volumes": [
        "zenoh:/root/.zenoh"
    ],
    "labels": [
        "mqtt",
        "realtime"
    ]
}
"#;
    const VALID_MANIFEST_3: &str = r#"{
    "app": "tech.flecs.flunder",
    "_schemaVersion": "3.0.0",
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "multiInstance": false,
    "editors": [
        {
            "name": "Editor 1",
            "port": 1234,
            "supportsReverseProxy": true
        }
    ],
    "args": [
        "--adminspace-permissions=rw",
        "--rest-http-port=8000"
    ],
    "env": [
        "RUST_LOG=debug"
    ],
    "interactive": false,
    "ports": [
        "7447:7447"
    ],
    "volumes": [
        "zenoh:/root/.zenoh"
    ],
    "labels": [
        "mqtt",
        "realtime"
    ]
}
"#;

    const VALID_MANIFEST_2: &str = r#"{
    "app": "tech.flecs.flunder",
    "_schemaVersion": "2.0.0",
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "multiInstance": false,
    "editor": ":1234",
    "args": [
        "--adminspace-permissions=rw",
        "--rest-http-port=8000"
    ],
    "env": [
        "RUST_LOG=debug"
    ],
    "interactive": false,
    "ports": [
        "7447:7447"
    ],
    "volumes": [
        "zenoh:/root/.zenoh"
    ],
    "labels": [
        "mqtt",
        "realtime"
    ]
}
"#;

    #[test]
    fn parse_valid_manifest_3_test() {
        match AppManifestVersion::from_str(VALID_MANIFEST_3).unwrap() {
            AppManifestVersion::V3_0_0(val) => val,
            _ => panic!("Wrong enum variant"),
        };
    }

    #[test]
    fn parse_valid_manifest_2_test() {
        match AppManifestVersion::from_str(VALID_MANIFEST_2).unwrap() {
            AppManifestVersion::V2_0_0(val) => val,
            _ => panic!("Wrong enum variant"),
        };
    }

    #[test]
    fn manifest_serialized_as_original_v2() {
        let original = AppManifestVersion::from_str(VALID_MANIFEST_2).unwrap();
        let manifest = AppManifest::try_from(original.clone()).unwrap();
        assert_eq!(
            serde_json::to_string(&original).unwrap(),
            serde_json::to_string(&manifest).unwrap()
        );
    }

    #[test]
    fn manifest_serialized_as_original_v3() {
        let original = AppManifestVersion::from_str(VALID_MANIFEST_3).unwrap();
        let manifest = AppManifest::try_from(original.clone()).unwrap();
        assert_eq!(
            serde_json::to_string(&original).unwrap(),
            serde_json::to_string(&manifest).unwrap()
        );
    }

    #[test]
    fn manifest_serialized_as_original_no_schema_version() {
        let original = AppManifestVersion::from_str(VALID_MANIFEST_NO_SCHEMA_VERSION).unwrap();
        let manifest = AppManifest::try_from(original.clone()).unwrap();
        assert_eq!(
            serde_json::to_string(&original).unwrap(),
            serde_json::to_string(&manifest).unwrap()
        );
    }

    #[test]
    fn parse_valid_manifest_no_schema_version_test() {
        match AppManifestVersion::from_str(VALID_MANIFEST_NO_SCHEMA_VERSION).unwrap() {
            AppManifestVersion::V2_0_0(val) => val,
            _ => panic!("Wrong enum variant"),
        };
    }

    #[test]
    #[should_panic]
    fn parse_unknown_schema_version_test() {
        const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "3.0.0",
    "_schemaVersion": "1.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "multiInstance": false,
    "editor": ":1234",
    "args": [
        "--adminspace-permissions=rw",
        "--rest-http-port=8000"
    ],
    "env": [
        "RUST_LOG=debug"
    ],
    "interactive": false,
    "ports": [
        "7447:7447"
    ],
    "volumes": [
        "zenoh:/root/.zenoh"
    ],
    "labels": [
        "mqtt",
        "realtime"
    ]
}
"#;

        let _ = AppManifestVersion::from_str(MANIFEST_STR).unwrap();
    }
}
