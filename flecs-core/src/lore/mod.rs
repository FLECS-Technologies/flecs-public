pub mod console_client_config {
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
            ..Configuration::default()
        })
    }
}

pub const BASE_PATH: &str = "/var/lib/flecs/";
