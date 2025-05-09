use crate::sorcerer::manifesto::Manifesto;
use crate::vault::Vault;
use crate::vault::pouch::AppKey;
use flecs_app_manifest::generated::manifest_3_1_0::FlecsAppManifest;
use flecsd_axum_server::apis::manifests::ManifestsAppNameVersionGetResponse as GetResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::ManifestsAppNameVersionGetPathParams as GetPathParams;
use serde::Serialize;
use std::str::FromStr;
use std::sync::Arc;

pub async fn get<M: Manifesto>(
    vault: Arc<Vault>,
    manifesto: Arc<M>,
    path_params: GetPathParams,
) -> GetResponse {
    let app_key = AppKey {
        name: path_params.app_name,
        version: path_params.version,
    };
    match manifesto.get_manifest(&vault, &app_key).await {
        Some(manifest) => match try_model_from_manifest(manifest) {
            Ok(manifest) => GetResponse::Status200_Success(manifest),
            Err(e) => GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(
                e.to_string(),
            )),
        },
        None => GetResponse::Status404_ManifestNotFound,
    }
}

#[derive(Debug, Serialize)]
struct TaggedManifest {
    #[serde(flatten)]
    manifest: FlecsAppManifest,
    #[serde(rename = "_schemaVersion")]
    schema_version: &'static str,
}

impl From<FlecsAppManifest> for TaggedManifest {
    fn from(manifest: FlecsAppManifest) -> Self {
        Self {
            manifest,
            schema_version: "3.1.0",
        }
    }
}

pub fn try_model_from_manifest(manifest: FlecsAppManifest) -> crate::Result<models::AppManifest> {
    let manifest = TaggedManifest::from(manifest);
    let manifest = serde_json::to_string_pretty(&manifest)?;
    Ok(models::AppManifest::from_str(&manifest)?)
}
