#![cfg_attr(any(), rustfmt::skip)]#![allow(clippy::clone_on_copy)]#![allow(clippy::to_string_trait_impl)]
#[doc = "Generated types for `FLECS App Manifest` for tag/branch 3.0.0 - 6758dd9251986a8afbb20f867bf92f38ad07ea3d"]
use serde::{Deserialize, Serialize};
/// Error types.
pub mod error {
    /// Error from a TryFrom or FromStr implementation.
    pub struct ConversionError(std::borrow::Cow<'static, str>);
    impl std::error::Error for ConversionError {}
    impl std::fmt::Display for ConversionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            std::fmt::Display::fmt(&self.0, f)
        }
    }
    impl std::fmt::Debug for ConversionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            std::fmt::Debug::fmt(&self.0, f)
        }
    }
    impl From<&'static str> for ConversionError {
        fn from(value: &'static str) -> Self {
            Self(value.into())
        }
    }
    impl From<String> for ConversionError {
        fn from(value: String) -> Self {
            Self(value.into())
        }
    }
}
///Schema for the FLECS App Manifest
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "$id": "https://raw.githubusercontent.com/FLECS-Technologies/app-sdk/main/manifest.schema.json",
///  "title": "FLECS App Manifest",
///  "description": "Schema for the FLECS App Manifest",
///  "type": "object",
///  "required": [
///    "app",
///    "image",
///    "version"
///  ],
///  "properties": {
///    "_minimumFlecsVersion": {
///      "description": "Minimum FLECS version needed for the app",
///      "examples": [
///        "2.0.0",
///        "3.0.0-beta.1"
///      ],
///      "type": "string",
///      "pattern": "^(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)(?:-((?:0|[1-9]\\d*|\\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\\.(?:0|[1-9]\\d*|\\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\\+([0-9a-zA-Z-]+(?:\\.[0-9a-zA-Z-]+)*))?$",
///      "$comment": "https://regex101.com/r/y9GIZD/1"
///    },
///    "app": {
///      "description": "Unique App identifier in reverse domain name notation",
///      "examples": [
///        "tech.flecs.flunder",
///        "io.anyviz.cloudadapter",
///        "com.example.some-app"
///      ],
///      "type": "string",
///      "pattern": "^((?:[a-z])+[a-z0-9.\\-_]+[a-z0-9])$",
///      "$comment": "https://regex101.com/r/psUZll/1"
///    },
///    "args": {
///      "description": "Command line arguments passed to App entrypoint",
///      "examples": [
///        [
///          "--launch-arg1",
///          "--launch-arg2=value"
///        ]
///      ],
///      "type": "array",
///      "items": {
///        "type": "string"
///      }
///    },
///    "capabilities": {
///      "description": "Permissions required for the App to function correctly",
///      "type": "array",
///      "items": {
///        "enum": [
///          "DOCKER",
///          "NET_ADMIN",
///          "SYS_NICE",
///          "IPC_LOCK",
///          "NET_RAW"
///        ]
///      },
///      "uniqueItems": true
///    },
///    "conffiles": {
///      "description": "Configuration files used by the App",
///      "type": "array",
///      "items": {
///        "examples": [
///          [
///            "default.conf:/etc/my-app/default.conf",
///            "default.conf:/etc/my-app/default.conf:rw",
///            "default.conf:/etc/my-app/default.conf:ro"
///          ]
///        ],
///        "type": "string",
///        "pattern": "(^[^/:]+):([^:]+)(?:$|:(r[ow](?:,(?:no_)?init$|)$|(?:no_)?init)$)"
///      },
///      "$comment": "https://regex101.com/r/0LtIRV/1"
///    },
///    "devices": {
///      "description": "Devices passed through to the App instances",
///      "type": "array",
///      "items": {
///        "examples": [
///          [
///            "/dev/net/tun"
///          ]
///        ],
///        "type": "string",
///        "pattern": "^(/dev/.+)$"
///      },
///      "$comment": "https://regex101.com/r/6utwD1/1"
///    },
///    "editors": {
///      "description": "Set of web-based UIs of the app",
///      "examples": [
///        [
///          {
///            "name": "Example config UI",
///            "port": 5678,
///            "supportsReverseProxy": false
///          },
///          {
///            "name:": "Example log UI",
///            "port": 7890
///          }
///        ]
///      ],
///      "type": "array",
///      "items": {
///        "type": "object",
///        "required": [
///          "name",
///          "port"
///        ],
///        "properties": {
///          "name": {
///            "type": "string"
///          },
///          "port": {
///            "description": "Port on which the editor is reachable on the docker container",
///            "type": "integer",
///            "maximum": 65535.0,
///            "minimum": 1.0
///          },
///          "supportsReverseProxy": {
///            "default": true,
///            "type": "boolean"
///          }
///        }
///      }
///    },
///    "env": {
///      "description": "Environment variables for the App instances",
///      "type": "array",
///      "items": {
///        "examples": [
///          [
///            "MY_ENV=value",
///            "tech.flecs.some-app_value=any"
///          ]
///        ],
///        "type": "string",
///        "pattern": "^[a-zA-Z]+(?:[a-zA-Z0-9_\\-\\.])*=.+$"
///      },
///      "$comment": "https://regex101.com/r/MNDmTF/1"
///    },
///    "image": {
///      "description": "Docker image for the App",
///      "examples": [
///        "flecs/tech.flecs.flunder",
///        "flecs.azurecr.io/io.anyviz.cloudadapter",
///        "registry.example.com/some-app",
///        "debian:bookworm-slim"
///      ],
///      "type": "string",
///      "pattern": "^((?:(?:(?:[a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9])(?:\\.(?:[a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9]))*|\\[(?:[a-fA-F0-9:]+)\\])(?::[0-9]+)?/)?[a-z0-9]+(?:(?:[._]|__|[-]+)[a-z0-9]+)*(?:/[a-z0-9]+(?:(?:[._]|__|[-]+)[a-z0-9]+)*)*)(?::([\\w][\\w.-]{0,127}))?(?:@([A-Za-z][A-Za-z0-9]*(?:[-_+.][A-Za-z][A-Za-z0-9]*)*[:][A-Fa-f0-9]{32,}))?$",
///      "$comment": "https://regex101.com/r/EkbfNE/1"
///    },
///    "interactive": {
///      "description": "DEPRECATED: true if App requires allocation of an interactive shell",
///      "deprecated": true,
///      "type": "boolean"
///    },
///    "labels": {
///      "description": "Labels for the App instances",
///      "type": "array",
///      "items": {
///        "examples": [
///          [
///            "tech.flecs",
///            "tech.flecs.some-label=Some custom label value"
///          ]
///        ],
///        "type": "string",
///        "pattern": "^[a-z](?:(?:[\\-\\.]?[a-zA-Z0-9])*[\\-\\.]?[a-z])?(?:=.*)?$"
///      },
///      "$comment": "https://regex101.com/r/xOiJXu/1"
///    },
///    "multiInstance": {
///      "description": "'true' if App can be instantiated more than once (requires no editor, no ports)",
///      "examples": [
///        true,
///        false
///      ],
///      "type": "boolean"
///    },
///    "ports": {
///      "description": "Port mappings for the App's instances (not allowed for multiInstance Apps)",
///      "type": "array",
///      "items": {
///        "examples": [
///          [
///            "8001:8001",
///            "5000",
///            "5001-5008:6001-6008",
///            "6001-6008"
///          ]
///        ],
///        "type": "string",
///        "pattern": "(?=\\d|:)^(?:([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3})(?:$|[:-](?=\\d))|:)?(?:(?<=-)([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3})(?:$|:(?=\\d)))?(?:(?<=:)([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3})(?:$|-(?=\\d)))?(?:(?<=-)([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3}))?$"
///      },
///      "$comment": "https://regex101.com/r/GgJ78T/1"
///    },
///    "revision": {
///      "description": "App manifest revision. Increment if Manifest is changed within the same App version",
///      "examples": [
///        "0",
///        "1",
///        "2"
///      ],
///      "type": "string"
///    },
///    "version": {
///      "description": "App version, naturally sortable",
///      "examples": [
///        "1.0.0",
///        "2022-12",
///        "v3.14.159-alpha.2",
///        "version 21"
///      ],
///      "type": "string"
///    },
///    "volumes": {
///      "description": "Virtual volumes and bind mounts for an App's instances",
///      "type": "array",
///      "items": {
///        "examples": [
///          [
///            "my-app-etc:/etc/my-app",
///            "/etc/my-app:/etc/my-app"
///          ]
///        ],
///        "type": "string",
///        "pattern": "(?:^([a-zA-Z0-9\\-_.]+)|^/[a-zA-Z0-9\\-_./]+):([a-zA-Z0-9\\-_./]+)$",
///        "$comment": "https://regex101.com/r/WjJro3/1"
///      }
///    }
///  }
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FlecsAppManifest {
    ///Unique App identifier in reverse domain name notation
    pub app: FlecsAppManifestApp,
    ///Command line arguments passed to App entrypoint
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    ///Permissions required for the App to function correctly
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Vec<FlecsAppManifestCapabilitiesItem>>,
    ///Configuration files used by the App
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conffiles: Vec<FlecsAppManifestConffilesItem>,
    ///Devices passed through to the App instances
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub devices: Vec<FlecsAppManifestDevicesItem>,
    ///Set of web-based UIs of the app
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub editors: Vec<FlecsAppManifestEditorsItem>,
    ///Environment variables for the App instances
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<FlecsAppManifestEnvItem>,
    ///Docker image for the App
    pub image: FlecsAppManifestImage,
    ///DEPRECATED: true if App requires allocation of an interactive shell
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interactive: Option<bool>,
    ///Labels for the App instances
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<FlecsAppManifestLabelsItem>,
    ///Minimum FLECS version needed for the app
    #[serde(
        rename = "_minimumFlecsVersion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub minimum_flecs_version: Option<FlecsAppManifestMinimumFlecsVersion>,
    ///'true' if App can be instantiated more than once (requires no editor, no ports)
    #[serde(rename = "multiInstance", default, skip_serializing_if = "Option::is_none")]
    pub multi_instance: Option<bool>,
    ///Port mappings for the App's instances (not allowed for multiInstance Apps)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<FlecsAppManifestPortsItem>,
    ///App manifest revision. Increment if Manifest is changed within the same App version
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    ///App version, naturally sortable
    pub version: String,
    ///Virtual volumes and bind mounts for an App's instances
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub volumes: Vec<FlecsAppManifestVolumesItem>,
}
impl From<&FlecsAppManifest> for FlecsAppManifest {
    fn from(value: &FlecsAppManifest) -> Self {
        value.clone()
    }
}
impl FlecsAppManifest {
    pub fn builder() -> builder::FlecsAppManifest {
        Default::default()
    }
}
///Unique App identifier in reverse domain name notation
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Unique App identifier in reverse domain name notation",
///  "examples": [
///    "tech.flecs.flunder",
///    "io.anyviz.cloudadapter",
///    "com.example.some-app"
///  ],
///  "type": "string",
///  "pattern": "^((?:[a-z])+[a-z0-9.\\-_]+[a-z0-9])$",
///  "$comment": "https://regex101.com/r/psUZll/1"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FlecsAppManifestApp(String);
impl std::ops::Deref for FlecsAppManifestApp {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<FlecsAppManifestApp> for String {
    fn from(value: FlecsAppManifestApp) -> Self {
        value.0
    }
}
impl From<&FlecsAppManifestApp> for FlecsAppManifestApp {
    fn from(value: &FlecsAppManifestApp) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for FlecsAppManifestApp {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if regress::Regex::new("^((?:[a-z])+[a-z0-9.\\-_]+[a-z0-9])$")
            .unwrap()
            .find(value)
            .is_none()
        {
            return Err(
                "doesn't match pattern \"^((?:[a-z])+[a-z0-9.\\-_]+[a-z0-9])$\"".into(),
            );
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for FlecsAppManifestApp {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for FlecsAppManifestApp {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for FlecsAppManifestApp {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for FlecsAppManifestApp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as serde::de::Error>::custom(e.to_string())
            })
    }
}
///FlecsAppManifestCapabilitiesItem
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "enum": [
///    "DOCKER",
///    "NET_ADMIN",
///    "SYS_NICE",
///    "IPC_LOCK",
///    "NET_RAW"
///  ]
///}
/// ```
/// </details>
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize
)]
pub enum FlecsAppManifestCapabilitiesItem {
    #[serde(rename = "DOCKER")]
    Docker,
    #[serde(rename = "NET_ADMIN")]
    NetAdmin,
    #[serde(rename = "SYS_NICE")]
    SysNice,
    #[serde(rename = "IPC_LOCK")]
    IpcLock,
    #[serde(rename = "NET_RAW")]
    NetRaw,
}
impl From<&FlecsAppManifestCapabilitiesItem> for FlecsAppManifestCapabilitiesItem {
    fn from(value: &FlecsAppManifestCapabilitiesItem) -> Self {
        value.clone()
    }
}
impl ToString for FlecsAppManifestCapabilitiesItem {
    fn to_string(&self) -> String {
        match *self {
            Self::Docker => "DOCKER".to_string(),
            Self::NetAdmin => "NET_ADMIN".to_string(),
            Self::SysNice => "SYS_NICE".to_string(),
            Self::IpcLock => "IPC_LOCK".to_string(),
            Self::NetRaw => "NET_RAW".to_string(),
        }
    }
}
impl std::str::FromStr for FlecsAppManifestCapabilitiesItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        match value {
            "DOCKER" => Ok(Self::Docker),
            "NET_ADMIN" => Ok(Self::NetAdmin),
            "SYS_NICE" => Ok(Self::SysNice),
            "IPC_LOCK" => Ok(Self::IpcLock),
            "NET_RAW" => Ok(Self::NetRaw),
            _ => Err("invalid value".into()),
        }
    }
}
impl std::convert::TryFrom<&str> for FlecsAppManifestCapabilitiesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for FlecsAppManifestCapabilitiesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for FlecsAppManifestCapabilitiesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
///FlecsAppManifestConffilesItem
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "examples": [
///    [
///      "default.conf:/etc/my-app/default.conf",
///      "default.conf:/etc/my-app/default.conf:rw",
///      "default.conf:/etc/my-app/default.conf:ro"
///    ]
///  ],
///  "type": "string",
///  "pattern": "(^[^/:]+):([^:]+)(?:$|:(r[ow](?:,(?:no_)?init$|)$|(?:no_)?init)$)"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FlecsAppManifestConffilesItem(String);
impl std::ops::Deref for FlecsAppManifestConffilesItem {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<FlecsAppManifestConffilesItem> for String {
    fn from(value: FlecsAppManifestConffilesItem) -> Self {
        value.0
    }
}
impl From<&FlecsAppManifestConffilesItem> for FlecsAppManifestConffilesItem {
    fn from(value: &FlecsAppManifestConffilesItem) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for FlecsAppManifestConffilesItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if regress::Regex::new(
                "(^[^/:]+):([^:]+)(?:$|:(r[ow](?:,(?:no_)?init$|)$|(?:no_)?init)$)",
            )
            .unwrap()
            .find(value)
            .is_none()
        {
            return Err(
                "doesn't match pattern \"(^[^/:]+):([^:]+)(?:$|:(r[ow](?:,(?:no_)?init$|)$|(?:no_)?init)$)\""
                    .into(),
            );
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for FlecsAppManifestConffilesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for FlecsAppManifestConffilesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for FlecsAppManifestConffilesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for FlecsAppManifestConffilesItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as serde::de::Error>::custom(e.to_string())
            })
    }
}
///FlecsAppManifestDevicesItem
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "examples": [
///    [
///      "/dev/net/tun"
///    ]
///  ],
///  "type": "string",
///  "pattern": "^(/dev/.+)$"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FlecsAppManifestDevicesItem(String);
impl std::ops::Deref for FlecsAppManifestDevicesItem {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<FlecsAppManifestDevicesItem> for String {
    fn from(value: FlecsAppManifestDevicesItem) -> Self {
        value.0
    }
}
impl From<&FlecsAppManifestDevicesItem> for FlecsAppManifestDevicesItem {
    fn from(value: &FlecsAppManifestDevicesItem) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for FlecsAppManifestDevicesItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if regress::Regex::new("^(/dev/.+)$").unwrap().find(value).is_none() {
            return Err("doesn't match pattern \"^(/dev/.+)$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for FlecsAppManifestDevicesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for FlecsAppManifestDevicesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for FlecsAppManifestDevicesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for FlecsAppManifestDevicesItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as serde::de::Error>::custom(e.to_string())
            })
    }
}
///FlecsAppManifestEditorsItem
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "type": "object",
///  "required": [
///    "name",
///    "port"
///  ],
///  "properties": {
///    "name": {
///      "type": "string"
///    },
///    "port": {
///      "description": "Port on which the editor is reachable on the docker container",
///      "type": "integer",
///      "maximum": 65535.0,
///      "minimum": 1.0
///    },
///    "supportsReverseProxy": {
///      "default": true,
///      "type": "boolean"
///    }
///  }
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FlecsAppManifestEditorsItem {
    pub name: String,
    ///Port on which the editor is reachable on the docker container
    pub port: std::num::NonZeroU16,
    #[serde(rename = "supportsReverseProxy", default = "defaults::default_bool::<true>")]
    pub supports_reverse_proxy: bool,
}
impl From<&FlecsAppManifestEditorsItem> for FlecsAppManifestEditorsItem {
    fn from(value: &FlecsAppManifestEditorsItem) -> Self {
        value.clone()
    }
}
impl FlecsAppManifestEditorsItem {
    pub fn builder() -> builder::FlecsAppManifestEditorsItem {
        Default::default()
    }
}
///FlecsAppManifestEnvItem
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "examples": [
///    [
///      "MY_ENV=value",
///      "tech.flecs.some-app_value=any"
///    ]
///  ],
///  "type": "string",
///  "pattern": "^[a-zA-Z]+(?:[a-zA-Z0-9_\\-\\.])*=.+$"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FlecsAppManifestEnvItem(String);
impl std::ops::Deref for FlecsAppManifestEnvItem {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<FlecsAppManifestEnvItem> for String {
    fn from(value: FlecsAppManifestEnvItem) -> Self {
        value.0
    }
}
impl From<&FlecsAppManifestEnvItem> for FlecsAppManifestEnvItem {
    fn from(value: &FlecsAppManifestEnvItem) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for FlecsAppManifestEnvItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if regress::Regex::new("^[a-zA-Z]+(?:[a-zA-Z0-9_\\-\\.])*=.+$")
            .unwrap()
            .find(value)
            .is_none()
        {
            return Err(
                "doesn't match pattern \"^[a-zA-Z]+(?:[a-zA-Z0-9_\\-\\.])*=.+$\"".into(),
            );
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for FlecsAppManifestEnvItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for FlecsAppManifestEnvItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for FlecsAppManifestEnvItem {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for FlecsAppManifestEnvItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as serde::de::Error>::custom(e.to_string())
            })
    }
}
///Docker image for the App
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Docker image for the App",
///  "examples": [
///    "flecs/tech.flecs.flunder",
///    "flecs.azurecr.io/io.anyviz.cloudadapter",
///    "registry.example.com/some-app",
///    "debian:bookworm-slim"
///  ],
///  "type": "string",
///  "pattern": "^((?:(?:(?:[a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9])(?:\\.(?:[a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9]))*|\\[(?:[a-fA-F0-9:]+)\\])(?::[0-9]+)?/)?[a-z0-9]+(?:(?:[._]|__|[-]+)[a-z0-9]+)*(?:/[a-z0-9]+(?:(?:[._]|__|[-]+)[a-z0-9]+)*)*)(?::([\\w][\\w.-]{0,127}))?(?:@([A-Za-z][A-Za-z0-9]*(?:[-_+.][A-Za-z][A-Za-z0-9]*)*[:][A-Fa-f0-9]{32,}))?$",
///  "$comment": "https://regex101.com/r/EkbfNE/1"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FlecsAppManifestImage(String);
impl std::ops::Deref for FlecsAppManifestImage {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<FlecsAppManifestImage> for String {
    fn from(value: FlecsAppManifestImage) -> Self {
        value.0
    }
}
impl From<&FlecsAppManifestImage> for FlecsAppManifestImage {
    fn from(value: &FlecsAppManifestImage) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for FlecsAppManifestImage {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if regress::Regex::new(
                "^((?:(?:(?:[a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9])(?:\\.(?:[a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9]))*|\\[(?:[a-fA-F0-9:]+)\\])(?::[0-9]+)?/)?[a-z0-9]+(?:(?:[._]|__|[-]+)[a-z0-9]+)*(?:/[a-z0-9]+(?:(?:[._]|__|[-]+)[a-z0-9]+)*)*)(?::([\\w][\\w.-]{0,127}))?(?:@([A-Za-z][A-Za-z0-9]*(?:[-_+.][A-Za-z][A-Za-z0-9]*)*[:][A-Fa-f0-9]{32,}))?$",
            )
            .unwrap()
            .find(value)
            .is_none()
        {
            return Err(
                "doesn't match pattern \"^((?:(?:(?:[a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9])(?:\\.(?:[a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9]))*|\\[(?:[a-fA-F0-9:]+)\\])(?::[0-9]+)?/)?[a-z0-9]+(?:(?:[._]|__|[-]+)[a-z0-9]+)*(?:/[a-z0-9]+(?:(?:[._]|__|[-]+)[a-z0-9]+)*)*)(?::([\\w][\\w.-]{0,127}))?(?:@([A-Za-z][A-Za-z0-9]*(?:[-_+.][A-Za-z][A-Za-z0-9]*)*[:][A-Fa-f0-9]{32,}))?$\""
                    .into(),
            );
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for FlecsAppManifestImage {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for FlecsAppManifestImage {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for FlecsAppManifestImage {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for FlecsAppManifestImage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as serde::de::Error>::custom(e.to_string())
            })
    }
}
///FlecsAppManifestLabelsItem
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "examples": [
///    [
///      "tech.flecs",
///      "tech.flecs.some-label=Some custom label value"
///    ]
///  ],
///  "type": "string",
///  "pattern": "^[a-z](?:(?:[\\-\\.]?[a-zA-Z0-9])*[\\-\\.]?[a-z])?(?:=.*)?$"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FlecsAppManifestLabelsItem(String);
impl std::ops::Deref for FlecsAppManifestLabelsItem {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<FlecsAppManifestLabelsItem> for String {
    fn from(value: FlecsAppManifestLabelsItem) -> Self {
        value.0
    }
}
impl From<&FlecsAppManifestLabelsItem> for FlecsAppManifestLabelsItem {
    fn from(value: &FlecsAppManifestLabelsItem) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for FlecsAppManifestLabelsItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if regress::Regex::new(
                "^[a-z](?:(?:[\\-\\.]?[a-zA-Z0-9])*[\\-\\.]?[a-z])?(?:=.*)?$",
            )
            .unwrap()
            .find(value)
            .is_none()
        {
            return Err(
                "doesn't match pattern \"^[a-z](?:(?:[\\-\\.]?[a-zA-Z0-9])*[\\-\\.]?[a-z])?(?:=.*)?$\""
                    .into(),
            );
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for FlecsAppManifestLabelsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for FlecsAppManifestLabelsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for FlecsAppManifestLabelsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for FlecsAppManifestLabelsItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as serde::de::Error>::custom(e.to_string())
            })
    }
}
///Minimum FLECS version needed for the app
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Minimum FLECS version needed for the app",
///  "examples": [
///    "2.0.0",
///    "3.0.0-beta.1"
///  ],
///  "type": "string",
///  "pattern": "^(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)(?:-((?:0|[1-9]\\d*|\\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\\.(?:0|[1-9]\\d*|\\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\\+([0-9a-zA-Z-]+(?:\\.[0-9a-zA-Z-]+)*))?$",
///  "$comment": "https://regex101.com/r/y9GIZD/1"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FlecsAppManifestMinimumFlecsVersion(String);
impl std::ops::Deref for FlecsAppManifestMinimumFlecsVersion {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<FlecsAppManifestMinimumFlecsVersion> for String {
    fn from(value: FlecsAppManifestMinimumFlecsVersion) -> Self {
        value.0
    }
}
impl From<&FlecsAppManifestMinimumFlecsVersion> for FlecsAppManifestMinimumFlecsVersion {
    fn from(value: &FlecsAppManifestMinimumFlecsVersion) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for FlecsAppManifestMinimumFlecsVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if regress::Regex::new(
                "^(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)(?:-((?:0|[1-9]\\d*|\\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\\.(?:0|[1-9]\\d*|\\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\\+([0-9a-zA-Z-]+(?:\\.[0-9a-zA-Z-]+)*))?$",
            )
            .unwrap()
            .find(value)
            .is_none()
        {
            return Err(
                "doesn't match pattern \"^(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)(?:-((?:0|[1-9]\\d*|\\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\\.(?:0|[1-9]\\d*|\\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\\+([0-9a-zA-Z-]+(?:\\.[0-9a-zA-Z-]+)*))?$\""
                    .into(),
            );
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for FlecsAppManifestMinimumFlecsVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for FlecsAppManifestMinimumFlecsVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for FlecsAppManifestMinimumFlecsVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for FlecsAppManifestMinimumFlecsVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as serde::de::Error>::custom(e.to_string())
            })
    }
}
///FlecsAppManifestPortsItem
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "examples": [
///    [
///      "8001:8001",
///      "5000",
///      "5001-5008:6001-6008",
///      "6001-6008"
///    ]
///  ],
///  "type": "string",
///  "pattern": "(?=\\d|:)^(?:([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3})(?:$|[:-](?=\\d))|:)?(?:(?<=-)([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3})(?:$|:(?=\\d)))?(?:(?<=:)([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3})(?:$|-(?=\\d)))?(?:(?<=-)([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3}))?$"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FlecsAppManifestPortsItem(String);
impl std::ops::Deref for FlecsAppManifestPortsItem {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<FlecsAppManifestPortsItem> for String {
    fn from(value: FlecsAppManifestPortsItem) -> Self {
        value.0
    }
}
impl From<&FlecsAppManifestPortsItem> for FlecsAppManifestPortsItem {
    fn from(value: &FlecsAppManifestPortsItem) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for FlecsAppManifestPortsItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if regress::Regex::new(
                "(?=\\d|:)^(?:([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3})(?:$|[:-](?=\\d))|:)?(?:(?<=-)([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3})(?:$|:(?=\\d)))?(?:(?<=:)([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3})(?:$|-(?=\\d)))?(?:(?<=-)([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3}))?$",
            )
            .unwrap()
            .find(value)
            .is_none()
        {
            return Err(
                "doesn't match pattern \"(?=\\d|:)^(?:([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3})(?:$|[:-](?=\\d))|:)?(?:(?<=-)([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3})(?:$|:(?=\\d)))?(?:(?<=:)([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3})(?:$|-(?=\\d)))?(?:(?<=-)([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3}))?$\""
                    .into(),
            );
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for FlecsAppManifestPortsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for FlecsAppManifestPortsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for FlecsAppManifestPortsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for FlecsAppManifestPortsItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as serde::de::Error>::custom(e.to_string())
            })
    }
}
///FlecsAppManifestVolumesItem
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "examples": [
///    [
///      "my-app-etc:/etc/my-app",
///      "/etc/my-app:/etc/my-app"
///    ]
///  ],
///  "type": "string",
///  "pattern": "(?:^([a-zA-Z0-9\\-_.]+)|^/[a-zA-Z0-9\\-_./]+):([a-zA-Z0-9\\-_./]+)$",
///  "$comment": "https://regex101.com/r/WjJro3/1"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FlecsAppManifestVolumesItem(String);
impl std::ops::Deref for FlecsAppManifestVolumesItem {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<FlecsAppManifestVolumesItem> for String {
    fn from(value: FlecsAppManifestVolumesItem) -> Self {
        value.0
    }
}
impl From<&FlecsAppManifestVolumesItem> for FlecsAppManifestVolumesItem {
    fn from(value: &FlecsAppManifestVolumesItem) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for FlecsAppManifestVolumesItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if regress::Regex::new(
                "(?:^([a-zA-Z0-9\\-_.]+)|^/[a-zA-Z0-9\\-_./]+):([a-zA-Z0-9\\-_./]+)$",
            )
            .unwrap()
            .find(value)
            .is_none()
        {
            return Err(
                "doesn't match pattern \"(?:^([a-zA-Z0-9\\-_.]+)|^/[a-zA-Z0-9\\-_./]+):([a-zA-Z0-9\\-_./]+)$\""
                    .into(),
            );
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for FlecsAppManifestVolumesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for FlecsAppManifestVolumesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for FlecsAppManifestVolumesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for FlecsAppManifestVolumesItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as serde::de::Error>::custom(e.to_string())
            })
    }
}
/// Types for composing complex structures.
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct FlecsAppManifest {
        app: Result<super::FlecsAppManifestApp, String>,
        args: Result<Vec<String>, String>,
        capabilities: Result<
            Option<Vec<super::FlecsAppManifestCapabilitiesItem>>,
            String,
        >,
        conffiles: Result<Vec<super::FlecsAppManifestConffilesItem>, String>,
        devices: Result<Vec<super::FlecsAppManifestDevicesItem>, String>,
        editors: Result<Vec<super::FlecsAppManifestEditorsItem>, String>,
        env: Result<Vec<super::FlecsAppManifestEnvItem>, String>,
        image: Result<super::FlecsAppManifestImage, String>,
        interactive: Result<Option<bool>, String>,
        labels: Result<Vec<super::FlecsAppManifestLabelsItem>, String>,
        minimum_flecs_version: Result<
            Option<super::FlecsAppManifestMinimumFlecsVersion>,
            String,
        >,
        multi_instance: Result<Option<bool>, String>,
        ports: Result<Vec<super::FlecsAppManifestPortsItem>, String>,
        revision: Result<Option<String>, String>,
        version: Result<String, String>,
        volumes: Result<Vec<super::FlecsAppManifestVolumesItem>, String>,
    }
    impl Default for FlecsAppManifest {
        fn default() -> Self {
            Self {
                app: Err("no value supplied for app".to_string()),
                args: Ok(Default::default()),
                capabilities: Ok(Default::default()),
                conffiles: Ok(Default::default()),
                devices: Ok(Default::default()),
                editors: Ok(Default::default()),
                env: Ok(Default::default()),
                image: Err("no value supplied for image".to_string()),
                interactive: Ok(Default::default()),
                labels: Ok(Default::default()),
                minimum_flecs_version: Ok(Default::default()),
                multi_instance: Ok(Default::default()),
                ports: Ok(Default::default()),
                revision: Ok(Default::default()),
                version: Err("no value supplied for version".to_string()),
                volumes: Ok(Default::default()),
            }
        }
    }
    impl FlecsAppManifest {
        pub fn app<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<super::FlecsAppManifestApp>,
            T::Error: std::fmt::Display,
        {
            self.app = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for app: {}", e));
            self
        }
        pub fn args<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Vec<String>>,
            T::Error: std::fmt::Display,
        {
            self.args = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for args: {}", e));
            self
        }
        pub fn capabilities<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<
                Option<Vec<super::FlecsAppManifestCapabilitiesItem>>,
            >,
            T::Error: std::fmt::Display,
        {
            self.capabilities = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for capabilities: {}", e)
                });
            self
        }
        pub fn conffiles<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Vec<super::FlecsAppManifestConffilesItem>>,
            T::Error: std::fmt::Display,
        {
            self.conffiles = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for conffiles: {}", e)
                });
            self
        }
        pub fn devices<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Vec<super::FlecsAppManifestDevicesItem>>,
            T::Error: std::fmt::Display,
        {
            self.devices = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for devices: {}", e)
                });
            self
        }
        pub fn editors<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Vec<super::FlecsAppManifestEditorsItem>>,
            T::Error: std::fmt::Display,
        {
            self.editors = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for editors: {}", e)
                });
            self
        }
        pub fn env<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Vec<super::FlecsAppManifestEnvItem>>,
            T::Error: std::fmt::Display,
        {
            self.env = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for env: {}", e));
            self
        }
        pub fn image<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<super::FlecsAppManifestImage>,
            T::Error: std::fmt::Display,
        {
            self.image = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for image: {}", e)
                });
            self
        }
        pub fn interactive<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<bool>>,
            T::Error: std::fmt::Display,
        {
            self.interactive = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for interactive: {}", e)
                });
            self
        }
        pub fn labels<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Vec<super::FlecsAppManifestLabelsItem>>,
            T::Error: std::fmt::Display,
        {
            self.labels = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for labels: {}", e)
                });
            self
        }
        pub fn minimum_flecs_version<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<super::FlecsAppManifestMinimumFlecsVersion>>,
            T::Error: std::fmt::Display,
        {
            self.minimum_flecs_version = value
                .try_into()
                .map_err(|e| {
                    format!(
                        "error converting supplied value for minimum_flecs_version: {}",
                        e
                    )
                });
            self
        }
        pub fn multi_instance<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<bool>>,
            T::Error: std::fmt::Display,
        {
            self.multi_instance = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for multi_instance: {}", e)
                });
            self
        }
        pub fn ports<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Vec<super::FlecsAppManifestPortsItem>>,
            T::Error: std::fmt::Display,
        {
            self.ports = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for ports: {}", e)
                });
            self
        }
        pub fn revision<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<String>>,
            T::Error: std::fmt::Display,
        {
            self.revision = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for revision: {}", e)
                });
            self
        }
        pub fn version<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<String>,
            T::Error: std::fmt::Display,
        {
            self.version = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for version: {}", e)
                });
            self
        }
        pub fn volumes<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Vec<super::FlecsAppManifestVolumesItem>>,
            T::Error: std::fmt::Display,
        {
            self.volumes = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for volumes: {}", e)
                });
            self
        }
    }
    impl std::convert::TryFrom<FlecsAppManifest> for super::FlecsAppManifest {
        type Error = super::error::ConversionError;
        fn try_from(
            value: FlecsAppManifest,
        ) -> Result<Self, super::error::ConversionError> {
            Ok(Self {
                app: value.app?,
                args: value.args?,
                capabilities: value.capabilities?,
                conffiles: value.conffiles?,
                devices: value.devices?,
                editors: value.editors?,
                env: value.env?,
                image: value.image?,
                interactive: value.interactive?,
                labels: value.labels?,
                minimum_flecs_version: value.minimum_flecs_version?,
                multi_instance: value.multi_instance?,
                ports: value.ports?,
                revision: value.revision?,
                version: value.version?,
                volumes: value.volumes?,
            })
        }
    }
    impl From<super::FlecsAppManifest> for FlecsAppManifest {
        fn from(value: super::FlecsAppManifest) -> Self {
            Self {
                app: Ok(value.app),
                args: Ok(value.args),
                capabilities: Ok(value.capabilities),
                conffiles: Ok(value.conffiles),
                devices: Ok(value.devices),
                editors: Ok(value.editors),
                env: Ok(value.env),
                image: Ok(value.image),
                interactive: Ok(value.interactive),
                labels: Ok(value.labels),
                minimum_flecs_version: Ok(value.minimum_flecs_version),
                multi_instance: Ok(value.multi_instance),
                ports: Ok(value.ports),
                revision: Ok(value.revision),
                version: Ok(value.version),
                volumes: Ok(value.volumes),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct FlecsAppManifestEditorsItem {
        name: Result<String, String>,
        port: Result<std::num::NonZeroU16, String>,
        supports_reverse_proxy: Result<bool, String>,
    }
    impl Default for FlecsAppManifestEditorsItem {
        fn default() -> Self {
            Self {
                name: Err("no value supplied for name".to_string()),
                port: Err("no value supplied for port".to_string()),
                supports_reverse_proxy: Ok(super::defaults::default_bool::<true>()),
            }
        }
    }
    impl FlecsAppManifestEditorsItem {
        pub fn name<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<String>,
            T::Error: std::fmt::Display,
        {
            self.name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for name: {}", e));
            self
        }
        pub fn port<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<std::num::NonZeroU16>,
            T::Error: std::fmt::Display,
        {
            self.port = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for port: {}", e));
            self
        }
        pub fn supports_reverse_proxy<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<bool>,
            T::Error: std::fmt::Display,
        {
            self.supports_reverse_proxy = value
                .try_into()
                .map_err(|e| {
                    format!(
                        "error converting supplied value for supports_reverse_proxy: {}",
                        e
                    )
                });
            self
        }
    }
    impl std::convert::TryFrom<FlecsAppManifestEditorsItem>
    for super::FlecsAppManifestEditorsItem {
        type Error = super::error::ConversionError;
        fn try_from(
            value: FlecsAppManifestEditorsItem,
        ) -> Result<Self, super::error::ConversionError> {
            Ok(Self {
                name: value.name?,
                port: value.port?,
                supports_reverse_proxy: value.supports_reverse_proxy?,
            })
        }
    }
    impl From<super::FlecsAppManifestEditorsItem> for FlecsAppManifestEditorsItem {
        fn from(value: super::FlecsAppManifestEditorsItem) -> Self {
            Self {
                name: Ok(value.name),
                port: Ok(value.port),
                supports_reverse_proxy: Ok(value.supports_reverse_proxy),
            }
        }
    }
}
/// Generation of default values for serde.
pub mod defaults {
    pub(super) fn default_bool<const V: bool>() -> bool {
        V
    }
}

#[cfg(test)]
mod tests;