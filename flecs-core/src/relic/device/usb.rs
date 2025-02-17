use rusb::Device;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::str::FromStr;
use thiserror::Error;
use usb_ids::FromId;

#[cfg(not(test))]
const USB_DEVICE_PATH: &str = "/sys/bus/usb/devices/";
#[cfg(test)]
const USB_DEVICE_PATH: &str = "/tmp/flecs-tests/sys/bus/usb/devices/";

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Rusb(#[from] rusb::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct UsbDevice {
    pub vid: u16,
    pub pid: u16,
    pub port: String,
    pub device: String,
    pub vendor: String,
}

type Result<T> = std::result::Result<T, Error>;

pub fn read_usb_devices() -> Result<HashSet<UsbDevice>> {
    let devices = rusb::devices()?
        .iter()
        .flat_map(|device| device.try_into())
        .collect();
    Ok(devices)
}
impl<T: rusb::UsbContext> TryFrom<Device<T>> for UsbDevice {
    type Error = Error;

    fn try_from(device: Device<T>) -> Result<Self> {
        let device_desc = device.device_descriptor()?;
        let pid = device_desc.product_id();
        let vid = device_desc.vendor_id();
        let bus = device.bus_number();
        let port_numbers = device.port_numbers()?;
        let port = if port_numbers.is_empty() {
            format!("usb{bus}")
        } else {
            let mut port = format!("{bus}-{}", port_numbers[0]);
            for p in port_numbers.into_iter().skip(1) {
                port.push_str(format!(".{p}").as_str());
            }
            port
        };
        Ok(UsbDevice {
            pid,
            vid,
            vendor: get_vendor(vid, &port).unwrap_or_else(|| format!("Unknown vendor {vid}")),
            device: get_device_name(vid, pid, &port)
                .unwrap_or_else(|| format!("Unknown device {pid}")),
            port,
        })
    }
}

fn get_vendor(vid: u16, port: &str) -> Option<String> {
    usb_ids::Vendor::from_id(vid)
        .map(|v| v.name().to_string())
        .or_else(|| get_manufacturer(port))
}

fn get_device_name(vid: u16, pid: u16, port: &str) -> Option<String> {
    usb_ids::Device::from_vid_pid(vid, pid)
        .map(|device| device.name().to_string())
        .or_else(|| get_product(port))
}

fn get_product(port: &str) -> Option<String> {
    get_usb_value("product", port)
}

fn get_manufacturer(port: &str) -> Option<String> {
    get_usb_value("manufacturer", port)
}

fn get_bus_num(port: &str) -> Option<u16> {
    let bus_num = get_usb_value("busnum", port)?;
    u16::from_str(&bus_num).ok()
}

fn get_dev_num(port: &str) -> Option<u16> {
    let dev_num = get_usb_value("devnum", port)?;
    u16::from_str(&dev_num).ok()
}

fn get_usb_value(value_name: &str, port: &str) -> Option<String> {
    fs::read_to_string(format!("{USB_DEVICE_PATH}{port}/{value_name}"))
        .ok()
        .map(|content| content.trim_end().to_string())
}

impl UsbDevice {
    pub fn get_bus_num(&self) -> Option<u16> {
        get_bus_num(&self.port)
    }

    pub fn get_dev_num(&self) -> Option<u16> {
        get_dev_num(&self.port)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    pub fn prepare_usb_device_test_path(test_name: &str) -> PathBuf {
        let path = Path::new(USB_DEVICE_PATH).join(test_name);
        println!("Preparing {:?}", path);
        let _ = fs::remove_dir_all(&path);
        assert!(!path.try_exists().unwrap());
        fs::create_dir_all(&path).unwrap();
        assert!(path.try_exists().unwrap());
        path
    }

    #[test]
    fn get_product_ok() {
        let path = prepare_usb_device_test_path("get_product_ok");
        fs::write(path.join("product"), b"Test Product 9000").unwrap();
        assert_eq!(
            get_product("get_product_ok"),
            Some("Test Product 9000".to_string())
        );
    }

    #[test]
    fn get_product_none() {
        prepare_usb_device_test_path("get_product_none");
        assert_eq!(get_product("get_product_none"), None);
    }

    #[test]
    fn get_manufacturer_ok() {
        let path = prepare_usb_device_test_path("get_manufacturer_ok");
        fs::write(path.join("manufacturer"), b"Test Manufacturer Inc").unwrap();
        assert_eq!(
            get_manufacturer("get_manufacturer_ok"),
            Some("Test Manufacturer Inc".to_string())
        );
    }

    #[test]
    fn get_manufacturer_none() {
        prepare_usb_device_test_path("get_manufacturer_none");
        assert_eq!(get_manufacturer("get_manufacturer_none"), None);
    }

    #[test]
    fn get_bus_num_ok() {
        let path = prepare_usb_device_test_path("get_bus_num_ok");
        fs::write(path.join("busnum"), b"123").unwrap();
        assert_eq!(get_bus_num("get_bus_num_ok"), Some(123));
        fs::write(path.join("busnum"), b"123\n").unwrap();
        assert_eq!(get_bus_num("get_bus_num_ok"), Some(123));
    }

    #[test]
    fn get_bus_num_none() {
        prepare_usb_device_test_path("get_bus_num_none");
        assert_eq!(get_product("get_bus_num_none"), None);
    }

    #[test]
    fn get_bus_num_err() {
        let path = prepare_usb_device_test_path("get_bus_num_err");
        fs::write(path.join("busnum"), b"invalid number").unwrap();
        assert_eq!(get_product("get_bus_num_err"), None);
    }

    #[test]
    fn get_dev_num_ok() {
        let path = prepare_usb_device_test_path("get_dev_num_ok");
        fs::write(path.join("devnum"), b"123").unwrap();
        assert_eq!(get_dev_num("get_dev_num_ok"), Some(123));
        fs::write(path.join("devnum"), b"123\n").unwrap();
        assert_eq!(get_dev_num("get_dev_num_ok"), Some(123));
    }

    #[test]
    fn get_dev_num_none() {
        prepare_usb_device_test_path("get_dev_num_none");
        assert_eq!(get_product("get_dev_num_none"), None);
    }

    #[test]
    fn get_dev_num_err() {
        let path = prepare_usb_device_test_path("get_dev_num_err");
        fs::write(path.join("devnum"), b"invalid number").unwrap();
        assert_eq!(get_product("get_dev_num_err"), None);
    }
}
