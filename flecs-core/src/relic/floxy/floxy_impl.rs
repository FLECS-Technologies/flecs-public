use crate::jeweler::gem::instance::InstanceId;
use crate::lore::{FloxyLore, FloxyLoreRef};
use crate::relic::floxy::{AdditionalLocationInfo, Floxy};
use crate::relic::network::get_random_free_port;
use std::fmt::{Display, Formatter};
use std::fs::DirEntry;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info};

const CONFIG_EXTENSION: &str = "conf";
#[derive(Default)]
pub struct FloxyImpl;

impl Floxy for FloxyImpl {
    fn add_instance_reverse_proxy_config(
        &self,
        lore: FloxyLoreRef,
        app_name: &str,
        instance_id: InstanceId,
        instance_ip: IpAddr,
        dest_ports: &[u16],
        auth_provider_port: Option<u16>,
    ) -> anyhow::Result<()> {
        let config_content = Self::create_instance_reverse_proxy_config(
            instance_id,
            instance_ip,
            dest_ports.iter(),
            auth_provider_port,
        );
        let config_path = Self::build_instance_config_path(&lore, app_name, instance_id);
        Self::add_reverse_proxy_config(lore, &config_content, &config_path)?;
        debug!("Added reverse proxy config for instance {instance_id} at {config_path:?}");
        Ok(())
    }

    fn add_additional_locations_proxy_config(
        &self,
        lore: FloxyLoreRef,
        app_name: &str,
        instance_id: InstanceId,
        additional_locations: &[AdditionalLocationInfo],
    ) -> anyhow::Result<()> {
        let config_content =
            Self::create_additional_location_proxy_config(instance_id, additional_locations.iter());
        let config_path = Self::build_instance_locations_config_path(&lore, app_name, instance_id);
        Self::add_reverse_proxy_config(lore, &config_content, &config_path)?;
        debug!(
            "Added additional location proxy config for instance {instance_id} at {config_path:?}"
        );
        Ok(())
    }

    fn delete_additional_locations_proxy_config(
        &self,
        lore: FloxyLoreRef,
        app_name: &str,
        instance_id: InstanceId,
    ) -> anyhow::Result<()> {
        let config_path = Self::build_instance_locations_config_path(&lore, app_name, instance_id);
        if matches!(config_path.try_exists(), Ok(false)) {
            return Ok(());
        }
        match std::fs::remove_file(&config_path) {
            Ok(()) => {
                debug!(
                    "Removed additional locations reverse proxy config for instance {instance_id} at {config_path:?}."
                );
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Error deleting {config_path:?}: {e}")),
        }
    }

    fn delete_reverse_proxy_config(
        &self,
        lore: FloxyLoreRef,
        app_name: &str,
        instance_id: InstanceId,
    ) -> crate::Result<()> {
        let config_path = Self::build_instance_config_path(&lore, app_name, instance_id);
        if matches!(config_path.try_exists(), Ok(false)) {
            return Ok(());
        }
        match std::fs::remove_file(&config_path) {
            Ok(()) => {
                debug!(
                    "Removed reverse proxy config for instance {instance_id} at {config_path:?}."
                );
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Error deleting {config_path:?}: {e}")),
        }
    }

    fn delete_server_config(
        &self,
        lore: FloxyLoreRef,
        app_name: &str,
        instance_id: InstanceId,
        host_port: u16,
    ) -> anyhow::Result<()> {
        let config_path = Self::build_server_config_path(&lore, app_name, instance_id, host_port);
        if matches!(config_path.try_exists(), Ok(false)) {
            return Ok(());
        }
        match std::fs::remove_file(&config_path) {
            Ok(()) => {
                debug!(
                    "Removed server config for instance {instance_id} and port {host_port} at {config_path:?}."
                );
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Error deleting {config_path:?}: {e}")),
        }
    }

    fn delete_server_proxy_configs(
        &self,
        lore: FloxyLoreRef,
        app_name: &str,
        instance_id: InstanceId,
        host_ports: &[u16],
    ) -> crate::Result<()> {
        let mut delete_failures = Vec::new();
        for host_port in host_ports {
            if let Err(e) =
                self.delete_server_config(lore.clone(), app_name, instance_id, *host_port)
            {
                delete_failures.push(e.to_string());
            }
        }
        if delete_failures.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Could not delete all server proxy configs: [{}], {self}",
                delete_failures.join(",")
            ))
        }
    }

    /// Returns a pair of a bool which indicates whether a reload is necessary and an u16 which
    /// is the chosen free port to which will be redirected.
    fn add_instance_editor_redirect_to_free_port(
        &self,
        lore: FloxyLoreRef,
        app_name: &str,
        instance_id: InstanceId,
        instance_ip: IpAddr,
        dest_port: u16,
    ) -> crate::Result<u16> {
        let free_port = get_random_free_port()?;
        self.add_instance_redirect(
            lore,
            app_name,
            instance_id,
            instance_ip,
            free_port,
            dest_port,
        )?;
        Ok(free_port)
    }

    fn add_instance_redirect(
        &self,
        lore: FloxyLoreRef,
        app_name: &str,
        instance_id: InstanceId,
        instance_ip: IpAddr,
        src_port: u16,
        dest_port: u16,
    ) -> anyhow::Result<()> {
        let config_content = Self::create_server_config(instance_ip, src_port, dest_port);
        let config_path = Self::build_server_config_path(&lore, app_name, instance_id, src_port);
        Self::add_reverse_proxy_config(lore, &config_content, &config_path)?;
        debug!(
            "Added redirect for instance {instance_id} at {config_path:?}: host:{src_port} -> {instance_ip}:{dest_port}"
        );
        Ok(())
    }

    fn clear_server_configs(&self, lore: FloxyLoreRef) -> anyhow::Result<()> {
        let server_dir = lore.as_ref().as_ref().server_config_path();
        Self::clear_configs(&lore, &server_dir)
    }

    fn clear_instance_configs(&self, lore: FloxyLoreRef) -> anyhow::Result<()> {
        let instance_dir = lore.as_ref().as_ref().instance_config_path();
        Self::clear_configs(&lore, &instance_dir)
    }
}

impl Display for FloxyImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FloxyImpl",)
    }
}

impl FloxyImpl {
    fn create_instance_reverse_proxy_config<'a, I: Iterator<Item = &'a u16>>(
        instance_id: InstanceId,
        instance_ip: IpAddr,
        dest_ports: I,
        auth_provider_port: Option<u16>,
    ) -> String {
        dest_ports
            .map(|port| {
                Self::create_instance_config(
                    instance_ip,
                    *port,
                    &FloxyLore::instance_editor_location(instance_id, *port),
                )
            })
            .chain(auth_provider_port.map(|auth_provider_port| {
                Self::create_instance_config(
                    instance_ip,
                    auth_provider_port,
                    &FloxyLore::auth_provider_location(instance_id),
                )
            }))
            .collect::<String>()
    }

    fn create_additional_location_proxy_config<
        'a,
        I: Iterator<Item = &'a AdditionalLocationInfo>,
    >(
        instance_id: InstanceId,
        additional_locations: I,
    ) -> String {
        additional_locations
            .map(|additional_location| {
                Self::create_location_config(
                    &format!(
                        "/api/{}",
                        &FloxyLore::instance_editor_api_location(
                            instance_id,
                            additional_location.port
                        )
                    ),
                    &additional_location.location,
                )
            })
            .collect::<String>()
    }

    /// Creates a config with the given content at the given path. Returns Ok(true) if the file
    /// was created and Ok(false) if the file with the exact content already exists.
    fn add_reverse_proxy_config(
        lore: FloxyLoreRef,
        config: &str,
        config_path: &Path,
    ) -> crate::Result<bool> {
        anyhow::ensure!(
            config_path.starts_with(&lore.as_ref().as_ref().base_path),
            "The config path ({config_path:?}) has to be inside the floxy base directory"
        );
        if let Ok(true) = config_path.try_exists() {
            if std::fs::read_to_string(config_path)? == config {
                return Ok(false);
            }
        }
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(config_path, config.as_bytes())?;
        Ok(true)
    }

    fn build_server_config_path(
        lore: &FloxyLoreRef,
        app_name: &str,
        instance_id: InstanceId,
        host_port: u16,
    ) -> PathBuf {
        lore.as_ref().as_ref().server_config_path().join(format!(
            "{app_name}-{instance_id}_{host_port}.{CONFIG_EXTENSION}"
        ))
    }

    fn build_instance_config_path(
        lore: &FloxyLoreRef,
        app_name: &str,
        instance_id: InstanceId,
    ) -> PathBuf {
        lore.as_ref()
            .as_ref()
            .instance_config_path()
            .join(format!("{app_name}-{instance_id}.{CONFIG_EXTENSION}"))
    }

    fn build_instance_locations_config_path(
        lore: &FloxyLoreRef,
        app_name: &str,
        instance_id: InstanceId,
    ) -> PathBuf {
        lore.as_ref().as_ref().instance_config_path().join(format!(
            "{app_name}-{instance_id}-locations.{CONFIG_EXTENSION}"
        ))
    }

    fn delete_config_entry(lore: &FloxyLoreRef, entry: &DirEntry) -> crate::Result<()> {
        let path = entry.path();
        let base_path = &lore.as_ref().as_ref().base_path;
        anyhow::ensure!(
            path.starts_with(base_path),
            "The config path ({path:?}) has to be inside the floxy base directory {base_path:?}",
        );
        let meta = entry.metadata()?;
        if (meta.is_symlink() || meta.is_file())
            && path.extension() == Some(CONFIG_EXTENSION.as_ref())
        {
            std::fs::remove_file(&path)?;
            debug!("Removed config entry {path:?} {}", Self);
        }
        Ok(())
    }

    fn create_instance_config(instance_ip: IpAddr, dest_port: u16, location: &str) -> String {
        format!(
            "
location {location}/ {{
  proxy_pass http://{instance_ip}:{dest_port}/;
  proxy_redirect / {location}/;

  include conf.d/include/proxy_headers.conf;

  client_max_body_size 0;
  client_body_timeout 30m;
}}"
        )
    }

    fn create_location_config(location: &str, additional_location: &str) -> String {
        format!(
            "
location {additional_location} {{
  return 307 {location};
}}
location ~ ^{additional_location}/(.*) {{
  return 307 {location}/$1;
}}"
        )
    }

    fn create_server_config(instance_ip: IpAddr, host_port: u16, dest_port: u16) -> String {
        format!(
            "
server {{
  listen {host_port};
  location / {{
    proxy_pass http://{instance_ip}:{dest_port}/;

    include conf.d/include/proxy_headers.conf;

    client_max_body_size 0;
    client_body_timeout 30m;
  }}
}}"
        )
    }

    fn clear_configs(lore: &FloxyLoreRef, path: &Path) -> anyhow::Result<()> {
        let mut failed_deletes = Vec::new();
        for entry in std::fs::read_dir(path)? {
            match entry {
                Err(e) => error!("Error during deletion of floxy config from {path:?}: {e}"),
                Ok(entry) => {
                    if let Err(e) = Self::delete_config_entry(lore, &entry) {
                        failed_deletes.push(format!("{:?}: {e}", entry.path()));
                    }
                }
            }
        }
        if failed_deletes.is_empty() {
            info!("All floxy configs deleted from {path:?} {}", Self);
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Could not delete all floxy configs from {path:?} ({})",
                failed_deletes.join(",")
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lore;
    use crate::relic::var::test::MockVarReader;
    use std::fs;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use testdir::testdir;

    #[test]
    fn display_test() {
        assert_eq!(FloxyImpl.to_string(), "FloxyImpl");
    }

    const EXPECTED_TRIPLE_CONFIG: &str = "
location /flecs/instances/1234abcd/editor/5000/ {
  proxy_pass http://123.123.234.234:5000/;
  proxy_redirect / /flecs/instances/1234abcd/editor/5000/;

  include conf.d/include/proxy_headers.conf;

  client_max_body_size 0;
  client_body_timeout 30m;
}
location /flecs/instances/1234abcd/editor/6000/ {
  proxy_pass http://123.123.234.234:6000/;
  proxy_redirect / /flecs/instances/1234abcd/editor/6000/;

  include conf.d/include/proxy_headers.conf;

  client_max_body_size 0;
  client_body_timeout 30m;
}
location /flecs/instances/1234abcd/editor/7000/ {
  proxy_pass http://123.123.234.234:7000/;
  proxy_redirect / /flecs/instances/1234abcd/editor/7000/;

  include conf.d/include/proxy_headers.conf;

  client_max_body_size 0;
  client_body_timeout 30m;
}";

    const TRIPLE_DEST_PORTS: [u16; 3] = [5000, 6000, 7000];

    #[test]
    fn create_instance_reverse_proxy_config_test() {
        let config = FloxyImpl::create_instance_reverse_proxy_config(
            InstanceId::new(0x1234abcd),
            IpAddr::V4(Ipv4Addr::new(123, 123, 234, 234)),
            TRIPLE_DEST_PORTS.iter(),
            None,
        );
        assert_eq!(config, EXPECTED_TRIPLE_CONFIG);
    }

    #[test]
    fn add_instance_reverse_proxy_config_new() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let config_path = lore
            .floxy
            .instance_config_path()
            .join("test_app-1234abcd.conf");
        assert!(matches!(
            FloxyImpl.add_instance_reverse_proxy_config(
                lore,
                "test_app",
                InstanceId::new(0x1234abcd),
                IpAddr::V4(Ipv4Addr::new(123, 123, 234, 234)),
                &TRIPLE_DEST_PORTS,
                None
            ),
            Ok(())
        ));
        assert_eq!(
            std::fs::read_to_string(config_path).unwrap(),
            EXPECTED_TRIPLE_CONFIG
        );
    }

    #[test]
    fn add_instance_reverse_proxy_config_unchanged() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let config_path = lore
            .floxy
            .instance_config_path()
            .join("test_app-1234abcd.conf");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, EXPECTED_TRIPLE_CONFIG).unwrap();
        assert!(matches!(
            FloxyImpl.add_instance_reverse_proxy_config(
                lore,
                "test_app",
                InstanceId::new(0x1234abcd),
                IpAddr::V4(Ipv4Addr::new(123, 123, 234, 234)),
                &TRIPLE_DEST_PORTS,
                None
            ),
            Ok(())
        ));
        assert_eq!(
            std::fs::read_to_string(config_path).unwrap(),
            EXPECTED_TRIPLE_CONFIG
        );
    }

    #[test]
    fn add_instance_reverse_proxy_config_changed() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let config_path = lore
            .floxy
            .instance_config_path()
            .join("test_app-1234abcd.conf");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, "test config").unwrap();
        assert!(matches!(
            FloxyImpl.add_instance_reverse_proxy_config(
                lore,
                "test_app",
                InstanceId::new(0x1234abcd),
                IpAddr::V4(Ipv4Addr::new(123, 123, 234, 234)),
                &TRIPLE_DEST_PORTS,
                None
            ),
            Ok(())
        ));
        assert_eq!(
            std::fs::read_to_string(config_path).unwrap(),
            EXPECTED_TRIPLE_CONFIG
        );
    }

    #[test]
    fn create_server_config_test() {
        const EXPECTED_CONFIG: &str = "
server {
  listen 5050;
  location / {
    proxy_pass http://123.123.234.234:9090/;

    include conf.d/include/proxy_headers.conf;

    client_max_body_size 0;
    client_body_timeout 30m;
  }
}";
        assert_eq!(
            FloxyImpl::create_server_config(
                IpAddr::V4(Ipv4Addr::new(123, 123, 234, 234)),
                5050,
                9090
            ),
            EXPECTED_CONFIG
        )
    }

    #[test]
    fn create_instance_config_test() {
        const EXPECTED_CONFIG: &str = "
location TEST_LOCATION/ {
  proxy_pass http://30.60.120.240:7799/;
  proxy_redirect / TEST_LOCATION/;

  include conf.d/include/proxy_headers.conf;

  client_max_body_size 0;
  client_body_timeout 30m;
}";
        assert_eq!(
            FloxyImpl::create_instance_config(
                IpAddr::V4(Ipv4Addr::new(30, 60, 120, 240)),
                7799,
                "TEST_LOCATION",
            ),
            EXPECTED_CONFIG
        )
    }

    #[test]
    fn delete_reverse_proxy_config_ok() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let config_path = lore
            .floxy
            .instance_config_path()
            .join("test_app-abcd1234.conf");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, "test config").unwrap();
        FloxyImpl
            .delete_reverse_proxy_config(lore, "test_app", InstanceId::new(0xabcd1234))
            .unwrap();
        assert!(matches!(config_path.try_exists(), Ok(false)));
    }

    #[test]
    fn delete_reverse_proxy_config_not_existing() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        FloxyImpl
            .delete_reverse_proxy_config(lore, "test_app", InstanceId::new(0xabcd1234))
            .unwrap();
    }

    #[test]
    fn delete_reverse_proxy_config_err() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let config_path = lore
            .floxy
            .instance_config_path()
            .join("test_app-abcd1234.conf");
        std::fs::create_dir_all(config_path).unwrap();
        assert!(
            FloxyImpl
                .delete_reverse_proxy_config(lore, "test_app", InstanceId::new(0xabcd1234))
                .is_err()
        );
    }

    #[test]
    fn delete_server_config_ok() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let config_path = lore
            .floxy
            .server_config_path()
            .join("test_app-abcd1234_1234.conf");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, "test config").unwrap();
        FloxyImpl
            .delete_server_config(lore, "test_app", InstanceId::new(0xabcd1234), 1234)
            .unwrap();
        assert!(matches!(config_path.try_exists(), Ok(false)));
    }

    #[test]
    fn delete_server_config_err() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let config_path = lore
            .floxy
            .server_config_path()
            .join("test_app-abcd1234_1234.conf");
        std::fs::create_dir_all(config_path).unwrap();
        assert!(
            FloxyImpl
                .delete_server_config(lore, "test_app", InstanceId::new(0xabcd1234), 1234)
                .is_err()
        );
    }

    #[test]
    fn build_server_config_path_test() {
        let lore: FloxyLoreRef = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let config_path = lore
            .as_ref()
            .as_ref()
            .server_config_path()
            .join("test_app-ab12cd34_1234.conf");
        assert_eq!(
            FloxyImpl::build_server_config_path(
                &lore,
                "test_app",
                InstanceId::new(0xab12cd34),
                1234
            ),
            config_path
        )
    }

    #[test]
    fn build_instance_config_path_test() {
        let lore: FloxyLoreRef = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let config_path = lore
            .as_ref()
            .as_ref()
            .instance_config_path()
            .join("test_app-ab12cd34.conf");
        assert_eq!(
            FloxyImpl::build_instance_config_path(&lore, "test_app", InstanceId::new(0xab12cd34)),
            config_path
        )
    }

    #[test]
    fn delete_config_entry_outside_base_path_err() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let config_path = lore.base_path.join("test.conf");
        let config_test_dir = lore.base_path.clone();
        let lore: FloxyLoreRef = lore;
        std::fs::create_dir_all(&config_test_dir).unwrap();
        std::fs::write(config_path, "test config").unwrap();
        let entry = std::fs::read_dir(config_test_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        assert!(FloxyImpl::delete_config_entry(&lore, &entry).is_err());
    }

    #[test]
    fn delete_config_entry_not_file() {
        let lore: FloxyLoreRef = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let config_path = lore.as_ref().as_ref().base_path.join("test.conf");
        let config_test_dir = lore.as_ref().as_ref().base_path.clone();
        std::fs::create_dir_all(&config_path).unwrap();
        let entry = std::fs::read_dir(config_test_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        FloxyImpl::delete_config_entry(&lore, &entry).unwrap();
        assert!(matches!(config_path.try_exists(), Ok(true)));
    }

    #[test]
    fn delete_config_entry_not_config() {
        let lore: FloxyLoreRef = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let config_path = lore.as_ref().as_ref().base_path.join("test.txt");
        let config_test_dir = lore.as_ref().as_ref().base_path.clone();
        std::fs::create_dir_all(&config_test_dir).unwrap();
        std::fs::write(&config_path, "test config").unwrap();
        let entry = std::fs::read_dir(config_test_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        FloxyImpl::delete_config_entry(&lore, &entry).unwrap();
        assert!(matches!(config_path.try_exists(), Ok(true)));
    }

    #[test]
    fn delete_config_entry_ok() {
        let lore: FloxyLoreRef = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let config_path = lore.as_ref().as_ref().base_path.join("test.conf");
        let config_test_dir = lore.as_ref().as_ref().base_path.clone();
        std::fs::create_dir_all(&config_test_dir).unwrap();
        std::fs::write(&config_path, "test config").unwrap();
        let entry = std::fs::read_dir(config_test_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        FloxyImpl::delete_config_entry(&lore, &entry).unwrap();
        assert!(matches!(config_path.try_exists(), Ok(false)));
    }

    #[test]
    fn clear_server_configs_ok() {
        let lore: FloxyLoreRef = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let server_dir = lore.as_ref().as_ref().server_config_path();
        std::fs::create_dir_all(&server_dir).unwrap();
        for i in 0..10 {
            std::fs::write(server_dir.join(format!("test{i}.conf")), "test config").unwrap();
        }
        std::fs::write(server_dir.join("test.file"), "test file").unwrap();
        std::fs::create_dir_all(server_dir.join("test.dir")).unwrap();
        FloxyImpl.clear_server_configs(lore).unwrap();
        for i in 0..10 {
            assert!(matches!(
                server_dir.join(format!("test{i}.conf")).try_exists(),
                Ok(false)
            ));
        }
        assert!(matches!(
            server_dir.join("test.file").try_exists(),
            Ok(true)
        ));
        assert!(matches!(server_dir.join("test.dir").try_exists(), Ok(true)));
    }

    #[test]
    fn clear_server_proxy_configs_ok() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let server_dir = lore.floxy.server_config_path();
        std::fs::create_dir_all(&server_dir).unwrap();
        std::fs::write(
            server_dir.join("test_app-cdab3412_1234.conf"),
            "test config",
        )
        .unwrap();
        std::fs::write(
            server_dir.join("test_app-cdab3412_5678.conf"),
            "test config",
        )
        .unwrap();
        std::fs::write(server_dir.join("test_app-cdab3412_910.conf"), "test config").unwrap();
        let ports = [1234, 5678, 910];
        FloxyImpl
            .delete_server_proxy_configs(lore, "test_app", InstanceId::new(0xcdab3412), &ports)
            .unwrap();
        assert!(matches!(
            server_dir.join("test_app-cdab3412_1234.conf").try_exists(),
            Ok(false)
        ));
        assert!(matches!(
            server_dir.join("test_app-cdab3412_5678.conf").try_exists(),
            Ok(false)
        ));
        assert!(matches!(
            server_dir.join("test_app-cdab3412_910.conf").try_exists(),
            Ok(false)
        ));
    }

    #[test]
    fn clear_server_proxy_configs_err() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let server_dir = lore.floxy.server_config_path();
        std::fs::create_dir_all(&server_dir).unwrap();
        std::fs::write(
            server_dir.join("test_app-cdab3412_1234.conf"),
            "test config",
        )
        .unwrap();
        std::fs::create_dir_all(server_dir.join("test_app-cdab3412_5678.conf")).unwrap();
        std::fs::write(server_dir.join("test_app-cdab3412_910.conf"), "test config").unwrap();
        let ports = [1234, 5678, 910];
        assert!(
            FloxyImpl
                .delete_server_proxy_configs(lore, "test_app", InstanceId::new(0xcdab3412), &ports)
                .is_err()
        );
        assert!(matches!(
            server_dir.join("test_app-cdab3412_1234.conf").try_exists(),
            Ok(false)
        ));
        assert!(matches!(
            server_dir.join("test_app-cdab3412_5678.conf").try_exists(),
            Ok(true)
        ));
        assert!(matches!(
            server_dir.join("test_app-cdab3412_910.conf").try_exists(),
            Ok(false)
        ));
    }

    #[test]
    fn create_reverse_proxy_config_no_changes() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        fs::create_dir_all(&lore.floxy.base_path).unwrap();
        let config_path = lore.floxy.base_path.join("test.conf");
        std::fs::write(&config_path, "test content").unwrap();
        assert!(matches!(
            FloxyImpl::add_reverse_proxy_config(lore, "test content", &config_path),
            Ok(false)
        ));
        assert_eq!(
            "test content",
            std::fs::read_to_string(config_path).unwrap()
        );
    }

    #[test]
    fn create_reverse_proxy_config_with_changes() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        fs::create_dir_all(&lore.floxy.base_path).unwrap();
        let config_path = lore.floxy.base_path.join("test.conf");
        std::fs::write(&config_path, "old test content").unwrap();
        assert!(matches!(
            FloxyImpl::add_reverse_proxy_config(lore, "test content", &config_path),
            Ok(true)
        ));
        assert_eq!(
            "test content",
            std::fs::read_to_string(config_path).unwrap()
        );
    }

    #[test]
    fn create_reverse_proxy_config_new_config() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let config_path = lore.floxy.base_path.join("test.conf");
        assert!(matches!(
            FloxyImpl::add_reverse_proxy_config(lore, "test content", &config_path),
            Ok(true)
        ));
        assert_eq!(
            "test content",
            std::fs::read_to_string(config_path).unwrap()
        );
    }

    #[test]
    fn create_reverse_proxy_config_outside_base_path() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let config_dir = lore.base_path.join("config");
        let config_path = config_dir.join("test.conf");
        assert!(FloxyImpl::add_reverse_proxy_config(lore, "test content", &config_path).is_err());
        assert!(matches!(config_path.try_exists(), Ok(false)));
    }

    #[test]
    fn create_instance_editor_redirect_to_free_port_ok() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let server_config_path = lore.floxy.server_config_path();
        let port = FloxyImpl
            .add_instance_editor_redirect_to_free_port(
                lore,
                "test app",
                InstanceId::new(0x12345678),
                IpAddr::V4(Ipv4Addr::new(123, 123, 123, 123)),
                50000,
            )
            .unwrap();
        let config_path = server_config_path.join(format!("test app-12345678_{port}.conf"));
        let expected_config_content = format!(
            "
server {{
  listen {port};
  location / {{
    proxy_pass http://123.123.123.123:50000/;

    include conf.d/include/proxy_headers.conf;

    client_max_body_size 0;
    client_body_timeout 30m;
  }}
}}"
        );
        assert_eq!(
            std::fs::read_to_string(config_path).unwrap(),
            expected_config_content
        );
        // Check if port is really free
        std::net::TcpListener::bind(std::net::SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port))
            .unwrap();
    }
}
