use crate::enchantment::floxy::Floxy;
use crate::fsm::server_impl::{ServerImpl, api};
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
use flecsd_axum_server::apis::experimental::{
    Experimental, InstancesInstanceIdDependsFeatureDeleteResponse,
    InstancesInstanceIdDependsFeatureGetResponse, InstancesInstanceIdDependsFeaturePutResponse,
    InstancesInstanceIdDependsGetResponse, InstancesInstanceIdProvidesFeatureGetResponse,
    InstancesInstanceIdProvidesGetResponse, ProvidersAuthCoreFirstTimeSetupSuperAdminGetResponse,
    ProvidersAuthCoreFirstTimeSetupSuperAdminPostResponse, ProvidersAuthCoreGetResponse,
    ProvidersAuthCorePutResponse, ProvidersAuthDefaultDeleteResponse,
    ProvidersAuthDefaultFirstTimeSetupSuperAdminGetResponse,
    ProvidersAuthDefaultFirstTimeSetupSuperAdminPostResponse, ProvidersAuthDefaultGetResponse,
    ProvidersAuthDefaultPutResponse, ProvidersAuthFirstTimeSetupFlecsportPostResponse,
    ProvidersAuthGetResponse, ProvidersAuthIdFirstTimeSetupSuperAdminGetResponse,
    ProvidersAuthIdFirstTimeSetupSuperAdminPostResponse, ProvidersAuthIdGetResponse,
    ProvidersFeatureDefaultDeleteResponse, ProvidersFeatureDefaultGetResponse,
    ProvidersFeatureDefaultPutResponse, ProvidersFeatureGetResponse, ProvidersFeatureIdGetResponse,
    ProvidersGetResponse,
};
use flecsd_axum_server::models::{
    InstancesInstanceIdDependsFeatureDeletePathParams,
    InstancesInstanceIdDependsFeatureGetPathParams, InstancesInstanceIdDependsFeaturePutPathParams,
    InstancesInstanceIdDependsGetPathParams, InstancesInstanceIdProvidesFeatureGetPathParams,
    InstancesInstanceIdProvidesGetPathParams, ProviderReference,
    ProvidersAuthIdFirstTimeSetupSuperAdminGetPathParams,
    ProvidersAuthIdFirstTimeSetupSuperAdminPostPathParams, ProvidersAuthIdGetPathParams,
    ProvidersFeatureDefaultDeletePathParams, ProvidersFeatureDefaultGetPathParams,
    ProvidersFeatureDefaultPutPathParams, ProvidersFeatureGetPathParams,
    ProvidersFeatureIdGetPathParams, PutDefaultProviderRequest, SuperAdmin,
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
    SYS: Systemus,
    D: Deploymento,
    E: Exportius,
    IMP: Importius,
    F: Floxy,
    T: UsbDeviceReader,
    NET: NetworkAdapterReader,
    NetDev: NetDeviceReader,
> Experimental for ServerImpl<APP, AUTH, I, L, Q, M, SYS, D, E, IMP, F, T, NET, NetDev>
{
    async fn instances_instance_id_depends_feature_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdDependsFeatureDeletePathParams,
    ) -> Result<InstancesInstanceIdDependsFeatureDeleteResponse, ()> {
        Ok(api::v2::instances::instance_id::depends::feature::delete(path_params).await)
    }

    async fn instances_instance_id_depends_feature_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdDependsFeatureGetPathParams,
    ) -> Result<InstancesInstanceIdDependsFeatureGetResponse, ()> {
        Ok(api::v2::instances::instance_id::depends::feature::get(path_params).await)
    }

    async fn instances_instance_id_depends_feature_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdDependsFeaturePutPathParams,
        body: ProviderReference,
    ) -> Result<InstancesInstanceIdDependsFeaturePutResponse, ()> {
        Ok(api::v2::instances::instance_id::depends::feature::put(body, path_params).await)
    }

    async fn instances_instance_id_depends_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdDependsGetPathParams,
    ) -> Result<InstancesInstanceIdDependsGetResponse, ()> {
        Ok(api::v2::instances::instance_id::depends::get(path_params).await)
    }

    async fn instances_instance_id_provides_feature_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdProvidesFeatureGetPathParams,
    ) -> Result<InstancesInstanceIdProvidesFeatureGetResponse, ()> {
        Ok(api::v2::instances::instance_id::provides::feature::get(path_params).await)
    }

    async fn instances_instance_id_provides_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdProvidesGetPathParams,
    ) -> Result<InstancesInstanceIdProvidesGetResponse, ()> {
        Ok(api::v2::instances::instance_id::provides::get(path_params).await)
    }

    async fn providers_auth_core_first_time_setup_super_admin_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ProvidersAuthCoreFirstTimeSetupSuperAdminGetResponse, ()> {
        Ok(api::v2::providers::auth::core::first_time_setup::super_admin::get().await)
    }

    async fn providers_auth_core_first_time_setup_super_admin_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: SuperAdmin,
    ) -> Result<ProvidersAuthCoreFirstTimeSetupSuperAdminPostResponse, ()> {
        Ok(api::v2::providers::auth::core::first_time_setup::super_admin::post(body).await)
    }

    async fn providers_auth_core_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ProvidersAuthCoreGetResponse, ()> {
        Ok(api::v2::providers::auth::core::get().await)
    }

    async fn providers_auth_core_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: ProviderReference,
    ) -> Result<ProvidersAuthCorePutResponse, ()> {
        Ok(api::v2::providers::auth::core::put(body).await)
    }

    async fn providers_auth_default_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ProvidersAuthDefaultDeleteResponse, ()> {
        Ok(api::v2::providers::auth::default::delete().await)
    }

    async fn providers_auth_default_first_time_setup_super_admin_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ProvidersAuthDefaultFirstTimeSetupSuperAdminGetResponse, ()> {
        Ok(api::v2::providers::auth::default::first_time_setup::super_admin::get().await)
    }

    async fn providers_auth_default_first_time_setup_super_admin_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: SuperAdmin,
    ) -> Result<ProvidersAuthDefaultFirstTimeSetupSuperAdminPostResponse, ()> {
        Ok(api::v2::providers::auth::default::first_time_setup::super_admin::post(body).await)
    }

    async fn providers_auth_default_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ProvidersAuthDefaultGetResponse, ()> {
        Ok(api::v2::providers::auth::default::get().await)
    }

    async fn providers_auth_default_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: PutDefaultProviderRequest,
    ) -> Result<ProvidersAuthDefaultPutResponse, ()> {
        Ok(api::v2::providers::auth::default::put(body).await)
    }

    async fn providers_auth_first_time_setup_flecsport_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ProvidersAuthFirstTimeSetupFlecsportPostResponse, ()> {
        Ok(api::v2::providers::auth::first_time_setup::flecsport::post().await)
    }

    async fn providers_auth_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ProvidersAuthGetResponse, ()> {
        Ok(api::v2::providers::auth::get().await)
    }

    async fn providers_auth_id_first_time_setup_super_admin_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ProvidersAuthIdFirstTimeSetupSuperAdminGetPathParams,
    ) -> Result<ProvidersAuthIdFirstTimeSetupSuperAdminGetResponse, ()> {
        Ok(api::v2::providers::auth::id::first_time_setup::super_admin::get(path_params).await)
    }

    async fn providers_auth_id_first_time_setup_super_admin_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ProvidersAuthIdFirstTimeSetupSuperAdminPostPathParams,
        body: SuperAdmin,
    ) -> Result<ProvidersAuthIdFirstTimeSetupSuperAdminPostResponse, ()> {
        Ok(
            api::v2::providers::auth::id::first_time_setup::super_admin::post(body, path_params)
                .await,
        )
    }

    async fn providers_auth_id_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ProvidersAuthIdGetPathParams,
    ) -> Result<ProvidersAuthIdGetResponse, ()> {
        Ok(api::v2::providers::auth::id::get(path_params).await)
    }

    async fn providers_feature_default_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ProvidersFeatureDefaultDeletePathParams,
    ) -> Result<ProvidersFeatureDefaultDeleteResponse, ()> {
        Ok(api::v2::providers::feature::default::delete(path_params).await)
    }

    async fn providers_feature_default_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ProvidersFeatureDefaultGetPathParams,
    ) -> Result<ProvidersFeatureDefaultGetResponse, ()> {
        Ok(api::v2::providers::feature::default::get(path_params).await)
    }

    async fn providers_feature_default_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ProvidersFeatureDefaultPutPathParams,
        body: PutDefaultProviderRequest,
    ) -> Result<ProvidersFeatureDefaultPutResponse, ()> {
        Ok(api::v2::providers::feature::default::put(body, path_params).await)
    }

    async fn providers_feature_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ProvidersFeatureGetPathParams,
    ) -> Result<ProvidersFeatureGetResponse, ()> {
        Ok(api::v2::providers::feature::get(path_params).await)
    }

    async fn providers_feature_id_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ProvidersFeatureIdGetPathParams,
    ) -> Result<ProvidersFeatureIdGetResponse, ()> {
        Ok(api::v2::providers::feature::id::get(path_params).await)
    }

    async fn providers_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ProvidersGetResponse, ()> {
        Ok(api::v2::providers::get().await)
    }
}
