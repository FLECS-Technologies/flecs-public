use crate::jeweler::GetAppKey;
use crate::vault::pouch::AppKey;
use serde::Serialize;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct AppManifestMulti {
    #[serde(skip_serializing)]
    pub key: AppKey,
    #[serde(flatten)]
    original: flecs_app_manifest::AppManifestMulti,
}

impl GetAppKey for AppManifestMulti {
    fn app_key(&self) -> &AppKey {
        &self.key
    }
}

impl TryFrom<flecs_app_manifest::AppManifestMulti> for AppManifestMulti {
    type Error = anyhow::Error;

    fn try_from(value: flecs_app_manifest::AppManifestMulti) -> Result<Self, Self::Error> {
        Ok(Self {
            key: AppKey {
                name: value.app.to_string(),
                version: value.version.to_string(),
            },
            original: value,
        })
    }
}

impl AppManifestMulti {
    pub fn revision(&self) -> Option<&String> {
        self.original.revision.as_deref()
    }
}
