pub mod app_name;

use crate::fsm::server_impl::api::v2::manifests::app_name::version::try_model_from_manifest;
use crate::sorcerer::manifesto::Manifesto;
use crate::vault::Vault;
use crate::vault::pouch::AppKey;
use flecsd_axum_server::apis::manifests::ManifestsGetResponse as GetResponse;
use std::ops::Deref;
use std::sync::Arc;
use tracing::error;

pub async fn get<M: Manifesto>(vault: Arc<Vault>, manifesto: Arc<M>) -> GetResponse {
    let manifests = manifesto.get_manifests(&vault).await;
    GetResponse::Status200_Success(
        manifests
            .into_iter()
            .filter_map(|manifest| {
                let app_key = app_key_from_manifest(&manifest);
                match try_model_from_manifest(manifest) {
                    Err(e) => {
                        error!("Could not convert manifest {app_key} to model: {e}",);
                        None
                    }
                    Ok(manifest) => Some(manifest),
                }
            })
            .collect(),
    )
}

fn app_key_from_manifest(
    manifest: &flecs_app_manifest::generated::manifest_3_2_0::FlecsAppManifest,
) -> AppKey {
    match manifest {
        flecs_app_manifest::generated::manifest_3_2_0::FlecsAppManifest::Single(single) => AppKey {
            name: single.app.deref().clone(),
            version: single.version.deref().clone(),
        },
        flecs_app_manifest::generated::manifest_3_2_0::FlecsAppManifest::Multi(multi) => AppKey {
            name: multi.app.deref().clone(),
            version: multi.version.deref().clone(),
        },
    }
}
