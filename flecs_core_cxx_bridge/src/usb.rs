use crate::ffi;
use flecs_core::relic::device::usb;

impl From<usb::UsbDevice> for ffi::UsbDevice {
    fn from(value: usb::UsbDevice) -> Self {
        Self {
            vid: value.vid,
            pid: value.pid,
            port: value.port,
            device: value.device,
            vendor: value.vendor,
        }
    }
}

pub fn read_usb_devices() -> Result<Vec<ffi::UsbDevice>, usb::Error> {
    usb::read_usb_devices().map(|set| set.into_iter().map(|dev| dev.into()).collect())
}
