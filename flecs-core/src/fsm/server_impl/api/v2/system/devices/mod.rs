pub mod usb;
use crate::relic::device::usb::UsbDeviceReader;
use flecsd_axum_server::apis::system::SystemDevicesGetResponse as GetResponse;
use flecsd_axum_server::models;
use std::sync::Arc;

pub fn get<U: UsbDeviceReader>(usb_device_reader: Arc<U>) -> GetResponse {
    match usb::get_usb_devices(usb_device_reader) {
        Ok(usb_devices) => GetResponse::Status200_Success(models::Devices {
            usb: Some(usb_devices),
        }),
        Err(e) => {
            GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
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
        let expected_devices = create_expected_usb_devices();
        let GetResponse::Status200_Success(models::Devices {
            usb: Some(result_devices),
        }) = get(Arc::new(usb_reader))
        else {
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
}
