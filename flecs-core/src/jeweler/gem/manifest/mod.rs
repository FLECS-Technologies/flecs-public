use crate::vault::pouch::AppKey;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;

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

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub enum Dependency {
    OneOf(HashMap<String, serde_json::Value>),
    One(String, serde_json::Value),
}

#[derive(Debug, Eq, PartialEq, Clone, DeserializeFromStr, Serialize, Hash)]
pub struct DependencyKey(String);

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

    pub fn features(&self) -> Vec<&str> {
        self.0.split('|').collect()
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
                serde_json::Map::from_iter([(feature.clone(), config.clone())])
            }
            Self::OneOf(configs) => serde_json::Map::from_iter(configs.clone()),
        })
    }
}

#[derive(Debug, Clone, Error)]
pub enum ParseDependencyError {
    #[error("")]
    EmptyFeature,
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
        let features: Vec<&str> = key.features();
        match features.len() {
            1 if features[0].is_empty() => Err(Self::Error::EmptyFeature),
            1 => Ok(Self::One(features[0].to_string(), value.clone())),
            len => {
                let serde_json::Value::Object(properties) = value else {
                    return Err(Self::Error::NoProperties);
                };
                if properties.len() != len {
                    return Err(Self::Error::FeaturesNotMatchingProperties);
                };
                let dependencies: Result<HashMap<String, serde_json::Value>, Self::Error> =
                    features
                        .iter()
                        .map(|feature| {
                            Ok::<_, Self::Error>((
                                feature.to_string(),
                                properties.get(*feature).cloned().ok_or_else(|| {
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

    pub fn provides(&self) -> &HashMap<String, serde_json::Value> {
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
}
