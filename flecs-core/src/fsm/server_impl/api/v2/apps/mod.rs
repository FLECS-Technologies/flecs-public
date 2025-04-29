pub mod app;
pub mod install;
pub mod sideload;

use crate::sorcerer::appraiser::AppRaiser;
use crate::vault::Vault;
use flecsd_axum_server::apis::apps::AppsGetResponse as GetResponse;
use flecsd_axum_server::models;
use std::sync::Arc;

pub async fn get<A: AppRaiser>(vault: Arc<Vault>, appraiser: Arc<A>) -> GetResponse {
    match appraiser.get_apps(vault).await {
        Ok(apps) => GetResponse::Status200_Success(apps),
        Err(e) => {
            GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}
