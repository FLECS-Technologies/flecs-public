use crate::vault::VaultConfig;
use std::path::PathBuf;

impl Default for Config {
    fn default() -> Self {
        Self {
            floxy_base_path: PathBuf::from(crate::lore::floxy::BASE_PATH),
            floxy_config_path: PathBuf::from(crate::lore::floxy::CONFIG_PATH),
            socket_path: PathBuf::from(crate::lore::FLECSD_SOCKET_PATH),
            vault_config: VaultConfig::default(),
        }
    }
}

pub struct Config {
    pub floxy_base_path: PathBuf,
    pub floxy_config_path: PathBuf,
    pub socket_path: PathBuf,
    pub vault_config: VaultConfig,
}
