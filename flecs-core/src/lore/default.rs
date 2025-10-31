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
    pub const BASE_DIRECTORY_NAME: &str = "floxy";
    pub const CONFIG_PATH: &str = "/etc/nginx/floxy.conf";
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
    use crate::jeweler::network::NetworkKind;
    use crate::relic::network::Ipv4Network;
    use std::collections::HashMap;
    use std::net::Ipv4Addr;

    pub const DEFAULT_NETWORK_NAME: &str = "flecs";
    pub const DEFAULT_CIDR_SUBNET: Ipv4Network = Ipv4Network::default();
    pub const DEFAULT_GATEWAY: Ipv4Addr = Ipv4Addr::new(172, 21, 0, 1);
    pub const DEFAULT_NETWORK_KIND: NetworkKind = NetworkKind::Bridge;

    pub fn default_network_options() -> HashMap<String, String> {
        HashMap::new()
    }
}

pub mod provider {
    pub const BASE_DIRECTORY_NAME: &str = "providers";
}

#[cfg(feature = "auth")]
pub mod auth {
    use std::time::Duration;

    pub const ISSUER_CERTIFICATE_CACHE_LIFETIME: Duration = Duration::from_secs(300);
    pub const BASE_PATH: &str = "/usr/local/share/flecs/auth";
    pub const CASBIN_POLICY_FILE_NAME: &str = "casbin_policy.csv";
    pub const CASBIN_MODEL_FILE_NAME: &str = "casbin_model.conf";
    pub const INITIAL_AUTH_PROVIDER_FLECSPORT_FILE_NAME: &str = "initial_auth_provider.tar.gz";
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
