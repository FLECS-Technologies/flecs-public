use generated::manifest_2_0_0;
use generated::manifest_3_0_0;
use generated::manifest_3_1_0;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

pub mod conversion;
pub mod generated;

#[derive(Debug)]
pub enum ManifestError {
    ConversionV3V31(manifest_3_1_0::error::ConversionError),
    ConversionV2V3(manifest_3_0_0::error::ConversionError),
    MultiAppManifestNotSupported,
}

impl Display for ManifestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::ConversionV3V31(error) => {
                write!(
                    f,
                    "Could not convert app manifest from v3.0.0 to v3.1.0: {error}"
                )
            }
            ManifestError::ConversionV2V3(error) => {
                write!(
                    f,
                    "Could not convert app manifest from v2.0.0 to v3.1.0: {error}"
                )
            }
            ManifestError::MultiAppManifestNotSupported => {
                write!(f, "Multi image app manifests are not supported yet")
            }
        }
    }
}

impl std::error::Error for ManifestError {}

impl From<manifest_3_1_0::error::ConversionError> for ManifestError {
    fn from(value: generated::manifest_3_1_0::error::ConversionError) -> Self {
        Self::ConversionV3V31(value)
    }
}

impl From<manifest_3_0_0::error::ConversionError> for ManifestError {
    fn from(value: generated::manifest_3_0_0::error::ConversionError) -> Self {
        Self::ConversionV2V3(value)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(tag = "_schemaVersion")]
pub enum AppManifestVersion {
    #[serde(rename = "2.0.0")]
    V2_0_0(manifest_2_0_0::FlecsAppManifest),
    #[serde(rename = "3.0.0")]
    V3_0_0(manifest_3_0_0::FlecsAppManifest),
    #[serde(rename = "3.1.0")]
    V3_1_0(manifest_3_1_0::FlecsAppManifest),
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
#[serde(untagged)]
pub enum AppManifest {
    Single(AppManifestSingle),
    Multi(AppManifestMulti),
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub struct AppManifestMulti {
    #[serde(skip_serializing)]
    manifest: manifest_3_1_0::Multi,
    #[serde(flatten)]
    original: AppManifestVersion,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub struct AppManifestSingle {
    #[serde(skip_serializing)]
    manifest: manifest_3_1_0::Single,
    #[serde(flatten)]
    original: AppManifestVersion,
}

impl Deref for AppManifestSingle {
    type Target = manifest_3_1_0::Single;

    fn deref(&self) -> &Self::Target {
        &self.manifest
    }
}

impl Deref for AppManifestMulti {
    type Target = manifest_3_1_0::Multi;

    fn deref(&self) -> &Self::Target {
        &self.manifest
    }
}

impl TryFrom<AppManifestVersion> for AppManifest {
    type Error = ManifestError;

    fn try_from(value: AppManifestVersion) -> Result<Self, Self::Error> {
        match &value {
            AppManifestVersion::V2_0_0(val) => Ok(Self::Single(AppManifestSingle {
                manifest: val.try_into()?,
                original: value,
            })),
            AppManifestVersion::V3_0_0(val) => Ok(Self::Single(AppManifestSingle {
                manifest: val.try_into()?,
                original: value,
            })),
            AppManifestVersion::V3_1_0(manifest_3_1_0::FlecsAppManifest::Single(val)) => {
                Ok(Self::Single(AppManifestSingle {
                    manifest: val.clone(),
                    original: value,
                }))
            }
            AppManifestVersion::V3_1_0(manifest_3_1_0::FlecsAppManifest::Multi(val)) => {
                Ok(Self::Multi(AppManifestMulti {
                    manifest: val.clone(),
                    original: value,
                }))
            }
        }
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
    const VALID_MANIFEST_3_1_SINGLE: &str = r#"{
    "app": "tech.flecs.flunder",
    "_schemaVersion": "3.1.0",
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
    "hostname": "test-hostname",
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
    const VALID_MANIFEST_3_1_MULTI: &str = r#"{
    "app": "tech.flecs.flunder",
    "_schemaVersion": "3.1.0",
    "version": "3.0.0",
    "deployment": {
        "compose": {
            "yaml": {
                "some": "yaml",
                "converted": "to",
                "corresponding": "json"
            }
        }
    }
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
    fn parse_valid_manifest_3_1_single_test() {
        assert!(matches!(
            AppManifestVersion::from_str(VALID_MANIFEST_3_1_SINGLE),
            Ok(AppManifestVersion::V3_1_0(
                manifest_3_1_0::FlecsAppManifest::Single(_)
            ))
        ));
    }

    #[test]
    fn parse_valid_manifest_3_1_multi_test() {
        assert!(matches!(
            AppManifestVersion::from_str(VALID_MANIFEST_3_1_MULTI),
            Ok(AppManifestVersion::V3_1_0(
                manifest_3_1_0::FlecsAppManifest::Multi(_)
            ))
        ));
    }

    #[test]
    fn parse_valid_manifest_3_test() {
        assert!(matches!(
            AppManifestVersion::from_str(VALID_MANIFEST_3),
            Ok(AppManifestVersion::V3_0_0(_))
        ));
    }

    #[test]
    fn parse_valid_manifest_2_test() {
        assert!(matches!(
            AppManifestVersion::from_str(VALID_MANIFEST_2),
            Ok(AppManifestVersion::V2_0_0(_))
        ));
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

        assert!(AppManifestVersion::from_str(MANIFEST_STR).is_err());
    }
}
