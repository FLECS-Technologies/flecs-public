mod floxy_impl;
mod operation;
use crate::enchantment::Enchantment;
use crate::jeweler::gem::instance::InstanceId;
pub use floxy_impl::FloxyImpl;
#[cfg(test)]
use mockall::{automock, predicate::*};
pub use operation::FloxyOperation;
use std::net::IpAddr;

#[cfg_attr(test, automock)]
pub trait Floxy: Enchantment {
    fn start(&self) -> crate::Result<()>;

    fn stop(&self) -> crate::Result<()>;

    fn add_instance_reverse_proxy_config(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        instance_ip: IpAddr,
        dest_ports: &[u16],
    ) -> crate::Result<bool>;

    fn delete_reverse_proxy_config(
        &self,
        app_name: &str,
        instance_id: InstanceId,
    ) -> crate::Result<bool>;

    fn delete_server_config(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        host_port: u16,
    ) -> crate::Result<bool>;

    fn delete_server_proxy_configs(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        host_ports: &[u16],
    ) -> Result<bool, (bool, crate::Error)>;

    fn add_instance_editor_redirect_to_free_port(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        instance_ip: IpAddr,
        dest_port: u16,
    ) -> crate::Result<(bool, u16)>;

    fn reload_config(&self) -> crate::Result<()>;

    fn clear_server_configs(&self) -> crate::Result<()>;
}

#[cfg(test)]
impl std::fmt::Display for MockFloxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockFloxy")
    }
}

#[cfg(test)]
impl Enchantment for MockFloxy {}
