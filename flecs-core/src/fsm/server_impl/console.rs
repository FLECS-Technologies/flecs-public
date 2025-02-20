use crate::fsm::server_impl::ServerImpl;
use crate::relic::device::usb::UsbDeviceReader;
use async_trait::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecsd_axum_server::apis::console::{
    Console, ConsoleAuthenticationDeleteResponse, ConsoleAuthenticationPutResponse,
};
use flecsd_axum_server::models::AuthResponseData;
use http::Method;

#[async_trait]
impl<T: UsbDeviceReader + Sync> Console for ServerImpl<T> {
    async fn console_authentication_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ConsoleAuthenticationDeleteResponse, ()> {
        crate::sorcerer::authmancer::delete_authentication(&self.vault).await;
        Ok(ConsoleAuthenticationDeleteResponse::Status204_NoContent)
    }

    async fn console_authentication_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: AuthResponseData,
    ) -> Result<ConsoleAuthenticationPutResponse, ()> {
        crate::sorcerer::authmancer::store_authentication(body, &self.vault).await;
        Ok(ConsoleAuthenticationPutResponse::Status204_NoContent)
    }
}
