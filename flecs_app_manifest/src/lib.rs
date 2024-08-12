use generated::manifest_2_0_0;
use generated::manifest_3_0_0;
use serde::{Deserialize, Serialize};
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

#[derive(Debug)]
pub struct AppManifest {
    pub manifest: manifest_3_0_0::FlecsAppManifest,
    pub original: AppManifestVersion,
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

    #[test]
    fn parse_valid_manifest_3_test() {
        const MANIFEST_STR: &str = r#"{
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

        match AppManifestVersion::from_str(MANIFEST_STR).unwrap() {
            AppManifestVersion::V3_0_0(val) => val,
            _ => panic!("Wrong enum variant"),
        };
    }
    #[test]
    fn parse_valid_manifest_2_test() {
        const MANIFEST_STR: &str = r#"{
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

        match AppManifestVersion::from_str(MANIFEST_STR).unwrap() {
            AppManifestVersion::V2_0_0(val) => val,
            _ => panic!("Wrong enum variant"),
        };
    }

    #[test]
    fn parse_valid_manifest_no_schema_version_test() {
        const MANIFEST_STR: &str = r#"{
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

        match AppManifestVersion::from_str(MANIFEST_STR).unwrap() {
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
