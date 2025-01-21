use crate::relic::process;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use tracing::{info, warn};

const DEFAULT_PID_PATH: &str = "/var/run/nginx.pid";

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum NginxStatus {
    Running,
    Stopped,
    Corrupted,
}

pub struct Nginx {
    pid_path: PathBuf,
    config_path: Option<PathBuf>,
}

impl Default for Nginx {
    fn default() -> Self {
        Nginx {
            pid_path: PathBuf::from(DEFAULT_PID_PATH),
            config_path: None,
        }
    }
}

impl Display for Nginx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(pid {}, config {})",
            self.pid_path.to_string_lossy(),
            self.config_path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_else(|| "Default".to_string())
        )
    }
}

impl Nginx {
    pub fn pid_path(&self) -> &Path {
        &self.pid_path
    }

    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    pub fn from_config(config_path: PathBuf) -> crate::Result<Self> {
        Ok(Self {
            pid_path: Self::pid_from_config_path(&config_path)?,
            config_path: Some(config_path),
        })
    }

    fn pid_from_config_path(config_path: &Path) -> crate::Result<PathBuf> {
        let file = std::fs::File::open(config_path)?;
        Self::read_pid_path(file)
    }

    fn read_pid_path(reader: impl Read) -> crate::Result<PathBuf> {
        let buf_reader = BufReader::new(reader);

        for line in buf_reader.lines() {
            let line = line?;
            if line.starts_with("pid") && line.ends_with(';') {
                let mut split = line.split_whitespace().skip(1);
                if let (Some(pid_path), None) = (split.next(), split.next()) {
                    return Ok(PathBuf::from(&pid_path[..pid_path.len() - 1]));
                }
            }
        }
        Ok(PathBuf::from(DEFAULT_PID_PATH))
    }

    pub fn pid(&self) -> crate::Result<Option<u32>> {
        match std::fs::read_to_string(&self.pid_path) {
            Ok(pid) => match u32::from_str(pid.trim_end()) {
                Ok(pid) => Ok(Some(pid)),
                Err(e) => anyhow::bail!("Failed to parse {pid} into an pid: {e}"),
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => anyhow::bail!(e),
        }
    }

    pub fn fast_shutdown(&self) -> crate::Result<()> {
        if let Some(pid) = self.pid()? {
            process::send_signal(pid as i32, process::signal::SIGINT)?;
        }
        Ok(())
    }

    pub fn graceful_shutdown(&self) -> crate::Result<()> {
        if let Some(pid) = self.pid()? {
            process::send_signal(pid as i32, process::signal::SIGQUIT)?;
        }
        Ok(())
    }

    pub fn kill(&self) -> crate::Result<()> {
        if let Some(pid) = self.pid()? {
            process::send_signal(pid as i32, process::signal::SIGKILL)?;
        }
        Ok(())
    }

    pub fn reload_config(&self) -> crate::Result<()> {
        match self.pid()? {
            Some(pid) => process::send_signal(pid as i32, process::signal::SIGHUP),
            None => Err(anyhow::anyhow!("Process not running")),
        }
    }

    pub fn status(&self) -> crate::Result<NginxStatus> {
        match self.pid()? {
            Some(pid) => {
                if process::is_running(pid as i32)? {
                    Ok(NginxStatus::Running)
                } else {
                    Ok(NginxStatus::Corrupted)
                }
            }
            None => Ok(NginxStatus::Stopped),
        }
    }

    pub fn start(&self) -> crate::Result<()> {
        match self.status()? {
            NginxStatus::Running => {
                self.reload_config()?;
                warn!("Reusing running nginx {:?}", self.pid_path);
                return Ok(());
            }
            NginxStatus::Corrupted => {
                warn!("Cleanup old nginx pid file {:?}", self.pid_path);
                std::fs::remove_file(&self.pid_path)?;
            }
            NginxStatus::Stopped => {}
        }
        let mut command = Command::new("nginx");
        if let Some(config) = &self.config_path {
            command.args(["-c", config.to_string_lossy().as_ref()]);
        };
        let output = command.spawn()?.wait_with_output()?;
        if !output.status.success() {
            anyhow::bail!(
                "Failed to start nginx: {}",
                String::from_utf8_lossy(&output.stderr)
            )
        }
        match self.pid() {
            Ok(Some(pid)) => {
                info!("Nginx {self} started with pid {pid}")
            }
            Ok(None) => {
                warn!("Nginx {self} start triggered, but process not operational yet")
            }
            Err(e) => anyhow::bail!("Could not read pid file of nginx {self}: {e}"),
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::relic::process::tests::sleepy_child;
    use crate::tests::prepare_test_path;
    use ntest::timeout;
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    pub const NGINX_CONFIG_EXAMPLE: &[u8; 341] = b"user  www www;

worker_processes  2;

pid /abc/def/nginx.pid;
events {
  worker_connections  1024;
}

http {
  access_log        /var/log/floxy/access.log;
  sendfile          on;
  keepalive_timeout 65;

  include /var/lib/flecs/floxy/servers/*.conf;

  map $http_upgrade $connection_upgrade {
    default upgrade;
    ''      close;
  }
}
";

    #[test]
    fn display_test() {
        assert_eq!(
            Nginx::default().to_string(),
            "(pid /var/run/nginx.pid, config Default)"
        );
    }

    #[test]
    fn pid_ok() {
        let test_dir = prepare_test_path(module_path!(), "pid_ok");
        let test_pid_path = test_dir.join("test.pid");
        std::fs::write(&test_pid_path, "1234").unwrap();
        let nginx = Nginx {
            pid_path: test_pid_path,
            config_path: None,
        };
        assert_eq!(nginx.pid().unwrap(), Some(1234));
    }

    #[test]
    fn pid_none() {
        let test_dir = prepare_test_path(module_path!(), "pid_none");
        let test_pid_path = test_dir.join("test.pid");
        let nginx = Nginx {
            pid_path: test_pid_path,
            config_path: None,
        };
        assert_eq!(nginx.pid().unwrap(), None);
    }

    #[test]
    fn pid_io_error() {
        let test_dir = prepare_test_path(module_path!(), "pid_io_error");
        let test_pid_path = test_dir.join("test.pid");
        // Create directory to provoke error if read as a file
        std::fs::create_dir_all(&test_pid_path).unwrap();
        let nginx = Nginx {
            pid_path: test_pid_path,
            config_path: None,
        };
        assert!(nginx.pid().is_err());
    }

    #[test]
    fn pid_parse_error() {
        let test_dir = prepare_test_path(module_path!(), "pid_parse_error");
        let test_pid_path = test_dir.join("test.pid");
        std::fs::write(&test_pid_path, "xyz").unwrap();
        let nginx = Nginx {
            pid_path: test_pid_path,
            config_path: None,
        };
        assert!(nginx.pid().is_err());
    }

    #[test]
    fn read_pid_path_ok() {
        assert_eq!(
            Nginx::read_pid_path(&NGINX_CONFIG_EXAMPLE[..]).unwrap(),
            PathBuf::from("/abc/def/nginx.pid")
        );
    }

    #[test]
    fn read_pid_path_none() {
        const NGINX_CONFIG: &[u8; 317] = b"user  www www;

worker_processes  2;

events {
  worker_connections  1024;
}

http {
  access_log        /var/log/floxy/access.log;
  sendfile          on;
  keepalive_timeout 65;

  include /var/lib/flecs/floxy/servers/*.conf;

  map $http_upgrade $connection_upgrade {
    default upgrade;
    ''      close;
  }
}
";
        assert_eq!(
            Nginx::read_pid_path(&NGINX_CONFIG[..]).unwrap(),
            PathBuf::from(DEFAULT_PID_PATH)
        );
    }

    #[test]
    fn read_pid_path_err() {
        const NGINX_CONFIG: &[u8; 4] = &[0xff, 0x80, 0xff, 0x80];
        assert!(Nginx::read_pid_path(&NGINX_CONFIG[..]).is_err());
    }

    #[test]
    fn pid_from_config_path_ok() {
        let test_dir = prepare_test_path(module_path!(), "pid_from_config_path_ok");
        let test_config_path = test_dir.join("test.conf");
        std::fs::write(&test_config_path, NGINX_CONFIG_EXAMPLE).unwrap();
        assert_eq!(
            Nginx::pid_from_config_path(&test_config_path).unwrap(),
            PathBuf::from("/abc/def/nginx.pid")
        );
    }

    #[test]
    fn pid_from_config_path_err() {
        let test_dir = prepare_test_path(module_path!(), "pid_from_config_path_err");
        let test_config_path = test_dir.join("test.conf");
        assert!(Nginx::pid_from_config_path(&test_config_path).is_err());
    }

    #[test]
    fn from_config_ok() {
        let test_dir = prepare_test_path(module_path!(), "from_config_ok");
        let test_config_path = test_dir.join("test.conf");
        std::fs::write(&test_config_path, NGINX_CONFIG_EXAMPLE).unwrap();
        let nginx = Nginx::from_config(test_config_path.clone()).unwrap();
        assert_eq!(nginx.pid_path, PathBuf::from("/abc/def/nginx.pid"));
        assert_eq!(nginx.config_path, Some(test_config_path));
    }

    #[test]
    fn from_config_err() {
        let test_dir = prepare_test_path(module_path!(), "from_config_err");
        let test_config_path = test_dir.join("test.conf");
        assert!(Nginx::from_config(test_config_path).is_err());
    }

    #[test]
    fn default_test() {
        let nginx = Nginx::default();
        assert_eq!(nginx.pid_path, PathBuf::from(DEFAULT_PID_PATH));
        assert!(nginx.config_path.is_none());
    }

    #[test]
    fn get_pid_path() {
        let pid_path = PathBuf::from("/some/test/path.pid");
        let nginx = Nginx {
            pid_path: pid_path.clone(),
            config_path: None,
        };
        assert_eq!(nginx.pid_path(), pid_path)
    }

    #[test]
    fn get_config_path() {
        let config_path = Some(PathBuf::from("/some/test/config.conf"));
        let nginx = Nginx {
            pid_path: PathBuf::from("/some/test/path.pid"),
            config_path: config_path.clone(),
        };
        assert_eq!(nginx.config_path(), config_path.as_deref())
    }

    #[test]
    #[timeout(10000)]
    fn fast_shutdown_ok() {
        let mut child = sleepy_child();
        let test_dir = prepare_test_path(module_path!(), "fast_shutdown_ok");
        let test_pid_path = test_dir.join("sleepy.pid");
        std::fs::write(&test_pid_path, format!("{}\n", child.id())).unwrap();
        let nginx = Nginx {
            pid_path: test_pid_path,
            config_path: None,
        };
        match nginx.fast_shutdown() {
            Ok(_) => {
                let output = child.wait_with_output().unwrap();
                assert_eq!(output.status, ExitStatus::from_raw(2))
            }
            Err(e) => {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("fast_shutdown failed: {e}")
            }
        }
    }

    #[test]
    #[timeout(10000)]
    fn fast_shutdown_no_pid() {
        let test_dir = prepare_test_path(module_path!(), "fast_shutdown_no_pid");
        let test_pid_path = test_dir.join("missing.pid");
        let nginx = Nginx {
            pid_path: test_pid_path,
            config_path: None,
        };
        nginx.fast_shutdown().unwrap();
    }

    #[test]
    #[timeout(10000)]
    fn kill_ok() {
        let mut child = sleepy_child();
        let test_dir = prepare_test_path(module_path!(), "kill_ok");
        let test_pid_path = test_dir.join("sleepy.pid");
        std::fs::write(&test_pid_path, format!("{}\n", child.id())).unwrap();
        let nginx = Nginx {
            pid_path: test_pid_path,
            config_path: None,
        };
        match nginx.kill() {
            Ok(_) => {
                let output = child.wait_with_output().unwrap();
                assert_eq!(output.status, ExitStatus::from_raw(9))
            }
            Err(e) => {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("kill failed: {e}")
            }
        }
    }

    #[test]
    #[timeout(10000)]
    fn kill_no_pid() {
        let test_dir = prepare_test_path(module_path!(), "kill_no_pid");
        let test_pid_path = test_dir.join("missing.pid");
        let nginx = Nginx {
            pid_path: test_pid_path,
            config_path: None,
        };
        nginx.kill().unwrap();
    }

    #[test]
    #[timeout(10000)]
    fn graceful_shutdown_ok() {
        let mut child = sleepy_child();
        let test_dir = prepare_test_path(module_path!(), "graceful_shutdown_ok");
        let test_pid_path = test_dir.join("sleepy.pid");
        std::fs::write(&test_pid_path, format!("{}\n", child.id())).unwrap();
        let nginx = Nginx {
            pid_path: test_pid_path,
            config_path: None,
        };
        match nginx.graceful_shutdown() {
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
    #[timeout(10000)]
    fn graceful_shutdown_no_pid() {
        let test_dir = prepare_test_path(module_path!(), "graceful_shutdown_no_pid");
        let test_pid_path = test_dir.join("missing.pid");
        let nginx = Nginx {
            pid_path: test_pid_path,
            config_path: None,
        };
        nginx.graceful_shutdown().unwrap();
    }

    #[test]
    #[timeout(10000)]
    fn reload_ok() {
        let mut child = sleepy_child();
        let test_dir = prepare_test_path(module_path!(), "reload_ok");
        let test_pid_path = test_dir.join("sleepy.pid");
        std::fs::write(&test_pid_path, format!("{}\n", child.id())).unwrap();
        let nginx = Nginx {
            pid_path: test_pid_path,
            config_path: None,
        };
        match nginx.reload_config() {
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
    fn reload_no_pid() {
        let test_dir = prepare_test_path(module_path!(), "reload_no_pid");
        let test_pid_path = test_dir.join("missing.pid");
        let nginx = Nginx {
            pid_path: test_pid_path,
            config_path: None,
        };
        assert!(nginx.reload_config().is_err());
    }

    #[test]
    #[timeout(10000)]
    fn reload_pid_not_running() {
        let test_dir = prepare_test_path(module_path!(), "reload_pid_not_running");
        let test_pid_path = test_dir.join("missing.pid");
        let mut child = sleepy_child();
        let child_id = child.id() as i32;
        child.kill().unwrap();
        child.wait().unwrap();
        std::fs::write(&test_pid_path, format!("{}\n", child_id)).unwrap();
        let nginx = Nginx {
            pid_path: test_pid_path,
            config_path: None,
        };
        assert!(nginx.reload_config().is_err());
    }

    #[test]
    #[timeout(10000)]
    fn status_running() {
        let test_dir = prepare_test_path(module_path!(), "status_running");
        let test_pid_path = test_dir.join("sleepy.pid");
        let mut child = sleepy_child();
        let child_id = child.id() as i32;
        std::fs::write(&test_pid_path, format!("{}\n", child_id)).unwrap();
        let nginx = Nginx {
            pid_path: test_pid_path,
            config_path: None,
        };
        assert_eq!(nginx.status().unwrap(), NginxStatus::Running);
        child.kill().unwrap();
        child.wait().unwrap();
    }

    #[test]
    #[timeout(10000)]
    fn status_corrupted() {
        let test_dir = prepare_test_path(module_path!(), "status_corrupted");
        let test_pid_path = test_dir.join("sleepy.pid");
        let mut child = sleepy_child();
        let child_id = child.id() as i32;
        std::fs::write(&test_pid_path, format!("{}\n", child_id)).unwrap();
        let nginx = Nginx {
            pid_path: test_pid_path,
            config_path: None,
        };
        child.kill().unwrap();
        child.wait().unwrap();
        assert_eq!(nginx.status().unwrap(), NginxStatus::Corrupted);
    }

    #[test]
    #[timeout(10000)]
    fn status_stopped() {
        let test_dir = prepare_test_path(module_path!(), "status_stopped");
        let test_pid_path = test_dir.join("sleepy.pid");
        let nginx = Nginx {
            pid_path: test_pid_path,
            config_path: None,
        };
        assert_eq!(nginx.status().unwrap(), NginxStatus::Stopped);
    }
}
