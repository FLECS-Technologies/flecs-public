use crate::jeweler::gem::instance::InstanceId;
use crate::lore::conf::Mergeable;
use crate::relic::var::VarReader;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tracing_subscriber::EnvFilter;

pub mod conf;
pub mod default;
pub mod var;

pub const SPECIAL_CORE_GATEWAY_HOST: &str = "tech.flecs.core";

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unsupported config version {0}")]
    Version(u8),
    #[error(transparent)]
    VarReader(#[from] var::Error),
    #[error("Error reading config file: {0}")]
    File(#[from] conf::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub type ExportLoreRef = Arc<dyn AsRef<ExportLore> + Sync + Send>;
pub type ImportLoreRef = Arc<dyn AsRef<ImportLore> + Sync + Send>;
pub type FloxyLoreRef = Arc<dyn AsRef<FloxyLore> + Sync + Send>;
pub type ConsoleLoreRef = Arc<dyn AsRef<ConsoleLore> + Sync + Send>;
pub trait LoreRef<T>: AsRef<T> + std::fmt::Debug {}
pub type InstanceLoreRef = Arc<dyn LoreRef<InstanceLore> + Sync + Send>;
pub type NetworkLoreRef = Arc<dyn AsRef<NetworkLore> + Sync + Send>;
pub type AppLoreRef = Arc<dyn AsRef<AppLore> + Sync + Send>;
pub type DeploymentLoreRef = Arc<dyn AsRef<DeploymentLore> + Sync + Send>;
pub type ManifestLoreRef = Arc<dyn AsRef<ManifestLore> + Sync + Send>;
pub type SecretLoreRef = Arc<dyn AsRef<SecretLore> + Sync + Send>;
#[cfg(feature = "auth")]
pub type AuthLoreRef = Arc<dyn AsRef<AuthLore> + Sync + Send>;
pub type ProviderLoreRef = Arc<dyn AsRef<ProviderLore> + Sync + Send>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Listener {
    UnixSocket(PathBuf),
    TCP {
        port: u16,
        bind_address: Option<IpAddr>,
    },
}

#[derive(Debug)]
pub struct Lore {
    pub tracing_filter: EnvFilter,
    pub base_path: PathBuf,
    pub listener: Listener,
    pub export: ExportLore,
    pub import: ImportLore,
    pub floxy: FloxyLore,
    pub console: ConsoleLore,
    pub instance: InstanceLore,
    pub network: NetworkLore,
    pub app: AppLore,
    pub deployment: DeploymentLore,
    pub manifest: ManifestLore,
    pub secret: SecretLore,
    #[cfg(feature = "auth")]
    pub auth: AuthLore,
    pub provider: ProviderLore,
    pub system: SystemLore,
}

impl LoreRef<InstanceLore> for Lore {}

impl AsRef<ExportLore> for Lore {
    fn as_ref(&self) -> &ExportLore {
        &self.export
    }
}

impl AsRef<ImportLore> for Lore {
    fn as_ref(&self) -> &ImportLore {
        &self.import
    }
}

impl AsRef<FloxyLore> for Lore {
    fn as_ref(&self) -> &FloxyLore {
        &self.floxy
    }
}

impl AsRef<ConsoleLore> for Lore {
    fn as_ref(&self) -> &ConsoleLore {
        &self.console
    }
}

impl AsRef<InstanceLore> for Lore {
    fn as_ref(&self) -> &InstanceLore {
        &self.instance
    }
}

impl AsRef<NetworkLore> for Lore {
    fn as_ref(&self) -> &NetworkLore {
        &self.network
    }
}

impl AsRef<AppLore> for Lore {
    fn as_ref(&self) -> &AppLore {
        &self.app
    }
}

impl AsRef<DeploymentLore> for Lore {
    fn as_ref(&self) -> &DeploymentLore {
        &self.deployment
    }
}

impl AsRef<ManifestLore> for Lore {
    fn as_ref(&self) -> &ManifestLore {
        &self.manifest
    }
}

impl AsRef<SecretLore> for Lore {
    fn as_ref(&self) -> &SecretLore {
        &self.secret
    }
}

impl AsRef<ProviderLore> for Lore {
    fn as_ref(&self) -> &ProviderLore {
        &self.provider
    }
}

#[cfg(feature = "auth")]
impl AsRef<AuthLore> for Lore {
    fn as_ref(&self) -> &AuthLore {
        &self.auth
    }
}

pub fn config_path(reader: &impl VarReader) -> PathBuf {
    var::config_path(reader).unwrap_or_else(|| PathBuf::from(default::CONFIG_PATH))
}

#[derive(Debug)]
pub struct ExportLore {
    pub base_path: PathBuf,
    pub timeout: Duration,
}

#[derive(Debug)]
pub struct ImportLore {
    pub base_path: PathBuf,
    pub timeout: Duration,
}

#[derive(Debug)]
pub struct FloxyLore {
    pub base_path: PathBuf,
}

#[derive(Debug)]
pub struct ConsoleLore {
    pub uri: http::Uri,
}

#[derive(Debug)]
pub struct InstanceLore {
    pub base_path: PathBuf,
}

#[derive(Debug)]
pub struct NetworkLore {
    pub default_network_name: String,
}

#[derive(Debug)]
pub struct AppLore {
    pub base_path: PathBuf,
}

#[derive(Debug)]
pub struct DeploymentLore {
    pub base_path: PathBuf,
}

#[derive(Debug)]
pub struct ManifestLore {
    pub base_path: PathBuf,
}

#[derive(Debug)]
pub struct SecretLore {
    pub base_path: PathBuf,
}

#[cfg(feature = "auth")]
#[derive(Debug)]
pub struct AuthLore {
    pub issuer_url: Option<openidconnect::IssuerUrl>,
    pub issuer_certificate_cache_lifetime: Duration,
    pub casbin_policy_path: PathBuf,
    pub casbin_model_path: PathBuf,
    pub initial_auth_provider_flecsport_path: PathBuf,
}

#[derive(Debug)]
pub struct ProviderLore {
    pub base_path: PathBuf,
}

#[derive(Debug)]
pub struct SystemLore {
    pub core_sbom_spdx_path: PathBuf,
}

impl Lore {
    pub fn from_confs_with_defaults(
        confs: impl IntoIterator<Item = conf::FlecsConfig>,
    ) -> Result<Self> {
        let merged_conf = confs.into_iter().reduce(|mut current, new| {
            current.merge(new);
            current
        });
        Self::from_conf_with_defaults(merged_conf.unwrap_or_default())
    }

    pub fn from_conf_with_defaults(conf: conf::FlecsConfig) -> Result<Self> {
        if conf.version != 1 {
            return Err(Error::Version(conf.version));
        }
        let tracing_filter = conf
            .tracing_filter
            .map(|filter| filter.0)
            .unwrap_or_else(default::tracing::default_filter);
        let base_path = conf
            .base_path
            .unwrap_or_else(|| PathBuf::from(default::BASE_PATH));
        let listener = conf
            .listener
            .map(|listener| match listener {
                conf::Listener::UnixSocket {
                    socket_path: Some(socket_path),
                } => Listener::UnixSocket(socket_path),
                conf::Listener::UnixSocket { socket_path: None } => {
                    Listener::UnixSocket(PathBuf::from(default::FLECSD_SOCKET_PATH))
                }
                conf::Listener::TCP {
                    port: Some(port),
                    bind_address,
                } => Listener::TCP { port, bind_address },
                conf::Listener::TCP {
                    port: None,
                    bind_address: Some(bind_address),
                } => Listener::TCP {
                    port: default::FLECSD_PORT,
                    bind_address: Some(bind_address),
                },
                conf::Listener::TCP {
                    port: None,
                    bind_address: None,
                } => Listener::TCP {
                    port: default::FLECSD_PORT,
                    bind_address: None,
                },
            })
            .unwrap_or_else(|| Listener::UnixSocket(PathBuf::from(default::FLECSD_SOCKET_PATH)));
        Ok(Self {
            export: ExportLore::from_conf_with_defaults(
                conf.export.unwrap_or_default(),
                &base_path,
            ),
            import: ImportLore::from_conf_with_defaults(
                conf.import.unwrap_or_default(),
                &base_path,
            ),
            floxy: FloxyLore::from_conf_with_defaults(conf.floxy.unwrap_or_default()),
            console: ConsoleLore::from_conf_with_defaults(conf.console.unwrap_or_default()),
            instance: InstanceLore::from_conf_with_defaults(
                conf.instance.unwrap_or_default(),
                &base_path,
            ),
            network: NetworkLore::from_conf_with_defaults(conf.network.unwrap_or_default()),
            app: AppLore::from_conf_with_defaults(conf.app.unwrap_or_default(), &base_path),
            deployment: DeploymentLore::from_conf_with_defaults(
                conf.deployment.unwrap_or_default(),
                &base_path,
            ),
            manifest: ManifestLore::from_conf_with_defaults(
                conf.manifest.unwrap_or_default(),
                &base_path,
            ),
            secret: SecretLore::from_conf_with_defaults(
                conf.secret.unwrap_or_default(),
                &base_path,
            ),
            #[cfg(feature = "auth")]
            auth: AuthLore::from_conf_with_defaults(conf.auth.unwrap_or_default()),
            provider: ProviderLore::from_conf_with_defaults(
                conf.provider.unwrap_or_default(),
                &base_path,
            ),
            tracing_filter,
            base_path,
            listener,
            system: SystemLore::from_conf_with_defaults(conf.system.unwrap_or_default()),
        })
    }
}

impl ExportLore {
    pub fn from_conf_with_defaults(conf: conf::ExportConfig, base_path: &Path) -> Self {
        let base_path = conf
            .base_path
            .unwrap_or_else(|| base_path.join(default::export::BASE_DIRECTORY_NAME));
        let timeout = conf
            .timeout
            .map(Duration::from_secs)
            .unwrap_or_else(|| default::export::TIMEOUT);
        Self { timeout, base_path }
    }
}

impl ImportLore {
    pub fn from_conf_with_defaults(conf: conf::ImportConfig, base_path: &Path) -> Self {
        let base_path = conf
            .base_path
            .unwrap_or_else(|| base_path.join(default::import::BASE_DIRECTORY_NAME));
        let timeout = conf
            .timeout
            .map(Duration::from_secs)
            .unwrap_or_else(|| default::import::TIMEOUT);
        Self { timeout, base_path }
    }
}

impl FloxyLore {
    pub fn from_conf_with_defaults(conf: conf::FloxyConfig) -> Self {
        let base_path = conf
            .base_path
            .unwrap_or_else(|| PathBuf::from(default::floxy::BASE_DIRECTORY));
        Self { base_path }
    }

    pub fn instance_editor_location(instance_id: InstanceId, port: u16) -> String {
        format!("/v2/instances/{instance_id}/editor/{port}")
    }

    pub fn server_config_path(&self) -> PathBuf {
        self.base_path.join(default::floxy::SERVER_CONFIGS_DIR_NAME)
    }

    pub fn instance_config_path(&self) -> PathBuf {
        self.base_path
            .join(default::floxy::INSTANCE_CONFIGS_DIR_NAME)
    }
}

impl ConsoleLore {
    pub fn from_conf_with_defaults(conf: conf::ConsoleConfig) -> Self {
        let uri = conf
            .uri
            .map(|wrapper| wrapper.0)
            .unwrap_or_else(default::console::uri);
        Self { uri }
    }
}

impl InstanceLore {
    pub fn from_conf_with_defaults(conf: conf::InstanceConfig, base_path: &Path) -> Self {
        let base_path = conf
            .base_path
            .unwrap_or_else(|| base_path.join(default::instance::BASE_DIRECTORY_NAME));
        Self { base_path }
    }
    pub fn instance_config_path(&self, instance_id: &impl AsRef<str>) -> PathBuf {
        self.base_path.join(instance_id.as_ref()).join("conf")
    }

    pub fn instance_workdir_path(&self, instance_id: &impl AsRef<str>) -> PathBuf {
        self.base_path.join(instance_id.as_ref()).join("work")
    }
}

impl NetworkLore {
    pub fn from_conf_with_defaults(conf: conf::NetworkConfig) -> Self {
        let default_network_name = conf
            .default_network_name
            .unwrap_or_else(|| default::network::DEFAULT_NETWORK_NAME.to_string());
        Self {
            default_network_name,
        }
    }
}

impl AppLore {
    pub fn from_conf_with_defaults(conf: conf::AppConfig, base_path: &Path) -> Self {
        let base_path = conf
            .base_path
            .unwrap_or_else(|| base_path.join(default::app::BASE_DIRECTORY_NAME));
        Self { base_path }
    }
}

impl DeploymentLore {
    pub fn from_conf_with_defaults(conf: conf::DeploymentConfig, base_path: &Path) -> Self {
        let base_path = conf
            .base_path
            .unwrap_or_else(|| base_path.join(default::deployment::BASE_DIRECTORY_NAME));
        Self { base_path }
    }
}

impl ManifestLore {
    pub fn from_conf_with_defaults(conf: conf::ManifestConfig, base_path: &Path) -> Self {
        let base_path = conf
            .base_path
            .unwrap_or_else(|| base_path.join(default::manifest::BASE_DIRECTORY_NAME));
        Self { base_path }
    }
}

impl SecretLore {
    pub fn from_conf_with_defaults(conf: conf::SecretConfig, base_path: &Path) -> Self {
        let base_path = conf
            .base_path
            .unwrap_or_else(|| base_path.join(default::secret::BASE_DIRECTORY_NAME));
        Self { base_path }
    }
}

#[cfg(feature = "auth")]
impl AuthLore {
    pub fn from_conf_with_defaults(conf: conf::AuthConfig) -> Self {
        let issuer_url = conf.issuer_url.map(openidconnect::IssuerUrl::from_url);
        let issuer_certificate_cache_lifetime = conf
            .issuer_certificate_cache_lifetime
            .map(Duration::from_secs)
            .unwrap_or(default::auth::ISSUER_CERTIFICATE_CACHE_LIFETIME);
        let casbin_policy_path = conf.casbin_policy_path.unwrap_or_else(|| {
            Path::new(default::auth::SHARE_BASE_PATH).join(default::auth::CASBIN_POLICY_FILE_NAME)
        });
        let casbin_model_path = conf.casbin_model_path.unwrap_or_else(|| {
            Path::new(default::auth::SHARE_BASE_PATH).join(default::auth::CASBIN_MODEL_FILE_NAME)
        });
        let initial_auth_provider_flecsport_path = conf
            .initial_auth_provider_flecsport_path
            .unwrap_or_else(|| {
                Path::new(default::auth::LIB_BASE_PATH)
                    .join(default::auth::INITIAL_AUTH_PROVIDER_FLECSPORT_FILE_NAME)
            });
        Self {
            issuer_url,
            issuer_certificate_cache_lifetime,
            casbin_policy_path,
            casbin_model_path,
            initial_auth_provider_flecsport_path,
        }
    }
}

impl ProviderLore {
    pub fn from_conf_with_defaults(conf: conf::ProviderConfig, base_path: &Path) -> Self {
        let base_path = conf
            .base_path
            .unwrap_or_else(|| base_path.join(default::provider::BASE_DIRECTORY_NAME));
        Self { base_path }
    }
}

impl SystemLore {
    pub fn from_conf_with_defaults(conf: conf::SystemConfig) -> Self {
        let core_sbom_spdx_path = conf
            .core_sbom_spdx_path
            .unwrap_or_else(default::system::sbom_spdx_file_path_path);
        Self {
            core_sbom_spdx_path,
        }
    }
}

#[cfg(test)]
pub fn test_lore(
    base_path: PathBuf,
    mock_var_reader: &crate::relic::var::test::MockVarReader,
) -> Lore {
    let mut conf = crate::lore::conf::FlecsConfig::from_var_reader(mock_var_reader).unwrap();
    conf.merge(crate::lore::conf::FlecsConfig {
        floxy: Some(crate::lore::conf::FloxyConfig {
            base_path: Some(base_path.join("floxy")),
        }),
        base_path: Some(base_path),
        ..crate::lore::conf::FlecsConfig::default()
    });
    Lore::from_conf_with_defaults(conf).unwrap()
}

pub const MAX_SUPPORTED_APP_MANIFEST_VERSION: &str = "3.0.0";
pub const API_VERSION: &str = env!("FLECS_API_VERSION");
pub const CORE_VERSION: &str = concat!(env!("FLECS_VERSION"), "-", env!("FLECS_GIT_SHA"));

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forge::serde::{EnvFilterWrapper, UriWrapper};
    use std::str::FromStr;

    #[test]
    fn from_conf_unsupported_version() {
        const VERSION: u8 = 2;
        let conf = conf::FlecsConfig {
            version: VERSION,
            ..conf::FlecsConfig::default()
        };
        assert!(matches!(
            Lore::from_conf_with_defaults(conf),
            Err(Error::Version(VERSION))
        ));
    }

    #[test]
    fn from_conf_tracing_filter() {
        const FILTER: &str = "error";
        let conf = conf::FlecsConfig {
            tracing_filter: Some(EnvFilterWrapper::from_str(FILTER).unwrap()),
            ..conf::FlecsConfig::default()
        };
        assert_eq!(
            Lore::from_conf_with_defaults(conf)
                .unwrap()
                .tracing_filter
                .to_string(),
            FILTER
        );
    }

    #[test]
    fn from_conf_tracing_filter_default() {
        let conf = conf::FlecsConfig::default();
        assert_eq!(
            Lore::from_conf_with_defaults(conf)
                .unwrap()
                .tracing_filter
                .to_string(),
            default::tracing::default_filter().to_string()
        );
    }

    #[test]
    fn from_conf_base_path() {
        const BASE_PATH: &str = "/base/path";
        let conf = conf::FlecsConfig {
            base_path: Some(PathBuf::from(BASE_PATH)),
            ..conf::FlecsConfig::default()
        };
        assert_eq!(
            Lore::from_conf_with_defaults(conf).unwrap().base_path,
            PathBuf::from(BASE_PATH)
        );
    }

    #[test]
    fn from_conf_base_path_default() {
        let conf = conf::FlecsConfig::default();
        assert_eq!(
            Lore::from_conf_with_defaults(conf).unwrap().base_path,
            PathBuf::from(default::BASE_PATH)
        );
    }

    #[test]
    fn from_conf_socket_path() {
        const SOCKET_PATH: &str = "/socket/path.sock";
        let conf = conf::FlecsConfig {
            listener: Some(conf::Listener::UnixSocket {
                socket_path: Some(PathBuf::from(SOCKET_PATH)),
            }),
            ..conf::FlecsConfig::default()
        };
        assert!(matches!(
            Lore::from_conf_with_defaults(conf).unwrap().listener,
            Listener::UnixSocket(socket_path) if socket_path == PathBuf::from(SOCKET_PATH)
        ));
    }

    #[test]
    fn from_conf_socket_path_default() {
        let conf = conf::FlecsConfig::default();
        assert!(matches!(
            Lore::from_conf_with_defaults(conf).unwrap().listener,
            Listener::UnixSocket(socket_path) if socket_path == PathBuf::from(default::FLECSD_SOCKET_PATH)
        ));
    }

    #[test]
    fn export_lore_from_conf_base_path() {
        let base_path = PathBuf::from("/some/base/path");
        let conf = conf::ExportConfig {
            base_path: Some(base_path.clone()),
            ..conf::ExportConfig::default()
        };
        assert_eq!(
            ExportLore::from_conf_with_defaults(conf, Path::new("/")).base_path,
            base_path
        );
    }

    #[test]
    fn export_lore_from_conf_base_path_default() {
        let base_path = PathBuf::from("/some/base/path");
        let conf = conf::ExportConfig::default();
        assert_eq!(
            ExportLore::from_conf_with_defaults(conf, &base_path).base_path,
            base_path.join(default::export::BASE_DIRECTORY_NAME)
        );
    }

    #[test]
    fn export_lore_from_conf_timeout() {
        const TIMEOUT: u64 = 1234;
        let conf = conf::ExportConfig {
            timeout: Some(TIMEOUT),
            ..conf::ExportConfig::default()
        };
        assert_eq!(
            ExportLore::from_conf_with_defaults(conf, Path::new("/")).timeout,
            Duration::from_secs(TIMEOUT)
        );
    }

    #[test]
    fn export_lore_from_conf_timeout_default() {
        let conf = conf::ExportConfig::default();
        assert_eq!(
            ExportLore::from_conf_with_defaults(conf, Path::new("/")).timeout,
            default::import::TIMEOUT,
        );
    }

    #[test]
    fn import_lore_from_conf_base_path() {
        let base_path = PathBuf::from("/some/base/path");
        let conf = conf::ImportConfig {
            base_path: Some(base_path.clone()),
            ..conf::ImportConfig::default()
        };
        assert_eq!(
            ImportLore::from_conf_with_defaults(conf, Path::new("/")).base_path,
            base_path
        );
    }

    #[test]
    fn import_lore_from_conf_base_path_default() {
        let base_path = PathBuf::from("/some/base/path");
        let conf = conf::ImportConfig::default();
        assert_eq!(
            ImportLore::from_conf_with_defaults(conf, &base_path).base_path,
            base_path.join(default::import::BASE_DIRECTORY_NAME)
        );
    }

    #[test]
    fn import_lore_from_conf_timeout() {
        const TIMEOUT: u64 = 1234;
        let conf = conf::ImportConfig {
            timeout: Some(TIMEOUT),
            ..conf::ImportConfig::default()
        };
        assert_eq!(
            ImportLore::from_conf_with_defaults(conf, Path::new("/")).timeout,
            Duration::from_secs(TIMEOUT)
        );
    }

    #[test]
    fn import_lore_from_conf_timeout_default() {
        let conf = conf::ImportConfig::default();
        assert_eq!(
            ImportLore::from_conf_with_defaults(conf, Path::new("/")).timeout,
            default::import::TIMEOUT,
        );
    }

    #[test]
    fn floxy_lore_from_conf_base_path() {
        let base_path = PathBuf::from("/some/base/path");
        let conf = conf::FloxyConfig {
            base_path: Some(base_path.clone()),
        };
        assert_eq!(
            FloxyLore::from_conf_with_defaults(conf).base_path,
            base_path
        );
    }

    #[test]
    fn floxy_lore_from_conf_base_path_default() {
        let base_path = PathBuf::from("/some/base/path");
        let conf = conf::FloxyConfig::default();
        assert_eq!(
            FloxyLore::from_conf_with_defaults(conf).base_path,
            base_path.join(default::floxy::BASE_DIRECTORY)
        );
    }

    #[test]
    fn instance_lore_from_conf_base_path() {
        let base_path = PathBuf::from("/some/base/path");
        let conf = conf::InstanceConfig {
            base_path: Some(base_path.clone()),
        };
        assert_eq!(
            InstanceLore::from_conf_with_defaults(conf, Path::new("/")).base_path,
            base_path
        );
    }

    #[test]
    fn instance_lore_from_conf_base_path_default() {
        let base_path = PathBuf::from("/some/base/path");
        let conf = conf::InstanceConfig::default();
        assert_eq!(
            InstanceLore::from_conf_with_defaults(conf, &base_path).base_path,
            base_path.join(default::instance::BASE_DIRECTORY_NAME)
        );
    }

    #[test]
    fn app_lore_from_conf_base_path() {
        let base_path = PathBuf::from("/some/base/path");
        let conf = conf::AppConfig {
            base_path: Some(base_path.clone()),
        };
        assert_eq!(
            AppLore::from_conf_with_defaults(conf, Path::new("/")).base_path,
            base_path
        );
    }

    #[test]
    fn app_lore_from_conf_base_path_default() {
        let base_path = PathBuf::from("/some/base/path");
        let conf = conf::AppConfig::default();
        assert_eq!(
            AppLore::from_conf_with_defaults(conf, &base_path).base_path,
            base_path.join(default::app::BASE_DIRECTORY_NAME)
        );
    }

    #[test]
    fn deployment_lore_from_conf_base_path() {
        let base_path = PathBuf::from("/some/base/path");
        let conf = conf::DeploymentConfig {
            base_path: Some(base_path.clone()),
        };
        assert_eq!(
            DeploymentLore::from_conf_with_defaults(conf, Path::new("/")).base_path,
            base_path
        );
    }

    #[test]
    fn deployment_lore_from_conf_base_path_default() {
        let base_path = PathBuf::from("/some/base/path");
        let conf = conf::DeploymentConfig::default();
        assert_eq!(
            DeploymentLore::from_conf_with_defaults(conf, &base_path).base_path,
            base_path.join(default::deployment::BASE_DIRECTORY_NAME)
        );
    }

    #[test]
    fn manifest_lore_from_conf_base_path() {
        let base_path = PathBuf::from("/some/base/path");
        let conf = conf::ManifestConfig {
            base_path: Some(base_path.clone()),
        };
        assert_eq!(
            ManifestLore::from_conf_with_defaults(conf, Path::new("/")).base_path,
            base_path
        );
    }

    #[test]
    fn manifest_lore_from_conf_base_path_default() {
        let base_path = PathBuf::from("/some/base/path");
        let conf = conf::ManifestConfig::default();
        assert_eq!(
            ManifestLore::from_conf_with_defaults(conf, &base_path).base_path,
            base_path.join(default::manifest::BASE_DIRECTORY_NAME)
        );
    }

    #[test]
    fn secret_lore_from_conf_base_path() {
        let base_path = PathBuf::from("/some/base/path");
        let conf = conf::SecretConfig {
            base_path: Some(base_path.clone()),
        };
        assert_eq!(
            SecretLore::from_conf_with_defaults(conf, Path::new("/")).base_path,
            base_path
        );
    }

    #[test]
    fn secret_lore_from_conf_base_path_default() {
        let base_path = PathBuf::from("/some/base/path");
        let conf = conf::SecretConfig::default();
        assert_eq!(
            SecretLore::from_conf_with_defaults(conf, &base_path).base_path,
            base_path.join(default::secret::BASE_DIRECTORY_NAME)
        );
    }

    #[test]
    fn console_lore_from_conf_uri() {
        let uri = http::Uri::from_static("http://cloud.my/console");
        let conf = conf::ConsoleConfig {
            uri: Some(UriWrapper(uri.clone())),
        };
        assert_eq!(ConsoleLore::from_conf_with_defaults(conf).uri, uri);
    }

    #[test]
    fn console_lore_from_conf_uri_default() {
        let conf = conf::ConsoleConfig::default();
        assert_eq!(
            ConsoleLore::from_conf_with_defaults(conf).uri,
            default::console::uri()
        );
    }

    #[test]
    fn network_lore_from_conf_default_network_name() {
        const NETWORK_NAME: &str = "TESTNET";
        let conf = conf::NetworkConfig {
            default_network_name: Some(NETWORK_NAME.to_string()),
        };
        assert_eq!(
            NetworkLore::from_conf_with_defaults(conf).default_network_name,
            NETWORK_NAME
        );
    }

    #[test]
    fn network_lore_from_conf_default_network_name_default() {
        let conf = conf::NetworkConfig::default();
        assert_eq!(
            NetworkLore::from_conf_with_defaults(conf).default_network_name,
            default::network::DEFAULT_NETWORK_NAME
        );
    }
}
