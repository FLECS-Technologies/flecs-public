use crate::enchantment::floxy::Floxy;
use crate::fsm::server_impl::ServerImpl;
use crate::relic::device::net::NetDeviceReader;
use crate::relic::device::usb::UsbDeviceReader;
use crate::relic::network::NetworkAdapterReader;
use crate::sorcerer::appraiser::AppRaiser;
use crate::sorcerer::authmancer::Authmancer;
use crate::sorcerer::deploymento::Deploymento;
use crate::sorcerer::instancius::Instancius;
use crate::sorcerer::licenso::Licenso;
use crate::sorcerer::mage_quester::MageQuester;
use crate::sorcerer::manifesto::Manifesto;
use crate::sorcerer::systemus::Systemus;
use async_trait::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecsd_axum_server::apis::device::{
    Device, DeviceLicenseActivationPostResponse, DeviceLicenseActivationStatusGetResponse,
    DeviceLicenseInfoGetResponse, DeviceOnboardingPostResponse,
};
use flecsd_axum_server::models::Dosschema;
use http::Method;

#[async_trait]
impl<
        APP: AppRaiser + 'static,
        AUTH: Authmancer,
        I: Instancius,
        L: Licenso,
        Q: MageQuester,
        M: Manifesto,
        SYS: Systemus,
        D: Deploymento,
        F: Floxy,
        T: UsbDeviceReader,
        NET: NetworkAdapterReader,
        NetDev: NetDeviceReader,
    > Device for ServerImpl<APP, AUTH, I, L, Q, M, SYS, D, F, T, NET, NetDev>
{
    async fn device_license_activation_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<DeviceLicenseActivationPostResponse, ()> {
        Ok(super::api::v2::device::license::activation::post(
            self.vault.clone(),
            self.sorcerers.licenso.clone(),
            crate::lore::console_client_config::default().await,
        )
        .await)
    }

    async fn device_license_activation_status_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<DeviceLicenseActivationStatusGetResponse, ()> {
        Ok(super::api::v2::device::license::activation::status::get(
            self.vault.clone(),
            self.sorcerers.licenso.clone(),
            crate::lore::console_client_config::default().await,
        )
        .await)
    }

    async fn device_license_info_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<DeviceLicenseInfoGetResponse, ()> {
        Ok(super::api::v2::device::license::info::get(self.vault.clone()).await)
    }

    async fn device_onboarding_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: Dosschema,
    ) -> Result<DeviceOnboardingPostResponse, ()> {
        super::api::v2::device::onboarding::post(
            self.vault.clone(),
            self.sorcerers.app_raiser.clone(),
            body,
        )
        .await
    }
}
