mod floxy;
mod manifest;
mod network;
mod system;
mod token;
mod usb;

pub use crate::manifest::download_manifest;
use flecs_core::enchantment::floxy::{Floxy, FloxyImpl};
use flecs_core::enchantment::Enchantments;
pub use floxy::{
    create_instance_editor_redirect_to_free_port, delete_reverse_proxy_configs,
    delete_server_proxy_configs, load_instance_reverse_proxy_config,
};
pub use network::read_network_adapters;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
pub use system::read_system_info;
pub use token::acquire_download_token;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tracing::{error, info};
pub use usb::read_usb_devices;
#[cxx::bridge]
mod ffi {
    #[derive(Debug)]
    pub enum NetType {
        Unknown,
        Wired,
        Wireless,
        Local,
        Bridge,
        Virtual,
    }

    #[derive(Debug)]
    pub struct IpAddr {
        addr: String,
        subnet_mask: String,
    }

    #[derive(Debug)]
    pub struct NetInfo {
        mac: String,
        net_type: NetType,
        ipv4addresses: Vec<IpAddr>,
        ipv6addresses: Vec<IpAddr>,
        gateway: String,
    }

    pub struct NetAdapter {
        name: String,
        info: NetInfo,
    }

    pub struct UsbDevice {
        vid: u16,
        pid: u16,
        port: String,
        device: String,
        vendor: String,
    }

    #[derive(Debug, Default)]
    pub struct Token {
        username: String,
        password: String,
    }

    #[derive(Debug, Default)]
    pub struct Kernel {
        version: String,
        build: String,
        machine: String,
    }

    #[derive(Debug, Default)]
    pub struct Distro {
        id: String,
        codename: String,
        name: String,
        version: String,
    }

    #[derive(Debug, Default)]
    pub struct SystemInfo {
        arch: String,
        platform: String,
        kernel: Kernel,
        distro: Distro,
    }

    extern "Rust" {
        fn download_manifest(app: &str, version: &str) -> Result<String>;
        fn read_usb_devices() -> Result<Vec<UsbDevice>>;
        fn read_network_adapters() -> Result<Vec<NetAdapter>>;
        fn read_system_info() -> SystemInfo;
        fn start_server();
        fn stop_server();
        fn acquire_download_token(app: &str, version: &str) -> Result<Token>;
        fn create_instance_editor_redirect_to_free_port(
            app_name: &str,
            instance_id: u32,
            instance_ip: &str,
            dest_port: u16,
        ) -> Result<u16>;
        fn delete_reverse_proxy_configs(app: &str, instance_id: u32, ports: Vec<u16>)
            -> Result<()>;
        fn delete_server_proxy_configs(app: &str, instance_id: u32, ports: Vec<u16>);
        fn load_instance_reverse_proxy_config(
            app: &str,
            instance_id: u32,
            instance_ip: &str,
            ports: Vec<u16>,
        ) -> Result<()>;
    }
}

struct Server {
    runtime: Runtime,
    handle: Option<JoinHandle<()>>,
    floxy: Arc<FloxyImpl>,
}

fn get_server() -> Arc<Mutex<Server>> {
    static SERVER: OnceLock<Arc<Mutex<Server>>> = OnceLock::new();
    SERVER
        .get_or_init(|| {
            Arc::new(Mutex::new(Server {
                runtime: Runtime::new().unwrap(),
                handle: None,
                floxy: Arc::new(
                    FloxyImpl::from_config(
                        PathBuf::from("/var/lib/flecs/floxy"),
                        PathBuf::from("/etc/nginx/floxy.conf"),
                    )
                    .unwrap(),
                ),
            }))
        })
        .clone()
}

pub fn start_server() {
    let server = get_server();
    let mut server = server.lock().unwrap();
    assert!(server.handle.is_none());
    flecs_core::fsm::init_backtracing();
    flecs_core::fsm::init_tracing();
    match server.floxy.start() {
        Ok(()) => info!("Floxy started {}", server.floxy),
        Err(e) => error!("Failed to start floxy {}: {}", server.floxy, e),
    }
    info!("Spawning rust server");
    let floxy = server.floxy.clone();
    server.handle = Some(server.runtime.spawn(async {
        info!("Starting rust server");
        flecs_core::fsm::server(
            PathBuf::from("/run/flecs/flecsd-rs.sock"),
            Enchantments { floxy },
        )
        .await
        .unwrap();
    }));
}

pub fn stop_server() {
    let server = get_server();
    let mut server = server.lock().unwrap();
    match server.floxy.stop() {
        Err(e) => error!("Failed to stop floxy {}: {}", server.floxy, e),
        Ok(()) => info!("Signalled floxy {} to shut down", server.floxy),
    }
    if server.handle.is_some() {
        info!("Dropping rust server");
        _ = server.handle.take();
    }
}
