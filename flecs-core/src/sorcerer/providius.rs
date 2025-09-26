use super::spell;
use crate::jeweler::gem::instance::{InstanceId, ProviderReference, StoredProviderReference};
use crate::jeweler::gem::manifest::providers::auth::AuthProvider;
use crate::jeweler::gem::manifest::{DependencyKey, FeatureKey};
use crate::sorcerer::Sorcerer;
use crate::vault::Vault;
use crate::vault::pouch::AppKey;
use crate::vault::pouch::provider::{CoreProviders, ProviderId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
pub use spell::provider::{
    ClearDependencyError, DeleteDefaultProviderError, GetDependenciesError, GetDependencyError,
    GetFeatureProvidesError, GetProviderError, GetProvidesError, PutCoreAuthProviderError,
    SetDefaultProviderError, SetDependencyError,
};
use std::collections::HashMap;
use std::sync::Arc;
use utoipa::ToSchema;

pub struct ProvidersAndDefaults {
    pub providers: HashMap<FeatureKey, HashMap<ProviderId, Provider>>,
    pub defaults: HashMap<FeatureKey, ProviderId>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AuthProvidersAndDefaults {
    pub default: Option<ProviderId>,
    pub providers: HashMap<ProviderId, AuthProvider>,
    pub core: Option<ProviderReference>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct Provider {
    pub app_key: AppKey,
    pub config: serde_json::Value,
}

pub type Provides = StoredProviderReference;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Dependency {
    pub provider: Option<Provides>,
    pub config: serde_json::Value,
}

pub mod providius_impl;
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Providius: Sorcerer {
    async fn get_core_providers(&self, vault: Arc<Vault>) -> CoreProviders;
    async fn put_core_auth_provider(
        &self,
        vault: Arc<Vault>,
        provider: ProviderReference,
    ) -> Result<Option<ProviderReference>, PutCoreAuthProviderError>;
    async fn get_auth_providers_and_default(&self, vault: Arc<Vault>) -> AuthProvidersAndDefaults;
    async fn get_providers(
        &self,
        vault: Arc<Vault>,
    ) -> HashMap<FeatureKey, HashMap<ProviderId, Provider>>;
    async fn get_default_providers(&self, vault: Arc<Vault>) -> HashMap<FeatureKey, ProviderId>;
    async fn get_default_provider(
        &self,
        vault: Arc<Vault>,
        feature: &FeatureKey,
    ) -> Result<Option<Provider>, GetProviderError>;
    async fn get_providers_and_defaults(&self, vault: Arc<Vault>) -> ProvidersAndDefaults;
    async fn get_provider(
        &self,
        vault: Arc<Vault>,
        feature: &FeatureKey,
        id: ProviderId,
    ) -> Result<Provider, GetProviderError>;
    async fn delete_default_provider(
        &self,
        vault: Arc<Vault>,
        feature: &FeatureKey,
    ) -> Result<Option<ProviderId>, DeleteDefaultProviderError>;
    async fn set_default_provider(
        &self,
        vault: Arc<Vault>,
        feature: FeatureKey,
        id: ProviderId,
    ) -> Result<Option<ProviderId>, SetDefaultProviderError>;
    async fn get_provides(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Result<HashMap<FeatureKey, Provider>, GetProvidesError>;
    async fn get_feature_provides(
        &self,
        vault: Arc<Vault>,
        feature: &FeatureKey,
        id: InstanceId,
    ) -> Result<Provider, GetFeatureProvidesError>;
    async fn get_dependencies(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Result<HashMap<DependencyKey, Dependency>, GetDependenciesError>;
    async fn get_dependency(
        &self,
        vault: Arc<Vault>,
        key: &DependencyKey,
        id: InstanceId,
    ) -> Result<Dependency, GetDependencyError>;
    async fn clear_dependency(
        &self,
        vault: Arc<Vault>,
        key: &DependencyKey,
        id: InstanceId,
    ) -> Result<Option<ProviderReference>, ClearDependencyError>;
    async fn set_dependency(
        &self,
        vault: Arc<Vault>,
        dependency_key: DependencyKey,
        feature: FeatureKey,
        id: InstanceId,
        provider_reference: ProviderReference,
    ) -> Result<Option<ProviderReference>, SetDependencyError>;
}

#[cfg(test)]
impl Sorcerer for MockProvidius {}
