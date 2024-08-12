use rusb::Device;
use std::collections::HashSet;
use std::fs;
use thiserror::Error;
use usb_ids::FromId;

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
        .or_else(|| fs::read_to_string(format!("/sys/bus/usb/devices/{port}/manufacturer")).ok())
}

fn get_device_name(vid: u16, pid: u16, port: &str) -> Option<String> {
    usb_ids::Device::from_vid_pid(vid, pid)
        .map(|device| device.name().to_string())
        .or_else(|| fs::read_to_string(format!("/sys/bus/usb/devices/{port}/product")).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore]
    fn print_devices() {
        for device in read_usb_devices().unwrap() {
            println!("{device:?}");
        }
    }
}
