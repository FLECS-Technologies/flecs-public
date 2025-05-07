use crate::jeweler::GetAppKey;
use crate::vault::pouch::AppKey;
use docker_compose_types::{Compose, ComposeVolume, ExternalVolume, MapOrEmpty};
use serde::Serialize;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct AppManifestMulti {
    #[serde(skip_serializing)]
    pub key: AppKey,
    #[serde(skip_serializing)]
    pub compose: Compose,
    #[serde(flatten)]
    original: flecs_app_manifest::AppManifestMulti,
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
            original: value,
        })
    }
}

#[derive(Debug)]
pub struct ServiceWithImage {
    pub name: String,
    pub image: String,
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

    pub fn services_with_image_without_repo(&self) -> Vec<ServiceWithImage> {
        self.compose
            .services
            .0
            .iter()
            .filter_map(|(name, service)| {
                service
                    .as_ref()
                    .and_then(|service| service.image.as_ref())
                    .map(|image| ServiceWithImage {
                        name: name.clone(),
                        image: match image.split_once('/') {
                            Some((_, s)) => s.to_string(),
                            _ => image.to_string(),
                        },
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
}
