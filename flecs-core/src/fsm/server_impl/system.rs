use crate::fsm::server_impl::ServerImpl;
use crate::relic::device::net::NetDeviceReader;
use crate::relic::device::usb::UsbDeviceReader;
use crate::relic::network::NetworkAdapterReader;
use crate::sorcerer::appraiser::AppRaiser;
use crate::sorcerer::authmancer::Authmancer;
use crate::sorcerer::deploymento::Deploymento;
use crate::sorcerer::exportius::Exportius;
use crate::sorcerer::importius::Importius;
use crate::sorcerer::instancius::Instancius;
use crate::sorcerer::licenso::Licenso;
use crate::sorcerer::mage_quester::MageQuester;
use crate::sorcerer::manifesto::Manifesto;
use crate::sorcerer::systemus::Systemus;
use async_trait::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecsd_axum_server::apis::system::{
    System, SystemDevicesGetResponse, SystemDevicesUsbGetResponse, SystemDevicesUsbPortGetResponse,
    SystemInfoGetResponse, SystemNetworkAdaptersGetResponse,
    SystemNetworkAdaptersNetworkAdapterIdGetResponse, SystemPingGetResponse,
    SystemVersionGetResponse,
};
use flecsd_axum_server::models::{
    SystemDevicesUsbPortGetPathParams, SystemNetworkAdaptersNetworkAdapterIdGetPathParams,
};
use http::Method;

#[async_trait]
impl<
    APP: AppRaiser,
    AUTH: Authmancer,
    I: Instancius,
    L: Licenso,
    Q: MageQuester,
    M: Manifesto,
    SYS: Systemus + 'static,
    D: Deploymento,
    E: Exportius,
    IMP: Importius,
    T: UsbDeviceReader,
    NET: NetworkAdapterReader + 'static,
    NetDev: NetDeviceReader + 'static,
> System for ServerImpl<APP, AUTH, I, L, Q, M, SYS, D, E, IMP, T, NET, NetDev>
{
    async fn system_devices_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemDevicesGetResponse, ()> {
        Ok(super::api::v2::system::devices::get(
            self.relics.usb_device_reader.clone(),
        ))
    }

    async fn system_devices_usb_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemDevicesUsbGetResponse, ()> {
        Ok(super::api::v2::system::devices::usb::get(
            self.relics.usb_device_reader.clone(),
        ))
    }

    async fn system_devices_usb_port_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: SystemDevicesUsbPortGetPathParams,
    ) -> Result<SystemDevicesUsbPortGetResponse, ()> {
        Ok(super::api::v2::system::devices::usb::port::get(
            self.relics.usb_device_reader.clone(),
            path_params,
        ))
    }

    async fn system_info_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemInfoGetResponse, ()> {
        super::api::v2::system::info::get()
    }

    async fn system_network_adapters_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemNetworkAdaptersGetResponse, ()> {
        Ok(super::api::v2::system::network_adapters::get(
            self.sorcerers.systemus.clone(),
            self.relics.network_adapter_reader.clone(),
            self.relics.net_device_reader.clone(),
        ))
    }

    async fn system_network_adapters_network_adapter_id_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: SystemNetworkAdaptersNetworkAdapterIdGetPathParams,
    ) -> Result<SystemNetworkAdaptersNetworkAdapterIdGetResponse, ()> {
        Ok(
            super::api::v2::system::network_adapters::network_adapter_id::get(
                self.sorcerers.systemus.clone(),
                self.relics.network_adapter_reader.clone(),
                self.relics.net_device_reader.clone(),
                path_params,
            ),
        )
    }

    async fn system_ping_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemPingGetResponse, ()> {
        Ok(super::api::v2::system::ping::get())
    }

    async fn system_version_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<SystemVersionGetResponse, ()> {
        Ok(super::api::v2::system::version::get())
    }
}
