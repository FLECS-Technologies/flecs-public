use crate::vault::pouch::AppKey;
use serde::Serialize;
use std::sync::Arc;

pub mod multi;
pub mod single;

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(untagged)]
pub enum AppManifest {
    Single(Arc<single::AppManifestSingle>),
    Multi(Arc<multi::AppManifestMulti>),
}

impl TryFrom<flecs_app_manifest::AppManifest> for AppManifest {
    type Error = anyhow::Error;

    fn try_from(value: flecs_app_manifest::AppManifest) -> Result<Self, Self::Error> {
        match value {
            flecs_app_manifest::AppManifest::Single(single) => Ok(Self::Single(Arc::new(
                single::AppManifestSingle::try_from(single)?,
            ))),
            flecs_app_manifest::AppManifest::Multi(multi) => Ok(Self::Multi(Arc::new(
                multi::AppManifestMulti::try_from(multi)?,
            ))),
        }
    }
}

impl AppManifest {
    pub fn key(&self) -> &AppKey {
        match self {
            Self::Multi(multi) => &multi.key,
            Self::Single(single) => &single.key,
        }
    }

    pub fn revision(&self) -> Option<&String> {
        match self {
            AppManifest::Single(single) => single.revision(),
            AppManifest::Multi(multi) => multi.revision(),
        }
    }
}
