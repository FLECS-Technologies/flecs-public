pub mod device;
pub mod network;
pub mod system;
pub use super::{Error, Result};
use crate::relic::device::net::{NetDeviceReader, NetDeviceReaderImpl};
use crate::relic::device::usb::{UsbDeviceReader, UsbDeviceReaderImpl};
use crate::relic::network::{NetworkAdapterReader, NetworkAdapterReaderImpl};
use std::sync::Arc;

/// Helper functions that provide async versions of [flecstract::tar::extract] and [flecstract::tar::archive]
pub mod async_flecstract;
pub mod docker;
pub mod docker_cli;
pub mod nginx;
pub mod process;
pub mod serde;

pub struct Relics<UDR: UsbDeviceReader, NAR: NetworkAdapterReader, NDR: NetDeviceReader> {
    pub usb_device_reader: Arc<UDR>,
    pub network_adapter_reader: Arc<NAR>,
    pub net_device_reader: Arc<NDR>,
}

pub type FlecsRelics = Relics<UsbDeviceReaderImpl, NetworkAdapterReaderImpl, NetDeviceReaderImpl>;

impl Default for FlecsRelics {
    fn default() -> Self {
        Self {
            usb_device_reader: Default::default(),
            network_adapter_reader: Default::default(),
            net_device_reader: Default::default(),
        }
    }
}
