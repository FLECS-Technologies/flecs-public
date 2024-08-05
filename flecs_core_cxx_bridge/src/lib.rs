mod manifest;
mod usb;

pub use crate::manifest::download_manifest;
pub use usb::read_usb_devices;

#[cxx::bridge]
mod ffi {

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
    }
}
