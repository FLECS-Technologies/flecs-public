use crate::forge::serde::{EnvFilterWrapper, UriWrapper};
use crate::jeweler::network::NetworkKind;
use crate::lore::{
    AppLore, ConsoleLore, DeploymentLore, ExportLore, FloxyLore, ImportLore, InstanceLore, Lore,
    ManifestLore, NetworkLore, SecretLore,
};
use crate::relic::network::Ipv4Network;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;
use tracing::debug;

pub trait Mergeable {
    fn merge(&mut self, other: Self);
}

pub trait TriviallyMergeable {
    fn trivial_merge(&mut self, other: Self);
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("transparent")]
    TomlSer(#[from] toml::ser::Error),
    #[error("transparent")]
    TomlDe(#[from] toml::de::Error),
    #[error("transparent")]
    Json(#[from] serde_json::Error),
    #[error("transparent")]
    IO(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FlecsConfig {
    pub version: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracing_filter: Option<EnvFilterWrapper>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flecsd_socket_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<ExportConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import: Option<ImportConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floxy: Option<FloxyConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub console: Option<ConsoleConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<InstanceConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<AppConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment: Option<DeploymentConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest: Option<ManifestConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<SecretConfig>,
}

impl Default for FlecsConfig {
    fn default() -> Self {
        Self {
            version: 1,
            tracing_filter: None,
            base_path: None,
            flecsd_socket_path: None,
            export: None,
            import: None,
            floxy: None,
            console: None,
            instance: None,
            network: None,
            app: None,
            deployment: None,
            manifest: None,
            secret: None,
        }
    }
}

impl From<&Lore> for FlecsConfig {
    fn from(value: &Lore) -> Self {
        Self {
            version: 1,
            tracing_filter: Some(EnvFilterWrapper(
                tracing_subscriber::EnvFilter::from_str(&value.tracing_filter.to_string())
                    .expect("String from existing EnvFilter should be valid"),
            )),
            base_path: Some(value.base_path.clone()),
            flecsd_socket_path: Some(value.flecsd_socket_path.clone()),
            export: Some((&value.export).into()),
            import: Some((&value.import).into()),
            floxy: Some((&value.floxy).into()),
            console: Some((&value.console).into()),
            instance: Some((&value.instance).into()),
            network: Some((&value.network).into()),
            app: Some((&value.app).into()),
            deployment: Some((&value.deployment).into()),
            manifest: Some((&value.manifest).into()),
            secret: Some((&value.secret).into()),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExportConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

impl From<&ExportLore> for ExportConfig {
    fn from(value: &ExportLore) -> Self {
        Self {
            base_path: Some(value.base_path.clone()),
            timeout: Some(value.timeout.as_secs()),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

impl From<&ImportLore> for ImportConfig {
    fn from(value: &ImportLore) -> Self {
        Self {
            base_path: Some(value.base_path.clone()),
            timeout: Some(value.timeout.as_secs()),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FloxyConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<PathBuf>,
}

impl From<&FloxyLore> for FloxyConfig {
    fn from(value: &FloxyLore) -> Self {
        Self {
            base_path: Some(value.base_path.clone()),
            config_path: Some(value.config_path.clone()),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsoleConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<UriWrapper>,
}

impl From<&ConsoleLore> for ConsoleConfig {
    fn from(value: &ConsoleLore) -> Self {
        Self {
            uri: Some(UriWrapper(value.uri.clone())),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstanceConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_path: Option<PathBuf>,
}

impl From<&InstanceLore> for InstanceConfig {
    fn from(value: &InstanceLore) -> Self {
        Self {
            base_path: Some(value.base_path.clone()),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_network_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_cidr_subnet: Option<Ipv4Network>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_gateway: Option<Ipv4Addr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_options: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_parent_adapter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_network_kind: Option<NetworkKind>,
}

impl From<&NetworkLore> for NetworkConfig {
    fn from(value: &NetworkLore) -> Self {
        Self {
            default_network_name: Some(value.default_network_name.clone()),
            default_cidr_subnet: Some(value.default_cidr_subnet),
            default_gateway: Some(value.default_gateway),
            default_options: Some(value.default_options.clone()),
            default_parent_adapter: if value.default_parent_adapter.is_some() {
                value.default_parent_adapter.clone()
            } else {
                Some(String::new())
            },
            default_network_kind: Some(value.default_network_kind),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_path: Option<PathBuf>,
}

impl From<&AppLore> for AppConfig {
    fn from(value: &AppLore) -> Self {
        Self {
            base_path: Some(value.base_path.clone()),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeploymentConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_path: Option<PathBuf>,
}

impl From<&DeploymentLore> for DeploymentConfig {
    fn from(value: &DeploymentLore) -> Self {
        Self {
            base_path: Some(value.base_path.clone()),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_path: Option<PathBuf>,
}

impl From<&ManifestLore> for ManifestConfig {
    fn from(value: &ManifestLore) -> Self {
        Self {
            base_path: Some(value.base_path.clone()),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecretConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_path: Option<PathBuf>,
}

impl From<&SecretLore> for SecretConfig {
    fn from(value: &SecretLore) -> Self {
        Self {
            base_path: Some(value.base_path.clone()),
        }
    }
}

impl FlecsConfig {
    pub async fn from_path(path: &Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Ok(match path.extension().and_then(OsStr::to_str) {
            Some("json") => serde_json::from_str(&content)?,
            _ => toml::from_str(&content)?,
        })
    }

    pub async fn to_path(&self, path: &Path) -> Result<()> {
        let content = match path.extension().and_then(OsStr::to_str) {
            Some("json") => serde_json::to_string_pretty(&self)?,
            _ => toml::to_string_pretty(&self)?,
        };

        tokio::fs::write(path, &content).await?;
        debug!("Written config to {}: {content}", path.display());
        Ok(())
    }
}

impl Mergeable for FlecsConfig {
    fn merge(&mut self, other: Self) {
        self.tracing_filter.trivial_merge(other.tracing_filter);
        self.base_path.trivial_merge(other.base_path);
        self.flecsd_socket_path
            .trivial_merge(other.flecsd_socket_path);
        self.app.merge(other.app);
        self.console.merge(other.console);
        self.deployment.merge(other.deployment);
        self.export.merge(other.export);
        self.floxy.merge(other.floxy);
        self.import.merge(other.import);
        self.instance.merge(other.instance);
        self.manifest.merge(other.manifest);
        self.network.merge(other.network);
        self.secret.merge(other.secret);
    }
}

impl Mergeable for AppConfig {
    fn merge(&mut self, other: Self) {
        self.base_path.trivial_merge(other.base_path)
    }
}

impl Mergeable for ConsoleConfig {
    fn merge(&mut self, other: Self) {
        self.uri.trivial_merge(other.uri)
    }
}

impl Mergeable for DeploymentConfig {
    fn merge(&mut self, other: Self) {
        self.base_path.trivial_merge(other.base_path)
    }
}

impl Mergeable for ExportConfig {
    fn merge(&mut self, other: Self) {
        self.base_path.trivial_merge(other.base_path);
        self.timeout.trivial_merge(other.timeout);
    }
}

impl Mergeable for FloxyConfig {
    fn merge(&mut self, other: Self) {
        self.base_path.trivial_merge(other.base_path);
        self.config_path.trivial_merge(other.config_path);
    }
}
impl Mergeable for ImportConfig {
    fn merge(&mut self, other: Self) {
        self.base_path.trivial_merge(other.base_path);
        self.timeout.trivial_merge(other.timeout);
    }
}
impl Mergeable for InstanceConfig {
    fn merge(&mut self, other: Self) {
        self.base_path.trivial_merge(other.base_path);
    }
}
impl Mergeable for ManifestConfig {
    fn merge(&mut self, other: Self) {
        self.base_path.trivial_merge(other.base_path);
    }
}
impl Mergeable for NetworkConfig {
    fn merge(&mut self, other: Self) {
        self.default_cidr_subnet
            .trivial_merge(other.default_cidr_subnet);
        self.default_gateway.trivial_merge(other.default_gateway);
        self.default_network_kind
            .trivial_merge(other.default_network_kind);
        self.default_network_name
            .trivial_merge(other.default_network_name);
        self.default_options.trivial_merge(other.default_options);
        (self)
            .default_parent_adapter
            .trivial_merge(other.default_parent_adapter);
    }
}

impl Mergeable for SecretConfig {
    fn merge(&mut self, other: Self) {
        self.base_path.trivial_merge(other.base_path);
    }
}

impl<T> Mergeable for Option<T>
where
    T: Mergeable,
{
    fn merge(&mut self, other: Self) {
        match (self.as_mut(), other) {
            (Some(s), Some(other)) => s.merge(other),
            (None, Some(other)) => {
                _ = self.replace(other);
            }
            _ => {}
        }
    }
}

impl<T> TriviallyMergeable for Option<T> {
    fn trivial_merge(&mut self, other: Self) {
        if self.is_none() {
            let _ = std::mem::replace(self, other);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct TestMergeable(u64);
    impl Mergeable for TestMergeable {
        fn merge(&mut self, other: Self) {
            self.0 += other.0;
        }
    }

    #[test]
    fn trivial_merge() {
        let mut current = Some(10);
        current.trivial_merge(Some(230));
        assert_eq!(current, Some(10));
        let mut current = Some(10);
        current.trivial_merge(None);
        assert_eq!(current, Some(10));
        let mut current = None;
        current.trivial_merge(Some(230));
        assert_eq!(current, Some(230));
        let mut current: Option<i32> = None;
        current.trivial_merge(None);
        assert!(current.is_none());
    }

    #[test]
    fn merge() {
        let mut current = Some(TestMergeable(10));
        current.merge(Some(TestMergeable(230)));
        assert_eq!(current, Some(TestMergeable(240)));
        let mut current = Some(TestMergeable(10));
        current.merge(None);
        assert_eq!(current, Some(TestMergeable(10)));
        let mut current = None;
        current.merge(Some(TestMergeable(230)));
        assert_eq!(current, Some(TestMergeable(230)));
        let mut current: Option<TestMergeable> = None;
        current.merge(None);
        assert!(current.is_none());
    }

    #[test]
    fn merge_app_config_base_path_both() {
        const BASE_PATH: &str = "/test/base/path";
        let mut current = AppConfig {
            base_path: Some(PathBuf::from(BASE_PATH)),
        };
        current.merge(AppConfig {
            base_path: Some(PathBuf::from("other")),
        });
        assert_eq!(current.base_path, Some(PathBuf::from(BASE_PATH)));
    }

    #[test]
    fn merge_deployment_config_base_path_both() {
        const BASE_PATH: &str = "/test/base/path";
        let mut current = DeploymentConfig {
            base_path: Some(PathBuf::from(BASE_PATH)),
        };
        current.merge(DeploymentConfig {
            base_path: Some(PathBuf::from("other")),
        });
        assert_eq!(current.base_path, Some(PathBuf::from(BASE_PATH)));
    }

    #[test]
    fn merge_instance_config_base_path_both() {
        const BASE_PATH: &str = "/test/base/path";
        let mut current = InstanceConfig {
            base_path: Some(PathBuf::from(BASE_PATH)),
        };
        current.merge(InstanceConfig {
            base_path: Some(PathBuf::from("other")),
        });
        assert_eq!(current.base_path, Some(PathBuf::from(BASE_PATH)));
    }

    #[test]
    fn merge_manifest_config_base_path_both() {
        const BASE_PATH: &str = "/test/base/path";
        let mut current = ManifestConfig {
            base_path: Some(PathBuf::from(BASE_PATH)),
        };
        current.merge(ManifestConfig {
            base_path: Some(PathBuf::from("other")),
        });
        assert_eq!(current.base_path, Some(PathBuf::from(BASE_PATH)));
    }

    #[test]
    fn merge_secret_config_base_path_both() {
        const BASE_PATH: &str = "/test/base/path";
        let mut current = SecretConfig {
            base_path: Some(PathBuf::from(BASE_PATH)),
        };
        current.merge(SecretConfig {
            base_path: Some(PathBuf::from("other")),
        });
        assert_eq!(current.base_path, Some(PathBuf::from(BASE_PATH)));
    }

    #[test]
    fn merge_import_config_base_path_both() {
        const BASE_PATH: &str = "/test/base/path";
        let mut current = ImportConfig {
            base_path: Some(PathBuf::from(BASE_PATH)),
            ..ImportConfig::default()
        };
        current.merge(ImportConfig {
            base_path: Some(PathBuf::from("other")),
            ..ImportConfig::default()
        });
        assert_eq!(current.base_path, Some(PathBuf::from(BASE_PATH)));
    }

    #[test]
    fn merge_import_config_timeout_both() {
        const TIMEOUT: u64 = 1234;
        let mut current = ImportConfig {
            timeout: Some(TIMEOUT),
            ..ImportConfig::default()
        };
        current.merge(ImportConfig {
            timeout: Some(5678),
            ..ImportConfig::default()
        });
        assert_eq!(current.timeout, Some(TIMEOUT));
    }

    #[test]
    fn merge_export_config_base_path_both() {
        const BASE_PATH: &str = "/test/base/path";
        let mut current = ExportConfig {
            base_path: Some(PathBuf::from(BASE_PATH)),
            ..ExportConfig::default()
        };
        current.merge(ExportConfig {
            base_path: Some(PathBuf::from("other")),
            ..ExportConfig::default()
        });
        assert_eq!(current.base_path, Some(PathBuf::from(BASE_PATH)));
    }

    #[test]
    fn merge_export_config_timeout_both() {
        const TIMEOUT: u64 = 1234;
        let mut current = ExportConfig {
            timeout: Some(TIMEOUT),
            ..ExportConfig::default()
        };
        current.merge(ExportConfig {
            timeout: Some(5678),
            ..ExportConfig::default()
        });
        assert_eq!(current.timeout, Some(TIMEOUT));
    }

    #[test]
    fn merge_floxy_config_base_path_both() {
        const BASE_PATH: &str = "/test/base/path";
        let mut current = FloxyConfig {
            base_path: Some(PathBuf::from(BASE_PATH)),
            ..FloxyConfig::default()
        };
        current.merge(FloxyConfig {
            base_path: Some(PathBuf::from("other")),
            ..FloxyConfig::default()
        });
        assert_eq!(current.base_path, Some(PathBuf::from(BASE_PATH)));
    }

    #[test]
    fn merge_floxy_config_config_path_both() {
        const CONFIG_PATH: &str = "/test/config/path.conf";
        let mut current = FloxyConfig {
            config_path: Some(PathBuf::from(CONFIG_PATH)),
            ..FloxyConfig::default()
        };
        current.merge(FloxyConfig {
            config_path: Some(PathBuf::from("other")),
            ..FloxyConfig::default()
        });
        assert_eq!(current.config_path, Some(PathBuf::from(CONFIG_PATH)));
    }

    #[test]
    fn merge_console_config_uri_both() {
        const URI: &str = "http://some.uri";
        let mut current = ConsoleConfig {
            uri: Some(UriWrapper(http::Uri::from_static(URI))),
        };
        current.merge(ConsoleConfig {
            uri: Some(UriWrapper(http::Uri::from_static("other"))),
        });
        assert_eq!(current.uri, Some(UriWrapper(http::Uri::from_static(URI))));
    }

    #[test]
    fn merge_network_config_default_network_name_both() {
        const DEFAULT_NETWORK_NAME: &str = "DefNet";
        let mut current = NetworkConfig {
            default_network_name: Some(DEFAULT_NETWORK_NAME.to_string()),
            ..NetworkConfig::default()
        };
        current.merge(NetworkConfig {
            default_network_name: Some("other".to_string()),
            ..NetworkConfig::default()
        });
        assert_eq!(
            current.default_network_name,
            Some(DEFAULT_NETWORK_NAME.to_string())
        );
    }

    #[test]
    fn merge_network_config_default_cidr_subnet_both() {
        let default_cidr_subnet =
            Ipv4Network::try_new(Ipv4Addr::new(123, 123, 123, 0), 24).unwrap();
        let mut current = NetworkConfig {
            default_cidr_subnet: Some(default_cidr_subnet),
            ..NetworkConfig::default()
        };
        current.merge(NetworkConfig {
            default_cidr_subnet: Some(
                Ipv4Network::try_new(Ipv4Addr::new(111, 111, 0, 0), 16).unwrap(),
            ),
            ..NetworkConfig::default()
        });
        assert_eq!(current.default_cidr_subnet, Some(default_cidr_subnet));
    }

    #[test]
    fn merge_network_config_default_gateway_both() {
        const DEFAULT_GATEWAY: Ipv4Addr = Ipv4Addr::new(123, 123, 123, 1);
        let mut current = NetworkConfig {
            default_gateway: Some(DEFAULT_GATEWAY),
            ..NetworkConfig::default()
        };
        current.merge(NetworkConfig {
            default_gateway: Some(Ipv4Addr::new(111, 111, 0, 1)),
            ..NetworkConfig::default()
        });
        assert_eq!(current.default_gateway, Some(DEFAULT_GATEWAY));
    }

    #[test]
    fn merge_network_config_default_options_both() {
        let default_options = HashMap::from([("opt_1".to_string(), "val_1".to_string())]);
        let mut current = NetworkConfig {
            default_options: Some(default_options.clone()),
            ..NetworkConfig::default()
        };
        current.merge(NetworkConfig {
            default_options: Some(HashMap::new()),
            ..NetworkConfig::default()
        });
        assert_eq!(current.default_options, Some(default_options));
    }

    #[test]
    fn merge_network_config_default_parent_adapter_both() {
        const DEFAULT_PARENT_ADAPTER: &str = "eth_parent";
        let mut current = NetworkConfig {
            default_parent_adapter: Some(DEFAULT_PARENT_ADAPTER.to_string()),
            ..NetworkConfig::default()
        };
        current.merge(NetworkConfig {
            default_parent_adapter: Some("other".to_string()),
            ..NetworkConfig::default()
        });
        assert_eq!(
            current.default_parent_adapter,
            Some(DEFAULT_PARENT_ADAPTER.to_string())
        );
    }

    #[test]
    fn merge_network_config_default_network_kind_both() {
        const DEFAULT_NETWORK_KIND: NetworkKind = NetworkKind::Internal;
        let mut current = NetworkConfig {
            default_network_kind: Some(DEFAULT_NETWORK_KIND),
            ..NetworkConfig::default()
        };
        current.merge(NetworkConfig {
            default_network_kind: Some(NetworkKind::Bridge),
            ..NetworkConfig::default()
        });
        assert_eq!(current.default_network_kind, Some(DEFAULT_NETWORK_KIND));
    }
}
