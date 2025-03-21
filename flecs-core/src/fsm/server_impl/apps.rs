use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::fsm::server_impl::ServerImpl;
use crate::jeweler::gem::manifest::AppManifest;
use crate::relic::device::net::NetDeviceReader;
use crate::relic::device::usb::UsbDeviceReader;
use crate::relic::network::NetworkAdapterReader;
use crate::sorcerer::appraiser::AppRaiser;
use crate::sorcerer::authmancer::Authmancer;
use crate::sorcerer::instancius::Instancius;
use crate::sorcerer::licenso::Licenso;
use crate::sorcerer::mage_quester::MageQuester;
use crate::sorcerer::manifesto::Manifesto;
use crate::sorcerer::systemus::Systemus;
use crate::vault::pouch::{AppKey, Pouch};
use async_trait::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecs_app_manifest::AppManifestVersion;
use flecsd_axum_server::apis::apps::{
    Apps, AppsAppDeleteResponse, AppsAppGetResponse, AppsGetResponse, AppsInstallPostResponse,
    AppsSideloadPostResponse,
};
use flecsd_axum_server::models::{
    AdditionalInfo, AppsAppDeletePathParams, AppsAppDeleteQueryParams, AppsAppGetPathParams,
    AppsAppGetQueryParams, AppsInstallPostRequest, AppsSideloadPostRequest, JobMeta,
};
use http::Method;
use std::sync::Arc;

#[async_trait]
impl<
        APP: AppRaiser + 'static,
        AUTH: Authmancer,
        I: Instancius,
        L: Licenso,
        Q: MageQuester,
        M: Manifesto,
        SYS: Systemus,
        F: Floxy + 'static,
        T: UsbDeviceReader,
        NET: NetworkAdapterReader,
        NetDev: NetDeviceReader,
    > Apps for ServerImpl<APP, AUTH, I, L, Q, M, SYS, F, T, NET, NetDev>
{
    async fn apps_app_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: AppsAppDeletePathParams,
        query_params: AppsAppDeleteQueryParams,
    ) -> Result<AppsAppDeleteResponse, ()> {
        match query_params.version {
            Some(app_version) => {
                let key = AppKey {
                    name: path_params.app,
                    version: app_version,
                };
                if !self
                    .vault
                    .reservation()
                    .reserve_app_pouch()
                    .grab()
                    .await
                    .app_pouch
                    .as_ref()
                    .expect("Vault reservations should never fail")
                    .gems()
                    .contains_key(&key)
                {
                    return Ok(AppsAppDeleteResponse::Status404_NoSuchAppOrApp);
                }
                let vault = self.vault.clone();
                let floxy = FloxyOperation::new_arc(self.enchantments.floxy.clone());
                let appraiser = self.sorcerers.app_raiser.clone();
                let (id, _) = crate::lore::quest::default()
                    .await
                    .lock()
                    .await
                    .schedule_quest(format!("Uninstall {key}"), move |quest| async move {
                        appraiser.uninstall_app(quest, vault, floxy, key).await
                    })
                    .await
                    // TODO: Add 500 Response to API
                    .map_err(|_| ())?;
                Ok(AppsAppDeleteResponse::Status202_Accepted(JobMeta {
                    job_id: id.0 as i32,
                }))
            }
            // TODO: Add 400 Response to API
            None => Err(()),
        }
    }

    async fn apps_app_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: AppsAppGetPathParams,
        query_params: AppsAppGetQueryParams,
    ) -> Result<AppsAppGetResponse, ()> {
        let apps = self
            .sorcerers
            .app_raiser
            .get_app(self.vault.clone(), path_params.app, query_params.version)
            .await
            // TODO: Add 500 Response to API
            .map_err(|_| ())?;
        if apps.is_empty() {
            Ok(AppsAppGetResponse::Status404_NoSuchAppOrApp)
        } else {
            Ok(AppsAppGetResponse::Status200_Success(apps))
        }
    }

    async fn apps_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<AppsGetResponse, ()> {
        let apps = self
            .sorcerers
            .app_raiser
            .get_apps(self.vault.clone())
            .await
            // TODO: Add 500 Response to API
            .map_err(|_| ())?;
        Ok(AppsGetResponse::Status200_Success(apps))
    }

    async fn apps_install_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: AppsInstallPostRequest,
    ) -> Result<AppsInstallPostResponse, ()> {
        let app_key = body.app_key.into();
        let config = crate::lore::console_client_config::default().await;
        let vault = self.vault.clone();
        let appraiser = self.sorcerers.app_raiser.clone();
        match crate::lore::quest::default()
            .await
            .lock()
            .await
            .schedule_quest(format!("Install {}", app_key), move |quest| async move {
                appraiser.install_app(quest, vault, app_key, config).await
            })
            .await
        {
            Ok((id, _)) => Ok(AppsInstallPostResponse::Status202_Accepted(JobMeta {
                job_id: id.0 as i32,
            })),
            Err(e) => Ok(AppsInstallPostResponse::Status500_InternalServerError(
                AdditionalInfo {
                    additional_info: e.to_string(),
                },
            )),
        }
    }

    async fn apps_sideload_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: AppsSideloadPostRequest,
    ) -> Result<AppsSideloadPostResponse, ()> {
        match serde_json::from_str::<AppManifestVersion>(&body.manifest).map(AppManifest::try_from)
        {
            Err(e) => Ok(AppsSideloadPostResponse::Status400_MalformedRequest(
                AdditionalInfo {
                    additional_info: e.to_string(),
                },
            )),
            Ok(Err(e)) => Ok(AppsSideloadPostResponse::Status400_MalformedRequest(
                AdditionalInfo {
                    additional_info: e.to_string(),
                },
            )),
            Ok(Ok(manifest)) => {
                let config = crate::lore::console_client_config::default().await;
                let vault = self.vault.clone();
                let appraiser = self.sorcerers.app_raiser.clone();
                match crate::lore::quest::default()
                    .await
                    .lock()
                    .await
                    .schedule_quest(
                        format!("Sideloading {}", manifest.key),
                        move |quest| async move {
                            appraiser
                                .install_app_from_manifest(quest, vault, Arc::new(manifest), config)
                                .await
                        },
                    )
                    .await
                {
                    Ok((id, _)) => Ok(AppsSideloadPostResponse::Status202_Accepted(JobMeta {
                        job_id: id.0 as i32,
                    })),
                    // TODO: Add 500 Response to API
                    Err(_) => Err(()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fsm::server_impl::ServerImpl;
    use crate::relic::device::net::MockNetDeviceReader;
    use crate::relic::device::usb::MockUsbDeviceReader;
    use crate::relic::network::MockNetworkAdapterReader;
    use crate::sorcerer::MockSorcerers;
    use crate::vault::tests::create_empty_test_vault;
    use axum::extract::Host;
    use axum_extra::extract::CookieJar;
    use flecsd_axum_server::apis::apps::{Apps, AppsAppDeleteResponse};
    use flecsd_axum_server::models::{AppsAppDeletePathParams, AppsAppDeleteQueryParams};
    use http::Method;

    #[tokio::test]
    async fn uninstall_no_version() {
        let server = ServerImpl::test_instance(
            create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockNetworkAdapterReader::default(),
            MockNetDeviceReader::default(),
            MockSorcerers::default(),
        );
        assert!(server
            .apps_app_delete(
                Method::default(),
                Host("host".to_string()),
                CookieJar::default(),
                AppsAppDeletePathParams {
                    app: "app".to_string(),
                },
                AppsAppDeleteQueryParams { version: None },
            )
            .await
            .is_err())
    }

    #[tokio::test]
    async fn uninstall_404() {
        let server = ServerImpl::test_instance(
            create_empty_test_vault(),
            MockUsbDeviceReader::new(),
            MockNetworkAdapterReader::default(),
            MockNetDeviceReader::default(),
            MockSorcerers::default(),
        );
        assert_eq!(
            Ok(AppsAppDeleteResponse::Status404_NoSuchAppOrApp),
            server
                .apps_app_delete(
                    Method::default(),
                    Host("host".to_string()),
                    CookieJar::default(),
                    AppsAppDeletePathParams {
                        app: "app".to_string(),
                    },
                    AppsAppDeleteQueryParams {
                        version: Some("version".to_string())
                    },
                )
                .await
        )
    }
}
