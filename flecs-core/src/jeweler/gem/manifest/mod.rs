use crate::vault::pouch::AppKey;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use std::string::ToString;
use std::sync::Arc;
use thiserror::Error;
use utoipa::openapi::schema::SchemaType;
use utoipa::openapi::{RefOr, Schema, Type};
use utoipa::{PartialSchema, ToSchema};

pub mod multi;
pub mod providers;
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

impl From<AppManifest> for flecs_app_manifest::generated::manifest_3_2_0::FlecsAppManifest {
    fn from(value: AppManifest) -> Self {
        match value {
            AppManifest::Single(single) => {
                flecs_app_manifest::generated::manifest_3_2_0::FlecsAppManifest::Single(
                    single.inner().clone(),
                )
            }
            AppManifest::Multi(multi) => {
                flecs_app_manifest::generated::manifest_3_2_0::FlecsAppManifest::Multi(
                    multi.inner().clone(),
                )
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FeatureKey(String);

impl PartialSchema for FeatureKey {
    fn schema() -> RefOr<Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .schema_type(SchemaType::Type(Type::String))
            .min_length(Some(1))
            .pattern(Some(r"^([\w.-]+)$"))
            .into()
    }
}

impl ToSchema for FeatureKey {}

impl AsRef<str> for FeatureKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for FeatureKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = Regex::new(r"^([\w.-]+)$").unwrap();
        if regex.is_match(s) {
            Ok(Self(s.to_string()))
        } else {
            Err(())
        }
    }
}

impl Display for FeatureKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl FeatureKey {
    pub fn auth() -> Self {
        Self("auth".to_string())
    }
}

impl From<flecs_app_manifest::generated::manifest_3_2_0::ProvidesKey> for FeatureKey {
    fn from(value: flecs_app_manifest::generated::manifest_3_2_0::ProvidesKey) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub enum Dependency {
    OneOf(HashMap<FeatureKey, serde_json::Value>),
    One(FeatureKey, serde_json::Value),
}

#[derive(Debug, Eq, PartialEq, Clone, DeserializeFromStr, Serialize, Hash)]
pub struct DependencyKey(String);

impl PartialSchema for DependencyKey {
    fn schema() -> RefOr<Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .schema_type(SchemaType::Type(Type::String))
            .min_length(Some(1))
            .pattern(Some("^([\\w.-]+)(?:\\s*\\|\\s*([\\w.-]+))*$"))
            .into()
    }
}
impl ToSchema for DependencyKey {}
impl Display for DependencyKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl FromStr for DependencyKey {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl DependencyKey {
    pub fn new(s: &str) -> Self {
        let value: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        let mut value: Vec<_> = value.split('|').collect();
        value.sort();
        Self(value.join("|"))
    }

    pub fn features(&self) -> Vec<FeatureKey> {
        self.0
            .split('|')
            .map(|f| FeatureKey(f.to_string()))
            .collect()
    }
}

impl AsRef<String> for DependencyKey {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl Dependency {
    pub fn config_json(&self) -> serde_json::Value {
        serde_json::Value::Object(match self {
            Self::One(feature, config) => {
                serde_json::Map::from_iter([(feature.to_string(), config.clone())])
            }
            Self::OneOf(configs) => serde_json::Map::from_iter(
                configs
                    .iter()
                    .map(|(feature, value)| (feature.to_string(), value.clone())),
            ),
        })
    }
}

#[derive(Debug, Clone, Error)]
pub enum ParseDependencyError {
    #[error("")]
    NoProperties,
    #[error("")]
    FeaturesNotMatchingProperties,
    #[error("")]
    NoMatchingProperty(String),
}

impl TryFrom<(&DependencyKey, &serde_json::Value)> for Dependency {
    type Error = ParseDependencyError;

    fn try_from((key, value): (&DependencyKey, &serde_json::Value)) -> Result<Self, Self::Error> {
        let features = key.features();
        match features.len() {
            1 => Ok(Self::One(features[0].clone(), value.clone())),
            len => {
                let serde_json::Value::Object(properties) = value else {
                    return Err(Self::Error::NoProperties);
                };
                if properties.len() != len {
                    return Err(Self::Error::FeaturesNotMatchingProperties);
                };
                let dependencies: Result<HashMap<FeatureKey, serde_json::Value>, Self::Error> =
                    features
                        .iter()
                        .map(|feature| {
                            Ok::<_, Self::Error>((
                                feature.clone(),
                                properties.get(feature.as_ref()).cloned().ok_or_else(|| {
                                    Self::Error::NoMatchingProperty(feature.to_string())
                                })?,
                            ))
                        })
                        .collect();
                Ok(Self::OneOf(dependencies?))
            }
        }
    }
}

fn parse_depends(
    flecs_app_manifest::generated::manifest_3_2_0::Depends(depends): &flecs_app_manifest::generated::manifest_3_2_0::Depends,
) -> Result<HashMap<DependencyKey, Dependency>, ParseDependencyError> {
    Ok(depends
        .iter()
        .map(|(key, value)| {
            let key = DependencyKey::new(key.deref());
            let dependency = Dependency::try_from((&key, value)).expect("TODO");
            (key, dependency)
        })
        .collect())
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

    pub fn provides(&self) -> &HashMap<FeatureKey, serde_json::Value> {
        match self {
            AppManifest::Single(single) => single.provides(),
            AppManifest::Multi(multi) => multi.provides(),
        }
    }

    pub fn depends(&self) -> &HashMap<DependencyKey, Dependency> {
        match self {
            AppManifest::Single(single) => single.depends(),
            AppManifest::Multi(multi) => multi.depends(),
        }
    }

    pub fn depend(&self, key: &DependencyKey) -> Option<&Dependency> {
        match self {
            AppManifest::Single(single) => single.depends(),
            AppManifest::Multi(multi) => multi.depends(),
        }
        .get(key)
    }

    pub fn specific_providers(&self) -> &providers::Providers {
        match self {
            AppManifest::Single(single) => single.specific_providers(),
            AppManifest::Multi(multi) => multi.specific_providers(),
        }
    }
}
