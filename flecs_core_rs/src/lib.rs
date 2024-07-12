mod console;
mod network;
mod usb;

pub use crate::console::new_console;
pub use crate::console::Console;
pub use crate::network::read_network_adapters;
pub use crate::usb::read_usb_devices;

#[cxx::bridge]
mod ffi {

    #[derive(Debug)]
    struct Device {
        vid: u16,
        pid: u16,
        port: String,
        device: String,
        vendor: String,
    }

    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    struct User {
        id: u64,
        email: String,
        login: String,
        display_name: String,
    }
    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    struct Jwt {
        token: String,
        token_expires: u64,
    }
    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    struct FeatureFlags {
        is_vendor: bool,
        is_white_labeled: bool,
    }
    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    struct Authentication {
        user: User,
        jwt: Jwt,
        feature_flags: FeatureFlags,
    }

    struct DownloadToken {
        username: String,
        password: String,
    }

    extern "Rust" {

        type Console;

        fn new_console(base_url: String) -> Box<Console>;
        fn activate_license(self: &Console, session_id: String) -> Result<String>;
        fn validate_license(self: &Console, session_id: String) -> Result<bool>;
        fn download_manifest(
            self: &Console,
            app: String,
            version: String,
            session_id: String,
        ) -> Result<String>;
        fn acquire_download_token(
            self: &Console,
            app: String,
            version: String,
            session_id: String,
        ) -> Result<DownloadToken>;
        fn authentication(self: &Console) -> Authentication;

        fn store_authentication(&mut self, authentication: Authentication) -> u16;
        fn delete_authentication(&mut self) -> u16;
    }

    #[derive(Debug)]
    enum NetType {
        Unknown,
        Wired,
        Wireless,
        Local,
        Bridge,
        Virtual,
    }
    #[derive(Debug)]
    struct IpAddr {
        addr: String,
        subnet_mask: String,
    }

    #[derive(Debug)]
    struct NetInfo {
        mac: String,
        net_type: NetType,
        ipv4addresses: Vec<IpAddr>,
        ipv6addresses: Vec<IpAddr>,
        gateway: String,
    }

    struct NetAdapter {
        name: String,
        info: NetInfo,
    }

    extern "Rust" {
        fn read_network_adapters() -> Result<Vec<NetAdapter>>;
        fn read_usb_devices() -> Result<Vec<Device>>;
    }
}
