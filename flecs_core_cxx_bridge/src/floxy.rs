use crate::get_server;
use flecs_core::jeweler::gem::instance::InstanceId;
use flecs_core::*;
use std::net::Ipv4Addr;
use std::str::FromStr;
use tracing::error;

pub fn create_instance_editor_redirect_to_free_port(
    app_name: &str,
    instance_id: u32,
    instance_ip: &str,
    dest_port: u16,
) -> Result<u16> {
    let server = get_server();
    let server = server.lock().unwrap();
    let (reload, port) = server.floxy.add_instance_editor_redirect_to_free_port(
        app_name,
        InstanceId::new(instance_id),
        Ipv4Addr::from_str(instance_ip)?,
        dest_port,
    )?;
    if reload {
        server.floxy.reload_config()?;
    }
    Ok(port)
}

pub fn delete_reverse_proxy_configs(
    app_name: &str,
    instance_id: u32,
    ports: Vec<u16>,
) -> Result<()> {
    let server = get_server();
    let server = server.lock().unwrap();
    let instance_id = InstanceId::new(instance_id);
    if let Err(e) = server
        .floxy
        .delete_server_proxy_configs(app_name, instance_id, ports.iter())
    {
        error!("{}", e);
    }
    if let Err(e) = server
        .floxy
        .delete_reverse_proxy_config(app_name, instance_id)
    {
        error!("{}", e);
    }
    server.floxy.reload_config()
}

pub fn delete_server_proxy_configs(
    app_name: &str,
    instance_id: u32,
    ports: Vec<u16>,
) -> Result<()> {
    let server = get_server();
    let server = server.lock().unwrap();
    let instance_id = InstanceId::new(instance_id);
    if let Err(e) = server
        .floxy
        .delete_server_proxy_configs(app_name, instance_id, ports.iter())
    {
        error!("{}", e);
    }
    server.floxy.reload_config()
}

pub fn load_instance_reverse_proxy_config(
    app: &str,
    instance_id: u32,
    instance_ip: &str,
    ports: Vec<u16>,
) -> Result<()> {
    let server = get_server();
    let server = server.lock().unwrap();
    let instance_id = InstanceId::new(instance_id);
    if server.floxy.add_instance_reverse_proxy_config(
        app,
        instance_id,
        Ipv4Addr::from_str(instance_ip)?,
        ports.iter(),
    )? {
        server.floxy.reload_config()?;
    }
    Ok(())
}
