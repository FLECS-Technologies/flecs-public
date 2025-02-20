mod apps;
mod console;
mod device;
mod instances;
mod jobs;
mod system;
use crate::relic::device::usb::UsbDeviceReader;
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

pub struct ServerImpl<T: UsbDeviceReader> {
    vault: Arc<Vault>,
    usb_reader: T,
}

impl<T: UsbDeviceReader> ServerImpl<T> {
    pub async fn new(usb_reader: T) -> Self {
        Self {
            vault: crate::lore::vault::default().await,
            usb_reader,
        }
    }
}

#[async_trait]
impl<T: UsbDeviceReader + Sync> Flunder for ServerImpl<T> {
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
