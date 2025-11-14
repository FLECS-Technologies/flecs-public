use crate::jeweler::GetAppKey;
use crate::jeweler::gem::manifest::{Dependency, DependencyKey, FeatureKey, parse_depends};
use crate::vault::pouch::AppKey;
use docker_compose_types::{Compose, ComposeVolume, ExternalVolume, MapOrEmpty};
use serde::Serialize;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct AppManifestMulti {
    #[serde(skip_serializing)]
    pub key: AppKey,
    #[serde(skip_serializing)]
    pub compose: Compose,
    #[serde(skip_serializing)]
    pub provides: HashMap<FeatureKey, serde_json::Value>,
    #[serde(skip_serializing)]
    pub depends: HashMap<DependencyKey, Dependency>,
    #[serde(flatten)]
    original: flecs_app_manifest::AppManifestMulti,
    #[serde(skip_serializing)]
    pub specific_providers: super::providers::Providers,
}

impl GetAppKey for AppManifestMulti {
    fn app_key(&self) -> &AppKey {
        &self.key
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum ComposeValidationError {
    #[error("Invalid app manifest: Network name 'flecs' is reserved")]
    FlecsNetworkReserved,
}

fn validate_compose(compose: &Compose) -> Result<(), ComposeValidationError> {
    for network_name in compose.networks.0.keys() {
        if network_name == "flecs" {
            return Err(ComposeValidationError::FlecsNetworkReserved);
        }
    }
    Ok(())
}

impl TryFrom<flecs_app_manifest::AppManifestMulti> for AppManifestMulti {
    type Error = anyhow::Error;

    fn try_from(value: flecs_app_manifest::AppManifestMulti) -> Result<Self, Self::Error> {
        let json_value = serde_json::Value::Object(value.deployment.compose.yaml.clone());
        let compose = serde_json::from_value(json_value)?;
        validate_compose(&compose)?;
        Ok(Self {
            compose,
            key: AppKey {
                name: value.app.to_string(),
                version: value.version.to_string(),
            },
            provides: value
                .provides
                .as_ref()
                .map(|provides| {
                    provides
                        .0
                        .iter()
                        .map(|(key, value)| (FeatureKey::from(key.clone()), value.clone()))
                        .collect()
                })
                .unwrap_or_default(),
            depends: value
                .depends
                .as_ref()
                .map(parse_depends)
                .transpose()?
                .unwrap_or_default(),
            specific_providers: value
                .provides
                .as_ref()
                .map(|provides| super::providers::Providers::try_from(&provides.0))
                .transpose()?
                .unwrap_or_default(),
            original: value,
        })
    }
}

#[derive(Debug)]
pub struct ServiceWithImageInfo {
    pub name: String,
    pub image: String,
    pub image_with_repo: String,
}

impl AppManifestMulti {
    pub fn revision(&self) -> Option<&String> {
        self.original.revision.as_deref()
    }

    pub fn project_name(&self) -> String {
        self.key.name.replace('.', "-")
    }

    pub fn compose_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.compose)
    }

    pub fn images(&self) -> Vec<String> {
        self.compose
            .services
            .0
            .values()
            .filter_map(|service| service.as_ref().and_then(|service| service.image.clone()))
            .collect()
    }

    pub fn services_with_image_info(&self) -> Vec<ServiceWithImageInfo> {
        self.compose
            .services
            .0
            .iter()
            .filter_map(|(name, service)| {
                service
                    .as_ref()
                    .and_then(|service| service.image.as_ref())
                    .map(|image| ServiceWithImageInfo {
                        name: name.clone(),
                        image: match image.split_once('/') {
                            Some((_, s)) => s.to_string(),
                            _ => image.to_string(),
                        },
                        image_with_repo: image.clone(),
                    })
            })
            .collect()
    }

    pub fn volume_names(&self) -> Vec<String> {
        let project_name = self.project_name();
        self.compose
            .volumes
            .0
            .iter()
            .map(|entry| Self::volume_name_from_map_entry(&project_name, entry))
            .collect()
    }

    pub fn external_volume_names(&self) -> Vec<String> {
        self.compose
            .volumes
            .0
            .iter()
            .filter_map(|(volume_name, entry)| match entry {
                MapOrEmpty::Map(ComposeVolume {
                    external: Some(ExternalVolume::Bool(true)),
                    ..
                }) => Some(volume_name.clone()),
                MapOrEmpty::Map(ComposeVolume {
                    external: Some(ExternalVolume::Name { name }),
                    ..
                }) => Some(name.clone()),
                _ => None,
            })
            .collect()
    }

    fn volume_name_from_map_entry(
        project_name: &String,
        (volume_name, volume): (&String, &MapOrEmpty<ComposeVolume>),
    ) -> String {
        match volume {
            // The name is explicitly given
            MapOrEmpty::Map(ComposeVolume {
                name: Some(name), ..
            }) => name.clone(),
            // The volume is marked as external and the given name will be used
            MapOrEmpty::Map(ComposeVolume {
                external: Some(ExternalVolume::Bool(true)),
                ..
            }) => volume_name.clone(),
            // Old v2/v3 compose "notation": The volume is marked external with a given name
            MapOrEmpty::Map(ComposeVolume {
                external: Some(ExternalVolume::Name { name }),
                ..
            }) => name.clone(),
            // The project name is prefixed in all other cases
            _ => format!("{project_name}_{volume_name}"),
        }
    }

    pub fn inner(&self) -> &flecs_app_manifest::generated::manifest_3_2_0::Multi {
        self.original.deref()
    }

    pub fn provides(&self) -> &HashMap<FeatureKey, serde_json::Value> {
        &self.provides
    }

    pub fn depends(&self) -> &HashMap<DependencyKey, Dependency> {
        &self.depends
    }

    pub fn specific_providers(&self) -> &super::providers::Providers {
        &self.specific_providers
    }
}
