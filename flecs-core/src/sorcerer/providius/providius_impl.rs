use crate::jeweler::gem::instance::{InstanceId, ProviderReference};
use crate::jeweler::gem::manifest::{DependencyKey, FeatureKey};
use crate::lore::FloxyLore;
#[cfg(feature = "auth")]
use crate::quest;
#[cfg(feature = "auth")]
use crate::quest::SyncQuest;
use crate::sorcerer::Sorcerer;
#[cfg(feature = "auth")]
use crate::sorcerer::providius::AuthProvidersAndDefaults;
use crate::sorcerer::providius::{
    ClearDependencyError, Dependency, GetDependenciesError, GetFeatureProvidesError,
    GetProvidesError, Provider, ProvidersAndDefaults, Providius, ReplacementUrlParts,
};
#[cfg(feature = "auth")]
use crate::sorcerer::spell::provider::{
    BuildWatchConfigError, build_watch_config_from_auth_provider, get_auth_providers,
};
use crate::sorcerer::spell::provider::{
    DeleteDefaultProviderError, GetDependencyError, GetProviderError, SetCoreAuthProviderError,
    SetDefaultProviderError, SetDependencyError, clear_dependency, delete_default_provider,
    get_core_providers, get_default_provider_id, get_default_provider_ids, get_dependencies,
    get_dependency, get_feature_provides, get_provider, get_providers, get_provides,
    set_core_auth_provider, set_default_provider, set_dependency,
};
use crate::vault::pouch::Pouch;
use crate::vault::pouch::provider::{CoreProviders, ProviderId};
use crate::vault::{GrabbedPouches, Vault};
#[cfg(feature = "auth")]
use crate::wall::watch;
#[cfg(feature = "auth")]
use crate::wall::watch::Watch;
use async_trait::async_trait;
use std::collections::HashMap;
#[cfg(feature = "auth")]
use std::ops::DerefMut;
use std::sync::Arc;

pub struct ProvidiusImpl;

impl Sorcerer for ProvidiusImpl {}

#[async_trait]
impl Providius for ProvidiusImpl {
    async fn get_core_providers(&self, vault: Arc<Vault>) -> CoreProviders {
        let grab = vault.reservation().reserve_provider_pouch().grab().await;
        let provider_pouch = grab
            .provider_pouch
            .as_ref()
            .expect("Vault reservations should never fail");
        get_core_providers(provider_pouch.gems()).clone()
    }

    async fn put_core_auth_provider(
        &self,
        vault: Arc<Vault>,
        #[cfg(feature = "auth")] watch: Arc<Watch>,
        provider: ProviderReference,
    ) -> Result<Option<ProviderReference>, SetCoreAuthProviderError> {
        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            provider_pouch_mut: Some(ref mut providers),
            ..
        } = vault
            .reservation()
            .reserve_instance_pouch()
            .reserve_provider_pouch_mut()
            .grab()
            .await
        else {
            unreachable!("Reservation should never fail");
        };
        #[cfg(feature = "auth")]
        let config =
            build_watch_config_from_auth_provider(instances.gems(), providers.gems(), provider)
                .await?;
        let previous = set_core_auth_provider(instances.gems(), providers.gems_mut(), provider)?;
        #[cfg(feature = "auth")]
        watch
            .data_mut()
            .await
            .deref_mut()
            .set_auth_provider_meta_data(config);
        Ok(previous)
    }

    #[cfg(feature = "auth")]
    async fn setup_core_auth_provider(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        watch: Arc<Watch>,
    ) -> Result<(), SetCoreAuthProviderError> {
        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            provider_pouch: Some(ref providers),
            ..
        } = vault
            .reservation()
            .reserve_instance_pouch()
            .reserve_provider_pouch()
            .grab()
            .await
        else {
            unreachable!("Reservation should never fail");
        };
        let Some(provider) = providers.gems().core_providers.auth else {
            let mut quest = quest.lock().await;
            quest.state = quest::State::Skipped;
            quest.detail = Some("No core auth provider configured".to_string());
            return Ok(());
        };
        let config =
            build_watch_config_from_auth_provider(instances.gems(), providers.gems(), provider)
                .await?;
        watch
            .data_mut()
            .await
            .deref_mut()
            .set_auth_provider_meta_data(config);
        Ok(())
    }

    #[cfg(feature = "auth")]
    async fn get_auth_providers_and_default(
        &self,
        vault: Arc<Vault>,
        replacement_url_parts: &ReplacementUrlParts,
    ) -> AuthProvidersAndDefaults {
        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            provider_pouch: Some(ref providers),
            ..
        } = vault
            .reservation()
            .reserve_instance_pouch()
            .reserve_provider_pouch()
            .grab()
            .await
        else {
            unreachable!("Reservation should never fail");
        };
        let default = get_default_provider_id(providers.gems(), &FeatureKey::auth());
        let core = get_core_providers(providers.gems()).auth;
        let providers = get_auth_providers(instances.gems());
        let providers = providers
            .into_iter()
            .map(|(id, mut provider)| {
                // Replace host, port, schema and path part of issuer url as accessible by the client
                provider.replace_url_parts(
                    replacement_url_parts,
                    &FloxyLore::auth_provider_location(id),
                );
                (id, provider)
            })
            .collect();
        AuthProvidersAndDefaults {
            default,
            core,
            providers,
        }
    }

    async fn get_providers(
        &self,
        vault: Arc<Vault>,
    ) -> HashMap<FeatureKey, HashMap<ProviderId, Provider>> {
        let grab = vault.reservation().reserve_instance_pouch().grab().await;
        let instance_pouch = grab
            .instance_pouch
            .as_ref()
            .expect("Vault reservations should never fail");
        get_providers(instance_pouch.gems())
    }

    async fn get_default_providers(&self, vault: Arc<Vault>) -> HashMap<FeatureKey, ProviderId> {
        let grab = vault.reservation().reserve_provider_pouch().grab().await;
        let provider_pouch = grab
            .provider_pouch
            .as_ref()
            .expect("Vault reservations should never fail");
        get_default_provider_ids(provider_pouch.gems()).clone()
    }

    async fn get_default_provider(
        &self,
        vault: Arc<Vault>,
        feature: &FeatureKey,
    ) -> Result<Option<Provider>, GetProviderError> {
        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            provider_pouch: Some(ref providers),
            ..
        } = vault
            .reservation()
            .reserve_instance_pouch()
            .reserve_provider_pouch()
            .grab()
            .await
        else {
            unreachable!("Reservation should never fail");
        };
        let Some(provider_id) = get_default_provider_id(providers.gems(), feature) else {
            return Ok(None);
        };
        Ok(Some(get_provider(instances.gems(), feature, provider_id)?))
    }

    async fn get_providers_and_defaults(&self, vault: Arc<Vault>) -> ProvidersAndDefaults {
        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            provider_pouch: Some(ref providers),
            ..
        } = vault
            .reservation()
            .reserve_instance_pouch()
            .reserve_provider_pouch()
            .grab()
            .await
        else {
            unreachable!("Reservation should never fail");
        };
        ProvidersAndDefaults {
            providers: get_providers(instances.gems()),
            defaults: get_default_provider_ids(providers.gems()).clone(),
        }
    }

    async fn get_provider(
        &self,
        vault: Arc<Vault>,
        feature: &FeatureKey,
        id: ProviderId,
    ) -> Result<Provider, GetProviderError> {
        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Reservation should never fail");
        };
        get_provider(instances.gems(), feature, id)
    }

    async fn delete_default_provider(
        &self,
        vault: Arc<Vault>,
        feature: &FeatureKey,
    ) -> Result<Option<ProviderId>, DeleteDefaultProviderError> {
        let GrabbedPouches {
            provider_pouch_mut: Some(ref mut providers),
            instance_pouch: Some(ref instances),
            ..
        } = vault
            .reservation()
            .reserve_provider_pouch_mut()
            .reserve_instance_pouch()
            .grab()
            .await
        else {
            unreachable!("Reservation should never fail");
        };
        delete_default_provider(feature, providers.gems_mut(), instances.gems()).await
    }

    async fn set_default_provider(
        &self,
        vault: Arc<Vault>,
        feature: FeatureKey,
        id: ProviderId,
    ) -> Result<Option<ProviderId>, SetDefaultProviderError> {
        let GrabbedPouches {
            provider_pouch_mut: Some(ref mut providers),
            instance_pouch: Some(ref instances),
            ..
        } = vault
            .reservation()
            .reserve_provider_pouch_mut()
            .reserve_instance_pouch()
            .grab()
            .await
        else {
            unreachable!("Reservation should never fail");
        };
        set_default_provider(feature, providers.gems_mut(), instances.gems(), id)
    }

    async fn get_provides(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Result<HashMap<FeatureKey, Provider>, GetProvidesError> {
        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Reservation should never fail");
        };
        get_provides(instances.gems(), id)
    }

    async fn get_feature_provides(
        &self,
        vault: Arc<Vault>,
        feature: &FeatureKey,
        id: InstanceId,
    ) -> Result<Provider, GetFeatureProvidesError> {
        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Reservation should never fail");
        };
        get_feature_provides(instances.gems(), feature, id)
    }

    async fn get_dependencies(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Result<HashMap<DependencyKey, Dependency>, GetDependenciesError> {
        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Reservation should never fail");
        };
        get_dependencies(instances.gems(), id)
    }

    async fn get_dependency(
        &self,
        vault: Arc<Vault>,
        key: &DependencyKey,
        id: InstanceId,
    ) -> Result<Dependency, GetDependencyError> {
        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Reservation should never fail");
        };
        get_dependency(instances.gems(), key, id)
    }

    async fn clear_dependency(
        &self,
        vault: Arc<Vault>,
        key: &DependencyKey,
        id: InstanceId,
    ) -> Result<Option<ProviderReference>, ClearDependencyError> {
        let GrabbedPouches {
            instance_pouch_mut: Some(ref mut instances),
            ..
        } = vault
            .reservation()
            .reserve_instance_pouch_mut()
            .grab()
            .await
        else {
            unreachable!("Reservation should never fail");
        };
        clear_dependency(instances.gems_mut(), key, id).await
    }

    async fn set_dependency(
        &self,
        vault: Arc<Vault>,
        dependency_key: DependencyKey,
        feature: FeatureKey,
        id: InstanceId,
        provider_reference: ProviderReference,
    ) -> Result<Option<ProviderReference>, SetDependencyError> {
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
        set_dependency(
            instances.gems_mut(),
            providers.gems(),
            dependency_key,
            feature,
            id,
            provider_reference,
        )
        .await
    }

    #[cfg(feature = "auth")]
    async fn build_watch_config_from_auth_provider(
        &self,
        vault: Arc<Vault>,
        auth_provider: ProviderReference,
    ) -> Result<watch::AuthProviderMetaData, BuildWatchConfigError> {
        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            provider_pouch: Some(ref providers),
            ..
        } = vault
            .reservation()
            .reserve_instance_pouch()
            .reserve_provider_pouch()
            .grab()
            .await
        else {
            unreachable!("Reservation should never fail");
        };
        build_watch_config_from_auth_provider(instances.gems(), providers.gems(), auth_provider)
            .await
    }
}
