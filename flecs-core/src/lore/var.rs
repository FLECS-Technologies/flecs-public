use crate::forge::serde::EnvFilterWrapper;
#[cfg(feature = "auth")]
use crate::lore::conf::AuthConfig;
use crate::lore::conf::{
    AppConfig, ConsoleConfig, DeploymentConfig, ExportConfig, FlecsConfig, FloxyConfig,
    ImportConfig, InstanceConfig, Listener, ManifestConfig, NetworkConfig, ProviderConfig,
    SecretConfig, SystemConfig,
};
use crate::relic::var;
use crate::relic::var::VarReader;
use std::net::IpAddr;
use std::path::PathBuf;
use thiserror::Error;
use tracing_subscriber::EnvFilter;

const TRACING_FILTER_ENV: &str = "RUST_LOG";
const BASE_PATH: &str = "FLECS_CORE_BASE_PATH";
const FLECSD_SOCKET_PATH: &str = "FLECS_CORE_SOCKET_PATH";
const FLECSD_PORT: &str = "FLECS_CORE_PORT";
const FLECSD_BIND_ADDRESS: &str = "FLECS_CORE_BIND_ADDRESS";
const CONFIG_PATH: &str = "FLECS_CORE_CONFIG_PATH";

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    NotUnicode(#[from] var::Error),
    #[error("Tracing filter {0} from {TRACING_FILTER_ENV} invalid: {1}")]
    InvalidTracingFilter(String, tracing_subscriber::filter::ParseError),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub fn env_filter(reader: &impl VarReader) -> Result<Option<EnvFilter>> {
    reader
        .read_var(TRACING_FILTER_ENV)?
        .map(|val| EnvFilter::try_new(&val).map_err(|e| Error::InvalidTracingFilter(val, e)))
        .transpose()
}

pub fn base_path(reader: &impl VarReader) -> Option<PathBuf> {
    reader.read_path(BASE_PATH)
}

pub fn config_path(reader: &impl VarReader) -> Option<PathBuf> {
    reader.read_path(CONFIG_PATH)
}

pub fn flecsd_socket_path(reader: &impl VarReader) -> Option<PathBuf> {
    reader.read_path(FLECSD_SOCKET_PATH)
}

pub fn flecsd_port(reader: &impl VarReader) -> Result<Option<u16>> {
    Ok(reader.read_u16(FLECSD_PORT)?)
}

pub fn flecsd_bind_address(reader: &impl VarReader) -> Result<Option<IpAddr>> {
    Ok(reader.read_ip(FLECSD_BIND_ADDRESS)?)
}

impl FlecsConfig {
    pub fn from_var_reader(reader: &impl VarReader) -> Result<Self> {
        Ok(Self {
            version: 1,
            tracing_filter: env_filter(reader)?.map(EnvFilterWrapper),
            base_path: base_path(reader),
            listener: match (
                flecsd_port(reader)?,
                flecsd_bind_address(reader)?,
                flecsd_socket_path(reader),
            ) {
                (Some(port), bind_address, _) => Some(Listener::TCP {
                    port: Some(port),
                    bind_address,
                }),
                (port, Some(bind_address), _) => Some(Listener::TCP {
                    port,
                    bind_address: Some(bind_address),
                }),
                (None, None, Some(socket_path)) => Some(Listener::UnixSocket {
                    socket_path: Some(socket_path),
                }),
                _ => None,
            },
            export: ExportConfig::from_var_reader(reader)?,
            import: ImportConfig::from_var_reader(reader)?,
            floxy: FloxyConfig::from_var_reader(reader),
            console: ConsoleConfig::from_var_reader(reader)?,
            instance: InstanceConfig::from_var_reader(reader),
            network: NetworkConfig::from_var_reader(reader)?,
            app: AppConfig::from_var_reader(reader),
            deployment: DeploymentConfig::from_var_reader(reader),
            manifest: ManifestConfig::from_var_reader(reader),
            secret: SecretConfig::from_var_reader(reader),
            #[cfg(feature = "auth")]
            auth: AuthConfig::from_var_reader(reader)?,
            provider: ProviderConfig::from_var_reader(reader),
            system: SystemConfig::from_var_reader(reader),
        })
    }
}

pub mod export {
    use super::Result;
    use crate::lore::conf::ExportConfig;
    use crate::relic::var::VarReader;
    use std::path::PathBuf;
    use std::time::Duration;

    const BASE_PATH: &str = "FLECS_CORE_EXPORT_BASE_PATH";
    const TIMEOUT: &str = "FLECS_CORE_EXPORT_TIMEOUT";

    fn base_path(reader: &impl VarReader) -> Option<PathBuf> {
        reader.read_path(BASE_PATH)
    }

    fn timeout(reader: &impl VarReader) -> Result<Option<Duration>> {
        Ok(reader.read_secs(TIMEOUT)?)
    }

    impl ExportConfig {
        pub fn from_var_reader(reader: &impl VarReader) -> Result<Option<Self>> {
            let base_path = base_path(reader);
            let timeout = timeout(reader)?.as_ref().map(Duration::as_secs);
            if base_path.is_some() && timeout.is_some() {
                Ok(Some(Self { base_path, timeout }))
            } else {
                Ok(None)
            }
        }
    }
}

pub mod import {
    use super::Result;
    use crate::lore::conf::ImportConfig;
    use crate::relic::var::VarReader;
    use std::path::PathBuf;
    use std::time::Duration;

    const BASE_PATH: &str = "FLECS_CORE_IMPORT_BASE_PATH";
    const TIMEOUT: &str = "FLECS_CORE_IMPORT_TIMEOUT";
    fn base_path(reader: &impl VarReader) -> Option<PathBuf> {
        reader.read_path(BASE_PATH)
    }

    fn timeout(reader: &impl VarReader) -> Result<Option<Duration>> {
        Ok(reader.read_secs(TIMEOUT)?)
    }

    impl ImportConfig {
        pub fn from_var_reader(reader: &impl VarReader) -> Result<Option<Self>> {
            let base_path = base_path(reader);
            let timeout = timeout(reader)?.as_ref().map(Duration::as_secs);
            if base_path.is_some() && timeout.is_some() {
                Ok(Some(Self { base_path, timeout }))
            } else {
                Ok(None)
            }
        }
    }
}

pub mod floxy {
    use crate::lore::conf::FloxyConfig;
    use crate::relic::var::VarReader;
    use std::path::PathBuf;

    const BASE_PATH: &str = "FLECS_CORE_FLOXY_BASE_PATH";

    fn base_path(reader: &impl VarReader) -> Option<PathBuf> {
        reader.read_path(BASE_PATH)
    }

    impl FloxyConfig {
        pub fn from_var_reader(reader: &impl VarReader) -> Option<Self> {
            base_path(reader).map(|base_path| Self {
                base_path: Some(base_path),
            })
        }
    }
}

pub mod console {
    use super::Result;
    use crate::forge::serde::UriWrapper;
    use crate::lore::conf::ConsoleConfig;
    use crate::relic::var::VarReader;

    const URI: &str = "FLECS_CORE_CONSOLE_URI";

    fn uri(reader: &impl VarReader) -> Result<Option<http::Uri>> {
        Ok(reader.read_uri(URI)?)
    }

    impl ConsoleConfig {
        pub fn from_var_reader(reader: &impl VarReader) -> Result<Option<Self>> {
            let uri = uri(reader)?.map(UriWrapper);
            Ok(uri.map(|uri| Self { uri: Some(uri) }))
        }
    }
}

pub mod instance {
    use crate::lore::conf::InstanceConfig;
    use crate::relic::var::VarReader;
    use std::path::PathBuf;

    const BASE_PATH: &str = "FLECS_CORE_INSTANCE_BASE_PATH";

    fn base_path(reader: &impl VarReader) -> Option<PathBuf> {
        reader.read_path(BASE_PATH)
    }

    impl InstanceConfig {
        pub fn from_var_reader(reader: &impl VarReader) -> Option<Self> {
            let base_path = base_path(reader);
            base_path.map(|base_path| Self {
                base_path: Some(base_path),
            })
        }
    }
}

pub mod app {
    use crate::lore::conf::AppConfig;
    use crate::relic::var::VarReader;
    use std::path::PathBuf;

    const BASE_PATH: &str = "FLECS_CORE_APP_BASE_PATH";

    fn base_path(reader: &impl VarReader) -> Option<PathBuf> {
        reader.read_path(BASE_PATH)
    }

    impl AppConfig {
        pub fn from_var_reader(reader: &impl VarReader) -> Option<Self> {
            let base_path = base_path(reader);
            base_path.map(|base_path| Self {
                base_path: Some(base_path),
            })
        }
    }
}

pub mod deployment {
    use crate::lore::conf::DeploymentConfig;
    use crate::relic::var::VarReader;
    use std::path::PathBuf;

    const BASE_PATH: &str = "FLECS_CORE_DEPLOYMENT_BASE_PATH";

    fn base_path(reader: &impl VarReader) -> Option<PathBuf> {
        reader.read_path(BASE_PATH)
    }

    impl DeploymentConfig {
        pub fn from_var_reader(reader: &impl VarReader) -> Option<Self> {
            let base_path = base_path(reader);
            base_path.map(|base_path| Self {
                base_path: Some(base_path),
            })
        }
    }
}

pub mod manifest {
    use crate::lore::conf::ManifestConfig;
    use crate::relic::var::VarReader;
    use std::path::PathBuf;

    const BASE_PATH: &str = "FLECS_CORE_MANIFEST_BASE_PATH";

    fn base_path(reader: &impl VarReader) -> Option<PathBuf> {
        reader.read_path(BASE_PATH)
    }

    impl ManifestConfig {
        pub fn from_var_reader(reader: &impl VarReader) -> Option<Self> {
            let base_path = base_path(reader);
            base_path.map(|base_path| Self {
                base_path: Some(base_path),
            })
        }
    }
}

pub mod secret {
    use crate::lore::conf::SecretConfig;
    use crate::relic::var::VarReader;
    use std::path::PathBuf;

    const BASE_PATH: &str = "FLECS_CORE_SECRET_BASE_PATH";

    fn base_path(reader: &impl VarReader) -> Option<PathBuf> {
        reader.read_path(BASE_PATH)
    }

    impl SecretConfig {
        pub fn from_var_reader(reader: &impl VarReader) -> Option<Self> {
            let base_path = base_path(reader);
            base_path.map(|base_path| Self {
                base_path: Some(base_path),
            })
        }
    }
}

#[cfg(feature = "auth")]
pub mod auth {
    use super::Result;
    use crate::lore::conf::AuthConfig;
    use crate::relic::var::VarReader;
    use std::path::PathBuf;
    use std::time::Duration;

    const ISSUER_URL: &str = "FLECS_CORE_ISSUER_URL";
    const ISSUER_CERTIFICATE_CACHE_LIFETIME: &str = "FLECS_CORE_ISSUER_CERTIFICATE_CACHE_LIFETIME";
    const CASBIN_POLICY_PATH: &str = "FLECS_CORE_CASBIN_POLICY_PATH";
    const CASBIN_MODEL_PATH: &str = "FLECS_CORE_CASBIN_MODEL_PATH";
    const INITIAL_AUTH_PROVIDER_FLECSPORT_PATH: &str =
        "FLECS_CORE_INITIAL_AUTH_PROVIDER_FLECSPORT_PATH";

    fn issuer_certificate_cache_lifetime(reader: &impl VarReader) -> Result<Option<Duration>> {
        Ok(reader.read_secs(ISSUER_CERTIFICATE_CACHE_LIFETIME)?)
    }

    fn casbin_policy_path(reader: &impl VarReader) -> Option<PathBuf> {
        reader.read_path(CASBIN_POLICY_PATH)
    }

    fn casbin_model_path(reader: &impl VarReader) -> Option<PathBuf> {
        reader.read_path(CASBIN_MODEL_PATH)
    }

    fn initial_auth_provider_flecsport_path(reader: &impl VarReader) -> Option<PathBuf> {
        reader.read_path(INITIAL_AUTH_PROVIDER_FLECSPORT_PATH)
    }

    impl AuthConfig {
        pub fn from_var_reader(reader: &impl VarReader) -> Result<Option<Self>> {
            let issuer_url = reader.read_url(ISSUER_URL)?;
            let issuer_certificate_cache_lifetime = issuer_certificate_cache_lifetime(reader)?
                .as_ref()
                .map(Duration::as_secs);
            let casbin_policy_path = casbin_policy_path(reader);
            let casbin_model_path = casbin_model_path(reader);
            let initial_auth_provider_flecsport_path = initial_auth_provider_flecsport_path(reader);
            Ok(
                if issuer_url.is_some()
                    || issuer_certificate_cache_lifetime.is_some()
                    || casbin_policy_path.is_some()
                    || casbin_model_path.is_some()
                    || initial_auth_provider_flecsport_path.is_some()
                {
                    Some(Self {
                        issuer_url,
                        issuer_certificate_cache_lifetime,
                        casbin_policy_path,
                        casbin_model_path,
                        initial_auth_provider_flecsport_path,
                    })
                } else {
                    None
                },
            )
        }
    }
}

pub mod provider {
    use crate::lore::conf::ProviderConfig;
    use crate::relic::var::VarReader;
    use std::path::PathBuf;

    const BASE_PATH: &str = "FLECS_CORE_PROVIDER_BASE_PATH";

    fn base_path(reader: &impl VarReader) -> Option<PathBuf> {
        reader.read_path(BASE_PATH)
    }

    impl ProviderConfig {
        pub fn from_var_reader(reader: &impl VarReader) -> Option<Self> {
            let base_path = base_path(reader);
            base_path.map(|base_path| Self {
                base_path: Some(base_path),
            })
        }
    }
}

pub mod system {
    use crate::lore::conf::SystemConfig;
    use crate::relic::var::VarReader;
    use std::path::PathBuf;
    const SBOM_SPDX_PATH: &str = "FLECS_CORE_SBOM_SPDX_PATH";

    fn sbom_spdx_path(reader: &impl VarReader) -> Option<PathBuf> {
        reader.read_path(SBOM_SPDX_PATH)
    }

    impl SystemConfig {
        pub fn from_var_reader(reader: &impl VarReader) -> Option<Self> {
            let core_sbom_spdx_path = sbom_spdx_path(reader)?;
            Some(Self {
                core_sbom_spdx_path: Some(core_sbom_spdx_path),
            })
        }
    }
}

pub mod network {
    use super::Result;
    use crate::lore::conf::NetworkConfig;
    use crate::relic::var::VarReader;

    const DEFAULT_NETWORK_NAME: &str = "FLECS_CORE_NETWORK_DEFAULT_NETWORK_NAME";

    fn default_network_name(reader: &impl VarReader) -> Result<Option<String>> {
        Ok(reader.read_var(DEFAULT_NETWORK_NAME)?)
    }

    impl NetworkConfig {
        pub fn from_var_reader(reader: &impl VarReader) -> Result<Option<Self>> {
            Ok(
                default_network_name(reader)?.map(|default_network_name| Self {
                    default_network_name: Some(default_network_name),
                }),
            )
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::relic::var::test::MockVarReader;

        #[test]
        fn from_var_reader_none() {
            let reader = &MockVarReader::new();
            assert!(NetworkConfig::from_var_reader(reader).unwrap().is_none());
        }
    }
}
