use crate::jeweler::gem;
#[cfg(feature = "auth")]
use crate::jeweler::gem::instance::Instance;
use crate::jeweler::gem::instance::status::InstanceStatus;
use crate::jeweler::gem::instance::{InstanceId, ProviderReference, StoredProviderReference};
#[cfg(feature = "auth")]
use crate::jeweler::gem::manifest::providers::auth::AuthProvider;
use crate::jeweler::gem::manifest::{DependencyKey, FeatureKey};
use crate::quest::{State, SyncQuest};
use crate::sorcerer::instancius::QueryInstanceConfigError;
use crate::sorcerer::providius::{Dependency, Provider};
use crate::vault::pouch::provider::{CoreProviders, ProviderId};
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::{GrabbedPouches, Vault, pouch};
use anyhow::Context;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeleteDefaultProviderError {
    #[error("The default provider is still in use ({0:?})")]
    ProviderInUse(Vec<(InstanceId, AppKey)>),
    #[error("Failed to check dependents: {0}")]
    FailedToCheckDependents(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum SetDefaultProviderError {
    #[error("Provider with id {0} does not exist")]
    ProviderNotFound(ProviderId),
    #[error("Provider with id {id} does not provide {feature}")]
    ProviderDoesNotProvide { id: ProviderId, feature: FeatureKey },
}

pub type GetProviderError = SetDefaultProviderError;

#[derive(Error, Debug)]
pub enum GetProvidesError {
    #[error("Instance with id {0} does not exist")]
    InstanceNotFound(ProviderId),
}

#[derive(Error, Debug)]
pub enum GetDependenciesError {
    #[error("Instance with id {0} does not exist")]
    InstanceNotFound(ProviderId),
}

#[derive(Error, Debug)]
pub enum GetDependencyError {
    #[error("Instance with id {0} does not exist")]
    InstanceNotFound(ProviderId),
    #[error("Instance with id {instance_id} does not depend on feature {feature}")]
    DoesNotDepend {
        instance_id: InstanceId,
        feature: String,
    },
}

#[derive(Error, Debug)]
pub enum ClearDependencyError {
    #[error("Instance with id {0} does not exist")]
    InstanceNotFound(ProviderId),
    #[error("Instance with id {instance_id} does not depend on feature {feature}")]
    DoesNotDepend {
        instance_id: InstanceId,
        feature: String,
    },
    #[error("Instance with id {instance_id} is running, dependency can not be cleared")]
    InstanceRunning { instance_id: InstanceId },
    #[error("Failed to check instance status: {0}")]
    FailedToCheckStatus(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum SetDependencyError {
    #[error("Instance with id {0} does not exist")]
    InstanceNotFound(InstanceId),
    #[error("Instance with id {instance_id} does not depend on {key}")]
    DoesNotDepend {
        instance_id: InstanceId,
        key: String,
    },
    #[error("Instance with id {instance_id} is running, dependency can not be set")]
    InstanceRunning { instance_id: InstanceId },
    #[error("Failed to check instance status: {0}")]
    FailedToCheckStatus(#[from] anyhow::Error),
    #[error("No default provider for feature {feature} present")]
    NoDefaultProvider { feature: FeatureKey },
    #[error("Provider with id {0} does not exist")]
    ProviderDoesNotExist(ProviderId),
    #[error("Provider with id {provider_id} does not provide feature {feature}")]
    ProviderDoesNotProvideFeature {
        feature: FeatureKey,
        provider_id: ProviderId,
    },
    #[error(
        "Provider with id {provider_id} provides feature {feature}, but config does not match: {error}"
    )]
    FeatureConfigNotMatching {
        feature: FeatureKey,
        provider_id: ProviderId,
        error: anyhow::Error,
    },
    #[error("Provided dependency key {key} does not contain feature {feature}")]
    KeyDoesNotContainFeature {
        feature: FeatureKey,
        key: DependencyKey,
    },
}

#[derive(Error, Debug)]
pub enum GetFeatureProvidesError {
    #[error("Instance with id {0} does not exist")]
    InstanceNotFound(ProviderId),
    #[error("Instance with id {id} does not provide feature {feature}")]
    DoesNotProvide { id: ProviderId, feature: String },
}

#[derive(Error, Debug)]
pub enum PutCoreAuthProviderError {
    #[error("Instance with id {0} does not exist")]
    InstanceNotFound(ProviderId),
    #[error("Can not use default auth provider as it is not set")]
    DefaultProviderNotSet,
    #[error("Instance with id {id} does not provide feature auth")]
    DoesNotProvide { id: ProviderId },
}

#[derive(Error, Debug)]
pub enum GetAuthProviderPortError {
    #[error("Core auth provider not set")]
    CoreProviderNotSet,
    #[error("Auth provider with id {0} does not exist")]
    ProviderNotFound(ProviderId),
    #[error("Can not use default auth provider as it is not set")]
    DefaultProviderNotSet,
    #[error("Instance with id {id} does not provide feature auth")]
    DoesNotProvide { id: ProviderId },
    #[error(transparent)]
    QueryConfig(#[from] QueryInstanceConfigError),
}

pub async fn delete_default_provider(
    feature: &FeatureKey,
    providers: &mut pouch::provider::Gems,
    instances: &pouch::instance::Gems,
) -> Result<Option<ProviderId>, DeleteDefaultProviderError> {
    if !providers.default_providers.contains_key(feature) {
        return Ok(None);
    };
    let mut depending_running_instances = Vec::new();
    for (id, instance) in instances.iter().filter(|(_, instance)| {
        instance.dependencies().iter().any(|(key, id)| {
            id.provided_feature == *feature && id.is_default() && key.features().contains(feature)
        })
    }) {
        if instance.status().await? == InstanceStatus::Running {
            depending_running_instances.push((*id, instance.app_key().clone()));
        }
    }
    if !depending_running_instances.is_empty() {
        return Err(DeleteDefaultProviderError::ProviderInUse(
            depending_running_instances,
        ));
    }
    Ok(providers.default_providers.remove(feature))
}

pub fn set_default_provider(
    feature: FeatureKey,
    providers: &mut pouch::provider::Gems,
    instances: &pouch::instance::Gems,
    id: ProviderId,
) -> Result<Option<ProviderId>, SetDefaultProviderError> {
    match instances.get(&id) {
        None => return Err(SetDefaultProviderError::ProviderNotFound(id)),
        Some(instance) if !instance.manifest().provides().contains_key(&feature) => {
            return Err(SetDefaultProviderError::ProviderDoesNotProvide { id, feature });
        }
        _ => {}
    }
    Ok(providers.default_providers.insert(feature, id))
}

pub fn get_provider(
    instances: &pouch::instance::Gems,
    feature: &FeatureKey,
    id: ProviderId,
) -> Result<Provider, GetProviderError> {
    match instances.get(&id) {
        None => Err(SetDefaultProviderError::ProviderNotFound(id)),
        Some(instance) => match instance.manifest().provides().get(feature) {
            Some(provider) => Ok(Provider {
                app_key: instance.app_key().clone(),
                config: provider.clone(),
            }),
            None => Err(SetDefaultProviderError::ProviderDoesNotProvide {
                id,
                feature: feature.clone(),
            }),
        },
    }
}

#[cfg(feature = "auth")]
pub fn get_auth_providers(
    instances: &pouch::instance::Gems,
) -> HashMap<ProviderId, (AuthProvider, u16)> {
    instances
        .iter()
        .filter_map(|(id, instance)| {
            let auth_provider = instance
                .manifest()
                .specific_providers()
                .auth
                .as_ref()?
                .clone();
            let port = match instance {
                Instance::Docker(instance) => Some(instance.config.providers.auth.as_ref()?.port),
                _ => None,
            }?;
            Some((*id, (auth_provider, port)))
        })
        .collect()
}

pub fn resolve_provider_reference(
    providers: &pouch::provider::Gems,
    feature: &FeatureKey,
    provider_reference: ProviderReference,
) -> Option<ProviderId> {
    match provider_reference {
        ProviderReference::Default => get_default_provider_id(providers, feature),
        ProviderReference::Provider(id) => Some(id),
    }
}

pub fn get_providers(
    instances: &pouch::instance::Gems,
) -> HashMap<FeatureKey, HashMap<ProviderId, Provider>> {
    let mut providers: HashMap<FeatureKey, HashMap<ProviderId, Provider>> = HashMap::new();
    for (id, instance) in instances {
        for (feature, value) in instance.manifest().provides() {
            providers.entry(feature.clone()).or_default().insert(
                *id,
                Provider {
                    app_key: instance.manifest().key().clone(),
                    config: value.clone(),
                },
            );
        }
    }
    providers
}

pub fn get_default_provider_id(
    providers: &pouch::provider::Gems,
    feature: &FeatureKey,
) -> Option<ProviderId> {
    providers.default_providers.get(feature).copied()
}

pub fn get_default_provider_ids(
    providers: &pouch::provider::Gems,
) -> &HashMap<FeatureKey, ProviderId> {
    &providers.default_providers
}

pub fn get_provides(
    instances: &pouch::instance::Gems,
    id: InstanceId,
) -> Result<HashMap<FeatureKey, Provider>, GetProvidesError> {
    match instances.get(&id) {
        None => Err(GetProvidesError::InstanceNotFound(id)),
        Some(instance) => Ok(instance
            .manifest()
            .provides()
            .iter()
            .map(|(feature, value)| {
                (
                    feature.clone(),
                    Provider {
                        app_key: instance.app_key().clone(),
                        config: value.clone(),
                    },
                )
            })
            .collect()),
    }
}

pub fn get_core_providers(providers: &pouch::provider::Gems) -> &CoreProviders {
    &providers.core_providers
}

pub fn put_core_auth_provider(
    instances: &pouch::instance::Gems,
    providers: &mut pouch::provider::Gems,
    provider: ProviderReference,
) -> Result<Option<ProviderReference>, PutCoreAuthProviderError> {
    let id = match provider {
        ProviderReference::Default => providers
            .default_providers
            .get(&FeatureKey::auth())
            .cloned()
            .ok_or(PutCoreAuthProviderError::DefaultProviderNotSet)?,
        ProviderReference::Provider(id) => id,
    };
    if instances
        .get(&id)
        .ok_or(PutCoreAuthProviderError::InstanceNotFound(id))?
        .manifest()
        .specific_providers()
        .auth
        .is_none()
    {
        Err(PutCoreAuthProviderError::DoesNotProvide { id })
    } else {
        Ok(providers.core_providers.auth.replace(provider))
    }
}

pub fn get_feature_provides(
    instances: &pouch::instance::Gems,
    feature: &FeatureKey,
    id: InstanceId,
) -> Result<Provider, GetFeatureProvidesError> {
    match instances.get(&id) {
        None => Err(GetFeatureProvidesError::InstanceNotFound(id)),
        Some(instance) => instance
            .manifest()
            .provides()
            .get(feature)
            .map(|value| Provider {
                app_key: instance.app_key().clone(),
                config: value.clone(),
            })
            .ok_or_else(|| GetFeatureProvidesError::DoesNotProvide {
                id,
                feature: feature.to_string(),
            }),
    }
}

pub fn get_dependencies(
    instances: &pouch::instance::Gems,
    id: InstanceId,
) -> Result<HashMap<DependencyKey, Dependency>, GetDependenciesError> {
    let Some(instance) = instances.get(&id) else {
        return Err(GetDependenciesError::InstanceNotFound(id));
    };
    Ok(instance
        .manifest()
        .depends()
        .iter()
        .map(|(feature, dependency)| {
            (
                feature.clone(),
                Dependency {
                    provider: instance.dependencies().get(feature).cloned(),
                    config: dependency.config_json(),
                },
            )
        })
        .collect())
}

pub fn get_dependency(
    instances: &pouch::instance::Gems,
    feature: &DependencyKey,
    id: InstanceId,
) -> Result<Dependency, GetDependencyError> {
    let Some(instance) = instances.get(&id) else {
        return Err(GetDependencyError::InstanceNotFound(id));
    };
    instance
        .manifest()
        .depends()
        .get(feature)
        .map(|dependency| Dependency {
            provider: instance.dependencies().get(feature).cloned(),
            config: dependency.config_json(),
        })
        .ok_or_else(|| GetDependencyError::DoesNotDepend {
            instance_id: id,
            feature: feature.to_string(),
        })
}

pub async fn clear_dependency(
    instances: &mut pouch::instance::Gems,
    feature: &DependencyKey,
    id: InstanceId,
) -> Result<Option<ProviderReference>, ClearDependencyError> {
    let Some(instance) = instances.get_mut(&id) else {
        return Err(ClearDependencyError::InstanceNotFound(id));
    };
    if !instance.manifest().depends().contains_key(feature) {
        return Err(ClearDependencyError::DoesNotDepend {
            feature: feature.to_string(),
            instance_id: id,
        });
    }
    if instance.dependencies().get(feature).is_none() {
        return Ok(None);
    }
    if instance.status().await? == InstanceStatus::Running {
        return Err(ClearDependencyError::InstanceRunning { instance_id: id });
    }
    Ok(instance
        .clear_dependency(feature)
        .map(|dependency| dependency.provider_reference))
}

fn config_str_matches(
    provider_config: &serde_json::Value,
    dependency_str: &str,
) -> Result<(), anyhow::Error> {
    match provider_config {
        serde_json::Value::Null => anyhow::bail!("Expected {dependency_str}, found null"),
        provider_val @ serde_json::Value::Bool(bool) => match bool::from_str(dependency_str) {
            Ok(expected_bool) if expected_bool == *bool => Ok(()),
            _ => anyhow::bail!("Expected {dependency_str}, found {provider_val:?}"),
        },
        provider_val @ serde_json::Value::Number(num) => {
            match serde_json::Number::from_str(dependency_str) {
                Ok(expected_num) if expected_num == *num => Ok(()),
                _ => anyhow::bail!("Expected {dependency_str}, found {provider_val:?}"),
            }
        }
        serde_json::Value::String(string) => {
            anyhow::ensure!(
                string == dependency_str,
                "Expected {dependency_str}, found {string}"
            );
            Ok(())
        }
        provider_val @ serde_json::Value::Array(array) => {
            anyhow::ensure!(
                array
                    .iter()
                    .any(|provider_val| config_str_matches(provider_val, dependency_str).is_ok()),
                "Found no match for {dependency_str} in {provider_val:?}"
            );
            Ok(())
        }
        provider_val @ serde_json::Value::Object(properties) => {
            anyhow::ensure!(
                properties.contains_key(dependency_str),
                "Found no match for {dependency_str} in {provider_val:?}"
            );
            Ok(())
        }
    }
}

fn split_escaped(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut chars = input.chars();

    while let Some(c) = chars.next() {
        match c {
            '\\' => cur.push(chars.next().unwrap_or('\\')),
            '|' => {
                out.push(cur);
                cur = String::new();
            }
            c => cur.push(c),
        }
    }
    out.push(cur);
    out
}

fn config_matches(
    provider_config: &serde_json::Value,
    dependency_config: &serde_json::Value,
) -> Result<(), anyhow::Error> {
    match dependency_config {
        serde_json::Value::Null => Ok(()),
        dependency_config @ serde_json::Value::Bool(dependency_bool) => match provider_config {
            serde_json::Value::Bool(provider_bool) if dependency_bool == provider_bool => Ok(()),
            val => {
                anyhow::bail!("Expected {dependency_config}, found {val}")
            }
        },
        dependency_config @ serde_json::Value::Number(dependency_num) => match provider_config {
            serde_json::Value::Number(provider_num) if dependency_num == provider_num => Ok(()),
            val => {
                anyhow::bail!("Expected {dependency_config}, found {val}")
            }
        },
        serde_json::Value::String(dependency_string) => {
            anyhow::ensure!(
                split_escaped(dependency_string)
                    .iter()
                    .any(
                        |dependency_str| config_str_matches(provider_config, dependency_str)
                            .is_ok()
                    ),
                "Could not find a match for any value in {dependency_string}, in {provider_config:?}"
            );
            Ok(())
        }
        dependency_config @ serde_json::Value::Array(dependency_array) => match provider_config {
            serde_json::Value::Array(provider_array) => {
                for dependency_value in dependency_array {
                    anyhow::ensure!(
                        provider_array.iter().any(|provider_value| {
                            config_matches(provider_value, dependency_value).is_ok()
                        }),
                        "Could not find a match for {dependency_value:?}, in {provider_array:?}"
                    )
                }
                Ok(())
            }
            val => {
                anyhow::bail!("Expected {dependency_config}, found {val}")
            }
        },
        serde_json::Value::Object(dependency_properties) => {
            let serde_json::Value::Object(provider_properties) = provider_config else {
                anyhow::bail!(
                    "Expected properties {dependency_properties:?}, found {provider_config}"
                )
            };
            for (key, property) in dependency_properties {
                let provider_property = provider_properties
                    .get(key)
                    .ok_or_else(|| anyhow::anyhow!("Expected property {key}"))?;
                config_matches(provider_property, property).context(format!(".{key}"))?;
            }
            Ok(())
        }
    }
}

pub async fn set_dependency(
    instances: &mut pouch::instance::Gems,
    providers: &pouch::provider::Gems,
    dependency_key: DependencyKey,
    feature: FeatureKey,
    id: InstanceId,
    provider_reference: ProviderReference,
) -> Result<Option<ProviderReference>, SetDependencyError> {
    let Some(instance) = instances.get(&id) else {
        return Err(SetDependencyError::InstanceNotFound(id));
    };

    if !dependency_key.features().contains(&feature) {
        return Err(SetDependencyError::KeyDoesNotContainFeature {
            key: dependency_key,
            feature,
        });
    }
    let manifest = instance.manifest();
    let dependency_config = match manifest.depends().get(&dependency_key) {
        None => {
            return Err(SetDependencyError::DoesNotDepend {
                key: feature.to_string(),
                instance_id: id,
            });
        }
        Some(gem::manifest::Dependency::One(_, value)) => value,
        Some(gem::manifest::Dependency::OneOf(map)) => {
            map.get(&feature)
                .ok_or_else(|| SetDependencyError::DoesNotDepend {
                    key: feature.to_string(),
                    instance_id: id,
                })?
        }
    };

    let provider_id = match provider_reference {
        ProviderReference::Provider(id) => id,
        ProviderReference::Default => providers
            .default_providers
            .get(&feature)
            .cloned()
            .ok_or_else(|| SetDependencyError::NoDefaultProvider {
                feature: feature.clone(),
            })?,
    };
    let Some(provider) = instances.get(&provider_id) else {
        return Err(SetDependencyError::ProviderDoesNotExist(provider_id));
    };
    let manifest = provider.manifest();
    let Some(provider_config) = manifest.provides().get(&feature) else {
        return Err(SetDependencyError::ProviderDoesNotProvideFeature {
            feature: feature.clone(),
            provider_id,
        });
    };
    config_matches(provider_config, dependency_config).map_err(|error| {
        SetDependencyError::FeatureConfigNotMatching {
            provider_id,
            error,
            feature: feature.clone(),
        }
    })?;
    if instance.status().await? == InstanceStatus::Running {
        return Err(SetDependencyError::InstanceRunning { instance_id: id });
    }
    let Some(instance) = instances.get_mut(&id) else {
        return Err(SetDependencyError::InstanceNotFound(id));
    };
    let provider_reference = StoredProviderReference {
        provider_reference,
        provided_feature: feature,
    };
    Ok(instance
        .set_dependency(dependency_key, provider_reference)
        .map(|dependency| dependency.provider_reference.clone()))
}

pub async fn set_default_dependencies(quest: SyncQuest, vault: Arc<Vault>, id: InstanceId) {
    let GrabbedPouches {
        provider_pouch: Some(ref providers),
        instance_pouch_mut: Some(ref mut instances),
        ..
    } = vault
        .reservation()
        .reserve_provider_pouch()
        .reserve_instance_pouch_mut()
        .grab()
        .await
    else {
        unreachable!("Reservation should never fail");
    };
    let Some(instance) = instances.gems().get(&id) else {
        let mut quest = quest.lock().await;
        quest.detail = Some(format!("Instance with id {id} does not exist"));
        quest.state = State::Failed;
        return;
    };
    let dependencies: Vec<_> = instance.dependencies().keys().cloned().collect();
    if dependencies.is_empty() {
        let mut quest = quest.lock().await;
        quest.detail = Some("Instance has no dependencies".to_string());
        quest.state = State::Skipped;
        return;
    }
    for dependency in dependencies {
        let detail = match set_default_dependency(
            instances.gems_mut(),
            providers.gems(),
            dependency.clone(),
            id,
        )
        .await
        {
            Ok(feature) => format!(
                "Solved dependency {dependency} with default provider for feature {feature}"
            ),
            Err(errors) => errors
                .into_iter()
                .map(|(feature, error)| {
                    format!("Could not use default provider for {feature} for dependency {dependency}: {error}")
                })
                .collect::<Vec<_>>()
                .join("\n"),
        };
        let mut quest = quest.lock().await;
        match quest.detail.as_mut() {
            None => quest.detail = Some(detail),
            Some(quest_detail) => {
                quest_detail.push('\n');
                quest_detail.push_str(&detail)
            }
        }
    }
}

pub async fn set_default_dependency(
    instances: &mut pouch::instance::Gems,
    providers: &pouch::provider::Gems,
    dependency_key: DependencyKey,
    id: InstanceId,
) -> Result<FeatureKey, Vec<(FeatureKey, SetDependencyError)>> {
    let mut errors = Vec::new();
    for feature in dependency_key.features() {
        if let Err(e) = set_dependency(
            instances,
            providers,
            dependency_key.clone(),
            feature.clone(),
            id,
            ProviderReference::Default,
        )
        .await
        {
            errors.push((feature, e));
        } else {
            return Ok(feature);
        }
    }
    Err(errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn config_str_matches_null() {
        assert!(config_str_matches(&serde_json::Value::Null, "null").is_err());
        assert!(config_str_matches(&serde_json::Value::Null, "").is_err());
        assert!(config_str_matches(&serde_json::Value::Null, "true").is_err());
    }

    #[test]
    fn config_str_matches_bool_ok() {
        config_str_matches(&json!(true), "true").unwrap();
        config_str_matches(&json!(false), "false").unwrap();
    }

    #[test]
    fn config_str_matches_bool_err() {
        assert!(config_str_matches(&json!(true), "false").is_err());
        assert!(config_str_matches(&json!(false), "true").is_err());
        assert!(config_str_matches(&json!(true), "a").is_err());
    }

    #[test]
    fn config_str_matches_num_ok() {
        config_str_matches(&json!(12.7), "12.7").unwrap();
        config_str_matches(&json!(-120), "-120").unwrap();
        config_str_matches(&json!(786812), "786812").unwrap();
    }

    #[test]
    fn config_str_matches_num_err() {
        assert!(config_str_matches(&json!(1), "2").is_err());
        assert!(config_str_matches(&json!(1), "a").is_err());
    }

    #[test]
    fn config_str_matches_string_ok() {
        config_str_matches(&json!("oehjoina"), "oehjoina").unwrap();
        config_str_matches(&json!("-120"), "-120").unwrap();
    }

    #[test]
    fn config_str_matches_string_err() {
        assert!(config_str_matches(&json!("oehjoina"), "as").is_err());
        assert!(config_str_matches(&json!("1"), "2").is_err());
    }

    #[test]
    fn config_str_matches_array_ok() {
        config_str_matches(&json!(["oehjoina"]), "oehjoina").unwrap();
        config_str_matches(&json!(["123", "oehjoina", "124"]), "oehjoina").unwrap();
        config_str_matches(&json!(["1", "2", 3.1, { "p": 100}]), "3.1").unwrap();
        config_str_matches(&json!(["1", "2", 3.1, { "p": 100}]), "p").unwrap();
    }

    #[test]
    fn config_str_matches_array_err() {
        assert!(config_str_matches(&json!(["oehjoina"]), "as").is_err());
        assert!(config_str_matches(&json!(["123", "oehjoina", "124"]), "125").is_err());
        assert!(config_str_matches(&json!(["1", "2", 3.1, { "p": 100}]), "x").is_err());
    }

    #[test]
    fn config_str_matches_object_ok() {
        config_str_matches(&json!({"oehjoina": 100}), "oehjoina").unwrap();
        config_str_matches(&json!({"eta": null}), "eta").unwrap();
        config_str_matches(&json!({"beta": {}}), "beta").unwrap();
        config_str_matches(&json!({"true": 100}), "true").unwrap();
    }

    #[test]
    fn config_str_matches_object_err() {
        assert!(config_str_matches(&json!({"oehjoina": 100}), "as").is_err());
        assert!(config_str_matches(&json!({}), "125").is_err());
    }

    #[test]
    fn config_matches_null() {
        config_matches(&json!(null), &serde_json::Value::Null).unwrap();
        config_matches(&json!(true), &serde_json::Value::Null).unwrap();
        config_matches(&json!("string"), &serde_json::Value::Null).unwrap();
        config_matches(&json!(12), &serde_json::Value::Null).unwrap();
        config_matches(&json!([1, "44", null]), &serde_json::Value::Null).unwrap();
        config_matches(&json!({ "some": 10, "thing": []}), &serde_json::Value::Null).unwrap();
    }

    #[test]
    fn config_matches_bool_ok() {
        config_matches(&json!(true), &json!(true)).unwrap();
        config_matches(&json!(false), &json!(false)).unwrap();
    }

    #[test]
    fn config_matches_bool_err() {
        assert!(config_matches(&json!(true), &json!(false)).is_err());
        assert!(config_matches(&json!(false), &json!(true)).is_err());
        assert!(config_matches(&json!("false"), &json!(true)).is_err());
        assert!(config_matches(&json!(1), &json!(true)).is_err());
    }

    #[test]
    fn config_matches_num_ok() {
        config_matches(&json!(12.7), &json!(12.7)).unwrap();
        config_matches(&json!(-120), &json!(-120)).unwrap();
        config_matches(&json!(786812), &json!(786812)).unwrap();
    }

    #[test]
    fn config_matches_num_err() {
        assert!(config_matches(&json!(1), &json!(786812)).is_err());
        assert!(config_matches(&json!("1"), &json!(1)).is_err());
    }

    #[test]
    fn config_matches_string_ok() {
        config_matches(&json!("oehjoina"), &json!("oehjoina")).unwrap();
        config_matches(&json!(-120), &json!("-120")).unwrap();
        config_matches(&json!(1), &json!("2|4|1")).unwrap();
        config_matches(&json!({"p": 100, "test": true}), &json!("test|4|2")).unwrap();
    }

    #[test]
    fn config_matches_string_err() {
        assert!(config_matches(&json!("oehjoina"), &json!("as")).is_err());
        assert!(config_matches(&json!("1"), &json!("2|3|4")).is_err());
        assert!(config_matches(&json!("1"), &json!(" 1|2|3|4")).is_err());
    }

    #[test]
    fn config_matches_array_ok() {
        config_matches(&json!(["oehjoina"]), &json!(["oehjoina"])).unwrap();
        config_matches(&json!(["123", "oehjoina", "124"]), &json!(["123", "124"])).unwrap();
        config_matches(&json!(["123", "oehjoina", "124"]), &json!(["124", "123"])).unwrap();
        config_matches(
            &json!(["1", "2", 3.1, { "p": 100}]),
            &json!(["3.1|be", "p|75"]),
        )
        .unwrap();
        config_matches(&json!([1, "2", 3.1, { "p": 100}]), &json!(["1"])).unwrap();
        config_matches(&json!([1, "2", 3.1, { "p": 100}]), &json!([1])).unwrap();
        config_matches(&json!(["1", "2", 3.1, { "p": 100}]), &json!([])).unwrap();
        config_matches(
            &json!([1, 2, 3, { "p": 100, "k": "ast", "t": null}]),
            &json!([{ "p": 100, "t": null}]),
        )
        .unwrap();
    }

    #[test]
    fn config_matches_array_err() {
        assert!(config_matches(&json!(["oehjoina"]), &json!(["as"])).is_err());
        assert!(
            config_matches(
                &json!(["123", "oehjoina", "124"]),
                &json!(["123", "124", "125"])
            )
            .is_err()
        );
        assert!(config_matches(&json!([]), &json!([true])).is_err());
        assert!(config_matches(&json!({ "p": 100}), &json!(["n"])).is_err());
    }

    #[test]
    fn config_matches_object_ok() {
        config_matches(&json!({"oehjoina": 100}), &json!({"oehjoina": 100})).unwrap();
        config_matches(
            &json!({"a": 1, "b": 2, "c": 3}),
            &json!({"a": 1, "b": 2, "c": 3}),
        )
        .unwrap();
        config_matches(&json!({"a": 1, "b": 2, "c": 3}), &json!({"b": 2, "c": 3})).unwrap();
        config_matches(&json!({"beta": {}}), &json!({"beta": null})).unwrap();
        config_matches(&json!({"true": 100}), &json!({})).unwrap();
    }

    #[test]
    fn config_matches_object_err() {
        assert!(config_matches(&json!({"oehjoina": 100}), &json!({"oehjoina": 101})).is_err());
        assert!(config_matches(&json!({"oehjoina": 100}), &json!({"as": 100})).is_err());
        assert!(
            config_matches(&json!({"a": 1, "c": 3}), &json!({"a": 1, "b": 2, "c": 3}),).is_err()
        );
    }

    fn str_vec(s: &[&str]) -> Vec<String> {
        s.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn split_escaped_empties() {
        assert_eq!(split_escaped(""), str_vec(&[""]));
        assert_eq!(split_escaped("a|b"), str_vec(&["a", "b"]));
        assert_eq!(split_escaped("a||b"), str_vec(&["a", "", "b"]));
        assert_eq!(split_escaped("a|||b"), str_vec(&["a", "", "", "b"]));
        assert_eq!(split_escaped("a||||b"), str_vec(&["a", "", "", "", "b"]));
        assert_eq!(
            split_escaped("a|||||b"),
            str_vec(&["a", "", "", "", "", "b"])
        );
        assert_eq!(
            split_escaped("a||||||b"),
            str_vec(&["a", "", "", "", "", "", "b"])
        );
    }

    #[test]
    fn split_escaped_escaped() {
        assert_eq!(split_escaped("\\\\"), str_vec(&["\\"]));
        assert_eq!(split_escaped("\\a"), str_vec(&["a"]));
        assert_eq!(split_escaped("a\\|b"), str_vec(&["a|b"]));
    }

    #[test]
    fn split_escaped_trailing_slash() {
        assert_eq!(split_escaped("1234\\"), str_vec(&["1234\\"]));
    }
}
