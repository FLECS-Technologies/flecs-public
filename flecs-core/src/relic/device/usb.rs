#[cfg(test)]
use mockall::{automock, predicate::*};
use rusb::{Device, UsbContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use thiserror::Error;
use usb_ids::FromId;

#[cfg(not(test))]
const USB_DEVICE_PATH: &str = "/sys/bus/usb/devices/";
#[cfg(test)]
const USB_DEVICE_PATH: &str = "/tmp/flecs-tests/sys/bus/usb/devices/";

#[cfg_attr(test, automock)]
pub trait UsbDeviceReader: Sync + Send {
    fn read_usb_devices(&self) -> Result<HashMap<UsbPort, UsbDevice>>;
    fn get_vendor(&self, vid: u16, port: &str) -> Result<String>;

    fn get_device_name(&self, vid: u16, pid: u16, port: &str) -> Result<String>;

    fn get_usb_value(&self, value_name: &str, port: &str) -> Result<String>;
}

pub trait UsbDeviceReaderExtension {
    fn get_product(&self, port: &str) -> Result<String>;
    fn get_manufacturer(&self, port: &str) -> Result<String>;
    fn get_bus_num(&self, port: &str) -> Result<u16>;
    fn get_dev_num(&self, port: &str) -> Result<u16>;
}

impl<T: ?Sized + UsbDeviceReader> UsbDeviceReaderExtension for T {
    fn get_product(&self, port: &str) -> Result<String> {
        self.get_usb_value("product", port)
    }

    fn get_manufacturer(&self, port: &str) -> Result<String> {
        self.get_usb_value("manufacturer", port)
    }

    fn get_bus_num(&self, port: &str) -> Result<u16> {
        let bus_num = self.get_usb_value("busnum", port)?;
        Ok(u16::from_str(&bus_num)?)
    }

    fn get_dev_num(&self, port: &str) -> Result<u16> {
        let dev_num = self.get_usb_value("devnum", port)?;
        Ok(u16::from_str(&dev_num)?)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct UsbDeviceReaderImpl {}

impl UsbDeviceReader for UsbDeviceReaderImpl {
    fn read_usb_devices(&self) -> Result<HashMap<UsbPort, UsbDevice>> {
        let devices = rusb::Context::new()?
            .devices()?
            .iter()
            .flat_map(Self::try_usb_device_from)
            .map(|device| (device.port.clone(), device))
            .collect();
        Ok(devices)
    }

    fn get_vendor(&self, vid: u16, port: &str) -> Result<String> {
        match usb_ids::Vendor::from_id(vid) {
            Some(vendor) => Ok(vendor.name().to_string()),
            None => self.get_manufacturer(port),
        }
    }

    fn get_device_name(&self, vid: u16, pid: u16, port: &str) -> Result<String> {
        match usb_ids::Device::from_vid_pid(vid, pid) {
            Some(device) => Ok(device.name().to_string()),
            None => self.get_product(port),
        }
    }

    fn get_usb_value(&self, value_name: &str, port: &str) -> Result<String> {
        let path = format!("{USB_DEVICE_PATH}{port}/{value_name}");
        Ok(fs::read_to_string(path)?.trim_end().to_string())
    }
}

impl Default for UsbDeviceReaderImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl UsbDeviceReaderImpl {
    pub fn new() -> Self {
        Self {}
    }

    fn try_usb_device_from<T: rusb::UsbContext>(device: Device<T>) -> Result<UsbDevice> {
        let reader = Self::new();
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
            vendor: reader
                .get_vendor(vid, &port)
                .unwrap_or_else(|_| format!("Unknown vendor {vid}")),
            device: reader
                .get_device_name(vid, pid, &port)
                .unwrap_or_else(|_| format!("Unknown device {pid}")),
            port,
        })
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Rusb(#[from] rusb::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Parse(#[from] core::num::ParseIntError),
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
type UsbPort = String;

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::io::ErrorKind;
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
    fn usb_device_reader_impl_get_usb_value_ok() {
        let reader = UsbDeviceReaderImpl::new();
        let path = prepare_usb_device_test_path("usb_device_reader_impl_get_usb_value_ok");
        let port = "usb_device_reader_impl_get_usb_value_ok";
        fs::write(path.join("test-property-1"), b"test-value-1").unwrap();
        fs::write(path.join("linebreak"), b"test-value-with-linebreak\n").unwrap();
        fs::write(
            path.join("whitespace"),
            b"\t value with lots of whitespace    \t\n",
        )
        .unwrap();
        assert_eq!(
            reader.get_usb_value("test-property-1", port).unwrap(),
            "test-value-1"
        );
        assert_eq!(
            reader.get_usb_value("linebreak", port).unwrap(),
            "test-value-with-linebreak"
        );
        assert_eq!(
            reader.get_usb_value("whitespace", port).unwrap(),
            "\t value with lots of whitespace"
        );
    }

    #[test]
    fn usb_device_reader_impl_get_usb_value_err() {
        let reader = UsbDeviceReaderImpl::new();
        let port = "usb_device_reader_impl_get_usb_value_err";
        assert!(reader.get_usb_value("whitespace", port).is_err(),);
    }

    #[test]
    fn usb_device_reader_impl_get_vendor_ok_in_db() {
        let reader = UsbDeviceReaderImpl::new();
        let port = "usb_device_reader_impl_get_vendor_ok_in_db";
        assert_eq!(reader.get_vendor(7531, port).unwrap(), "Linux Foundation");
    }

    #[test]
    fn usb_device_reader_impl_get_vendor_ok_on_file() {
        let reader = UsbDeviceReaderImpl::new();
        let path = prepare_usb_device_test_path("usb_device_reader_impl_get_vendor_ok_on_file");
        let port = "usb_device_reader_impl_get_vendor_ok_on_file";
        fs::write(path.join("manufacturer"), b"Non Existent Vendor #5").unwrap();
        assert_eq!(
            reader.get_vendor(4444, port).unwrap(),
            "Non Existent Vendor #5"
        );
    }

    #[test]
    fn usb_device_reader_impl_get_vendor_err() {
        let reader = UsbDeviceReaderImpl::new();
        let port = "usb_device_reader_impl_get_vendor_err";
        assert!(reader.get_vendor(4444, port).is_err());
    }

    #[test]
    fn usb_device_reader_impl_get_device_name_ok_in_db() {
        let reader = UsbDeviceReaderImpl::new();
        let port = "usb_device_reader_impl_get_device_name_ok_in_db";
        assert_eq!(reader.get_device_name(2199, 4, port).unwrap(), "PowerDebug");
    }

    #[test]
    fn usb_device_reader_impl_get_device_name_ok_on_file() {
        let reader = UsbDeviceReaderImpl::new();
        let path =
            prepare_usb_device_test_path("usb_device_reader_impl_get_device_name_ok_on_file");
        let port = "usb_device_reader_impl_get_device_name_ok_on_file";
        fs::write(path.join("product"), b"Non Existent product A").unwrap();
        assert_eq!(
            reader.get_device_name(4444, 55, port).unwrap(),
            "Non Existent product A"
        );
    }

    #[test]
    fn usb_device_reader_impl_get_device_name_err() {
        let reader = UsbDeviceReaderImpl::new();
        let port = "usb_device_reader_impl_get_device_name_err";
        assert!(reader.get_device_name(4444, 55, port).is_err());
    }

    #[test]
    fn get_product_ok() {
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_get_usb_value()
            .withf(|value_name, _| value_name == "product")
            .times(1)
            .returning(|_, _| Ok("Test Product 9000".to_string()));
        assert_eq!(
            reader.get_product("get_product_ok").unwrap(),
            "Test Product 9000".to_string()
        );
    }

    #[test]
    fn get_product_err() {
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_get_usb_value()
            .withf(|value_name, _| value_name == "product")
            .times(1)
            .returning(|_, _| {
                Err(Error::Io(std::io::Error::new(
                    ErrorKind::Other,
                    "test error",
                )))
            });
        assert!(reader.get_product("get_product_err").is_err());
    }

    #[test]
    fn get_manufacturer_ok() {
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_get_usb_value()
            .withf(|value_name, _| value_name == "manufacturer")
            .times(1)
            .returning(|_, _| Ok("Test Manufacturer Inc".to_string()));
        assert_eq!(
            reader.get_manufacturer("get_manufacturer_ok").unwrap(),
            "Test Manufacturer Inc".to_string()
        );
    }

    #[test]
    fn get_manufacturer_err() {
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_get_usb_value()
            .withf(|value_name, _| value_name == "manufacturer")
            .times(1)
            .returning(|_, _| {
                Err(Error::Io(std::io::Error::new(
                    ErrorKind::Other,
                    "test error",
                )))
            });
        assert!(reader.get_manufacturer("get_manufacturer_err").is_err());
    }

    #[test]
    fn get_bus_num_ok() {
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_get_usb_value()
            .withf(|value_name, _| value_name == "busnum")
            .times(1)
            .returning(|_, _| Ok("123".to_string()));
        assert_eq!(reader.get_bus_num("get_bus_num_ok").unwrap(), 123);
    }

    #[test]
    fn get_bus_num_err_none() {
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_get_usb_value()
            .withf(|value_name, _| value_name == "busnum")
            .times(1)
            .returning(|_, _| {
                Err(Error::Io(std::io::Error::new(
                    ErrorKind::Other,
                    "test error",
                )))
            });
        assert!(reader.get_bus_num("get_bus_num_err_none").is_err());
    }

    #[test]
    fn get_bus_num_err_parse() {
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_get_usb_value()
            .withf(|value_name, _| value_name == "busnum")
            .times(1)
            .returning(|_, _| Ok("invalid number".to_string()));
        assert!(reader.get_bus_num("get_bus_num_ok").is_err());
    }

    #[test]
    fn get_dev_num_ok() {
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_get_usb_value()
            .withf(|value_name, _| value_name == "devnum")
            .times(1)
            .returning(|_, _| Ok("123".to_string()));
        assert_eq!(reader.get_dev_num("get_dev_num_ok").unwrap(), 123);
    }

    #[test]
    fn get_dev_num_err_none() {
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_get_usb_value()
            .withf(|value_name, _| value_name == "devnum")
            .times(1)
            .returning(|_, _| {
                Err(Error::Io(std::io::Error::new(
                    ErrorKind::Other,
                    "test error",
                )))
            });
        assert!(reader.get_dev_num("get_dev_num_err_none").is_err());
    }

    #[test]
    fn get_dev_num_err_parse() {
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_get_usb_value()
            .withf(|value_name, _| value_name == "devnum")
            .times(1)
            .returning(|_, _| Ok("invalid number".to_string()));
        assert!(reader.get_dev_num("get_bus_num_ok").is_err());
    }
}
