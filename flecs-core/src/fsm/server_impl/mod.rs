mod apps;
mod console;
mod device;
mod instances;
mod jobs;
mod route_impl;
mod system;
use crate::enchantment::floxy::Floxy;
use crate::enchantment::Enchantments;
use crate::relic::device::usb::UsbDeviceReader;
use crate::sorcerer::appraiser::AppRaiser;
use crate::sorcerer::authmancer::Authmancer;
use crate::sorcerer::instancius::Instancius;
use crate::sorcerer::licenso::Licenso;
use crate::sorcerer::mage_quester::MageQuester;
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
    F: Floxy,
    T: UsbDeviceReader,
> {
    vault: Arc<Vault>,
    enchantments: Enchantments<F>,
    usb_reader: Arc<T>,
    sorcerers: SorcerersTemplate<APP, AUTH, I, L, Q>,
}

impl<
        APP: AppRaiser,
        AUTH: Authmancer,
        I: Instancius,
        L: Licenso,
        Q: MageQuester,
        F: Floxy,
        T: UsbDeviceReader,
    > ServerImpl<APP, AUTH, I, L, Q, F, T>
{
    pub async fn new(
        sorcerers: SorcerersTemplate<APP, AUTH, I, L, Q>,
        enchantments: Enchantments<F>,
        usb_reader: T,
    ) -> Self {
        Self {
            vault: crate::lore::vault::default().await,
            enchantments,
            usb_reader: Arc::new(usb_reader),
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
        crate::enchantment::floxy::MockFloxy,
        crate::relic::device::usb::MockUsbDeviceReader,
    >
{
    #[cfg(test)]
    pub fn test_instance(
        vault: Arc<Vault>,
        usb_reader: crate::relic::device::usb::MockUsbDeviceReader,
        sorcerers: crate::sorcerer::MockSorcerers,
    ) -> Self {
        Self {
            vault,
            enchantments: Enchantments::test_instance(),
            usb_reader: Arc::new(usb_reader),
            sorcerers,
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
        F: Floxy,
        T: UsbDeviceReader,
    > Flunder for ServerImpl<APP, AUTH, I, L, Q, F, T>
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
