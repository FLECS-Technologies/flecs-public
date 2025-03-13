mod api;
mod apps;
mod console;
mod deployments;
mod device;
mod instances;
mod jobs;
mod system;
use crate::enchantment::floxy::Floxy;
use crate::enchantment::Enchantments;
use crate::relic::device::net::NetDeviceReader;
use crate::relic::device::usb::UsbDeviceReader;
use crate::relic::network::NetworkAdapterReader;
use crate::sorcerer::appraiser::AppRaiser;
use crate::sorcerer::authmancer::Authmancer;
use crate::sorcerer::instancius::Instancius;
use crate::sorcerer::licenso::Licenso;
use crate::sorcerer::mage_quester::MageQuester;
use crate::sorcerer::manifesto::Manifesto;
use crate::sorcerer::systemus::Systemus;
use crate::sorcerer::SorcerersTemplate;
use crate::vault::Vault;
use anyhow::Error;
use axum::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecs_console_client::models::SessionId;
use flecsd_axum_server::apis::flunder::{Flunder, FlunderBrowseGetResponse};
use flecsd_axum_server::models::{AdditionalInfo, FlunderBrowseGetQueryParams};
use http::Method;
use std::sync::Arc;

fn additional_info_from_error(error: Error) -> AdditionalInfo {
    AdditionalInfo {
        additional_info: format!("{error:#}"),
    }
}

fn ok() -> AdditionalInfo {
    AdditionalInfo {
        additional_info: "OK".to_string(),
    }
}

pub struct ServerImpl<
    APP: AppRaiser,
    AUTH: Authmancer,
    I: Instancius,
    L: Licenso,
    Q: MageQuester,
    M: Manifesto,
    SYS: Systemus,
    F: Floxy,
    T: UsbDeviceReader,
    NET: NetworkAdapterReader,
    NetDev: NetDeviceReader,
> {
    vault: Arc<Vault>,
    enchantments: Enchantments<F>,
    usb_reader: Arc<T>,
    network_adapter_reader: Arc<NET>,
    net_device_reader: Arc<NetDev>,
    sorcerers: SorcerersTemplate<APP, AUTH, I, L, Q, M, SYS>,
}

impl<
        APP: AppRaiser,
        AUTH: Authmancer,
        I: Instancius,
        L: Licenso,
        Q: MageQuester,
        M: Manifesto,
        SYS: Systemus,
        F: Floxy,
        T: UsbDeviceReader,
        NET: NetworkAdapterReader,
        NetDev: NetDeviceReader,
    > ServerImpl<APP, AUTH, I, L, Q, M, SYS, F, T, NET, NetDev>
{
    pub async fn new(
        sorcerers: SorcerersTemplate<APP, AUTH, I, L, Q, M, SYS>,
        enchantments: Enchantments<F>,
        usb_reader: T,
        network_adapter_reader: NET,
        net_device_reader: NetDev,
    ) -> Self {
        Self {
            vault: crate::lore::vault::default().await,
            enchantments,
            usb_reader: Arc::new(usb_reader),
            net_device_reader: Arc::new(net_device_reader),
            network_adapter_reader: Arc::new(network_adapter_reader),
            sorcerers,
        }
    }
}

#[cfg(test)]
impl
    ServerImpl<
        crate::sorcerer::appraiser::MockAppRaiser,
        crate::sorcerer::authmancer::MockAuthmancer,
        crate::sorcerer::instancius::MockInstancius,
        crate::sorcerer::licenso::MockLicenso,
        crate::sorcerer::mage_quester::MockMageQuester,
        crate::sorcerer::manifesto::MockManifesto,
        crate::sorcerer::systemus::MockSystemus,
        crate::enchantment::floxy::MockFloxy,
        crate::relic::device::usb::MockUsbDeviceReader,
        crate::relic::network::MockNetworkAdapterReader,
        crate::relic::device::net::MockNetDeviceReader,
    >
{
    #[cfg(test)]
    pub fn test_instance(
        vault: Arc<Vault>,
        usb_reader: crate::relic::device::usb::MockUsbDeviceReader,
        network_adapter_reader: crate::relic::network::MockNetworkAdapterReader,
        net_device_reader: crate::relic::device::net::MockNetDeviceReader,
        sorcerers: crate::sorcerer::MockSorcerers,
    ) -> Self {
        Self {
            vault,
            enchantments: Enchantments::test_instance(),
            usb_reader: Arc::new(usb_reader),
            sorcerers,
            network_adapter_reader: Arc::new(network_adapter_reader),
            net_device_reader: Arc::new(net_device_reader),
        }
    }
}
#[async_trait]
impl<
        APP: AppRaiser,
        AUTH: Authmancer,
        I: Instancius,
        L: Licenso,
        Q: MageQuester,
        M: Manifesto,
        SYS: Systemus,
        F: Floxy,
        T: UsbDeviceReader,
        NET: NetworkAdapterReader,
        NetDev: NetDeviceReader,
    > Flunder for ServerImpl<APP, AUTH, I, L, Q, M, SYS, F, T, NET, NetDev>
{
    async fn flunder_browse_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _query_params: FlunderBrowseGetQueryParams,
    ) -> Result<FlunderBrowseGetResponse, ()> {
        todo!()
    }
}

fn console_session_id_to_core_session_id(
    session_id: SessionId,
) -> flecsd_axum_server::models::SessionId {
    flecsd_axum_server::models::SessionId {
        id: session_id.id,
        timestamp: session_id.timestamp,
    }
}
