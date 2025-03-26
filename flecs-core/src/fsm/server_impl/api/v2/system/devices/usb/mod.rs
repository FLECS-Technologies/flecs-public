pub mod port;
use crate::relic::device::usb::{UsbDevice, UsbDeviceReader};
use flecsd_axum_server::apis::system::SystemDevicesUsbGetResponse as GetResponse;
use flecsd_axum_server::models;
use std::sync::Arc;

pub fn get<U: UsbDeviceReader>(usb_device_reader: Arc<U>) -> GetResponse {
    match get_usb_devices(usb_device_reader) {
        Ok(usb_devices) => GetResponse::Status200_Success(usb_devices),
        Err(e) => {
            GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

pub(crate) fn get_usb_devices<U: UsbDeviceReader>(
    usb_device_reader: Arc<U>,
) -> Result<Vec<models::UsbDevice>, crate::Error> {
    Ok(usb_device_reader
        .read_usb_devices()?
        .into_values()
        .map(models::UsbDevice::from)
        .collect())
}

impl From<UsbDevice> for models::UsbDevice {
    fn from(value: UsbDevice) -> Self {
        Self {
            name: value.device,
            pid: value.pid as i32,
            port: value.port,
            vendor: value.vendor,
            vid: value.vid as i32,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::relic::device::usb::{Error, MockUsbDeviceReader, UsbDevice};
    use flecsd_axum_server::models;
    use std::collections::HashMap;
    use std::io::ErrorKind;
    use std::sync::Arc;

    pub fn create_mock_usb_reader_error() -> MockUsbDeviceReader {
        let mut usb_reader = MockUsbDeviceReader::new();
        usb_reader.expect_read_usb_devices().times(1).returning(|| {
            Err(Error::Io(std::io::Error::new(
                ErrorKind::Other,
                "test error",
            )))
        });
        usb_reader
    }

    pub fn create_mock_usb_reader_values() -> MockUsbDeviceReader {
        let mut usb_reader = MockUsbDeviceReader::new();
        let device_1 = UsbDevice {
            vid: 123,
            pid: 456,
            port: "test_port_1".to_string(),
            device: "test-dev-1".to_string(),
            vendor: "test-vendor-1".to_string(),
        };
        let device_2 = UsbDevice {
            vid: 10,
            pid: 100,
            port: "test_port_2".to_string(),
            device: "test-dev-2".to_string(),
            vendor: "test-vendor-2".to_string(),
        };
        let device_3 = UsbDevice {
            vid: 6,
            pid: 9,
            port: "test_port_3".to_string(),
            device: "test-dev-3".to_string(),
            vendor: "test-vendor-3".to_string(),
        };
        usb_reader
            .expect_read_usb_devices()
            .times(1)
            .return_once(|| {
                Ok(HashMap::from([
                    (device_1.port.clone(), device_1),
                    (device_2.port.clone(), device_2),
                    (device_3.port.clone(), device_3),
                ]))
            });
        usb_reader
    }

    pub fn create_expected_usb_devices() -> Vec<models::UsbDevice> {
        vec![
            models::UsbDevice {
                name: "test-dev-1".to_string(),
                pid: 456,
                port: "test_port_1".to_string(),
                vendor: "test-vendor-1".to_string(),
                vid: 123,
            },
            models::UsbDevice {
                name: "test-dev-2".to_string(),
                pid: 100,
                port: "test_port_2".to_string(),
                vendor: "test-vendor-2".to_string(),
                vid: 10,
            },
            models::UsbDevice {
                name: "test-dev-3".to_string(),
                pid: 9,
                port: "test_port_3".to_string(),
                vendor: "test-vendor-3".to_string(),
                vid: 6,
            },
        ]
    }

    #[tokio::test]
    async fn get_200() {
        let usb_reader = create_mock_usb_reader_values();
        let expected_devices = create_expected_usb_devices();
        let GetResponse::Status200_Success(result_devices) = get(Arc::new(usb_reader)) else {
            panic!()
        };
        assert_eq!(expected_devices.len(), result_devices.len());
        for expected_device in expected_devices.iter() {
            assert!(result_devices.contains(expected_device));
        }
    }

    #[tokio::test]
    async fn get_500() {
        let usb_reader = create_mock_usb_reader_error();
        assert!(matches!(
            get(Arc::new(usb_reader)),
            GetResponse::Status500_InternalServerError(_)
        ))
    }

    #[tokio::test]
    async fn get_usb_devices_err() {
        let usb_reader = create_mock_usb_reader_error();
        assert!(get_usb_devices(Arc::new(usb_reader)).is_err());
    }

    #[tokio::test]
    async fn get_usb_devices_ok() {
        let usb_reader = create_mock_usb_reader_values();
        let expected_devices = create_expected_usb_devices();
        let result_devices = get_usb_devices(Arc::new(usb_reader)).unwrap();
        assert_eq!(expected_devices.len(), result_devices.len());
        for expected_device in expected_devices.iter() {
            assert!(result_devices.contains(expected_device));
        }
    }

    #[test]
    fn from_usb_device() {
        let usb_device = UsbDevice {
            device: "device".to_string(),
            vendor: "vendor".to_string(),
            port: "from_usb_device".to_string(),
            pid: 2,
            vid: 5,
        };
        assert_eq!(
            models::UsbDevice::from(usb_device),
            models::UsbDevice {
                name: "device".to_string(),
                vid: 5,
                pid: 2,
                vendor: "vendor".to_string(),
                port: "from_usb_device".to_string(),
            }
        );
    }
}
