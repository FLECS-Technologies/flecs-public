use crate::enchantment::quest_master::QuestMaster;
use crate::fsm::console_client::ConsoleClient;
use crate::lore::Lore;
use crate::sorcerer::appraiser::AppRaiser;
use crate::vault::Vault;
use flecsd_axum_server::apis::device::DeviceOnboardingPostResponse as PostResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::DosManifest as PostRequest;
use std::sync::Arc;
use tracing::warn;

pub async fn post<A: AppRaiser + 'static>(
    vault: Arc<Vault>,
    appraiser: Arc<A>,
    lore: Arc<Lore>,
    quest_master: QuestMaster,
    console_client: ConsoleClient,
    request: PostRequest,
) -> PostResponse {
    if request.apps.is_empty() {
        return PostResponse::Status400_MalformedRequest(models::AdditionalInfo::new(
            "No apps to install given (field 'apps' is empty)".to_string(),
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
    match quest_master
        .lock()
        .await
        .schedule_quest(
            "Install apps via device onboarding".to_string(),
            move |quest| async move {
                appraiser
                    .install_apps(quest, vault, lore, app_keys, console_client)
                    .await
            },
        )
        .await
    {
        Ok((id, _)) => PostResponse::Status202_Accepted(models::JobMeta::new(id.0 as i32)),
        Err(e) => {
            PostResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}
