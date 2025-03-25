use crate::sorcerer::appraiser::AppRaiser;
use crate::vault::Vault;
use flecsd_axum_server::apis::device::DeviceOnboardingPostResponse as PostResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::Dosschema as PostRequest;
use std::sync::Arc;
use tracing::warn;

pub async fn post<A: AppRaiser + 'static>(
    vault: Arc<Vault>,
    appraiser: Arc<A>,
    request: PostRequest,
) -> Result<PostResponse, ()> {
    if request.apps.is_empty() {
        return Ok(PostResponse::Status400_MalformedRequest(
            models::AdditionalInfo::new(
                "No apps to install given (field 'apps' is empty)".to_string(),
            ),
        ));
    }
    let app_keys = request
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
    match crate::lore::quest::default()
        .await
        .lock()
        .await
        .schedule_quest("Install apps via device onboarding".to_string(), move |quest| async move {
            appraiser.install_apps(quest, vault, app_keys, config).await
        })
        .await
    {
        Ok((id, _)) => Ok(PostResponse::Status202_Accepted(models::JobMeta::new(id.0 as i32))),
        // TODO: Add 500 Response to API
        Err(_) => Err(()),
    }
}
