use crate::enchantment::floxy::Floxy;
use crate::fsm::server_impl::ServerImpl;
use crate::relic::device::usb::UsbDeviceReader;
use crate::sorcerer::appraiser::AppRaiser;
use crate::sorcerer::authmancer::Authmancer;
use crate::sorcerer::instancius::Instancius;
use crate::sorcerer::licenso::Licenso;
use crate::sorcerer::mage_quester::MageQuester;
use async_trait::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecsd_axum_server::apis::console::{
    Console, ConsoleAuthenticationDeleteResponse, ConsoleAuthenticationPutResponse,
};
use flecsd_axum_server::models::AuthResponseData;
use http::Method;

#[async_trait]
impl<
        APP: AppRaiser,
        AUTH: Authmancer,
        I: Instancius,
        L: Licenso,
        Q: MageQuester,
        F: Floxy,
        T: UsbDeviceReader,
    > Console for ServerImpl<APP, AUTH, I, L, Q, F, T>
{
    async fn console_authentication_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ConsoleAuthenticationDeleteResponse, ()> {
        self.sorcerers
            .authmancer
            .delete_authentication(&self.vault)
            .await;
        Ok(ConsoleAuthenticationDeleteResponse::Status204_NoContent)
    }

    async fn console_authentication_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: AuthResponseData,
    ) -> Result<ConsoleAuthenticationPutResponse, ()> {
        self.sorcerers
            .authmancer
            .store_authentication(body, &self.vault)
            .await;
        Ok(ConsoleAuthenticationPutResponse::Status204_NoContent)
    }
}
