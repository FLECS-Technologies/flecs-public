pub mod app;
pub mod install;
pub mod sideload;

use crate::sorcerer::appraiser::AppRaiser;
use crate::vault::Vault;
use flecsd_axum_server::apis::apps::AppsGetResponse as GetResponse;
use std::sync::Arc;

pub async fn get<A: AppRaiser>(vault: Arc<Vault>, appraiser: Arc<A>) -> Result<GetResponse, ()> {
    let apps = appraiser
        .get_apps(vault)
        .await
        // TODO: Add 500 Response to API
        .map_err(|_| ())?;
    Ok(GetResponse::Status200_Success(apps))
}
