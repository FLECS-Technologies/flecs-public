use crate::fsm::server_impl::{
    additional_info_from_error, console_session_id_to_core_session_id, ok, ServerImpl,
};
use async_trait::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecsd_axum_server::apis::device::{
    Device, DeviceLicenseActivationPostResponse, DeviceLicenseActivationStatusGetResponse,
    DeviceLicenseInfoGetResponse, DeviceOnboardingPostResponse,
};
use flecsd_axum_server::models::{
    AdditionalInfo, DeviceLicenseActivationStatusGet200Response, DeviceLicenseInfoGet200Response,
    Dosschema, JobMeta,
};
use http::Method;
use tracing::warn;

#[async_trait]
impl Device for ServerImpl {
    async fn device_license_activation_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<DeviceLicenseActivationPostResponse, ()> {
        match crate::sorcerer::licenso::activate_license(
            &self.vault,
            crate::lore::console_client_config::default().await,
        )
        .await
        {
            Ok(()) => Ok(DeviceLicenseActivationPostResponse::Status200_Success(ok())),
            Err(e) => Ok(
                DeviceLicenseActivationPostResponse::Status500_InternalServerError(
                    additional_info_from_error(e),
                ),
            ),
        }
    }

    async fn device_license_activation_status_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<DeviceLicenseActivationStatusGetResponse, ()> {
        match crate::sorcerer::licenso::validate_license(
            &self.vault,
            crate::lore::console_client_config::default().await,
        )
        .await
        {
            Ok(is_valid) => Ok(DeviceLicenseActivationStatusGetResponse::Status200_Success(
                DeviceLicenseActivationStatusGet200Response { is_valid },
            )),
            Err(e) => Ok(
                DeviceLicenseActivationStatusGetResponse::Status500_InternalServerError({
                    additional_info_from_error(e)
                }),
            ),
        }
    }

    async fn device_license_info_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
    ) -> Result<DeviceLicenseInfoGetResponse, ()> {
        let secrets = self.vault.get_secrets().await;
        Ok(DeviceLicenseInfoGetResponse::Status200_Success(
            DeviceLicenseInfoGet200Response {
                // TODO: Use correct type, as soon as serial numbers are implemented
                r#type: "Via user license".to_string(),
                session_id: Some(console_session_id_to_core_session_id(
                    secrets.get_session_id(),
                )),
                license: secrets.license_key,
            },
        ))
    }

    async fn device_onboarding_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: Dosschema,
    ) -> Result<DeviceOnboardingPostResponse, ()> {
        if body.apps.is_empty() {
            return Ok(DeviceOnboardingPostResponse::Status400_MalformedRequest(
                AdditionalInfo {
                    additional_info: "No apps to install given (field 'apps' is empty)".to_string(),
                },
            ));
        }
        let app_keys = body
            .apps
            .into_iter()
            .filter_map(|app| {
                if let Some(version) = app.version {
                    Some(crate::vault::pouch::AppKey {
                        name: app.name,
                        version,
                    })
                } else {
                    warn!(
                        "Skip installing newest version of app {}, not implemented yet",
                        app.name
                    );
                    None
                }
            })
            .collect();
        let config = crate::lore::console_client_config::default().await;
        let vault = self.vault.clone();
        match crate::lore::quest::default()
            .await
            .lock()
            .await
            .schedule_quest("Install apps via device onboarding".to_string(), |quest| {
                crate::sorcerer::appraiser::install_apps(quest, vault, app_keys, config)
            })
            .await
        {
            Ok((id, _)) => Ok(DeviceOnboardingPostResponse::Status202_Accepted(JobMeta {
                job_id: id.0 as i32,
            })),
            // TODO: Add 500 Response to API
            Err(_) => Err(()),
        }
    }
}
