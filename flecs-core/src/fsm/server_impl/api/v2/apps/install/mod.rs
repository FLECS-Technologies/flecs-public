use crate::sorcerer::appraiser::AppRaiser;
use crate::vault::Vault;
use flecsd_axum_server::apis::apps::AppsInstallPostResponse as PostResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::AppsInstallPostRequest as PostRequest;
use std::sync::Arc;

pub async fn post<A: AppRaiser + 'static>(
    vault: Arc<Vault>,
    appraiser: Arc<A>,
    request: PostRequest,
) -> PostResponse {
    let app_key = request.app_key.into();
    let config = crate::lore::console_client_config::default().await;
    match crate::lore::quest::default()
        .await
        .lock()
        .await
        .schedule_quest(format!("Install {}", app_key), move |quest| async move {
            appraiser.install_app(quest, vault, app_key, config).await
        })
        .await
    {
        Ok((id, _)) => PostResponse::Status202_Accepted(models::JobMeta::new(id.0 as i32)),
        Err(e) => {
            PostResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}
