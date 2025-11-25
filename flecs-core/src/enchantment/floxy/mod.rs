mod floxy_impl;

use crate::jeweler::gem::instance::InstanceId;
pub use floxy_impl::FloxyImpl;
#[cfg(test)]
use mockall::{automock, predicate::*};
use std::fmt::Display;
use std::net::IpAddr;

pub struct AdditionalLocationInfo {
    pub port: u16,
    pub location: String,
}

#[cfg_attr(test, automock)]
pub trait Floxy: Send + Sync + Display {
    fn add_instance_reverse_proxy_config(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        instance_ip: IpAddr,
        dest_ports: &[u16],
    ) -> crate::Result<()>;

    fn add_additional_locations_proxy_config(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        additional_locations: &[AdditionalLocationInfo],
    ) -> crate::Result<()>;

    fn delete_additional_locations_proxy_config(
        &self,
        app_name: &str,
        instance_id: InstanceId,
    ) -> crate::Result<()>;

    fn delete_reverse_proxy_config(
        &self,
        app_name: &str,
        instance_id: InstanceId,
    ) -> crate::Result<()>;

    fn delete_server_config(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        host_port: u16,
    ) -> crate::Result<()>;

    fn delete_server_proxy_configs(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        host_ports: &[u16],
    ) -> crate::Result<()>;

    fn add_instance_editor_redirect_to_free_port(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        instance_ip: IpAddr,
        dest_port: u16,
    ) -> crate::Result<u16>;

    fn add_instance_redirect(
        &self,
        app_name: &str,
        instance_id: InstanceId,
        instance_ip: IpAddr,
        src_port: u16,
        dest_port: u16,
    ) -> crate::Result<()>;

    fn clear_server_configs(&self) -> crate::Result<()>;
    fn clear_instance_configs(&self) -> crate::Result<()>;
}

#[cfg(test)]
impl std::fmt::Display for MockFloxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockFloxy")
    }
}
