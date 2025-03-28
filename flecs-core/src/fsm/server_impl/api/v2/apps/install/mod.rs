use crate::enchantment::quest_master::QuestMaster;
use crate::fsm::console_client::ConsoleClient;
use crate::sorcerer::appraiser::AppRaiser;
use crate::vault::Vault;
use flecsd_axum_server::apis::apps::AppsInstallPostResponse as PostResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::AppsInstallPostRequest as PostRequest;
use std::sync::Arc;

pub async fn post<A: AppRaiser + 'static>(
    vault: Arc<Vault>,
    appraiser: Arc<A>,
    console_client: ConsoleClient,
    quest_master: QuestMaster,
    request: PostRequest,
) -> PostResponse {
    let app_key = request.app_key.into();
    match quest_master
        .lock()
        .await
        .schedule_quest(format!("Install {}", app_key), move |quest| async move {
            appraiser
                .install_app(quest, vault, app_key, console_client)
                .await
        })
        .await
    {
        Ok((id, _)) => PostResponse::Status202_Accepted(models::JobMeta::new(id.0 as i32)),
        Err(e) => {
            PostResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}
