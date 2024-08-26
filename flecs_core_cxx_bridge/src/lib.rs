mod manifest;
mod network;
mod token;
mod usb;

pub use crate::manifest::download_manifest;
pub use network::read_network_adapters;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
pub use token::acquire_download_token;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tracing::info;
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

    extern "Rust" {
        fn download_manifest(app: &str, version: &str) -> Result<String>;
        fn read_usb_devices() -> Result<Vec<UsbDevice>>;
        fn read_network_adapters() -> Result<Vec<NetAdapter>>;
        fn start_server();
        fn stop_server();
        fn acquire_download_token(app: &str, version: &str) -> Result<Token>;
    }
}

struct Server {
    runtime: Runtime,
    handle: Option<JoinHandle<()>>,
}

fn get_server() -> Arc<Mutex<Server>> {
    static SERVER: OnceLock<Arc<Mutex<Server>>> = OnceLock::new();
    SERVER
        .get_or_init(|| {
            Arc::new(Mutex::new(Server {
                runtime: Runtime::new().unwrap(),
                handle: None,
            }))
        })
        .clone()
}

pub fn start_server() {
    let server = get_server();
    let mut server = server.lock().unwrap();
    assert!(server.handle.is_none());
    flecs_core::fsm::init_tracing();
    info!("Spawning rust server");
    server.handle = Some(server.runtime.spawn(async {
        info!("Starting rust server");
        flecs_core::fsm::server(PathBuf::from("/run/flecs/flecsd-rs.sock"))
            .await
            .unwrap();
    }));
}

pub fn stop_server() {
    let server = get_server();
    let mut server = server.lock().unwrap();
    if server.handle.is_some() {
        info!("Dropping rust server");
        _ = server.handle.take();
    }
}
