use crate::fsm::server_impl::{ok, ServerImpl};
use async_trait::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecsd_axum_server::apis::system::{
    System, SystemDevicesGetResponse, SystemDevicesUsbGetResponse, SystemDevicesUsbPortGetResponse,
    SystemInfoGetResponse, SystemPingGetResponse, SystemVersionGetResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::SystemDevicesUsbPortGetPathParams;
use http::Method;
use tracing::error;

#[async_trait]
impl System for ServerImpl {
    async fn system_devices_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemDevicesGetResponse, ()> {
        todo!()
    }

    async fn system_devices_usb_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemDevicesUsbGetResponse, ()> {
        todo!()
    }

    async fn system_devices_usb_port_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _path_params: SystemDevicesUsbPortGetPathParams,
    ) -> Result<SystemDevicesUsbPortGetResponse, ()> {
        todo!()
    }

    async fn system_info_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemInfoGetResponse, ()> {
        Ok(SystemInfoGetResponse::Status200_Sucess(
            crate::relic::system::info::try_create_system_info().map_err(|e| {
                error!("Could not create SystemInfo: {e}");
            })?,
        ))
    }

    async fn system_ping_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemPingGetResponse, ()> {
        Ok(SystemPingGetResponse::Status200_Success(ok()))
    }

    async fn system_version_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemVersionGetResponse, ()> {
        Ok(SystemVersionGetResponse::Status200_Success(
            models::SystemVersionGet200Response {
                api: crate::lore::API_VERSION.to_string(),
                core: crate::lore::CORE_VERSION.to_string(),
            },
        ))
    }
}
