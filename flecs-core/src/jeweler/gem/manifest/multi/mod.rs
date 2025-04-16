use crate::jeweler::GetAppKey;
use crate::vault::pouch::AppKey;
use docker_compose_types::Compose;
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
}
