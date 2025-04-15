use crate::enchantment::floxy::Floxy;
use crate::jeweler::gem::instance::InstanceId;
use std::net::IpAddr;
use std::sync::{Arc, atomic};
use tracing::error;

pub struct FloxyOperation<F: Floxy> {
    floxy: Arc<F>,
    reload: atomic::AtomicBool,
}

impl<F: Floxy> FloxyOperation<F> {
    pub fn new(floxy: Arc<F>) -> Self {
        Self {
            floxy,
            reload: atomic::AtomicBool::new(false),
        }
    }

    pub fn new_arc(floxy: Arc<F>) -> Arc<Self> {
        Arc::new(Self::new(floxy))
    }

    pub fn mark_for_reload(&self) {
        _ = self.reload.compare_exchange(
            false,
            true,
            atomic::Ordering::Relaxed,
            atomic::Ordering::Relaxed,
        );
    }

    pub fn add_instance_reverse_proxy_config(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        instance_ip: IpAddr,
        dest_ports: &[u16],
    ) -> crate::Result<()> {
        if self.floxy.add_instance_reverse_proxy_config(
            app_name,
            instance_id,
            instance_ip,
            dest_ports,
        )? {
            self.mark_for_reload();
        }
        Ok(())
    }

    pub fn delete_reverse_proxy_config(
        &self,
        app_name: &str,
        instance_id: InstanceId,
    ) -> crate::Result<()> {
        if self
            .floxy
            .delete_reverse_proxy_config(app_name, instance_id)?
        {
            self.mark_for_reload();
        }
        Ok(())
    }

    pub fn delete_server_config(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        host_port: u16,
    ) -> crate::Result<()> {
        if self
            .floxy
            .delete_server_config(app_name, instance_id, host_port)?
        {
            self.mark_for_reload();
        }
        Ok(())
    }

    pub fn delete_server_proxy_configs(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        host_ports: &[u16],
    ) -> crate::Result<()> {
        match self
            .floxy
            .delete_server_proxy_configs(app_name, instance_id, host_ports)
        {
            Err((true, e)) => {
                self.mark_for_reload();
                Err(e)
            }
            Err((false, e)) => Err(e),
            Ok(true) => {
                self.mark_for_reload();
                Ok(())
            }
            Ok(false) => Ok(()),
        }
    }

    pub fn add_instance_editor_redirect_to_free_port(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        instance_ip: IpAddr,
        dest_port: u16,
    ) -> crate::Result<u16> {
        let (reload, port) = self.floxy.add_instance_editor_redirect_to_free_port(
            app_name,
            instance_id,
            instance_ip,
            dest_port,
        )?;
        if reload {
            self.mark_for_reload();
        }
        Ok(port)
    }
}

impl<F: Floxy> Drop for FloxyOperation<F> {
    fn drop(&mut self) {
        if self.reload.load(atomic::Ordering::Relaxed) {
            if let Err(e) = self.floxy.reload_config() {
                error!("Floxy reload failed: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enchantment::floxy::MockFloxy;
    use crate::jeweler::gem::instance::InstanceId;
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, atomic};

    #[test]
    fn floxy_operation_drop_reload() {
        let mut floxy = MockFloxy::new();
        floxy.expect_reload_config().times(1).returning(|| Ok(()));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(true),
        };
        drop(operation);
    }

    #[test]
    fn floxy_operation_drop_no_reload() {
        let mut floxy = MockFloxy::new();
        floxy.expect_reload_config().times(0);
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        drop(operation);
    }

    #[test]
    fn floxy_operation_mark_for_reload() {
        let mut floxy = MockFloxy::new();
        floxy.expect_reload_config().returning(|| Ok(()));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        operation.mark_for_reload();
        assert!(operation.reload.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn floxy_operation_add_instance_reverse_proxy_config_ok_reload() {
        let mut floxy = MockFloxy::new();
        floxy.expect_reload_config().returning(|| Ok(()));
        floxy
            .expect_add_instance_reverse_proxy_config()
            .times(1)
            .returning(|_, _, _, _| Ok(true));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        operation
            .add_instance_reverse_proxy_config(
                "",
                InstanceId::new(1),
                IpAddr::from_str("172.10.10.10").unwrap(),
                &[],
            )
            .unwrap();
        assert!(operation.reload.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn floxy_operation_add_instance_reverse_proxy_config_ok_no_reload() {
        let mut floxy = MockFloxy::new();
        floxy
            .expect_add_instance_reverse_proxy_config()
            .times(1)
            .returning(|_, _, _, _| Ok(false));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        operation
            .add_instance_reverse_proxy_config(
                "",
                InstanceId::new(1),
                IpAddr::from_str("172.10.10.10").unwrap(),
                &[],
            )
            .unwrap();
        assert!(!operation.reload.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn floxy_operation_add_instance_reverse_proxy_config_err() {
        let mut floxy = MockFloxy::new();
        floxy
            .expect_add_instance_reverse_proxy_config()
            .times(1)
            .returning(|_, _, _, _| Err(anyhow::anyhow!("test error")));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        assert!(
            operation
                .add_instance_reverse_proxy_config(
                    "",
                    InstanceId::new(1),
                    IpAddr::from_str("172.10.10.10").unwrap(),
                    &[],
                )
                .is_err()
        );
        assert!(!operation.reload.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn floxy_operation_delete_reverse_proxy_config_ok_reload() {
        let mut floxy = MockFloxy::new();
        floxy.expect_reload_config().returning(|| Ok(()));
        floxy
            .expect_delete_reverse_proxy_config()
            .times(1)
            .returning(|_, _| Ok(true));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        operation
            .delete_reverse_proxy_config("", InstanceId::new(1))
            .unwrap();
        assert!(operation.reload.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn floxy_operation_delete_reverse_proxy_config_ok_no_reload() {
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_reverse_proxy_config()
            .times(1)
            .returning(|_, _| Ok(false));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        operation
            .delete_reverse_proxy_config("", InstanceId::new(1))
            .unwrap();
        assert!(!operation.reload.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn floxy_operation_delete_reverse_proxy_config_err() {
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_reverse_proxy_config()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("test error")));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        assert!(
            operation
                .delete_reverse_proxy_config("", InstanceId::new(1),)
                .is_err()
        );
        assert!(!operation.reload.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn floxy_operation_delete_server_config_ok_reload() {
        let mut floxy = MockFloxy::new();
        floxy.expect_reload_config().returning(|| Ok(()));
        floxy
            .expect_delete_server_config()
            .times(1)
            .returning(|_, _, _| Ok(true));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        operation
            .delete_server_config("", InstanceId::new(1), 100)
            .unwrap();
        assert!(operation.reload.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn floxy_operation_delete_server_config_ok_no_reload() {
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_server_config()
            .times(1)
            .returning(|_, _, _| Ok(false));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        operation
            .delete_server_config("", InstanceId::new(1), 100)
            .unwrap();
        assert!(!operation.reload.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn floxy_operation_delete_server_config_err() {
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_server_config()
            .times(1)
            .returning(|_, _, _| Err(anyhow::anyhow!("test error")));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        assert!(
            operation
                .delete_server_config("", InstanceId::new(1), 100)
                .is_err()
        );
        assert!(!operation.reload.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn floxy_operation_delete_server_proxy_configs_ok_reload() {
        let mut floxy = MockFloxy::new();
        floxy.expect_reload_config().returning(|| Ok(()));
        floxy
            .expect_delete_server_proxy_configs()
            .times(1)
            .returning(|_, _, _| Ok(true));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        operation
            .delete_server_proxy_configs("", InstanceId::new(1), &[])
            .unwrap();
        assert!(operation.reload.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn floxy_operation_delete_server_proxy_configs_ok_no_reload() {
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_server_proxy_configs()
            .times(1)
            .returning(|_, _, _| Ok(false));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        operation
            .delete_server_proxy_configs("", InstanceId::new(1), &[])
            .unwrap();
        assert!(!operation.reload.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn floxy_operation_delete_server_proxy_configs_err_reload() {
        let mut floxy = MockFloxy::new();
        floxy.expect_reload_config().returning(|| Ok(()));
        floxy
            .expect_delete_server_proxy_configs()
            .times(1)
            .returning(|_, _, _| Err((true, anyhow::anyhow!("test error"))));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        assert!(
            operation
                .delete_server_proxy_configs("", InstanceId::new(1), &[])
                .is_err()
        );
        assert!(operation.reload.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn floxy_operation_delete_server_proxy_configs_err_no_reload() {
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_server_proxy_configs()
            .times(1)
            .returning(|_, _, _| Err((false, anyhow::anyhow!("test error"))));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        assert!(
            operation
                .delete_server_proxy_configs("", InstanceId::new(1), &[])
                .is_err()
        );
        assert!(!operation.reload.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn floxy_operation_add_instance_editor_redirect_to_free_port_ok_reload() {
        let mut floxy = MockFloxy::new();
        floxy.expect_reload_config().returning(|| Ok(()));
        floxy
            .expect_add_instance_editor_redirect_to_free_port()
            .times(1)
            .returning(|_, _, _, _| Ok((true, 20)));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        assert_eq!(
            operation
                .add_instance_editor_redirect_to_free_port(
                    "",
                    InstanceId::new(1),
                    IpAddr::from_str("172.10.10.10").unwrap(),
                    100,
                )
                .unwrap(),
            20
        );
        assert!(operation.reload.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn floxy_operation_add_instance_editor_redirect_to_free_port_ok_no_reload() {
        let mut floxy = MockFloxy::new();
        floxy
            .expect_add_instance_editor_redirect_to_free_port()
            .times(1)
            .returning(|_, _, _, _| Ok((false, 20)));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        assert_eq!(
            operation
                .add_instance_editor_redirect_to_free_port(
                    "",
                    InstanceId::new(1),
                    IpAddr::from_str("172.10.10.10").unwrap(),
                    100,
                )
                .unwrap(),
            20
        );
        assert!(!operation.reload.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn floxy_operation_add_instance_editor_redirect_to_free_port_err() {
        let mut floxy = MockFloxy::new();
        floxy
            .expect_add_instance_editor_redirect_to_free_port()
            .times(1)
            .returning(|_, _, _, _| Err(anyhow::anyhow!("test error")));
        let operation = FloxyOperation {
            floxy: Arc::new(floxy),
            reload: AtomicBool::new(false),
        };
        assert!(
            operation
                .add_instance_editor_redirect_to_free_port(
                    "",
                    InstanceId::new(1),
                    IpAddr::from_str("172.10.10.10").unwrap(),
                    100
                )
                .is_err()
        );
        assert!(!operation.reload.load(atomic::Ordering::Relaxed));
    }
}
