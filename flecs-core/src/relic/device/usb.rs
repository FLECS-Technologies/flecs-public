use rusb::Device;
use std::collections::HashSet;
use std::ffi::OsStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Rusb(#[from] rusb::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, PartialEq, Eq, Hash)]
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
            vendor: get_vendor(vid)?.unwrap_or_else(|| format!("Unknown vendor {vid}")),
            device: get_device_name(vid, pid)?.unwrap_or_else(|| format!("Unknown device {pid}")),
            port,
        })
    }
}

fn get_vendor(vid: u16) -> Result<Option<String>> {
    let modalias = format!("usb:v{:04.4X}*", vid);
    query_hwdb_one(modalias.as_str(), "ID_VENDOR_FROM_DATABASE")
}

fn get_device_name(vid: u16, pid: u16) -> Result<Option<String>> {
    let modalias = format!("usb:v{:04.4X}p{:04.4X}*", vid, pid);
    query_hwdb_one(modalias.as_str(), "ID_MODEL_FROM_DATABASE")
}

fn query_hwdb_one<S: AsRef<OsStr>>(modalias: S, name: S) -> Result<Option<String>> {
    let hwdb = udev::Hwdb::new()?;
    let result = hwdb
        .query_one(modalias, name)
        .and_then(|s| s.to_os_string().into_string().ok());
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore]
    fn test_udev() {
        for device in read_usb_devices().unwrap() {
            println!("{device:?}");
        }
    }
}
