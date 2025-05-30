use crate::enchantment::Enchantment;
use crate::enchantment::floxy::{AdditionalLocationInfo, Floxy};
use crate::jeweler::gem::instance::InstanceId;
use crate::lore;
use crate::relic::network::get_random_free_port;
use crate::relic::nginx::Nginx;
use anyhow::Error;
use std::fmt::{Display, Formatter};
use std::fs::DirEntry;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info};

const SERVER_CONFIGS_DIR_NAME: &str = "servers";
const INSTANCE_CONFIGS_DIR_NAME: &str = "instances";
const CONFIG_EXTENSION: &str = "conf";
pub struct FloxyImpl {
    nginx: Nginx,
    base_path: PathBuf,
}

impl Enchantment for FloxyImpl {}

impl Floxy for FloxyImpl {
    fn start(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(self.servers_path())?;
        std::fs::create_dir_all(self.instances_path())?;
        self.clear_server_configs()?;
        self.nginx.start()
    }

    fn stop(&self) -> anyhow::Result<()> {
        self.nginx.graceful_shutdown()
    }

    fn add_instance_reverse_proxy_config(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        instance_ip: IpAddr,
        dest_ports: &[u16],
    ) -> anyhow::Result<bool> {
        let config_content =
            Self::create_instance_reverse_proxy_config(instance_id, instance_ip, dest_ports.iter());
        let config_path = self.build_instance_config_path(app_name, instance_id);
        let config_changed = self.add_reverse_proxy_config(&config_content, &config_path)?;
        debug!("Added reverse proxy config for instance {instance_id} at {config_path:?}");
        Ok(config_changed)
    }

    fn add_additional_locations_proxy_config(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        additional_locations: &[AdditionalLocationInfo],
    ) -> anyhow::Result<bool> {
        let config_content =
            Self::create_additional_location_proxy_config(instance_id, additional_locations.iter());
        let config_path = self.build_instance_locations_config_path(app_name, instance_id);
        let config_changed = self.add_reverse_proxy_config(&config_content, &config_path)?;
        debug!(
            "Added additional location proxy config for instance {instance_id} at {config_path:?}"
        );
        Ok(config_changed)
    }

    fn delete_additional_locations_proxy_config(
        &self,
        app_name: &str,
        instance_id: InstanceId,
    ) -> anyhow::Result<bool> {
        let config_path = self.build_instance_locations_config_path(app_name, instance_id);
        if matches!(config_path.try_exists(), Ok(false)) {
            return Ok(false);
        }
        match std::fs::remove_file(&config_path) {
            Ok(()) => {
                debug!(
                    "Removed additional locations reverse proxy config for instance {instance_id} at {config_path:?}."
                );
                Ok(true)
            }
            Err(e) => Err(anyhow::anyhow!("Error deleting {config_path:?}: {e}")),
        }
    }

    fn delete_reverse_proxy_config(
        &self,
        app_name: &str,
        instance_id: InstanceId,
    ) -> crate::Result<bool> {
        let config_path = self.build_instance_config_path(app_name, instance_id);
        if matches!(config_path.try_exists(), Ok(false)) {
            return Ok(false);
        }
        match std::fs::remove_file(&config_path) {
            Ok(()) => {
                debug!(
                    "Removed reverse proxy config for instance {instance_id} at {config_path:?}."
                );
                Ok(true)
            }
            Err(e) => Err(anyhow::anyhow!("Error deleting {config_path:?}: {e}")),
        }
    }

    fn delete_server_config(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        host_port: u16,
    ) -> anyhow::Result<bool> {
        let config_path = self.build_server_config_path(app_name, instance_id, host_port);
        if matches!(config_path.try_exists(), Ok(false)) {
            return Ok(false);
        }
        match std::fs::remove_file(&config_path) {
            Ok(()) => {
                debug!(
                    "Removed server config for instance {instance_id} and port {host_port} at {config_path:?}."
                );
                Ok(true)
            }
            Err(e) => Err(anyhow::anyhow!("Error deleting {config_path:?}: {e}")),
        }
    }

    fn delete_server_proxy_configs(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        host_ports: &[u16],
    ) -> Result<bool, (bool, Error)> {
        let mut delete_failures = Vec::new();
        let mut reload = false;
        for host_port in host_ports {
            match self.delete_server_config(app_name, instance_id, *host_port) {
                Err(e) => {
                    delete_failures.push(e.to_string());
                }
                Ok(delete_success) => reload |= delete_success,
            }
        }
        if delete_failures.is_empty() {
            Ok(reload)
        } else {
            Err((
                reload,
                anyhow::anyhow!(
                    "Could not delete all server proxy configs: [{}], {self}",
                    delete_failures.join(",")
                ),
            ))
        }
    }

    /// Returns a pair of a bool which indicates whether a reload is necessary and an u16 which
    /// is the chosen free port to which will be redirected.
    fn add_instance_editor_redirect_to_free_port(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        instance_ip: IpAddr,
        dest_port: u16,
    ) -> crate::Result<(bool, u16)> {
        let free_port = get_random_free_port()?;
        let config_content = Self::create_server_config(instance_ip, free_port, dest_port);
        let config_path = self.build_server_config_path(app_name, instance_id, free_port);
        let config_changed = self.add_reverse_proxy_config(&config_content, &config_path)?;
        debug!(
            "Added editor redirect for instance {instance_id} at {config_path:?}: host:{free_port} -> {instance_ip}:{dest_port}"
        );
        Ok((config_changed, free_port))
    }

    fn reload_config(&self) -> anyhow::Result<()> {
        self.nginx.reload_config()?;
        info!("Nginx config reload triggered {self}");
        Ok(())
    }

    fn clear_server_configs(&self) -> anyhow::Result<()> {
        let mut failed_deletes = Vec::new();
        let server_dir = self.servers_path();
        for entry in std::fs::read_dir(&server_dir)? {
            match entry {
                Err(e) => error!("Error during deletion of floxy servers from {server_dir:?}: {e}"),
                Ok(entry) => {
                    if let Err(e) = self.delete_config_entry(&entry) {
                        failed_deletes.push(format!("{:?}: {e}", entry.path()));
                    }
                }
            }
        }
        if failed_deletes.is_empty() {
            info!("All floxy server configs deleted {self}");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Could not delete all floxy server configs ({})",
                failed_deletes.join(",")
            ))
        }
    }
}

impl Display for FloxyImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(Floxy: (base_path = {}, nginx = {})",
            self.base_path.to_string_lossy(),
            self.nginx
        )
    }
}

impl FloxyImpl {
    pub fn from_config(base_path: PathBuf, config_path: PathBuf) -> crate::Result<Self> {
        Ok(Self {
            nginx: Nginx::from_config(config_path)?,
            base_path,
        })
    }

    fn servers_path(&self) -> PathBuf {
        self.base_path.join(SERVER_CONFIGS_DIR_NAME)
    }

    fn instances_path(&self) -> PathBuf {
        self.base_path.join(INSTANCE_CONFIGS_DIR_NAME)
    }

    fn create_instance_reverse_proxy_config<'a, I: Iterator<Item = &'a u16>>(
        instance_id: InstanceId,
        instance_ip: IpAddr,
        dest_ports: I,
    ) -> String {
        dest_ports
            .map(|port| {
                Self::create_instance_config(
                    instance_ip,
                    *port,
                    &lore::floxy::instance_editor_location(instance_id, *port),
                )
            })
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
                    &lore::floxy::instance_editor_location(instance_id, additional_location.port),
                    &additional_location.location,
                )
            })
            .collect::<String>()
    }

    /// Creates a config with the given content at the given path. Returns Ok(true) if the file
    /// was created and Ok(false) if the file with the exact content already exists.
    fn add_reverse_proxy_config(&self, config: &str, config_path: &Path) -> crate::Result<bool> {
        anyhow::ensure!(
            config_path.starts_with(&self.base_path),
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
        &self,
        app_name: &str,
        instance_id: InstanceId,
        host_port: u16,
    ) -> PathBuf {
        self.servers_path().join(format!(
            "{app_name}-{instance_id}_{host_port}.{CONFIG_EXTENSION}"
        ))
    }

    fn build_instance_config_path(&self, app_name: &str, instance_id: InstanceId) -> PathBuf {
        self.instances_path()
            .join(format!("{app_name}-{instance_id}.{CONFIG_EXTENSION}"))
    }

    fn build_instance_locations_config_path(
        &self,
        app_name: &str,
        instance_id: InstanceId,
    ) -> PathBuf {
        self.instances_path().join(format!(
            "{app_name}-{instance_id}-locations.{CONFIG_EXTENSION}"
        ))
    }

    fn delete_config_entry(&self, entry: &DirEntry) -> crate::Result<()> {
        let path = entry.path();
        anyhow::ensure!(
            path.starts_with(&self.base_path),
            "The config path ({path:?}) has to be inside the floxy base directory {:?}",
            self.base_path
        );
        let meta = entry.metadata()?;
        if (meta.is_symlink() || meta.is_file())
            && path.extension() == Some(CONFIG_EXTENSION.as_ref())
        {
            std::fs::remove_file(&path)?;
            debug!("Removed config entry {path:?} {self}");
        }
        Ok(())
    }

    fn create_instance_config(instance_ip: IpAddr, dest_port: u16, location: &str) -> String {
        format!(
            "
location {location} {{
   server_name_in_redirect on;
   return 307 $request_uri/;

   location ~ ^{location}/(.*) {{
      set $upstream http://{instance_ip}:{dest_port}/$1$is_args$args;
      proxy_pass $upstream;

      proxy_http_version 1.1;

      proxy_set_header Upgrade $http_upgrade;
      proxy_set_header Connection $connection_upgrade;
      proxy_set_header Host $host;
      proxy_set_header X-Forwarded-Proto $scheme;
      proxy_set_header X-Real-IP $remote_addr;
      proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
      proxy_set_header X-Forwarded-Host $host;
      proxy_set_header X-Forwarded-Port $server_port;

      client_max_body_size 0;
      client_body_timeout 30m;
   }}
}}"
        )
    }

    fn create_location_config(location: &str, additional_location: &str) -> String {
        format!(
            "
location {additional_location} {{
   server_name_in_redirect on;
   return 307 {location};                
}}
location ~ ^{additional_location}/(.*) {{
   server_name_in_redirect on;
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
      set $upstream http://{instance_ip}:{dest_port};
      proxy_pass $upstream;

      proxy_http_version 1.1;

      proxy_set_header Upgrade $http_upgrade;
      proxy_set_header Connection $connection_upgrade;
      proxy_set_header Host $host;
      proxy_set_header X-Forwarded-Proto $scheme;
      proxy_set_header X-Real-IP $remote_addr;
      proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
      proxy_set_header X-Forwarded-Host $host;
      proxy_set_header X-Forwarded-Port $server_port;

      client_max_body_size 0;
      client_body_timeout 30m;
   }}
}}"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relic::nginx::tests::NGINX_CONFIG_EXAMPLE;
    use crate::relic::process::tests::sleepy_child;
    use crate::tests::prepare_test_path;
    use ntest::timeout;
    use std::net::Ipv4Addr;
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    #[test]
    fn display_test() {
        let base_path = PathBuf::from("/some/test/dir");
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: base_path.clone(),
        };
        assert_eq!(
            floxy.to_string(),
            "(Floxy: (base_path = /some/test/dir, nginx = (pid /var/run/nginx.pid, config Default))"
        );
    }

    #[test]
    #[timeout(10000)]
    fn reload_ok() {
        let mut child = sleepy_child();
        let test_dir = prepare_test_path(module_path!(), "reload_ok");
        let test_pid_path = test_dir.join("sleepy.pid");
        std::fs::write(&test_pid_path, format!("{}\n", child.id())).unwrap();
        let test_conf_path = test_dir.join("nginx.conf");
        std::fs::write(
            &test_conf_path,
            format!("pid {};", test_pid_path.to_string_lossy()),
        )
        .unwrap();
        let floxy = FloxyImpl::from_config(test_dir, test_conf_path).unwrap();
        match floxy.reload_config() {
            Ok(_) => {
                let output = child.wait_with_output().unwrap();
                assert_eq!(output.status, ExitStatus::from_raw(1))
            }
            Err(e) => {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("reload_config failed: {e}")
            }
        }
    }

    #[test]
    #[timeout(10000)]
    fn stop_ok() {
        let mut child = sleepy_child();
        let test_dir = prepare_test_path(module_path!(), "stop_ok");
        let test_pid_path = test_dir.join("sleepy.pid");
        std::fs::write(&test_pid_path, format!("{}\n", child.id())).unwrap();
        let test_conf_path = test_dir.join("nginx.conf");
        std::fs::write(
            &test_conf_path,
            format!("pid {};", test_pid_path.to_string_lossy()),
        )
        .unwrap();
        let floxy = FloxyImpl::from_config(test_dir, test_conf_path).unwrap();
        match floxy.stop() {
            Ok(_) => {
                let output = child.wait_with_output().unwrap();
                assert_eq!(output.status, ExitStatus::from_raw(131))
            }
            Err(e) => {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("graceful_shutdown failed: {e}")
            }
        }
    }

    #[test]
    fn from_config_ok() {
        let test_dir = prepare_test_path(module_path!(), "from_config_ok");
        let test_config_path = test_dir.join("test.conf");
        std::fs::write(&test_config_path, NGINX_CONFIG_EXAMPLE).unwrap();
        let floxy = FloxyImpl::from_config(test_dir.clone(), test_config_path.clone()).unwrap();
        assert_eq!(floxy.base_path, test_dir);
        assert_eq!(floxy.nginx.config_path(), Some(test_config_path.as_path()));
        assert_eq!(floxy.nginx.pid_path(), Path::new("/abc/def/nginx.pid"));
    }

    #[test]
    fn get_server_path() {
        let base_path = PathBuf::from("/some/test/dir");
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: base_path.clone(),
        };
        assert_eq!(
            floxy.servers_path(),
            base_path.join(SERVER_CONFIGS_DIR_NAME)
        );
    }

    #[test]
    fn get_instances_path() {
        let base_path = PathBuf::from("/some/test/dir");
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: base_path.clone(),
        };
        assert_eq!(
            floxy.instances_path(),
            base_path.join(INSTANCE_CONFIGS_DIR_NAME)
        );
    }

    #[test]
    fn from_config_err() {
        let test_dir = prepare_test_path(module_path!(), "from_config_err");
        let test_config_path = test_dir.join("test.conf");
        assert!(FloxyImpl::from_config(test_dir.clone(), test_config_path.clone()).is_err());
    }

    const EXPECTED_TRIPLE_CONFIG: &str = "
location /v2/instances/1234abcd/editor/5000 {
   server_name_in_redirect on;
   return 307 $request_uri/;

   location ~ ^/v2/instances/1234abcd/editor/5000/(.*) {
      set $upstream http://123.123.234.234:5000/$1$is_args$args;
      proxy_pass $upstream;

      proxy_http_version 1.1;

      proxy_set_header Upgrade $http_upgrade;
      proxy_set_header Connection $connection_upgrade;
      proxy_set_header Host $host;
      proxy_set_header X-Forwarded-Proto $scheme;
      proxy_set_header X-Real-IP $remote_addr;
      proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
      proxy_set_header X-Forwarded-Host $host;
      proxy_set_header X-Forwarded-Port $server_port;

      client_max_body_size 0;
      client_body_timeout 30m;
   }
}
location /v2/instances/1234abcd/editor/6000 {
   server_name_in_redirect on;
   return 307 $request_uri/;

   location ~ ^/v2/instances/1234abcd/editor/6000/(.*) {
      set $upstream http://123.123.234.234:6000/$1$is_args$args;
      proxy_pass $upstream;

      proxy_http_version 1.1;

      proxy_set_header Upgrade $http_upgrade;
      proxy_set_header Connection $connection_upgrade;
      proxy_set_header Host $host;
      proxy_set_header X-Forwarded-Proto $scheme;
      proxy_set_header X-Real-IP $remote_addr;
      proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
      proxy_set_header X-Forwarded-Host $host;
      proxy_set_header X-Forwarded-Port $server_port;

      client_max_body_size 0;
      client_body_timeout 30m;
   }
}
location /v2/instances/1234abcd/editor/7000 {
   server_name_in_redirect on;
   return 307 $request_uri/;

   location ~ ^/v2/instances/1234abcd/editor/7000/(.*) {
      set $upstream http://123.123.234.234:7000/$1$is_args$args;
      proxy_pass $upstream;

      proxy_http_version 1.1;

      proxy_set_header Upgrade $http_upgrade;
      proxy_set_header Connection $connection_upgrade;
      proxy_set_header Host $host;
      proxy_set_header X-Forwarded-Proto $scheme;
      proxy_set_header X-Real-IP $remote_addr;
      proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
      proxy_set_header X-Forwarded-Host $host;
      proxy_set_header X-Forwarded-Port $server_port;

      client_max_body_size 0;
      client_body_timeout 30m;
   }
}";

    const TRIPLE_DEST_PORTS: [u16; 3] = [5000, 6000, 7000];

    #[test]
    fn create_instance_reverse_proxy_config_test() {
        let config = FloxyImpl::create_instance_reverse_proxy_config(
            InstanceId::new(0x1234abcd),
            IpAddr::V4(Ipv4Addr::new(123, 123, 234, 234)),
            TRIPLE_DEST_PORTS.iter(),
        );
        assert_eq!(config, EXPECTED_TRIPLE_CONFIG);
    }

    #[test]
    fn add_instance_reverse_proxy_config_new() {
        let test_dir = prepare_test_path(module_path!(), "add_instance_reverse_proxy_config_new");
        let config_path = test_dir
            .join(INSTANCE_CONFIGS_DIR_NAME)
            .join("test_app-1234abcd.conf");
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        assert!(matches!(
            floxy.add_instance_reverse_proxy_config(
                "test_app",
                InstanceId::new(0x1234abcd),
                IpAddr::V4(Ipv4Addr::new(123, 123, 234, 234)),
                &TRIPLE_DEST_PORTS,
            ),
            Ok(true)
        ));
        assert_eq!(
            std::fs::read_to_string(config_path).unwrap(),
            EXPECTED_TRIPLE_CONFIG
        );
    }

    #[test]
    fn add_instance_reverse_proxy_config_unchanged() {
        let test_dir = prepare_test_path(
            module_path!(),
            "add_instance_reverse_proxy_config_unchanged",
        );
        let config_path = test_dir
            .join(INSTANCE_CONFIGS_DIR_NAME)
            .join("test_app-1234abcd.conf");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, EXPECTED_TRIPLE_CONFIG).unwrap();
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        assert!(matches!(
            floxy.add_instance_reverse_proxy_config(
                "test_app",
                InstanceId::new(0x1234abcd),
                IpAddr::V4(Ipv4Addr::new(123, 123, 234, 234)),
                &TRIPLE_DEST_PORTS,
            ),
            Ok(false)
        ));
        assert_eq!(
            std::fs::read_to_string(config_path).unwrap(),
            EXPECTED_TRIPLE_CONFIG
        );
    }

    #[test]
    fn add_instance_reverse_proxy_config_changed() {
        let test_dir =
            prepare_test_path(module_path!(), "add_instance_reverse_proxy_config_changed");
        let config_path = test_dir
            .join(INSTANCE_CONFIGS_DIR_NAME)
            .join("test_app-1234abcd.conf");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, "test config").unwrap();
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        assert!(matches!(
            floxy.add_instance_reverse_proxy_config(
                "test_app",
                InstanceId::new(0x1234abcd),
                IpAddr::V4(Ipv4Addr::new(123, 123, 234, 234)),
                &TRIPLE_DEST_PORTS,
            ),
            Ok(true)
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
      set $upstream http://123.123.234.234:9090;
      proxy_pass $upstream;

      proxy_http_version 1.1;

      proxy_set_header Upgrade $http_upgrade;
      proxy_set_header Connection $connection_upgrade;
      proxy_set_header Host $host;
      proxy_set_header X-Forwarded-Proto $scheme;
      proxy_set_header X-Real-IP $remote_addr;
      proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
      proxy_set_header X-Forwarded-Host $host;
      proxy_set_header X-Forwarded-Port $server_port;

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
location TEST_LOCATION {
   server_name_in_redirect on;
   return 307 $request_uri/;

   location ~ ^TEST_LOCATION/(.*) {
      set $upstream http://30.60.120.240:7799/$1$is_args$args;
      proxy_pass $upstream;

      proxy_http_version 1.1;

      proxy_set_header Upgrade $http_upgrade;
      proxy_set_header Connection $connection_upgrade;
      proxy_set_header Host $host;
      proxy_set_header X-Forwarded-Proto $scheme;
      proxy_set_header X-Real-IP $remote_addr;
      proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
      proxy_set_header X-Forwarded-Host $host;
      proxy_set_header X-Forwarded-Port $server_port;

      client_max_body_size 0;
      client_body_timeout 30m;
   }
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
        let test_dir = prepare_test_path(module_path!(), "delete_reverse_proxy_config_ok");
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        let config_path = test_dir
            .join(INSTANCE_CONFIGS_DIR_NAME)
            .join("test_app-abcd1234.conf");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, "test config").unwrap();
        floxy
            .delete_reverse_proxy_config("test_app", InstanceId::new(0xabcd1234))
            .unwrap();
        assert!(matches!(config_path.try_exists(), Ok(false)));
    }

    #[test]
    fn delete_reverse_proxy_config_not_existing() {
        let test_dir =
            prepare_test_path(module_path!(), "delete_reverse_proxy_config_not_existing");
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        floxy
            .delete_reverse_proxy_config("test_app", InstanceId::new(0xabcd1234))
            .unwrap();
    }

    #[test]
    fn delete_reverse_proxy_config_err() {
        let test_dir = prepare_test_path(module_path!(), "delete_reverse_proxy_config_err");
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        let config_path = test_dir
            .join(INSTANCE_CONFIGS_DIR_NAME)
            .join("test_app-abcd1234.conf");
        std::fs::create_dir_all(config_path).unwrap();
        assert!(
            floxy
                .delete_reverse_proxy_config("test_app", InstanceId::new(0xabcd1234))
                .is_err()
        );
    }

    #[test]
    fn delete_server_config_ok() {
        let test_dir = prepare_test_path(module_path!(), "delete_server_config_ok");
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        let config_path = test_dir
            .join(SERVER_CONFIGS_DIR_NAME)
            .join("test_app-abcd1234_1234.conf");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, "test config").unwrap();
        floxy
            .delete_server_config("test_app", InstanceId::new(0xabcd1234), 1234)
            .unwrap();
        assert!(matches!(config_path.try_exists(), Ok(false)));
    }

    #[test]
    fn delete_server_config_err() {
        let test_dir = prepare_test_path(module_path!(), "delete_server_config_err");
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        let config_path = test_dir
            .join(SERVER_CONFIGS_DIR_NAME)
            .join("test_app-abcd1234_1234.conf");
        std::fs::create_dir_all(config_path).unwrap();
        assert!(
            floxy
                .delete_server_config("test_app", InstanceId::new(0xabcd1234), 1234)
                .is_err()
        );
    }

    #[test]
    fn build_server_config_path_test() {
        let test_dir = prepare_test_path(module_path!(), "build_server_config_path_test");
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        assert_eq!(
            floxy.build_server_config_path("test_app", InstanceId::new(0xab12cd34), 1234),
            test_dir
                .join(SERVER_CONFIGS_DIR_NAME)
                .join("test_app-ab12cd34_1234.conf")
        )
    }

    #[test]
    fn build_instance_config_path_test() {
        let test_dir = prepare_test_path(module_path!(), "build_instance_config_path_test");
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        assert_eq!(
            floxy.build_instance_config_path("test_app", InstanceId::new(0xab12cd34)),
            test_dir
                .join(INSTANCE_CONFIGS_DIR_NAME)
                .join("test_app-ab12cd34.conf")
        )
    }

    #[test]
    fn delete_config_entry_outside_base_path_err() {
        let test_dir =
            prepare_test_path(module_path!(), "delete_config_entry_outside_base_path_err");
        let config_test_dir = test_dir.join("config");
        std::fs::create_dir_all(&config_test_dir).unwrap();
        std::fs::write(config_test_dir.join("test.conf"), "test config").unwrap();
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.join("base"),
        };
        let entry = std::fs::read_dir(config_test_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        assert!(floxy.delete_config_entry(&entry).is_err());
    }

    #[test]
    fn delete_config_entry_not_file() {
        let test_dir = prepare_test_path(module_path!(), "delete_config_entry_not_file");
        let config_test_dir = test_dir.join("config");
        let config_path = config_test_dir.join("test.conf");
        std::fs::create_dir_all(&config_path).unwrap();
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: config_test_dir.clone(),
        };
        let entry = std::fs::read_dir(config_test_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        floxy.delete_config_entry(&entry).unwrap();
        assert!(matches!(config_path.try_exists(), Ok(true)));
    }

    #[test]
    fn delete_config_entry_not_config() {
        let test_dir = prepare_test_path(module_path!(), "delete_config_entry_not_config");
        let config_test_dir = test_dir.join("config");
        std::fs::create_dir_all(&config_test_dir).unwrap();
        let config_path = config_test_dir.join("test.txt");
        std::fs::write(&config_path, "test config").unwrap();
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: config_test_dir.clone(),
        };
        let entry = std::fs::read_dir(config_test_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        floxy.delete_config_entry(&entry).unwrap();
        assert!(matches!(config_path.try_exists(), Ok(true)));
    }

    #[test]
    fn delete_config_entry_ok() {
        let test_dir = prepare_test_path(module_path!(), "delete_config_entry_ok");
        let config_test_dir = test_dir.join("config");
        std::fs::create_dir_all(&config_test_dir).unwrap();
        let config_path = config_test_dir.join("test.conf");
        std::fs::write(&config_path, "test config").unwrap();
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: config_test_dir.clone(),
        };
        let entry = std::fs::read_dir(config_test_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        floxy.delete_config_entry(&entry).unwrap();
        assert!(matches!(config_path.try_exists(), Ok(false)));
    }

    #[test]
    fn clear_server_configs_ok() {
        let test_dir = prepare_test_path(module_path!(), "clear_server_configs_ok");
        let server_dir = test_dir.join(SERVER_CONFIGS_DIR_NAME);
        std::fs::create_dir_all(&server_dir).unwrap();
        for i in 0..10 {
            std::fs::write(server_dir.join(format!("test{i}.conf")), "test config").unwrap();
        }
        std::fs::write(server_dir.join("test.file"), "test file").unwrap();
        std::fs::create_dir_all(server_dir.join("test.dir")).unwrap();
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        floxy.clear_server_configs().unwrap();
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
        let test_dir = prepare_test_path(module_path!(), "clear_server_proxy_configs_ok");
        let server_dir = test_dir.join(SERVER_CONFIGS_DIR_NAME);
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
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        let ports = [1234, 5678, 910];
        floxy
            .delete_server_proxy_configs("test_app", InstanceId::new(0xcdab3412), &ports)
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
        let test_dir = prepare_test_path(module_path!(), "clear_server_proxy_configs_err");
        let server_dir = test_dir.join(SERVER_CONFIGS_DIR_NAME);
        std::fs::create_dir_all(&server_dir).unwrap();
        std::fs::write(
            server_dir.join("test_app-cdab3412_1234.conf"),
            "test config",
        )
        .unwrap();
        std::fs::create_dir_all(server_dir.join("test_app-cdab3412_5678.conf")).unwrap();
        std::fs::write(server_dir.join("test_app-cdab3412_910.conf"), "test config").unwrap();
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        let ports = [1234, 5678, 910];
        assert!(
            floxy
                .delete_server_proxy_configs("test_app", InstanceId::new(0xcdab3412), &ports)
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
        let test_dir = prepare_test_path(module_path!(), "create_reverse_proxy_config_no_changes");
        let config_path = test_dir.join("test.conf");
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        std::fs::write(&config_path, "test content").unwrap();
        assert!(matches!(
            floxy.add_reverse_proxy_config("test content", &config_path),
            Ok(false)
        ));
        assert_eq!(
            "test content",
            std::fs::read_to_string(config_path).unwrap()
        );
    }

    #[test]
    fn create_reverse_proxy_config_with_changes() {
        let test_dir =
            prepare_test_path(module_path!(), "create_reverse_proxy_config_with_changes");
        let config_path = test_dir.join("test.conf");
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        std::fs::write(&config_path, "old test content").unwrap();
        assert!(matches!(
            floxy.add_reverse_proxy_config("test content", &config_path),
            Ok(true)
        ));
        assert_eq!(
            "test content",
            std::fs::read_to_string(config_path).unwrap()
        );
    }

    #[test]
    fn create_reverse_proxy_config_new_config() {
        let test_dir = prepare_test_path(module_path!(), "create_reverse_proxy_config_new_config");
        let config_path = test_dir.join("test.conf");
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        assert!(matches!(
            floxy.add_reverse_proxy_config("test content", &config_path),
            Ok(true)
        ));
        assert_eq!(
            "test content",
            std::fs::read_to_string(config_path).unwrap()
        );
    }

    #[test]
    fn create_reverse_proxy_config_outside_base_path() {
        let test_dir = prepare_test_path(
            module_path!(),
            "create_reverse_proxy_config_outside_base_path",
        );
        let base_dir = test_dir.join("base");
        let config_dir = test_dir.join("config");
        let config_path = config_dir.join("test.conf");
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: base_dir,
        };
        assert!(
            floxy
                .add_reverse_proxy_config("test content", &config_path)
                .is_err()
        );
        assert!(matches!(config_path.try_exists(), Ok(false)));
    }

    #[test]
    fn create_instance_editor_redirect_to_free_port_ok() {
        let test_dir = prepare_test_path(
            module_path!(),
            "create_instance_editor_redirect_to_free_port_ok",
        );
        let floxy = FloxyImpl {
            nginx: Nginx::default(),
            base_path: test_dir.clone(),
        };
        let (reload_necessary, port) = floxy
            .add_instance_editor_redirect_to_free_port(
                "test app",
                InstanceId::new(0x12345678),
                IpAddr::V4(Ipv4Addr::new(123, 123, 123, 123)),
                50000,
            )
            .unwrap();
        assert!(reload_necessary);
        let config_path = test_dir
            .join(SERVER_CONFIGS_DIR_NAME)
            .join(format!("test app-12345678_{port}.conf"));
        let expected_config_content = format!(
            "
server {{
   listen {port};
   location / {{
      set $upstream http://123.123.123.123:50000;
      proxy_pass $upstream;

      proxy_http_version 1.1;

      proxy_set_header Upgrade $http_upgrade;
      proxy_set_header Connection $connection_upgrade;
      proxy_set_header Host $host;
      proxy_set_header X-Forwarded-Proto $scheme;
      proxy_set_header X-Real-IP $remote_addr;
      proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
      proxy_set_header X-Forwarded-Host $host;
      proxy_set_header X-Forwarded-Port $server_port;

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
