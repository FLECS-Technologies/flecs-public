pub mod tracing {
    use tracing_subscriber::EnvFilter;

    #[cfg(debug_assertions)]
    const DEFAULT_TRACING_FILTER: &str = "debug";
    #[cfg(not(debug_assertions))]
    const DEFAULT_TRACING_FILTER: &str = "info,tower_http=debug,axum::rejection=debug";
    pub fn default_filter() -> EnvFilter {
        EnvFilter::try_new(DEFAULT_TRACING_FILTER)
            .expect("const value {DEFAULT_TRACING_FILTER} is valid filter")
    }
}

pub mod import {
    use std::time::Duration;

    pub const BASE_DIRECTORY_NAME: &str = "import";
    pub const TIMEOUT: Duration = Duration::from_secs(i64::MAX as u64);
}

pub mod export {
    use std::time::Duration;

    pub const BASE_DIRECTORY_NAME: &str = "export";
    pub const TIMEOUT: Duration = Duration::from_secs(i64::MAX as u64);
}

pub mod floxy {
    pub const BASE_DIRECTORY: &str = "/tmp/floxy/conf.d";
    pub const SERVER_CONFIGS_DIR_NAME: &str = "servers";
    pub const INSTANCE_CONFIGS_DIR_NAME: &str = "instances";
}

pub mod console {
    #[cfg(debug_assertions)]
    pub const URI: &str = "https://console-dev.flecs.tech";
    #[cfg(not(debug_assertions))]
    pub const URI: &str = "https://console.flecs.tech";

    pub fn uri() -> http::Uri {
        http::Uri::from_static(URI)
    }
}

pub mod instance {
    pub const BASE_DIRECTORY_NAME: &str = "instances";
}

pub mod app {
    pub const BASE_DIRECTORY_NAME: &str = "apps";
}

pub mod deployment {
    pub const BASE_DIRECTORY_NAME: &str = "deployments";
}

pub mod manifest {
    pub const BASE_DIRECTORY_NAME: &str = "manifests";
}

pub mod secret {
    pub const BASE_DIRECTORY_NAME: &str = "device";
}

pub mod network {
    pub const DEFAULT_NETWORK_NAME: &str = "flecs";
}

pub mod provider {
    pub const BASE_DIRECTORY_NAME: &str = "providers";
}

#[cfg(feature = "auth")]
pub mod auth {
    use std::time::Duration;

    pub const ISSUER_CERTIFICATE_CACHE_LIFETIME: Duration = Duration::from_secs(3);
    pub const SHARE_BASE_PATH: &str = "/usr/local/share/flecs/auth";
    pub const LIB_BASE_PATH: &str = "/usr/local/lib/flecs/auth";
    pub const CASBIN_POLICY_FILE_NAME: &str = "casbin_policy.csv";
    pub const CASBIN_MODEL_FILE_NAME: &str = "casbin_model.conf";
    pub const INITIAL_AUTH_PROVIDER_FLECSPORT_FILE_NAME: &str = "initial_auth_provider.tar";
}

pub mod system {
    use std::path::PathBuf;

    const SBOM_BASE_PATH: &str = "/usr/local/lib/flecs/sbom";
    const SBOM_SPDX_FILE_NAME: &str = "sbom.spdx.json";
    pub fn sbom_spdx_file_path_path() -> PathBuf {
        PathBuf::from(SBOM_BASE_PATH).join(SBOM_SPDX_FILE_NAME)
    }
}

#[cfg(test)]
pub const BASE_PATH: &str = "/tmp/flecs-tests/var/lib/flecs/";
#[cfg(not(test))]
pub const BASE_PATH: &str = "/var/lib/flecs/";
#[cfg(test)]
pub const CONFIG_PATH: &str = "/tmp/flecs-tests/var/lib/flecs/config.toml";
#[cfg(not(test))]
pub const CONFIG_PATH: &str = "/var/lib/flecs/config.toml";
pub const FLECSD_SOCKET_PATH: &str = "/run/flecs/flecsd.sock";
pub const FLECSD_PORT: u16 = 8951;
