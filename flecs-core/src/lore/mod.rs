pub mod console_client_config {
    use crate::fsm::console_client::create_default_client_with_middleware;
    use flecs_console_client::apis::configuration::Configuration;
    use tokio::sync::OnceCell;

    pub async fn default() -> &'static Configuration {
        static CONSOLE_CLIENT_CONFIG: OnceCell<Configuration> = OnceCell::const_new();
        #[cfg(debug_assertions)]
        const BASE_PATH: &str = "https://console-dev.flecs.tech";
        #[cfg(not(debug_assertions))]
        const BASE_PATH: &str = "https://console.flecs.tech";
        CONSOLE_CLIENT_CONFIG
            .get_or_init(|| async {
                Configuration {
                    base_path: BASE_PATH.to_owned(),
                    client: create_default_client_with_middleware().await,
                    ..Configuration::default()
                }
            })
            .await
    }
}

pub mod vault {
    use crate::vault::{Vault, VaultConfig};
    use std::sync::Arc;
    use tokio::sync::OnceCell;

    pub async fn default() -> Arc<Vault> {
        static DEFAULT_VAULT: OnceCell<Arc<Vault>> = OnceCell::const_new();
        DEFAULT_VAULT
            .get_or_init(|| async {
                let vault = Vault::new(VaultConfig::default());
                vault.open().await;
                Arc::new(vault)
            })
            .await
            .clone()
    }
}

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

pub const BASE_PATH: &str = "/var/lib/flecs/";
pub const MAX_SUPPORTED_APP_MANIFEST_VERSION: &str = "3.0.0";
pub const API_VERSION: &str = env!("FLECS_API_VERSION");
pub const CORE_VERSION: &str = concat!(env!("FLECS_VERSION"), "-", env!("FLECS_GIT_SHA"));
