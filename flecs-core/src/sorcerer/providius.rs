use super::spell;
use crate::jeweler::gem::instance::{InstanceId, ProviderReference, StoredProviderReference};
use crate::jeweler::gem::manifest::providers::auth::AuthProvider;
use crate::jeweler::gem::manifest::{DependencyKey, FeatureKey};
#[cfg(feature = "auth")]
use crate::quest::SyncQuest;
use crate::sorcerer::Sorcerer;
#[cfg(feature = "auth")]
use crate::sorcerer::spell::provider::BuildWatchConfigError;
use crate::vault::Vault;
use crate::vault::pouch::AppKey;
use crate::vault::pouch::provider::{CoreProviders, ProviderId};
#[cfg(feature = "auth")]
use crate::wall::{watch, watch::Watch};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
pub use spell::provider::{
    ClearDependencyError, DeleteDefaultProviderError, GetAuthProviderPortError,
    GetDependenciesError, GetDependencyError, GetFeatureProvidesError, GetProviderError,
    GetProvidesError, SetCoreAuthProviderError, SetDefaultProviderError, SetDependencyError,
};
use std::collections::HashMap;
use std::sync::Arc;
use utoipa::ToSchema;

pub struct ForwardedHeaders {
    pub protocol: Option<String>,
    pub port: Option<u16>,
    pub host: Option<String>,
}

pub struct ReplacementUrlParts {
    pub protocol: String,
    pub port: u16,
    pub host: String,
}

impl ReplacementUrlParts {
    pub fn from_forwarded_and_host(forwarded: ForwardedHeaders, host: axum::extract::Host) -> Self {
        let protocol = forwarded.protocol.unwrap_or_else(|| "https".to_string());
        let port = forwarded.port.unwrap_or(match protocol.as_str() {
            "http" => 80,
            _ => 443,
        });
        let host = forwarded.host.unwrap_or(host.0);
        Self {
            protocol,
            port,
            host,
        }
    }
}

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
        #[cfg(feature = "auth")] watch: Arc<Watch>,
        provider: ProviderReference,
    ) -> Result<Option<ProviderReference>, SetCoreAuthProviderError>;
    #[cfg(feature = "auth")]
    async fn setup_core_auth_provider(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        watch: Arc<Watch>,
    ) -> Result<(), SetCoreAuthProviderError>;
    #[cfg(feature = "auth")]
    async fn get_auth_providers_and_default(
        &self,
        vault: Arc<Vault>,
        replacement_url_parts: &ReplacementUrlParts,
    ) -> AuthProvidersAndDefaults;
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
    #[cfg(feature = "auth")]
    async fn build_watch_config_from_auth_provider(
        &self,
        vault: Arc<Vault>,
        auth_provider: ProviderReference,
    ) -> Result<watch::AuthProviderMetaData, BuildWatchConfigError>;
}

#[cfg(test)]
impl Sorcerer for MockProvidius {}
