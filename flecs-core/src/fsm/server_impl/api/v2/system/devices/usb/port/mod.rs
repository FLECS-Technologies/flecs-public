use crate::relic::device::usb::UsbDeviceReader;
use flecsd_axum_server::apis::system::SystemDevicesUsbPortGetResponse as GetResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::SystemDevicesUsbPortGetPathParams as GetPathParams;
use std::sync::Arc;

pub fn get<U: UsbDeviceReader>(
    usb_device_reader: Arc<U>,
    path_params: GetPathParams,
) -> GetResponse {
    let mut usb_devices = match usb_device_reader.read_usb_devices() {
        Ok(usb_devices) => usb_devices,
        Err(e) => {
            return GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(
                e.to_string(),
            ));
        }
    };
    match usb_devices.remove(&path_params.port) {
        None => GetResponse::Status404_DeviceNotFound,
        Some(device) => GetResponse::Status200_Success(models::UsbDevice::from(device)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsm::server_impl::api::v2::system::devices::usb::tests::{
        create_expected_usb_devices, create_mock_usb_reader_error, create_mock_usb_reader_values,
    };

    #[tokio::test]
    async fn get_200() {
        let usb_reader = create_mock_usb_reader_values();
        assert_eq!(
            get(
                Arc::new(usb_reader),
                GetPathParams {
                    port: "test_port_2".to_string()
                }
            ),
            GetResponse::Status200_Success(create_expected_usb_devices()[1].clone())
        )
    }

    #[tokio::test]
    async fn get_404() {
        let usb_reader = create_mock_usb_reader_values();
        assert_eq!(
            get(
                Arc::new(usb_reader),
                GetPathParams {
                    port: "unknown-port".to_string()
                }
            ),
            GetResponse::Status404_DeviceNotFound
        )
    }

    #[tokio::test]
    async fn get_500() {
        let usb_reader = create_mock_usb_reader_error();
        assert!(matches!(
            get(
                Arc::new(usb_reader),
                GetPathParams {
                    port: "port".to_string()
                }
            ),
            GetResponse::Status500_InternalServerError(_)
        ))
    }
}
