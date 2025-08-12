use crate::enchantment::floxy::Floxy;
use crate::fsm::server_impl::ServerImpl;
#[cfg(feature = "auth")]
use crate::fsm::server_impl::api;
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
use flecsd_axum_server::apis::authentication::{
    AuthProvidersDefaultLocationGetResponse, AuthProvidersDefaultProtocolGetResponse,
    Authentication,
};

#[async_trait]
impl<
    APP: AppRaiser,
    AUTH: Authmancer,
    I: Instancius,
    L: Licenso,
    Q: MageQuester,
    M: Manifesto,
    SYS: Systemus,
    D: Deploymento,
    E: Exportius,
    IMP: Importius,
    F: Floxy,
    T: UsbDeviceReader,
    NET: NetworkAdapterReader,
    NetDev: NetDeviceReader,
> Authentication for ServerImpl<APP, AUTH, I, L, Q, M, SYS, D, E, IMP, F, T, NET, NetDev>
{
    async fn auth_providers_default_location_get(
        &self,
        _method: http::Method,
        _host: axum::extract::Host,
        _cookies: axum_extra::extract::CookieJar,
    ) -> Result<AuthProvidersDefaultLocationGetResponse, ()> {
        #[cfg(feature = "auth")]
        {
            Ok(api::v2::auth::providers::default::location::get(
                self.lore.clone(),
            ))
        }
        #[cfg(not(feature = "auth"))]
        Err(())
    }

    async fn auth_providers_default_protocol_get(
        &self,
        _method: http::Method,
        _host: axum::extract::Host,
        _cookies: axum_extra::extract::CookieJar,
    ) -> Result<AuthProvidersDefaultProtocolGetResponse, ()> {
        #[cfg(feature = "auth")]
        {
            Ok(api::v2::auth::providers::default::protocol::get())
        }
        #[cfg(not(feature = "auth"))]
        Err(())
    }
}
