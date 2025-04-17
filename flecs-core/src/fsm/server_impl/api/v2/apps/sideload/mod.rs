use crate::enchantment::quest_master::QuestMaster;
use crate::fsm::console_client::ConsoleClient;
use crate::jeweler::gem::manifest::AppManifest;
use crate::sorcerer::appraiser::AppRaiser;
use crate::vault::Vault;
use flecsd_axum_server::apis::apps::AppsSideloadPostResponse as PostResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::AppsSideloadPostRequest as PostRequest;
use std::sync::Arc;

pub async fn post<A: AppRaiser + 'static>(
    vault: Arc<Vault>,
    appraiser: Arc<A>,
    quest_master: QuestMaster,
    console_client: ConsoleClient,
    request: PostRequest,
) -> Result<PostResponse, ()> {
    match serde_json::from_str::<flecs_app_manifest::AppManifestVersion>(&request.manifest)
        .map(flecs_app_manifest::AppManifest::try_from)
    {
        Err(e) => Ok(PostResponse::Status400_MalformedRequest(
            models::AdditionalInfo::new(e.to_string()),
        )),
        Ok(Err(e)) => Ok(PostResponse::Status400_MalformedRequest(
            models::AdditionalInfo::new(e.to_string()),
        )),
        Ok(Ok(manifest)) => {
            let manifest = match AppManifest::try_from(manifest) {
                Err(e) => {
                    return Ok(PostResponse::Status400_MalformedRequest(
                        models::AdditionalInfo::new(e.to_string()),
                    ));
                }
                Ok(manifest) => manifest,
            };
            match quest_master
                .lock()
                .await
                .schedule_quest(
                    format!("Sideloading {}", manifest.key()),
                    move |quest| async move {
                        appraiser
                            .install_app_from_manifest(quest, vault, manifest, console_client)
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
