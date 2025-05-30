use std::path::{Path, PathBuf};

pub mod tracing {
    use tracing_subscriber::EnvFilter;

    #[cfg(debug_assertions)]
    const DEFAULT_TRACING_FILTER: &str = "debug";
    #[cfg(not(debug_assertions))]
    const DEFAULT_TRACING_FILTER: &str = "info,tower_http=debug,axum::rejection=debug";
    pub fn default_filter() -> EnvFilter {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| DEFAULT_TRACING_FILTER.into())
    }
}

pub mod flecsport {
    use std::time::Duration;

    #[cfg(test)]
    pub const BASE_PATH: &str = "/tmp/flecs-tests/var/lib/flecs/exports";
    #[cfg(not(test))]
    pub const BASE_PATH: &str = "/var/lib/flecs/exports";
    pub const APP_EXPORT_TIMEOUT: Duration = Duration::from_secs(u64::MAX);
}

pub mod flimport {
    use std::time::Duration;

    #[cfg(test)]
    pub const BASE_PATH: &str = "/tmp/flecs-tests/var/lib/flecs/imports";
    #[cfg(not(test))]
    pub const BASE_PATH: &str = "/var/lib/flecs/imports";
    pub const APP_IMPORT_TIMEOUT: Duration = Duration::from_secs(u64::MAX);
}

pub mod floxy {
    use crate::jeweler::gem::instance::InstanceId;

    pub const BASE_PATH: &str = "/var/lib/flecs/floxy";
    pub const CONFIG_PATH: &str = "/etc/nginx/floxy.conf";

    pub fn instance_editor_location(instance_id: InstanceId, port: u16) -> String {
        format!("/v2/instances/{instance_id}/editor/{port}")
    }
}

pub mod console {
    #[cfg(debug_assertions)]
    pub const BASE_PATH: &str = "https://console-dev.flecs.tech";
    #[cfg(not(debug_assertions))]
    pub const BASE_PATH: &str = "https://console.flecs.tech";
}

pub mod network {
    use crate::jeweler::network::{NetworkConfig, NetworkKind};
    use crate::relic;
    use std::net::Ipv4Addr;

    pub fn default_network_name() -> &'static str {
        "flecs"
    }

    pub fn default_cidr_subnet() -> relic::network::Ipv4Network {
        Default::default()
    }

    pub fn default_gateway() -> Ipv4Addr {
        Ipv4Addr::new(172, 21, 0, 1)
    }

    pub fn default_network_config() -> NetworkConfig {
        NetworkConfig {
            kind: NetworkKind::Bridge,
            name: default_network_name().to_string(),
            cidr_subnet: Some(default_cidr_subnet()),
            gateway: Some(default_gateway()),
            parent_adapter: None,
            options: Default::default(),
        }
    }
}

pub fn base_path() -> &'static Path {
    Path::new(BASE_PATH)
}

pub fn instance_config_path(instance_id: &impl AsRef<str>) -> PathBuf {
    Path::new(BASE_PATH)
        .join("instances")
        .join(instance_id.as_ref())
        .join("conf")
}

pub fn instance_workdir_path(instance_id: &impl AsRef<str>) -> PathBuf {
    Path::new(BASE_PATH)
        .join("instances")
        .join(instance_id.as_ref())
        .join("work")
}

#[cfg(test)]
pub const BASE_PATH: &str = "/tmp/flecs-tests/var/lib/flecs/";
#[cfg(not(test))]
pub const BASE_PATH: &str = "/var/lib/flecs/";
pub const MAX_SUPPORTED_APP_MANIFEST_VERSION: &str = "3.0.0";
pub const API_VERSION: &str = env!("FLECS_API_VERSION");
pub const CORE_VERSION: &str = concat!(env!("FLECS_VERSION"), "-", env!("FLECS_GIT_SHA"));
pub const FLECSD_SOCKET_PATH: &str = "/run/flecs/flecsd.sock";
