use crate::jeweler::gem::manifest::AppManifest;
use crate::sorcerer::appraiser::AppRaiser;
use crate::vault::Vault;
use flecs_app_manifest::AppManifestVersion;
use flecsd_axum_server::apis::apps::AppsSideloadPostResponse as PostResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::AppsSideloadPostRequest as PostRequest;
use std::sync::Arc;

pub async fn post<A: AppRaiser + 'static>(
    vault: Arc<Vault>,
    appraiser: Arc<A>,
    request: PostRequest,
) -> Result<PostResponse, ()> {
    match serde_json::from_str::<AppManifestVersion>(&request.manifest).map(AppManifest::try_from) {
        Err(e) => Ok(PostResponse::Status400_MalformedRequest(
            models::AdditionalInfo::new(e.to_string()),
        )),
        Ok(Err(e)) => Ok(PostResponse::Status400_MalformedRequest(
            models::AdditionalInfo::new(e.to_string()),
        )),
        Ok(Ok(manifest)) => {
            let config = crate::lore::console_client_config::default().await;
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
                Ok((id, _)) => Ok(PostResponse::Status202_Accepted(models::JobMeta::new(
                    id.0 as i32,
                ))),
                // TODO: Add 500 Response to API
                Err(_) => Err(()),
            }
        }
    }
}
