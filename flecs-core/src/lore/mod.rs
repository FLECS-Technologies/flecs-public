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
    #[cfg(test)]
    pub const BASE_PATH: &str = "/tmp/flecs-tests/var/lib/flecs/exports";
    #[cfg(not(test))]
    pub const BASE_PATH: &str = "/var/lib/flecs/exports";
}

pub mod floxy {
    pub const BASE_PATH: &str = "/var/lib/flecs/floxy";
    pub const CONFIG_PATH: &str = "/etc/nginx/floxy.conf";
}

pub mod console {
    #[cfg(debug_assertions)]
    pub const BASE_PATH: &str = "https://console-dev.flecs.tech";
    #[cfg(not(debug_assertions))]
    pub const BASE_PATH: &str = "https://console.flecs.tech";
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
#[cfg(test)]
pub const BASE_PATH: &str = "/tmp/flecs-tests/var/lib/flecs/";
#[cfg(not(test))]
pub const BASE_PATH: &str = "/var/lib/flecs/";
pub const MAX_SUPPORTED_APP_MANIFEST_VERSION: &str = "3.0.0";
pub const API_VERSION: &str = env!("FLECS_API_VERSION");
pub const CORE_VERSION: &str = concat!(env!("FLECS_VERSION"), "-", env!("FLECS_GIT_SHA"));
pub const FLECSD_SOCKET_PATH: &str = "/run/flecs/flecsd.sock";
