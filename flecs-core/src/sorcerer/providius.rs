use super::spell;
use crate::jeweler::gem::instance::{InstanceId, ProviderReference, StoredProviderReference};
use crate::jeweler::gem::manifest::DependencyKey;
use crate::sorcerer::Sorcerer;
use crate::vault::Vault;
use crate::vault::pouch::AppKey;
use crate::vault::pouch::provider::ProviderId;
use async_trait::async_trait;
pub use spell::provider::{
    ClearDependencyError, DeleteDefaultProviderError, GetDependenciesError, GetDependencyError,
    GetFeatureProvidesError, GetProviderError, GetProvidesError, SetDefaultProviderError,
    SetDependencyError,
};
use std::collections::HashMap;
use std::sync::Arc;

pub struct ProvidersAndDefaults {
    pub providers: HashMap<String, HashMap<ProviderId, Provider>>,
    pub defaults: HashMap<String, ProviderId>,
}

pub struct Provider {
    pub app_key: AppKey,
    pub config: serde_json::Value,
}

pub type Provides = StoredProviderReference;

pub struct Dependency {
    pub provider: Option<Provides>,
    pub config: serde_json::Value,
}

pub mod providius_impl;
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Providius: Sorcerer {
    async fn get_providers(
        &self,
        vault: Arc<Vault>,
    ) -> HashMap<String, HashMap<ProviderId, Provider>>;
    async fn get_default_providers(&self, vault: Arc<Vault>) -> HashMap<String, ProviderId>;
    async fn get_default_provider(
        &self,
        vault: Arc<Vault>,
        feature: &str,
    ) -> Result<Option<Provider>, GetProviderError>;
    async fn get_providers_and_defaults(&self, vault: Arc<Vault>) -> ProvidersAndDefaults;
    async fn get_provider(
        &self,
        vault: Arc<Vault>,
        feature: &str,
        id: ProviderId,
    ) -> Result<Provider, GetProviderError>;
    async fn delete_default_provider(
        &self,
        vault: Arc<Vault>,
        feature: &str,
    ) -> Result<Option<ProviderId>, DeleteDefaultProviderError>;
    async fn set_default_provider(
        &self,
        vault: Arc<Vault>,
        feature: String,
        id: ProviderId,
    ) -> Result<Option<ProviderId>, SetDefaultProviderError>;
    async fn get_provides(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Result<HashMap<String, Provider>, GetProvidesError>;
    async fn get_feature_provides(
        &self,
        vault: Arc<Vault>,
        feature: &str,
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
        feature: &str,
        id: InstanceId,
        provider_reference: ProviderReference,
    ) -> Result<Option<ProviderReference>, SetDependencyError>;
}

#[cfg(test)]
impl Sorcerer for MockProvidius {}
