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
    Experimental, InstancesInstanceIdDependsDependencyKeyDeleteResponse,
    InstancesInstanceIdDependsDependencyKeyFeaturePutResponse,
    InstancesInstanceIdDependsDependencyKeyGetResponse,
    InstancesInstanceIdDependsDependencyKeyPutResponse, InstancesInstanceIdDependsGetResponse,
    InstancesInstanceIdProvidesFeatureGetResponse, InstancesInstanceIdProvidesGetResponse,
    ProvidersAuthCoreFirstTimeSetupSuperAdminGetResponse,
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
    InstancesInstanceIdDependsDependencyKeyDeletePathParams,
    InstancesInstanceIdDependsDependencyKeyFeaturePutPathParams,
    InstancesInstanceIdDependsDependencyKeyGetPathParams,
    InstancesInstanceIdDependsDependencyKeyPutPathParams, InstancesInstanceIdDependsGetPathParams,
    InstancesInstanceIdProvidesFeatureGetPathParams, InstancesInstanceIdProvidesGetPathParams,
    ProviderReference, ProvidersAuthIdFirstTimeSetupSuperAdminGetPathParams,
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
    F: Floxy + 'static,
    T: UsbDeviceReader + 'static,
    NET: NetworkAdapterReader,
    NetDev: NetDeviceReader,
> Experimental for ServerImpl<APP, AUTH, I, L, Q, M, SYS, D, E, IMP, F, T, NET, NetDev>
{
    async fn instances_instance_id_depends_dependency_key_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdDependsDependencyKeyDeletePathParams,
    ) -> Result<InstancesInstanceIdDependsDependencyKeyDeleteResponse, ()> {
        Ok(
            api::v2::instances::instance_id::depends::dependency_key::delete(
                self.vault.clone(),
                self.sorcerers.providius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_depends_dependency_key_feature_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdDependsDependencyKeyFeaturePutPathParams,
        body: ProviderReference,
    ) -> Result<InstancesInstanceIdDependsDependencyKeyFeaturePutResponse, ()> {
        Ok(
            api::v2::instances::instance_id::depends::dependency_key::feature::put(
                self.vault.clone(),
                self.sorcerers.providius.clone(),
                body,
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_depends_dependency_key_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdDependsDependencyKeyGetPathParams,
    ) -> Result<InstancesInstanceIdDependsDependencyKeyGetResponse, ()> {
        Ok(
            api::v2::instances::instance_id::depends::dependency_key::get(
                self.vault.clone(),
                self.sorcerers.providius.clone(),
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_depends_dependency_key_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdDependsDependencyKeyPutPathParams,
        body: ProviderReference,
    ) -> Result<InstancesInstanceIdDependsDependencyKeyPutResponse, ()> {
        Ok(
            api::v2::instances::instance_id::depends::dependency_key::put(
                self.vault.clone(),
                self.sorcerers.providius.clone(),
                body,
                path_params,
            )
            .await,
        )
    }

    async fn instances_instance_id_depends_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdDependsGetPathParams,
    ) -> Result<InstancesInstanceIdDependsGetResponse, ()> {
        Ok(api::v2::instances::instance_id::depends::get(
            self.vault.clone(),
            self.sorcerers.providius.clone(),
            path_params,
        )
        .await)
    }

    async fn instances_instance_id_provides_feature_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdProvidesFeatureGetPathParams,
    ) -> Result<InstancesInstanceIdProvidesFeatureGetResponse, ()> {
        Ok(api::v2::instances::instance_id::provides::feature::get(
            self.vault.clone(),
            self.sorcerers.providius.clone(),
            path_params,
        )
        .await)
    }

    async fn instances_instance_id_provides_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: InstancesInstanceIdProvidesGetPathParams,
    ) -> Result<InstancesInstanceIdProvidesGetResponse, ()> {
        Ok(api::v2::instances::instance_id::provides::get(
            self.vault.clone(),
            self.sorcerers.providius.clone(),
            path_params,
        )
        .await)
    }

    async fn providers_auth_core_first_time_setup_super_admin_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ProvidersAuthCoreFirstTimeSetupSuperAdminGetResponse, ()> {
        #[cfg(feature = "auth")]
        {
            Ok(api::v2::providers::auth::core::first_time_setup::super_admin::get().await)
        }
        #[cfg(not(feature = "auth"))]
        Err(())
    }

    async fn providers_auth_core_first_time_setup_super_admin_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: SuperAdmin,
    ) -> Result<ProvidersAuthCoreFirstTimeSetupSuperAdminPostResponse, ()> {
        #[cfg(feature = "auth")]
        {
            Ok(api::v2::providers::auth::core::first_time_setup::super_admin::post(body).await)
        }
        #[cfg(not(feature = "auth"))]
        {
            let _ = body;
            Err(())
        }
    }

    async fn providers_auth_core_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ProvidersAuthCoreGetResponse, ()> {
        Ok(api::v2::providers::auth::core::get(
            self.vault.clone(),
            self.sorcerers.providius.clone(),
        )
        .await)
    }

    async fn providers_auth_core_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: ProviderReference,
    ) -> Result<ProvidersAuthCorePutResponse, ()> {
        Ok(api::v2::providers::auth::core::put(
            self.vault.clone(),
            self.sorcerers.providius.clone(),
            body,
        )
        .await)
    }

    async fn providers_auth_default_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ProvidersAuthDefaultDeleteResponse, ()> {
        Ok(api::v2::providers::auth::default::delete(
            self.vault.clone(),
            self.sorcerers.providius.clone(),
        )
        .await)
    }

    async fn providers_auth_default_first_time_setup_super_admin_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ProvidersAuthDefaultFirstTimeSetupSuperAdminGetResponse, ()> {
        #[cfg(feature = "auth")]
        {
            Ok(api::v2::providers::auth::default::first_time_setup::super_admin::get().await)
        }
        #[cfg(not(feature = "auth"))]
        Err(())
    }

    async fn providers_auth_default_first_time_setup_super_admin_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: SuperAdmin,
    ) -> Result<ProvidersAuthDefaultFirstTimeSetupSuperAdminPostResponse, ()> {
        #[cfg(feature = "auth")]
        {
            Ok(api::v2::providers::auth::default::first_time_setup::super_admin::post(body).await)
        }
        #[cfg(not(feature = "auth"))]
        {
            let _ = body;
            Err(())
        }
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
        Ok(api::v2::providers::auth::default::put(
            self.vault.clone(),
            self.sorcerers.providius.clone(),
            body,
        )
        .await)
    }

    async fn providers_auth_first_time_setup_flecsport_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ProvidersAuthFirstTimeSetupFlecsportPostResponse, ()> {
        #[cfg(feature = "auth")]
        {
            Ok(api::v2::providers::auth::first_time_setup::flecsport::post(
                self.vault.clone(),
                self.lore.clone(),
                self.sorcerers.importius.clone(),
                self.enchantments.floxy.clone(),
                self.usb_reader.clone(),
                self.enchantments.quest_master.clone(),
            )
            .await)
        }
        #[cfg(not(feature = "auth"))]
        Err(())
    }

    async fn providers_auth_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ProvidersAuthGetResponse, ()> {
        Ok(
            api::v2::providers::auth::get(self.vault.clone(), self.sorcerers.providius.clone())
                .await,
        )
    }

    async fn providers_auth_id_first_time_setup_super_admin_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ProvidersAuthIdFirstTimeSetupSuperAdminGetPathParams,
    ) -> Result<ProvidersAuthIdFirstTimeSetupSuperAdminGetResponse, ()> {
        #[cfg(feature = "auth")]
        {
            Ok(api::v2::providers::auth::id::first_time_setup::super_admin::get(path_params).await)
        }
        #[cfg(not(feature = "auth"))]
        {
            let _ = path_params;
            Err(())
        }
    }

    async fn providers_auth_id_first_time_setup_super_admin_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ProvidersAuthIdFirstTimeSetupSuperAdminPostPathParams,
        body: SuperAdmin,
    ) -> Result<ProvidersAuthIdFirstTimeSetupSuperAdminPostResponse, ()> {
        #[cfg(feature = "auth")]
        {
            Ok(
                api::v2::providers::auth::id::first_time_setup::super_admin::post(
                    body,
                    path_params,
                )
                .await,
            )
        }
        #[cfg(not(feature = "auth"))]
        {
            let _ = body;
            let _ = path_params;
            Err(())
        }
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
        Ok(api::v2::providers::feature::default::delete(
            self.vault.clone(),
            self.sorcerers.providius.clone(),
            path_params,
        )
        .await)
    }

    async fn providers_feature_default_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ProvidersFeatureDefaultGetPathParams,
    ) -> Result<ProvidersFeatureDefaultGetResponse, ()> {
        Ok(api::v2::providers::feature::default::get(
            self.vault.clone(),
            self.sorcerers.providius.clone(),
            path_params,
        )
        .await)
    }

    async fn providers_feature_default_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ProvidersFeatureDefaultPutPathParams,
        body: PutDefaultProviderRequest,
    ) -> Result<ProvidersFeatureDefaultPutResponse, ()> {
        Ok(api::v2::providers::feature::default::put(
            self.vault.clone(),
            self.sorcerers.providius.clone(),
            body,
            path_params,
        )
        .await)
    }

    async fn providers_feature_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ProvidersFeatureGetPathParams,
    ) -> Result<ProvidersFeatureGetResponse, ()> {
        Ok(api::v2::providers::feature::get(
            self.vault.clone(),
            self.sorcerers.providius.clone(),
            path_params,
        )
        .await)
    }

    async fn providers_feature_id_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: ProvidersFeatureIdGetPathParams,
    ) -> Result<ProvidersFeatureIdGetResponse, ()> {
        Ok(api::v2::providers::feature::id::get(
            self.vault.clone(),
            self.sorcerers.providius.clone(),
            path_params,
        )
        .await)
    }

    async fn providers_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<ProvidersGetResponse, ()> {
        Ok(api::v2::providers::get(self.vault.clone(), self.sorcerers.providius.clone()).await)
    }
}
