mod manifest;
mod network;
mod usb;

pub use crate::manifest::download_manifest;
pub use network::read_network_adapters;
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

    extern "Rust" {
        fn download_manifest(x_session_id: &str, app: &str, version: &str) -> Result<String>;
        fn read_usb_devices() -> Result<Vec<UsbDevice>>;
        fn read_network_adapters() -> Result<Vec<NetAdapter>>;
    }
}
