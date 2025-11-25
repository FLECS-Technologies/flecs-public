#![cfg_attr(any(), rustfmt::skip)]#![allow(clippy::clone_on_copy)]#![allow(clippy::to_string_trait_impl)]
#[doc = "Generated types for `FLECS App Manifest` for tag/branch 3.2.0 - d98070763a6fb76f20eb04a552d779e28811f50c"]
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
pub struct App(String);
impl std::ops::Deref for App {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<App> for String {
    fn from(value: App) -> Self {
        value.0
    }
}
impl From<&App> for App {
    fn from(value: &App) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for App {
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
impl std::convert::TryFrom<&str> for App {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for App {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for App {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for App {
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
///Command line arguments passed to App entrypoint
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Command line arguments passed to App entrypoint",
///  "examples": [
///    [
///      "--launch-arg1",
///      "--launch-arg2=value"
///    ]
///  ],
///  "type": "array",
///  "items": {
///    "type": "string"
///  }
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Args(pub Vec<String>);
impl std::ops::Deref for Args {
    type Target = Vec<String>;
    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}
impl From<Args> for Vec<String> {
    fn from(value: Args) -> Self {
        value.0
    }
}
impl From<&Args> for Args {
    fn from(value: &Args) -> Self {
        value.clone()
    }
}
impl From<Vec<String>> for Args {
    fn from(value: Vec<String>) -> Self {
        Self(value)
    }
}
///Permissions required for the App to function correctly
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Permissions required for the App to function correctly",
///  "type": "array",
///  "items": {
///    "enum": [
///      "DOCKER",
///      "NET_ADMIN",
///      "SYS_NICE",
///      "IPC_LOCK",
///      "NET_RAW"
///    ]
///  },
///  "uniqueItems": true
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Capabilities(pub Vec<CapabilitiesItem>);
impl std::ops::Deref for Capabilities {
    type Target = Vec<CapabilitiesItem>;
    fn deref(&self) -> &Vec<CapabilitiesItem> {
        &self.0
    }
}
impl From<Capabilities> for Vec<CapabilitiesItem> {
    fn from(value: Capabilities) -> Self {
        value.0
    }
}
impl From<&Capabilities> for Capabilities {
    fn from(value: &Capabilities) -> Self {
        value.clone()
    }
}
impl From<Vec<CapabilitiesItem>> for Capabilities {
    fn from(value: Vec<CapabilitiesItem>) -> Self {
        Self(value)
    }
}
///CapabilitiesItem
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
pub enum CapabilitiesItem {
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
impl From<&CapabilitiesItem> for CapabilitiesItem {
    fn from(value: &CapabilitiesItem) -> Self {
        value.clone()
    }
}
impl ToString for CapabilitiesItem {
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
impl std::str::FromStr for CapabilitiesItem {
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
impl std::convert::TryFrom<&str> for CapabilitiesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for CapabilitiesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for CapabilitiesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
///Configuration files used by the App
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Configuration files used by the App",
///  "type": "array",
///  "items": {
///    "examples": [
///      [
///        "default.conf:/etc/my-app/default.conf",
///        "default.conf:/etc/my-app/default.conf:rw",
///        "default.conf:/etc/my-app/default.conf:ro"
///      ]
///    ],
///    "type": "string",
///    "pattern": "(^[^/:]+):([^:]+)(?:$|:(r[ow](?:,(?:no_)?init$|)$|(?:no_)?init)$)"
///  },
///  "$comment": "https://regex101.com/r/0LtIRV/1"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Conffiles(pub Vec<ConffilesItem>);
impl std::ops::Deref for Conffiles {
    type Target = Vec<ConffilesItem>;
    fn deref(&self) -> &Vec<ConffilesItem> {
        &self.0
    }
}
impl From<Conffiles> for Vec<ConffilesItem> {
    fn from(value: Conffiles) -> Self {
        value.0
    }
}
impl From<&Conffiles> for Conffiles {
    fn from(value: &Conffiles) -> Self {
        value.clone()
    }
}
impl From<Vec<ConffilesItem>> for Conffiles {
    fn from(value: Vec<ConffilesItem>) -> Self {
        Self(value)
    }
}
///ConffilesItem
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
pub struct ConffilesItem(String);
impl std::ops::Deref for ConffilesItem {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<ConffilesItem> for String {
    fn from(value: ConffilesItem) -> Self {
        value.0
    }
}
impl From<&ConffilesItem> for ConffilesItem {
    fn from(value: &ConffilesItem) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for ConffilesItem {
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
impl std::convert::TryFrom<&str> for ConffilesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for ConffilesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for ConffilesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for ConffilesItem {
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
///Depends
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "type": "object",
///  "additionalProperties": true,
///  "propertyNames": {
///    "pattern": "^([\\w.-]+)(?:\\s*\\|\\s*([\\w.-]+))*$"
///  }
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Depends(pub std::collections::HashMap<DependsKey, serde_json::Value>);
impl std::ops::Deref for Depends {
    type Target = std::collections::HashMap<DependsKey, serde_json::Value>;
    fn deref(&self) -> &std::collections::HashMap<DependsKey, serde_json::Value> {
        &self.0
    }
}
impl From<Depends> for std::collections::HashMap<DependsKey, serde_json::Value> {
    fn from(value: Depends) -> Self {
        value.0
    }
}
impl From<&Depends> for Depends {
    fn from(value: &Depends) -> Self {
        value.clone()
    }
}
impl From<std::collections::HashMap<DependsKey, serde_json::Value>> for Depends {
    fn from(value: std::collections::HashMap<DependsKey, serde_json::Value>) -> Self {
        Self(value)
    }
}
///DependsKey
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "type": "string",
///  "pattern": "^([\\w.-]+)(?:\\s*\\|\\s*([\\w.-]+))*$"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct DependsKey(String);
impl std::ops::Deref for DependsKey {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<DependsKey> for String {
    fn from(value: DependsKey) -> Self {
        value.0
    }
}
impl From<&DependsKey> for DependsKey {
    fn from(value: &DependsKey) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for DependsKey {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if regress::Regex::new("^([\\w.-]+)(?:\\s*\\|\\s*([\\w.-]+))*$")
            .unwrap()
            .find(value)
            .is_none()
        {
            return Err(
                "doesn't match pattern \"^([\\w.-]+)(?:\\s*\\|\\s*([\\w.-]+))*$\"".into(),
            );
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for DependsKey {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for DependsKey {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for DependsKey {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for DependsKey {
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
///Method of deploying the App through FLECS
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Method of deploying the App through FLECS",
///  "type": "object",
///  "required": [
///    "compose"
///  ],
///  "properties": {
///    "compose": {
///      "type": "object",
///      "properties": {
///        "yaml": {
///          "description": "docker-compose.yml file converted to JSON",
///          "type": "object",
///          "$schema": "https://raw.githubusercontent.com/compose-spec/compose-go/refs/tags/v2.4.4/schema/compose-spec.json"
///        }
///      }
///    }
///  }
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Deployment {
    pub compose: DeploymentCompose,
}
impl From<&Deployment> for Deployment {
    fn from(value: &Deployment) -> Self {
        value.clone()
    }
}
impl Deployment {
    pub fn builder() -> builder::Deployment {
        Default::default()
    }
}
///DeploymentCompose
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "type": "object",
///  "properties": {
///    "yaml": {
///      "description": "docker-compose.yml file converted to JSON",
///      "type": "object",
///      "$schema": "https://raw.githubusercontent.com/compose-spec/compose-go/refs/tags/v2.4.4/schema/compose-spec.json"
///    }
///  }
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DeploymentCompose {
    ///docker-compose.yml file converted to JSON
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub yaml: serde_json::Map<String, serde_json::Value>,
}
impl From<&DeploymentCompose> for DeploymentCompose {
    fn from(value: &DeploymentCompose) -> Self {
        value.clone()
    }
}
impl DeploymentCompose {
    pub fn builder() -> builder::DeploymentCompose {
        Default::default()
    }
}
///Devices passed through to the App instances
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Devices passed through to the App instances",
///  "type": "array",
///  "items": {
///    "examples": [
///      [
///        "/dev/net/tun"
///      ]
///    ],
///    "type": "string",
///    "pattern": "^(/dev/.+)$"
///  },
///  "$comment": "https://regex101.com/r/6utwD1/1"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Devices(pub Vec<DevicesItem>);
impl std::ops::Deref for Devices {
    type Target = Vec<DevicesItem>;
    fn deref(&self) -> &Vec<DevicesItem> {
        &self.0
    }
}
impl From<Devices> for Vec<DevicesItem> {
    fn from(value: Devices) -> Self {
        value.0
    }
}
impl From<&Devices> for Devices {
    fn from(value: &Devices) -> Self {
        value.clone()
    }
}
impl From<Vec<DevicesItem>> for Devices {
    fn from(value: Vec<DevicesItem>) -> Self {
        Self(value)
    }
}
///DevicesItem
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
pub struct DevicesItem(String);
impl std::ops::Deref for DevicesItem {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<DevicesItem> for String {
    fn from(value: DevicesItem) -> Self {
        value.0
    }
}
impl From<&DevicesItem> for DevicesItem {
    fn from(value: &DevicesItem) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for DevicesItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if regress::Regex::new("^(/dev/.+)$").unwrap().find(value).is_none() {
            return Err("doesn't match pattern \"^(/dev/.+)$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for DevicesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for DevicesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for DevicesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for DevicesItem {
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
///Set of web-based UIs of the app
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Set of web-based UIs of the app",
///  "examples": [
///    [
///      {
///        "name": "Example config UI",
///        "port": 5678,
///        "supportsReverseProxy": false
///      },
///      {
///        "name:": "Example log UI",
///        "port": 7890
///      }
///    ]
///  ],
///  "type": "array",
///  "items": {
///    "type": "object",
///    "required": [
///      "name",
///      "port"
///    ],
///    "properties": {
///      "name": {
///        "type": "string"
///      },
///      "port": {
///        "description": "Port on which the editor is reachable on the docker container",
///        "type": "integer",
///        "maximum": 65535.0,
///        "minimum": 1.0
///      },
///      "supportsReverseProxy": {
///        "default": true,
///        "type": "boolean"
///      }
///    }
///  }
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Editors(pub Vec<EditorsItem>);
impl std::ops::Deref for Editors {
    type Target = Vec<EditorsItem>;
    fn deref(&self) -> &Vec<EditorsItem> {
        &self.0
    }
}
impl From<Editors> for Vec<EditorsItem> {
    fn from(value: Editors) -> Self {
        value.0
    }
}
impl From<&Editors> for Editors {
    fn from(value: &Editors) -> Self {
        value.clone()
    }
}
impl From<Vec<EditorsItem>> for Editors {
    fn from(value: Vec<EditorsItem>) -> Self {
        Self(value)
    }
}
///EditorsItem
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
pub struct EditorsItem {
    pub name: String,
    ///Port on which the editor is reachable on the docker container
    pub port: std::num::NonZeroU16,
    #[serde(rename = "supportsReverseProxy", default = "defaults::default_bool::<true>")]
    pub supports_reverse_proxy: bool,
}
impl From<&EditorsItem> for EditorsItem {
    fn from(value: &EditorsItem) -> Self {
        value.clone()
    }
}
impl EditorsItem {
    pub fn builder() -> builder::EditorsItem {
        Default::default()
    }
}
///Environment variables for the App instances
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Environment variables for the App instances",
///  "type": "array",
///  "items": {
///    "examples": [
///      [
///        "MY_ENV=value",
///        "tech.flecs.some-app_value=any"
///      ]
///    ],
///    "type": "string",
///    "pattern": "^[a-zA-Z]+(?:[a-zA-Z0-9_\\-\\.])*=.*$"
///  },
///  "$comment": "https://regex101.com/r/MNDmTF/1"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Env(pub Vec<EnvItem>);
impl std::ops::Deref for Env {
    type Target = Vec<EnvItem>;
    fn deref(&self) -> &Vec<EnvItem> {
        &self.0
    }
}
impl From<Env> for Vec<EnvItem> {
    fn from(value: Env) -> Self {
        value.0
    }
}
impl From<&Env> for Env {
    fn from(value: &Env) -> Self {
        value.clone()
    }
}
impl From<Vec<EnvItem>> for Env {
    fn from(value: Vec<EnvItem>) -> Self {
        Self(value)
    }
}
///EnvItem
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
///  "pattern": "^[a-zA-Z]+(?:[a-zA-Z0-9_\\-\\.])*=.*$"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct EnvItem(String);
impl std::ops::Deref for EnvItem {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<EnvItem> for String {
    fn from(value: EnvItem) -> Self {
        value.0
    }
}
impl From<&EnvItem> for EnvItem {
    fn from(value: &EnvItem) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for EnvItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if regress::Regex::new("^[a-zA-Z]+(?:[a-zA-Z0-9_\\-\\.])*=.*$")
            .unwrap()
            .find(value)
            .is_none()
        {
            return Err(
                "doesn't match pattern \"^[a-zA-Z]+(?:[a-zA-Z0-9_\\-\\.])*=.*$\"".into(),
            );
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for EnvItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for EnvItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for EnvItem {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for EnvItem {
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
///Schema for the FLECS App Manifest
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "$id": "https://raw.githubusercontent.com/FLECS-Technologies/app-manifest/3.x/manifest.schema.json",
///  "title": "FLECS App Manifest",
///  "description": "Schema for the FLECS App Manifest",
///  "type": "object",
///  "oneOf": [
///    {
///      "$ref": "#/definitions/single"
///    },
///    {
///      "$ref": "#/definitions/multi"
///    }
///  ]
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum FlecsAppManifest {
    Single(Single),
    Multi(Multi),
}
impl From<&FlecsAppManifest> for FlecsAppManifest {
    fn from(value: &FlecsAppManifest) -> Self {
        value.clone()
    }
}
impl From<Single> for FlecsAppManifest {
    fn from(value: Single) -> Self {
        Self::Single(value)
    }
}
impl From<Multi> for FlecsAppManifest {
    fn from(value: Multi) -> Self {
        Self::Multi(value)
    }
}
///DEPRECATED: hostname of the started app, using this with multiInstance = true will cause problems
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "DEPRECATED: hostname of the started app, using this with multiInstance = true will cause problems",
///  "deprecated": true,
///  "type": "string"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Hostname(pub String);
impl std::ops::Deref for Hostname {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<Hostname> for String {
    fn from(value: Hostname) -> Self {
        value.0
    }
}
impl From<&Hostname> for Hostname {
    fn from(value: &Hostname) -> Self {
        value.clone()
    }
}
impl From<String> for Hostname {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl std::str::FromStr for Hostname {
    type Err = std::convert::Infallible;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(value.to_string()))
    }
}
impl ToString for Hostname {
    fn to_string(&self) -> String {
        self.0.to_string()
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
pub struct Image(String);
impl std::ops::Deref for Image {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<Image> for String {
    fn from(value: Image) -> Self {
        value.0
    }
}
impl From<&Image> for Image {
    fn from(value: &Image) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for Image {
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
impl std::convert::TryFrom<&str> for Image {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for Image {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for Image {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for Image {
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
///DEPRECATED: true if App requires allocation of an interactive shell
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "DEPRECATED: true if App requires allocation of an interactive shell",
///  "deprecated": true,
///  "type": "boolean"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Interactive(pub bool);
impl std::ops::Deref for Interactive {
    type Target = bool;
    fn deref(&self) -> &bool {
        &self.0
    }
}
impl From<Interactive> for bool {
    fn from(value: Interactive) -> Self {
        value.0
    }
}
impl From<&Interactive> for Interactive {
    fn from(value: &Interactive) -> Self {
        value.clone()
    }
}
impl From<bool> for Interactive {
    fn from(value: bool) -> Self {
        Self(value)
    }
}
impl std::str::FromStr for Interactive {
    type Err = <bool as std::str::FromStr>::Err;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(value.parse()?))
    }
}
impl std::convert::TryFrom<&str> for Interactive {
    type Error = <bool as std::str::FromStr>::Err;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for Interactive {
    type Error = <bool as std::str::FromStr>::Err;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for Interactive {
    type Error = <bool as std::str::FromStr>::Err;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}
impl ToString for Interactive {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}
///Labels for the App instances
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Labels for the App instances",
///  "type": "array",
///  "items": {
///    "examples": [
///      [
///        "tech.flecs",
///        "tech.flecs.some-label=Some custom label value"
///      ]
///    ],
///    "type": "string",
///    "pattern": "^[a-z](?:(?:[\\-\\.]?[a-zA-Z0-9])*[\\-\\.]?[a-z])?(?:=.*)?$"
///  },
///  "$comment": "https://regex101.com/r/xOiJXu/1"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Labels(pub Vec<LabelsItem>);
impl std::ops::Deref for Labels {
    type Target = Vec<LabelsItem>;
    fn deref(&self) -> &Vec<LabelsItem> {
        &self.0
    }
}
impl From<Labels> for Vec<LabelsItem> {
    fn from(value: Labels) -> Self {
        value.0
    }
}
impl From<&Labels> for Labels {
    fn from(value: &Labels) -> Self {
        value.clone()
    }
}
impl From<Vec<LabelsItem>> for Labels {
    fn from(value: Vec<LabelsItem>) -> Self {
        Self(value)
    }
}
///LabelsItem
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
pub struct LabelsItem(String);
impl std::ops::Deref for LabelsItem {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<LabelsItem> for String {
    fn from(value: LabelsItem) -> Self {
        value.0
    }
}
impl From<&LabelsItem> for LabelsItem {
    fn from(value: &LabelsItem) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for LabelsItem {
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
impl std::convert::TryFrom<&str> for LabelsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for LabelsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for LabelsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for LabelsItem {
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
pub struct MinimumFlecsVersion(String);
impl std::ops::Deref for MinimumFlecsVersion {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<MinimumFlecsVersion> for String {
    fn from(value: MinimumFlecsVersion) -> Self {
        value.0
    }
}
impl From<&MinimumFlecsVersion> for MinimumFlecsVersion {
    fn from(value: &MinimumFlecsVersion) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for MinimumFlecsVersion {
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
impl std::convert::TryFrom<&str> for MinimumFlecsVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for MinimumFlecsVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for MinimumFlecsVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for MinimumFlecsVersion {
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
///Multi
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "type": "object",
///  "required": [
///    "app",
///    "deployment",
///    "version"
///  ],
///  "properties": {
///    "$schema": {
///      "$ref": "#/definitions/$schema"
///    },
///    "_minimumFlecsVersion": {
///      "$ref": "#/definitions/_minimumFlecsVersion"
///    },
///    "app": {
///      "$ref": "#/definitions/app"
///    },
///    "depends": {
///      "$ref": "#/definitions/depends"
///    },
///    "deployment": {
///      "$ref": "#/definitions/deployment"
///    },
///    "provides": {
///      "$ref": "#/definitions/provides"
///    },
///    "recommends": {
///      "$ref": "#/definitions/recommends"
///    },
///    "revision": {
///      "$ref": "#/definitions/revision"
///    },
///    "version": {
///      "$ref": "#/definitions/version"
///    }
///  },
///  "additionalProperties": false
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Multi {
    pub app: App,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depends: Option<Depends>,
    pub deployment: Deployment,
    #[serde(
        rename = "_minimumFlecsVersion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub minimum_flecs_version: Option<MinimumFlecsVersion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provides: Option<Provides>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommends: Option<Recommends>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<Revision>,
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,
    pub version: Version,
}
impl From<&Multi> for Multi {
    fn from(value: &Multi) -> Self {
        value.clone()
    }
}
impl Multi {
    pub fn builder() -> builder::Multi {
        Default::default()
    }
}
///'true' if App can be instantiated more than once (requires no editor, no ports)
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "'true' if App can be instantiated more than once (requires no editor, no ports)",
///  "examples": [
///    true,
///    false
///  ],
///  "type": "boolean"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MultiInstance(pub bool);
impl std::ops::Deref for MultiInstance {
    type Target = bool;
    fn deref(&self) -> &bool {
        &self.0
    }
}
impl From<MultiInstance> for bool {
    fn from(value: MultiInstance) -> Self {
        value.0
    }
}
impl From<&MultiInstance> for MultiInstance {
    fn from(value: &MultiInstance) -> Self {
        value.clone()
    }
}
impl From<bool> for MultiInstance {
    fn from(value: bool) -> Self {
        Self(value)
    }
}
impl std::str::FromStr for MultiInstance {
    type Err = <bool as std::str::FromStr>::Err;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(value.parse()?))
    }
}
impl std::convert::TryFrom<&str> for MultiInstance {
    type Error = <bool as std::str::FromStr>::Err;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for MultiInstance {
    type Error = <bool as std::str::FromStr>::Err;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for MultiInstance {
    type Error = <bool as std::str::FromStr>::Err;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}
impl ToString for MultiInstance {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}
///Port mappings for the App's instances (not allowed for multiInstance Apps)
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Port mappings for the App's instances (not allowed for multiInstance Apps)",
///  "type": "array",
///  "items": {
///    "examples": [
///      [
///        "8001:8001",
///        "5000",
///        "5001-5008:6001-6008",
///        "6001-6008"
///      ]
///    ],
///    "type": "string",
///    "pattern": "(?=\\d|:)^(?:([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3})(?:$|[:-](?=\\d))|:)?(?:(?<=-)([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3})(?:$|:(?=\\d)))?(?:(?<=:)([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3})(?:$|-(?=\\d)))?(?:(?<=-)([6][5][5][3][0-5]|[6][5][5][0-2][0-9]|[6][5][0-4][0-9]{2}|[6][0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{0,3}))?$"
///  },
///  "$comment": "https://regex101.com/r/GgJ78T/1"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Ports(pub Vec<PortsItem>);
impl std::ops::Deref for Ports {
    type Target = Vec<PortsItem>;
    fn deref(&self) -> &Vec<PortsItem> {
        &self.0
    }
}
impl From<Ports> for Vec<PortsItem> {
    fn from(value: Ports) -> Self {
        value.0
    }
}
impl From<&Ports> for Ports {
    fn from(value: &Ports) -> Self {
        value.clone()
    }
}
impl From<Vec<PortsItem>> for Ports {
    fn from(value: Vec<PortsItem>) -> Self {
        Self(value)
    }
}
///PortsItem
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
pub struct PortsItem(String);
impl std::ops::Deref for PortsItem {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<PortsItem> for String {
    fn from(value: PortsItem) -> Self {
        value.0
    }
}
impl From<&PortsItem> for PortsItem {
    fn from(value: &PortsItem) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for PortsItem {
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
impl std::convert::TryFrom<&str> for PortsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for PortsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for PortsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for PortsItem {
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
///Provides
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "type": "object",
///  "additionalProperties": true,
///  "propertyNames": {
///    "pattern": "^([\\w.-]+)$"
///  }
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Provides(pub std::collections::HashMap<ProvidesKey, serde_json::Value>);
impl std::ops::Deref for Provides {
    type Target = std::collections::HashMap<ProvidesKey, serde_json::Value>;
    fn deref(&self) -> &std::collections::HashMap<ProvidesKey, serde_json::Value> {
        &self.0
    }
}
impl From<Provides> for std::collections::HashMap<ProvidesKey, serde_json::Value> {
    fn from(value: Provides) -> Self {
        value.0
    }
}
impl From<&Provides> for Provides {
    fn from(value: &Provides) -> Self {
        value.clone()
    }
}
impl From<std::collections::HashMap<ProvidesKey, serde_json::Value>> for Provides {
    fn from(value: std::collections::HashMap<ProvidesKey, serde_json::Value>) -> Self {
        Self(value)
    }
}
///ProvidesKey
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "type": "string",
///  "pattern": "^([\\w.-]+)$"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ProvidesKey(String);
impl std::ops::Deref for ProvidesKey {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<ProvidesKey> for String {
    fn from(value: ProvidesKey) -> Self {
        value.0
    }
}
impl From<&ProvidesKey> for ProvidesKey {
    fn from(value: &ProvidesKey) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for ProvidesKey {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if regress::Regex::new("^([\\w.-]+)$").unwrap().find(value).is_none() {
            return Err("doesn't match pattern \"^([\\w.-]+)$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for ProvidesKey {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for ProvidesKey {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for ProvidesKey {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for ProvidesKey {
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
///Recommends
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "type": "object",
///  "additionalProperties": true,
///  "propertyNames": {
///    "pattern": "^([\\w.-]+)(?:\\s*\\|\\s*([\\w.-]+))*$"
///  }
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Recommends(pub std::collections::HashMap<RecommendsKey, serde_json::Value>);
impl std::ops::Deref for Recommends {
    type Target = std::collections::HashMap<RecommendsKey, serde_json::Value>;
    fn deref(&self) -> &std::collections::HashMap<RecommendsKey, serde_json::Value> {
        &self.0
    }
}
impl From<Recommends> for std::collections::HashMap<RecommendsKey, serde_json::Value> {
    fn from(value: Recommends) -> Self {
        value.0
    }
}
impl From<&Recommends> for Recommends {
    fn from(value: &Recommends) -> Self {
        value.clone()
    }
}
impl From<std::collections::HashMap<RecommendsKey, serde_json::Value>> for Recommends {
    fn from(value: std::collections::HashMap<RecommendsKey, serde_json::Value>) -> Self {
        Self(value)
    }
}
///RecommendsKey
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "type": "string",
///  "pattern": "^([\\w.-]+)(?:\\s*\\|\\s*([\\w.-]+))*$"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RecommendsKey(String);
impl std::ops::Deref for RecommendsKey {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<RecommendsKey> for String {
    fn from(value: RecommendsKey) -> Self {
        value.0
    }
}
impl From<&RecommendsKey> for RecommendsKey {
    fn from(value: &RecommendsKey) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for RecommendsKey {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if regress::Regex::new("^([\\w.-]+)(?:\\s*\\|\\s*([\\w.-]+))*$")
            .unwrap()
            .find(value)
            .is_none()
        {
            return Err(
                "doesn't match pattern \"^([\\w.-]+)(?:\\s*\\|\\s*([\\w.-]+))*$\"".into(),
            );
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for RecommendsKey {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for RecommendsKey {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for RecommendsKey {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for RecommendsKey {
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
///App manifest revision. Increment if Manifest is changed within the same App version
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "App manifest revision. Increment if Manifest is changed within the same App version",
///  "examples": [
///    "0",
///    "1",
///    "2"
///  ],
///  "type": "string"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Revision(pub String);
impl std::ops::Deref for Revision {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<Revision> for String {
    fn from(value: Revision) -> Self {
        value.0
    }
}
impl From<&Revision> for Revision {
    fn from(value: &Revision) -> Self {
        value.clone()
    }
}
impl From<String> for Revision {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl std::str::FromStr for Revision {
    type Err = std::convert::Infallible;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(value.to_string()))
    }
}
impl ToString for Revision {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}
///Location of the JSON schema to validate against
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Location of the JSON schema to validate against",
///  "examples": [
///    "https://raw.githubusercontent.com/FLECS-Technologies/app-manifest/2.x/manifest.schema.json",
///    "https://raw.githubusercontent.com/FLECS-Technologies/app-manifest/3.x/manifest.schema.json"
///  ],
///  "type": "string"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Schema(pub String);
impl std::ops::Deref for Schema {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<Schema> for String {
    fn from(value: Schema) -> Self {
        value.0
    }
}
impl From<&Schema> for Schema {
    fn from(value: &Schema) -> Self {
        value.clone()
    }
}
impl From<String> for Schema {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl std::str::FromStr for Schema {
    type Err = std::convert::Infallible;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(value.to_string()))
    }
}
impl ToString for Schema {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}
///Version of the implemented FLECS App Manifest schema
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Version of the implemented FLECS App Manifest schema",
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
pub struct SchemaVersion(String);
impl std::ops::Deref for SchemaVersion {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<SchemaVersion> for String {
    fn from(value: SchemaVersion) -> Self {
        value.0
    }
}
impl From<&SchemaVersion> for SchemaVersion {
    fn from(value: &SchemaVersion) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for SchemaVersion {
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
impl std::convert::TryFrom<&str> for SchemaVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for SchemaVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for SchemaVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for SchemaVersion {
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
///Single
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "type": "object",
///  "required": [
///    "app",
///    "image",
///    "version"
///  ],
///  "properties": {
///    "$schema": {
///      "$ref": "#/definitions/$schema"
///    },
///    "_minimumFlecsVersion": {
///      "$ref": "#/definitions/_minimumFlecsVersion"
///    },
///    "app": {
///      "$ref": "#/definitions/app"
///    },
///    "args": {
///      "$ref": "#/definitions/args"
///    },
///    "capabilities": {
///      "$ref": "#/definitions/capabilities"
///    },
///    "conffiles": {
///      "$ref": "#/definitions/conffiles"
///    },
///    "depends": {
///      "$ref": "#/definitions/depends"
///    },
///    "devices": {
///      "$ref": "#/definitions/devices"
///    },
///    "editors": {
///      "$ref": "#/definitions/editors"
///    },
///    "env": {
///      "$ref": "#/definitions/env"
///    },
///    "hostname": {
///      "$ref": "#/definitions/hostname"
///    },
///    "image": {
///      "$ref": "#/definitions/image"
///    },
///    "interactive": {
///      "$ref": "#/definitions/interactive"
///    },
///    "labels": {
///      "$ref": "#/definitions/labels"
///    },
///    "multiInstance": {
///      "$ref": "#/definitions/multiInstance"
///    },
///    "ports": {
///      "$ref": "#/definitions/ports"
///    },
///    "provides": {
///      "$ref": "#/definitions/provides"
///    },
///    "recommends": {
///      "$ref": "#/definitions/recommends"
///    },
///    "revision": {
///      "$ref": "#/definitions/revision"
///    },
///    "version": {
///      "$ref": "#/definitions/version"
///    },
///    "volumes": {
///      "$ref": "#/definitions/volumes"
///    }
///  },
///  "additionalProperties": false
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Single {
    pub app: App,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Args>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Capabilities>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conffiles: Option<Conffiles>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depends: Option<Depends>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub devices: Option<Devices>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editors: Option<Editors>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<Env>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hostname: Option<Hostname>,
    pub image: Image,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interactive: Option<Interactive>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<Labels>,
    #[serde(
        rename = "_minimumFlecsVersion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub minimum_flecs_version: Option<MinimumFlecsVersion>,
    #[serde(rename = "multiInstance", default, skip_serializing_if = "Option::is_none")]
    pub multi_instance: Option<MultiInstance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ports: Option<Ports>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provides: Option<Provides>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommends: Option<Recommends>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<Revision>,
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,
    pub version: Version,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volumes: Option<Volumes>,
}
impl From<&Single> for Single {
    fn from(value: &Single) -> Self {
        value.clone()
    }
}
impl Single {
    pub fn builder() -> builder::Single {
        Default::default()
    }
}
///App version, naturally sortable
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "App version, naturally sortable",
///  "examples": [
///    "1.0.0",
///    "2022-12",
///    "v3.14.159-alpha.2",
///    "version 21"
///  ],
///  "type": "string"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Version(pub String);
impl std::ops::Deref for Version {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<Version> for String {
    fn from(value: Version) -> Self {
        value.0
    }
}
impl From<&Version> for Version {
    fn from(value: &Version) -> Self {
        value.clone()
    }
}
impl From<String> for Version {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl std::str::FromStr for Version {
    type Err = std::convert::Infallible;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(value.to_string()))
    }
}
impl ToString for Version {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}
///Virtual volumes and bind mounts for an App's instances
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "description": "Virtual volumes and bind mounts for an App's instances",
///  "type": "array",
///  "items": {
///    "examples": [
///      [
///        "my-app-etc:/etc/my-app",
///        "/etc/my-app:/etc/my-app"
///      ]
///    ],
///    "type": "string",
///    "pattern": "(?:^([a-zA-Z0-9\\-_.]+)|^/[a-zA-Z0-9\\-_./]+):([a-zA-Z0-9\\-_./]+)$",
///    "$comment": "https://regex101.com/r/WjJro3/1"
///  }
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Volumes(pub Vec<VolumesItem>);
impl std::ops::Deref for Volumes {
    type Target = Vec<VolumesItem>;
    fn deref(&self) -> &Vec<VolumesItem> {
        &self.0
    }
}
impl From<Volumes> for Vec<VolumesItem> {
    fn from(value: Volumes) -> Self {
        value.0
    }
}
impl From<&Volumes> for Volumes {
    fn from(value: &Volumes) -> Self {
        value.clone()
    }
}
impl From<Vec<VolumesItem>> for Volumes {
    fn from(value: Vec<VolumesItem>) -> Self {
        Self(value)
    }
}
///VolumesItem
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
pub struct VolumesItem(String);
impl std::ops::Deref for VolumesItem {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<VolumesItem> for String {
    fn from(value: VolumesItem) -> Self {
        value.0
    }
}
impl From<&VolumesItem> for VolumesItem {
    fn from(value: &VolumesItem) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for VolumesItem {
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
impl std::convert::TryFrom<&str> for VolumesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for VolumesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for VolumesItem {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for VolumesItem {
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
    pub struct Deployment {
        compose: Result<super::DeploymentCompose, String>,
    }
    impl Default for Deployment {
        fn default() -> Self {
            Self {
                compose: Err("no value supplied for compose".to_string()),
            }
        }
    }
    impl Deployment {
        pub fn compose<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<super::DeploymentCompose>,
            T::Error: std::fmt::Display,
        {
            self.compose = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for compose: {}", e)
                });
            self
        }
    }
    impl std::convert::TryFrom<Deployment> for super::Deployment {
        type Error = super::error::ConversionError;
        fn try_from(value: Deployment) -> Result<Self, super::error::ConversionError> {
            Ok(Self { compose: value.compose? })
        }
    }
    impl From<super::Deployment> for Deployment {
        fn from(value: super::Deployment) -> Self {
            Self { compose: Ok(value.compose) }
        }
    }
    #[derive(Clone, Debug)]
    pub struct DeploymentCompose {
        yaml: Result<serde_json::Map<String, serde_json::Value>, String>,
    }
    impl Default for DeploymentCompose {
        fn default() -> Self {
            Self {
                yaml: Ok(Default::default()),
            }
        }
    }
    impl DeploymentCompose {
        pub fn yaml<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<serde_json::Map<String, serde_json::Value>>,
            T::Error: std::fmt::Display,
        {
            self.yaml = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for yaml: {}", e));
            self
        }
    }
    impl std::convert::TryFrom<DeploymentCompose> for super::DeploymentCompose {
        type Error = super::error::ConversionError;
        fn try_from(
            value: DeploymentCompose,
        ) -> Result<Self, super::error::ConversionError> {
            Ok(Self { yaml: value.yaml? })
        }
    }
    impl From<super::DeploymentCompose> for DeploymentCompose {
        fn from(value: super::DeploymentCompose) -> Self {
            Self { yaml: Ok(value.yaml) }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EditorsItem {
        name: Result<String, String>,
        port: Result<std::num::NonZeroU16, String>,
        supports_reverse_proxy: Result<bool, String>,
    }
    impl Default for EditorsItem {
        fn default() -> Self {
            Self {
                name: Err("no value supplied for name".to_string()),
                port: Err("no value supplied for port".to_string()),
                supports_reverse_proxy: Ok(super::defaults::default_bool::<true>()),
            }
        }
    }
    impl EditorsItem {
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
    impl std::convert::TryFrom<EditorsItem> for super::EditorsItem {
        type Error = super::error::ConversionError;
        fn try_from(value: EditorsItem) -> Result<Self, super::error::ConversionError> {
            Ok(Self {
                name: value.name?,
                port: value.port?,
                supports_reverse_proxy: value.supports_reverse_proxy?,
            })
        }
    }
    impl From<super::EditorsItem> for EditorsItem {
        fn from(value: super::EditorsItem) -> Self {
            Self {
                name: Ok(value.name),
                port: Ok(value.port),
                supports_reverse_proxy: Ok(value.supports_reverse_proxy),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Multi {
        app: Result<super::App, String>,
        depends: Result<Option<super::Depends>, String>,
        deployment: Result<super::Deployment, String>,
        minimum_flecs_version: Result<Option<super::MinimumFlecsVersion>, String>,
        provides: Result<Option<super::Provides>, String>,
        recommends: Result<Option<super::Recommends>, String>,
        revision: Result<Option<super::Revision>, String>,
        schema: Result<Option<super::Schema>, String>,
        version: Result<super::Version, String>,
    }
    impl Default for Multi {
        fn default() -> Self {
            Self {
                app: Err("no value supplied for app".to_string()),
                depends: Ok(Default::default()),
                deployment: Err("no value supplied for deployment".to_string()),
                minimum_flecs_version: Ok(Default::default()),
                provides: Ok(Default::default()),
                recommends: Ok(Default::default()),
                revision: Ok(Default::default()),
                schema: Ok(Default::default()),
                version: Err("no value supplied for version".to_string()),
            }
        }
    }
    impl Multi {
        pub fn app<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<super::App>,
            T::Error: std::fmt::Display,
        {
            self.app = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for app: {}", e));
            self
        }
        pub fn depends<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<super::Depends>>,
            T::Error: std::fmt::Display,
        {
            self.depends = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for depends: {}", e)
                });
            self
        }
        pub fn deployment<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<super::Deployment>,
            T::Error: std::fmt::Display,
        {
            self.deployment = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for deployment: {}", e)
                });
            self
        }
        pub fn minimum_flecs_version<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<super::MinimumFlecsVersion>>,
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
        pub fn provides<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<super::Provides>>,
            T::Error: std::fmt::Display,
        {
            self.provides = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for provides: {}", e)
                });
            self
        }
        pub fn recommends<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<super::Recommends>>,
            T::Error: std::fmt::Display,
        {
            self.recommends = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for recommends: {}", e)
                });
            self
        }
        pub fn revision<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<super::Revision>>,
            T::Error: std::fmt::Display,
        {
            self.revision = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for revision: {}", e)
                });
            self
        }
        pub fn schema<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<super::Schema>>,
            T::Error: std::fmt::Display,
        {
            self.schema = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for schema: {}", e)
                });
            self
        }
        pub fn version<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<super::Version>,
            T::Error: std::fmt::Display,
        {
            self.version = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for version: {}", e)
                });
            self
        }
    }
    impl std::convert::TryFrom<Multi> for super::Multi {
        type Error = super::error::ConversionError;
        fn try_from(value: Multi) -> Result<Self, super::error::ConversionError> {
            Ok(Self {
                app: value.app?,
                depends: value.depends?,
                deployment: value.deployment?,
                minimum_flecs_version: value.minimum_flecs_version?,
                provides: value.provides?,
                recommends: value.recommends?,
                revision: value.revision?,
                schema: value.schema?,
                version: value.version?,
            })
        }
    }
    impl From<super::Multi> for Multi {
        fn from(value: super::Multi) -> Self {
            Self {
                app: Ok(value.app),
                depends: Ok(value.depends),
                deployment: Ok(value.deployment),
                minimum_flecs_version: Ok(value.minimum_flecs_version),
                provides: Ok(value.provides),
                recommends: Ok(value.recommends),
                revision: Ok(value.revision),
                schema: Ok(value.schema),
                version: Ok(value.version),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Single {
        app: Result<super::App, String>,
        args: Result<Option<super::Args>, String>,
        capabilities: Result<Option<super::Capabilities>, String>,
        conffiles: Result<Option<super::Conffiles>, String>,
        depends: Result<Option<super::Depends>, String>,
        devices: Result<Option<super::Devices>, String>,
        editors: Result<Option<super::Editors>, String>,
        env: Result<Option<super::Env>, String>,
        hostname: Result<Option<super::Hostname>, String>,
        image: Result<super::Image, String>,
        interactive: Result<Option<super::Interactive>, String>,
        labels: Result<Option<super::Labels>, String>,
        minimum_flecs_version: Result<Option<super::MinimumFlecsVersion>, String>,
        multi_instance: Result<Option<super::MultiInstance>, String>,
        ports: Result<Option<super::Ports>, String>,
        provides: Result<Option<super::Provides>, String>,
        recommends: Result<Option<super::Recommends>, String>,
        revision: Result<Option<super::Revision>, String>,
        schema: Result<Option<super::Schema>, String>,
        version: Result<super::Version, String>,
        volumes: Result<Option<super::Volumes>, String>,
    }
    impl Default for Single {
        fn default() -> Self {
            Self {
                app: Err("no value supplied for app".to_string()),
                args: Ok(Default::default()),
                capabilities: Ok(Default::default()),
                conffiles: Ok(Default::default()),
                depends: Ok(Default::default()),
                devices: Ok(Default::default()),
                editors: Ok(Default::default()),
                env: Ok(Default::default()),
                hostname: Ok(Default::default()),
                image: Err("no value supplied for image".to_string()),
                interactive: Ok(Default::default()),
                labels: Ok(Default::default()),
                minimum_flecs_version: Ok(Default::default()),
                multi_instance: Ok(Default::default()),
                ports: Ok(Default::default()),
                provides: Ok(Default::default()),
                recommends: Ok(Default::default()),
                revision: Ok(Default::default()),
                schema: Ok(Default::default()),
                version: Err("no value supplied for version".to_string()),
                volumes: Ok(Default::default()),
            }
        }
    }
    impl Single {
        pub fn app<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<super::App>,
            T::Error: std::fmt::Display,
        {
            self.app = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for app: {}", e));
            self
        }
        pub fn args<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<super::Args>>,
            T::Error: std::fmt::Display,
        {
            self.args = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for args: {}", e));
            self
        }
        pub fn capabilities<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<super::Capabilities>>,
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
            T: std::convert::TryInto<Option<super::Conffiles>>,
            T::Error: std::fmt::Display,
        {
            self.conffiles = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for conffiles: {}", e)
                });
            self
        }
        pub fn depends<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<super::Depends>>,
            T::Error: std::fmt::Display,
        {
            self.depends = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for depends: {}", e)
                });
            self
        }
        pub fn devices<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<super::Devices>>,
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
            T: std::convert::TryInto<Option<super::Editors>>,
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
            T: std::convert::TryInto<Option<super::Env>>,
            T::Error: std::fmt::Display,
        {
            self.env = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for env: {}", e));
            self
        }
        pub fn hostname<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<super::Hostname>>,
            T::Error: std::fmt::Display,
        {
            self.hostname = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for hostname: {}", e)
                });
            self
        }
        pub fn image<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<super::Image>,
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
            T: std::convert::TryInto<Option<super::Interactive>>,
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
            T: std::convert::TryInto<Option<super::Labels>>,
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
            T: std::convert::TryInto<Option<super::MinimumFlecsVersion>>,
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
            T: std::convert::TryInto<Option<super::MultiInstance>>,
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
            T: std::convert::TryInto<Option<super::Ports>>,
            T::Error: std::fmt::Display,
        {
            self.ports = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for ports: {}", e)
                });
            self
        }
        pub fn provides<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<super::Provides>>,
            T::Error: std::fmt::Display,
        {
            self.provides = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for provides: {}", e)
                });
            self
        }
        pub fn recommends<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<super::Recommends>>,
            T::Error: std::fmt::Display,
        {
            self.recommends = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for recommends: {}", e)
                });
            self
        }
        pub fn revision<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<super::Revision>>,
            T::Error: std::fmt::Display,
        {
            self.revision = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for revision: {}", e)
                });
            self
        }
        pub fn schema<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<Option<super::Schema>>,
            T::Error: std::fmt::Display,
        {
            self.schema = value
                .try_into()
                .map_err(|e| {
                    format!("error converting supplied value for schema: {}", e)
                });
            self
        }
        pub fn version<T>(mut self, value: T) -> Self
        where
            T: std::convert::TryInto<super::Version>,
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
            T: std::convert::TryInto<Option<super::Volumes>>,
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
    impl std::convert::TryFrom<Single> for super::Single {
        type Error = super::error::ConversionError;
        fn try_from(value: Single) -> Result<Self, super::error::ConversionError> {
            Ok(Self {
                app: value.app?,
                args: value.args?,
                capabilities: value.capabilities?,
                conffiles: value.conffiles?,
                depends: value.depends?,
                devices: value.devices?,
                editors: value.editors?,
                env: value.env?,
                hostname: value.hostname?,
                image: value.image?,
                interactive: value.interactive?,
                labels: value.labels?,
                minimum_flecs_version: value.minimum_flecs_version?,
                multi_instance: value.multi_instance?,
                ports: value.ports?,
                provides: value.provides?,
                recommends: value.recommends?,
                revision: value.revision?,
                schema: value.schema?,
                version: value.version?,
                volumes: value.volumes?,
            })
        }
    }
    impl From<super::Single> for Single {
        fn from(value: super::Single) -> Self {
            Self {
                app: Ok(value.app),
                args: Ok(value.args),
                capabilities: Ok(value.capabilities),
                conffiles: Ok(value.conffiles),
                depends: Ok(value.depends),
                devices: Ok(value.devices),
                editors: Ok(value.editors),
                env: Ok(value.env),
                hostname: Ok(value.hostname),
                image: Ok(value.image),
                interactive: Ok(value.interactive),
                labels: Ok(value.labels),
                minimum_flecs_version: Ok(value.minimum_flecs_version),
                multi_instance: Ok(value.multi_instance),
                ports: Ok(value.ports),
                provides: Ok(value.provides),
                recommends: Ok(value.recommends),
                revision: Ok(value.revision),
                schema: Ok(value.schema),
                version: Ok(value.version),
                volumes: Ok(value.volumes),
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