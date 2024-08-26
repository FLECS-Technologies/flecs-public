pub mod console_client_config {
    use crate::fsm::console_client::create_default_client_with_middleware;
    use flecs_console_client::apis::configuration::Configuration;
    use std::sync::OnceLock;

    pub fn default() -> &'static Configuration {
        static CONSOLE_CLIENT_CONFIG: OnceLock<Configuration> = OnceLock::new();
        #[cfg(debug_assertions)]
        const BASE_PATH: &str = "https://console-dev.flecs.tech";
        #[cfg(not(debug_assertions))]
        const BASE_PATH: &str = "https://console.flecs.tech";
        CONSOLE_CLIENT_CONFIG.get_or_init(|| Configuration {
            base_path: BASE_PATH.to_owned(),
            client: create_default_client_with_middleware(),
            ..Configuration::default()
        })
    }
}

pub mod vault {
    use crate::vault::{Vault, VaultConfig};
    use std::sync::{Arc, OnceLock};

    pub fn default() -> Arc<Vault> {
        static DEFAULT_VAULT: OnceLock<Arc<Vault>> = OnceLock::new();
        DEFAULT_VAULT
            .get_or_init(|| {
                let vault = Vault::new(VaultConfig::default());
                vault.open();
                Arc::new(vault)
            })
            .clone()
    }
}

pub const BASE_PATH: &str = "/var/lib/flecs/";
