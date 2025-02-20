use crate::ffi;
use flecs_core::relic::device::usb;
use flecs_core::relic::device::usb::UsbDeviceReader;
use flecs_core::Result;

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

pub fn read_usb_devices() -> Result<Vec<ffi::UsbDevice>> {
    Ok(usb::UsbDeviceReaderImpl::new()
        .read_usb_devices()?
        .into_values()
        .map(ffi::UsbDevice::from)
        .collect())
}
