pub use super::{Error, Result};
use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::jeweler::gem::deployment::compose::ComposeDeployment;
use crate::jeweler::gem::deployment::docker::DockerDeployment;
use crate::jeweler::gem::instance::compose::ComposeInstance;
use crate::jeweler::gem::instance::docker::DockerInstance;
use crate::jeweler::gem::instance::docker::config::{InstanceConfig, ProviderConfig};
use crate::jeweler::gem::instance::status::InstanceStatus;
use crate::jeweler::gem::instance::{Instance, InstanceId, ProviderReference};
use crate::jeweler::gem::manifest::multi::AppManifestMulti;
use crate::jeweler::gem::manifest::single::AppManifestSingle;
use crate::jeweler::network::NetworkId;
use crate::lore::Lore;
use crate::quest::{State, SyncQuest};
use crate::relic::network::Ipv4NetworkAccess;
use crate::sorcerer::spell::provider::GetAuthProviderPortError;
use crate::vault::pouch::provider::ProviderId;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::{GrabbedPouches, Vault, pouch};
use futures_util::future::join_all;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::error;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum DisconnectInstanceError {
    #[error("Instance not found: {0}")]
    InstanceNotFound(InstanceId),
    #[error("Instance {instance} not connected to {network}")]
    InstanceNotConnected {
        network: NetworkId,
        instance: InstanceId,
    },
    #[error("Instance {0} does not support disconnecting from networks")]
    Unsupported(InstanceId),
    #[error("Failed to disconnect instance: {0}")]
    Other(String),
}

#[derive(thiserror::Error, Debug)]
pub enum UpdateInstanceError {
    #[error("App {0} is not installed")]
    AppNotInstalled(AppKey),
    #[error("No manifest found for {0}")]
    NoManifest(AppKey),
    #[error("Instance {0} does not exist")]
    NotFound(InstanceId),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub async fn create_docker_instance(
    quest: SyncQuest,
    lore: Arc<Lore>,
    deployment: Arc<dyn DockerDeployment>,
    manifest: Arc<AppManifestSingle>,
    name: String,
    address: IpAddr,
    provider_config: ProviderConfig,
) -> Result<DockerInstance> {
    DockerInstance::try_create_new(
        quest,
        lore,
        deployment,
        manifest,
        name,
        address,
        provider_config,
    )
    .await
}

pub async fn create_compose_instance(
    quest: SyncQuest,
    lore: Arc<Lore>,
    deployment: Arc<dyn ComposeDeployment>,
    manifest: Arc<AppManifestMulti>,
    name: String,
) -> Result<ComposeInstance> {
    Ok(ComposeInstance::try_create_new(quest, lore, deployment, manifest, name).await?)
}

pub async fn start_instance<F: Floxy>(
    _quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
    instance_id: InstanceId,
) -> Result<()> {
    let mut grab = vault
        .reservation()
        .reserve_instance_pouch_mut()
        .grab()
        .await;
    let instance = grab
        .instance_pouch_mut
        .as_mut()
        .expect("Vault reservations should never fail")
        .gems_mut()
        .get_mut(&instance_id)
        .ok_or_else(|| anyhow::anyhow!("Instance {instance_id} does not exist"))?;
    match instance {
        Instance::Docker(instance) => instance.start(floxy).await,
        Instance::Compose(instance) => instance.start().await,
    }
}

pub async fn resume_instance<F: Floxy>(
    _quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
    instance_id: InstanceId,
) -> Result<()> {
    let grab = vault.reservation().reserve_instance_pouch().grab().await;
    let instance = grab
        .instance_pouch
        .as_ref()
        .expect("Vault reservations should never fail")
        .gems()
        .get(&instance_id)
        .ok_or_else(|| anyhow::anyhow!("Instance {instance_id} does not exist"))?;
    match instance {
        Instance::Docker(instance) => instance.resume(floxy).await,
        Instance::Compose(instance) => instance.resume().await,
    }
}

pub async fn stop_instance<F: Floxy>(
    _quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
    instance_id: InstanceId,
) -> Result<()> {
    let mut grab = vault
        .reservation()
        .reserve_instance_pouch_mut()
        .grab()
        .await;
    let instance = grab
        .instance_pouch_mut
        .as_mut()
        .expect("Vault reservations should never fail")
        .gems_mut()
        .get_mut(&instance_id)
        .ok_or_else(|| anyhow::anyhow!("Instance {instance_id} does not exist"))?;
    match instance {
        Instance::Docker(instance) => instance.stop(floxy).await,
        Instance::Compose(instance) => instance.stop().await,
    }
}

pub async fn stop_instances<F: Floxy + 'static>(
    quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
    instance_ids: Vec<InstanceId>,
) -> Result<()> {
    let mut results = Vec::new();
    {
        let mut quest = quest.lock().await;
        for instance_id in instance_ids {
            let result = quest
                .create_sub_quest(format!("Stop instance {instance_id}"), |quest| {
                    stop_instance(quest, vault.clone(), floxy.clone(), instance_id)
                })
                .await
                .2;
            results.push(result);
        }
    }
    for result in join_all(results).await {
        result?;
    }
    Ok(())
}

/// The same as [stop_instances] but all instance_ids not in the vault are ignored
pub async fn stop_existing_instances<F: Floxy + 'static>(
    quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
    instance_ids: Vec<InstanceId>,
) -> Result<()> {
    let instance_ids: Vec<_> = {
        let grab = vault.reservation().reserve_instance_pouch().grab().await;
        let instances = grab
            .instance_pouch
            .as_ref()
            .expect("Vault reservations should never fail");
        instance_ids
            .into_iter()
            .filter(|id| instances.gems().contains_key(id))
            .collect()
    };
    stop_instances(quest, vault, floxy, instance_ids).await
}

pub async fn halt_all_instances(quest: SyncQuest, vault: Arc<Vault>) -> Result<()> {
    let mut instances_to_halt = Vec::new();
    let mut halt_results = Vec::new();
    {
        let grab = vault.reservation().reserve_instance_pouch().grab().await;
        let instances = grab
            .instance_pouch
            .as_ref()
            .expect("Vault reservations should never fail");
        let mut quest = quest.lock().await;
        for (id, instance) in instances.gems() {
            match instance.status().await {
                Ok(InstanceStatus::Stopped) => {}
                _ => {
                    instances_to_halt.push(id);
                    let result = quest
                        .spawn_sub_quest(format!("Halt instance {id}"), |quest| {
                            halt_instance(quest, vault.clone(), *id)
                        })
                        .await
                        .2;
                    halt_results.push(result);
                }
            }
        }
    }
    join_all(halt_results)
        .await
        .into_iter()
        .try_for_each(|result| match result {
            Err(e) => Err(anyhow::anyhow!(e)),
            Ok(Err(e)) => Err(anyhow::anyhow!(e)),
            Ok(Ok(())) => Ok(()),
        })
}

pub async fn delete_all_floxy_server_configs<F: Floxy + 'static>(
    quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
) -> anyhow::Result<()> {
    let mut halt_results = Vec::new();
    {
        let grab = vault.reservation().reserve_instance_pouch().grab().await;
        let instances = grab
            .instance_pouch
            .as_ref()
            .expect("Vault reservations should never fail");
        let mut quest = quest.lock().await;
        for (id, instance) in instances.gems() {
            if let Instance::Docker(_) = instance {
                let result = quest
                    .spawn_sub_quest(format!("Halt instance {id}"), |quest| {
                        delete_floxy_server_configs(quest, vault.clone(), floxy.clone(), *id)
                    })
                    .await
                    .2;
                halt_results.push(result);
            }
        }
    }
    let fails = join_all(halt_results)
        .await
        .iter()
        .filter(|result| !matches!(result, Ok(Ok(_))))
        .count();
    anyhow::ensure!(
        fails == 0,
        "Failed to delete floxy server configs of {fails} instances"
    );
    Ok(())
}

async fn delete_floxy_server_configs<F: Floxy + 'static>(
    quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
    instance_id: InstanceId,
) -> anyhow::Result<()> {
    let mut grab = vault
        .reservation()
        .reserve_instance_pouch_mut()
        .grab()
        .await;
    let instances = grab
        .instance_pouch_mut
        .as_mut()
        .expect("Vault reservations should never fail");
    match instances.gems_mut().get_mut(&instance_id) {
        None => anyhow::bail!("Instance with id {instance_id} not found"),
        Some(Instance::Docker(instance)) => {
            instance.delete_server_proxy_configs(floxy)?;
        }
        _ => {
            let mut quest = quest.lock().await;
            quest.state = State::Skipped;
            quest.detail = Some("Compose instances have no floxy configs".to_string());
        }
    }
    Ok(())
}

pub async fn start_all_instances_as_desired<F: Floxy + 'static>(
    quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
) -> Result<()> {
    let mut instances_to_start = Vec::new();
    let mut start_results = Vec::new();
    {
        let grab = vault.reservation().reserve_instance_pouch().grab().await;
        let instances = grab
            .instance_pouch
            .as_ref()
            .expect("Vault reservations should never fail");
        let mut quest = quest.lock().await;

        for (id, _) in instances
            .gems()
            .iter()
            .filter(|(_, instance)| instance.desired_status() == InstanceStatus::Running)
        {
            instances_to_start.push(id);
            let result = quest
                .spawn_sub_quest(format!("Start instance {id}"), |quest| {
                    resume_instance(quest, vault.clone(), floxy.clone(), *id)
                })
                .await
                .2;
            start_results.push(result);
        }
    }
    join_all(start_results)
        .await
        .into_iter()
        .try_for_each(|result| match result {
            Err(e) => Err(anyhow::anyhow!(e)),
            Ok(Err(e)) => Err(anyhow::anyhow!(e)),
            Ok(Ok(())) => Ok(()),
        })
}

pub async fn halt_instance(
    _quest: SyncQuest,
    vault: Arc<Vault>,
    instance_id: InstanceId,
) -> Result<()> {
    let mut grab = vault.reservation().reserve_instance_pouch().grab().await;
    let instance = grab
        .instance_pouch
        .as_mut()
        .expect("Vault reservations should never fail")
        .gems()
        .get(&instance_id)
        .ok_or_else(|| anyhow::anyhow!("Instance {instance_id} does not exist"))?;
    instance.halt().await
}

pub async fn get_instances_info(
    quest: SyncQuest,
    vault: Arc<Vault>,
    instance_ids: Vec<InstanceId>,
) -> Vec<flecsd_axum_server::models::AppInstance> {
    let mut info_results = Vec::new();
    for instance_id in instance_ids.iter() {
        let result = quest
            .lock()
            .await
            .create_sub_quest(format!("Get info of instance {instance_id}"), |_quest| {
                get_instance_info(vault.clone(), *instance_id)
            })
            .await
            .2;
        info_results.push(result);
    }
    join_all(info_results)
        .await
        .into_iter()
        .zip(instance_ids)
        .filter_map(|(result, id)| match result {
            Ok(Some(info)) => Some(info),
            Ok(None) => {
                error!("Could not get info for instance {id}: Not found");
                None
            }
            Err(e) => {
                error!("Could not get info for instance {id}: {e}");
                None
            }
        })
        .collect()
}

pub async fn get_instance_info(
    vault: Arc<Vault>,
    instance_id: InstanceId,
) -> Result<Option<flecsd_axum_server::models::AppInstance>> {
    match vault
        .reservation()
        .reserve_instance_pouch()
        .grab()
        .await
        .instance_pouch
        .as_ref()
        .expect("Vault reservations should never fail")
        .gems()
        .get(&instance_id)
    {
        None => Ok(None),
        Some(instance) => Ok(Some(instance.generate_info().await?)),
    }
}

pub async fn get_instance_detailed_info(
    vault: Arc<Vault>,
    instance_id: InstanceId,
) -> Result<Option<flecsd_axum_server::models::InstancesInstanceIdGet200Response>> {
    match vault
        .reservation()
        .reserve_instance_pouch()
        .grab()
        .await
        .instance_pouch
        .as_ref()
        .expect("Vault reservations should never fail")
        .gems()
        .get(&instance_id)
    {
        None => Ok(None),
        Some(instance) => Ok(Some(instance.generate_detailed_info().await?)),
    }
}

pub async fn get_instance_ids_by_app_key(vault: Arc<Vault>, key: AppKey) -> Vec<InstanceId> {
    vault
        .reservation()
        .reserve_instance_pouch()
        .grab()
        .await
        .instance_pouch
        .as_ref()
        .expect("Vault reservations should never fail")
        .instance_ids_by_app_key(AppKey {
            name: key.name,
            version: key.version,
        })
}

pub async fn delete_instances<F: Floxy + 'static>(
    quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
    instance_ids: Vec<InstanceId>,
) -> Result<(), Vec<(Error, InstanceId)>> {
    if instance_ids.is_empty() {
        let mut quest = quest.lock().await;
        quest.state = State::Skipped;
        quest.detail = Some("No instances to remove".to_string());
    }
    let mut results = Vec::new();
    for instance_id in instance_ids.iter() {
        let result = quest
            .lock()
            .await
            .create_sub_quest(format!("Delete instance {instance_id}"), |quest| {
                delete_instance(quest, vault.clone(), floxy.clone(), *instance_id)
            })
            .await
            .2;
        results.push(result);
    }
    let errors: Vec<_> = join_all(results)
        .await
        .into_iter()
        .zip(instance_ids)
        .filter_map(|(result, id)| match result {
            Err(e) => Some((e, id)),
            _ => None,
        })
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn validate_no_dependents_in_slice(
    providers: &pouch::provider::Gems,
    instances: &pouch::instance::Gems,
    provider_ids: &[ProviderId],
) -> Result<()> {
    for provider_id in provider_ids {
        validate_no_dependents(providers, instances, *provider_id)?;
    }
    Ok(())
}

pub fn validate_no_dependents(
    providers: &pouch::provider::Gems,
    instances: &pouch::instance::Gems,
    provider_id: ProviderId,
) -> Result<()> {
    let mut usages: Vec<String> = providers
        .default_providers
        .iter()
        .filter_map(|(feature, id)| {
            if *id == provider_id {
                Some(format!("default provider for feature {feature}"))
            } else {
                None
            }
        })
        .collect();
    for (dependent_instance_id, instance) in instances {
        for dependency in instance.dependencies().values() {
            match dependency.provider_reference {
                ProviderReference::Provider(id) if id == provider_id => usages.push(format!(
                    "providing {} for instance {dependent_instance_id}",
                    dependency.provided_feature
                )),
                _ => {}
            }
        }
    }
    match providers.core_providers.auth {
        Some(ProviderReference::Provider(core_auth_provider))
            if core_auth_provider == provider_id =>
        {
            usages.push("core auth provider".to_string())
        }
        _ => {}
    }
    if usages.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Instance {provider_id} is used as a provider: {}",
            usages.join(", ")
        ))
    }
}

pub async fn delete_instance<F: Floxy + 'static>(
    quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
    id: InstanceId,
) -> Result<()> {
    let GrabbedPouches {
        instance_pouch_mut: Some(ref mut instances),
        provider_pouch: Some(ref providers),
        ..
    } = vault
        .reservation()
        .reserve_instance_pouch_mut()
        .reserve_provider_pouch()
        .grab()
        .await
    else {
        unreachable!("Reservation should never fail");
    };
    let providers = providers.gems();
    let instances = instances.gems_mut();
    validate_no_dependents(providers, instances, id)?;
    match instances.remove(&id) {
        Some(instance) => {
            let result = match instance {
                Instance::Docker(instance) => instance
                    .stop_and_delete(quest, floxy)
                    .await
                    .map_err(|(e, instance)| (e, Instance::Docker(instance))),
                Instance::Compose(instance) => instance
                    .stop_and_delete()
                    .await
                    .map_err(|(e, instance)| (e, Instance::Compose(instance))),
            };
            if let Err((e, instance)) = result {
                instances.insert(id, instance);
                Err(e)
            } else {
                Ok(())
            }
        }
        None => anyhow::bail!("Instance {id} not found"),
    }
}

pub async fn clear_ip_reservation(vault: Arc<Vault>, ip_addr: IpAddr) {
    vault
        .reservation()
        .reserve_instance_pouch_mut()
        .grab()
        .await
        .instance_pouch_mut
        .as_mut()
        .expect("Vault reservations should never fail")
        .clear_ip_address_reservation(ip_addr);
}

pub async fn make_ipv4_reservation(
    vault: Arc<Vault>,
    network: Ipv4NetworkAccess,
) -> Option<Ipv4Addr> {
    vault
        .reservation()
        .reserve_instance_pouch_mut()
        .grab()
        .await
        .instance_pouch_mut
        .as_mut()
        .expect("Vault reservations should never fail")
        .reserve_free_ipv4_address(network)
}

pub async fn make_auth_port_reservation(vault: Arc<Vault>) -> Option<u16> {
    vault
        .reservation()
        .reserve_instance_pouch_mut()
        .grab()
        .await
        .instance_pouch_mut
        .as_mut()
        .expect("Vault reservations should never fail")
        .reserve_auth_port()
}

pub async fn clear_auth_port_reservation(vault: Arc<Vault>, port: u16) -> bool {
    vault
        .reservation()
        .reserve_instance_pouch_mut()
        .grab()
        .await
        .instance_pouch_mut
        .as_mut()
        .expect("Vault reservations should never fail")
        .clear_reserved_provider_port(port)
}

#[derive(Debug, thiserror::Error)]
pub enum QueryInstanceConfigError {
    #[error("Instance {0} not found")]
    NotFound(InstanceId),
    #[error("Instance {0} does not support configuring")]
    NotSupported(InstanceId),
}

pub async fn modify_instance_config_with<F, T>(
    vault: Arc<Vault>,
    instance_id: InstanceId,
    with: F,
) -> Result<T, QueryInstanceConfigError>
where
    F: FnOnce(&mut InstanceConfig) -> T,
{
    match vault
        .reservation()
        .reserve_instance_pouch_mut()
        .grab()
        .await
        .instance_pouch_mut
        .as_mut()
        .expect("Reservations should never fail")
        .gems_mut()
        .get_mut(&instance_id)
    {
        None => Err(QueryInstanceConfigError::NotFound(instance_id)),
        Some(Instance::Compose(_)) => Err(QueryInstanceConfigError::NotSupported(instance_id)),
        Some(Instance::Docker(instance)) => Ok(with(&mut instance.config)),
    }
}

pub async fn get_instance_config_part_with<F, T>(
    vault: Arc<Vault>,
    instance_id: InstanceId,
    with: F,
) -> Result<T, QueryInstanceConfigError>
where
    F: FnOnce(&InstanceConfig) -> T,
{
    get_instance_config_part_with_from_gems(
        vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .expect("Reservations should never fail")
            .gems(),
        instance_id,
        with,
    )
}

pub fn get_instance_config_part_with_from_gems<F, T>(
    instances: &pouch::instance::Gems,
    instance_id: InstanceId,
    with: F,
) -> Result<T, QueryInstanceConfigError>
where
    F: FnOnce(&InstanceConfig) -> T,
{
    match instances.get(&instance_id) {
        None => Err(QueryInstanceConfigError::NotFound(instance_id)),
        Some(Instance::Compose(_)) => Err(QueryInstanceConfigError::NotSupported(instance_id)),
        Some(Instance::Docker(instance)) => Ok(with(&instance.config)),
    }
}

pub fn get_auth_provider_port(
    instances: &pouch::instance::Gems,
    provider_id: ProviderId,
) -> Result<u16, GetAuthProviderPortError> {
    get_instance_config_part_with_from_gems(instances, provider_id, |config| {
        Ok::<u16, GetAuthProviderPortError>(
            config
                .providers
                .auth
                .as_ref()
                .ok_or(GetAuthProviderPortError::DoesNotProvide { id: provider_id })?
                .port,
        )
    })?
}

pub async fn query_instance<F, T>(vault: Arc<Vault>, instance_id: InstanceId, f: F) -> Option<T>
where
    F: FnOnce(&Instance) -> T,
{
    Some(f(vault
        .reservation()
        .reserve_instance_pouch()
        .grab()
        .await
        .instance_pouch
        .as_ref()
        .expect("Reservations should never fail")
        .gems()
        .get(&instance_id)?))
}

pub async fn disconnect_instance_from_network(
    vault: Arc<Vault>,
    id: InstanceId,
    network_id: NetworkId,
) -> Result<IpAddr, DisconnectInstanceError> {
    let mut grab = vault
        .reservation()
        .reserve_instance_pouch_mut()
        .grab()
        .await;
    let instance = grab
        .instance_pouch_mut
        .as_mut()
        .expect("Reservations should never fail")
        .gems_mut()
        .get_mut(&id)
        .ok_or(DisconnectInstanceError::InstanceNotFound(id))?;
    let Instance::Docker(instance) = instance else {
        return Err(DisconnectInstanceError::Unsupported(id));
    };
    match instance.disconnect_network(network_id.clone()).await {
        Ok(Some(address)) => Ok(address),
        Ok(None) => Err(DisconnectInstanceError::InstanceNotConnected {
            instance: id,
            network: network_id,
        }),
        Err(e) => Err(DisconnectInstanceError::Other(e.to_string())),
    }
}

pub async fn update_instance<F: Floxy + 'static>(
    quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
    instance_id: InstanceId,
    new_version: AppKey,
    base_path: PathBuf,
) -> Result<(), UpdateInstanceError> {
    let mut grab = vault
        .reservation()
        .reserve_manifest_pouch()
        .reserve_instance_pouch_mut()
        .grab()
        .await;
    let Some(new_manifest) = grab
        .manifest_pouch
        .as_ref()
        .expect("Vault reservations should never fail")
        .gems()
        .get(&new_version)
        .cloned()
    else {
        return Err(UpdateInstanceError::NoManifest(new_version));
    };
    match grab
        .instance_pouch_mut
        .as_mut()
        .expect("Vault reservations should never fail")
        .gems_mut()
        .get_mut(&instance_id)
    {
        None => Err(UpdateInstanceError::NotFound(instance_id)),
        Some(instance) => {
            let base_path = base_path.join(instance_id.to_string());
            instance
                .update(quest, floxy, new_manifest, &base_path)
                .await?;
            Ok(())
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::enchantment::floxy::MockFloxy;
    use crate::jeweler::gem::deployment::Deployment;
    use crate::jeweler::gem::deployment::docker::tests::MockedDockerDeployment;
    use crate::jeweler::gem::manifest::single::EnvironmentVariable;
    use crate::quest::Quest;
    use crate::relic::network::Ipv4Network;
    use crate::vault;
    use crate::vault::pouch::instance::tests::{
        EDITOR_INSTANCE, ENV_INSTANCE, LABEL_INSTANCE, MINIMAL_INSTANCE, MOUNT_INSTANCE,
        NETWORK_INSTANCE, PORT_MAPPING_INSTANCE, RUNNING_INSTANCE, UNKNOWN_INSTANCE_1,
        UNKNOWN_INSTANCE_2, UNKNOWN_INSTANCE_3, USB_DEV_INSTANCE,
    };
    use mockall::predicate;
    use mockall::predicate::eq;
    use std::collections::HashMap;

    #[tokio::test]
    async fn get_instance_info_details_ok() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = vault::tests::create_test_vault(
            HashMap::from([(RUNNING_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        assert!(
            get_instance_detailed_info(vault, RUNNING_INSTANCE)
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn get_instance_info_details_not_found() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(
            get_instance_detailed_info(vault, UNKNOWN_INSTANCE_2)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn get_instance_info_details_err() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = vault::tests::create_test_vault(
            HashMap::from([(RUNNING_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        assert!(
            get_instance_detailed_info(vault, RUNNING_INSTANCE)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn get_instance_info_ok() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = vault::tests::create_test_vault(
            HashMap::from([(RUNNING_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        assert!(
            get_instance_info(vault, RUNNING_INSTANCE)
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn get_instance_info_not_found() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(
            get_instance_info(vault, UNKNOWN_INSTANCE_2)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn get_instance_info_err() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = vault::tests::create_test_vault(
            HashMap::from([(RUNNING_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        assert!(get_instance_info(vault, RUNNING_INSTANCE).await.is_err());
    }

    #[tokio::test]
    async fn get_instances_info_ok() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault =
            vault::tests::create_test_vault(HashMap::new(), HashMap::new(), Some(deployment));
        let instance_ids = vec![
            RUNNING_INSTANCE,
            PORT_MAPPING_INSTANCE,
            ENV_INSTANCE,
            EDITOR_INSTANCE,
        ];
        let instance_infos = get_instances_info(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            instance_ids.clone(),
        )
        .await;
        assert_eq!(instance_infos.len(), instance_ids.len());
        for instance_id in instance_ids {
            assert!(
                instance_infos
                    .iter()
                    .any(|instance| instance.instance_id == instance_id.to_string())
            );
        }
    }

    #[tokio::test]
    async fn get_instances_info_part_not_found() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault =
            vault::tests::create_test_vault(HashMap::new(), HashMap::new(), Some(deployment));
        let unknown_instance_ids = [UNKNOWN_INSTANCE_1, UNKNOWN_INSTANCE_2];
        let known_instance_ids = [MINIMAL_INSTANCE, LABEL_INSTANCE, USB_DEV_INSTANCE];
        let instance_infos = get_instances_info(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            unknown_instance_ids
                .iter()
                .chain(known_instance_ids.iter())
                .cloned()
                .collect(),
        )
        .await;
        assert_eq!(instance_infos.len(), known_instance_ids.len());
        for known_instance_id in known_instance_ids {
            assert!(
                instance_infos
                    .iter()
                    .any(|instance| instance.instance_id == known_instance_id.to_string())
            );
        }
        for unknown_instance_id in unknown_instance_ids {
            assert!(
                !instance_infos
                    .iter()
                    .any(|instance| instance.instance_id == unknown_instance_id.to_string())
            );
        }
    }

    #[tokio::test]
    async fn get_instances_info_err() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault =
            vault::tests::create_test_vault(HashMap::new(), HashMap::new(), Some(deployment));
        let instance_ids = vec![MINIMAL_INSTANCE, LABEL_INSTANCE, USB_DEV_INSTANCE];
        let instance_infos = get_instances_info(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            instance_ids,
        )
        .await;
        assert!(instance_infos.is_empty());
    }

    #[tokio::test]
    async fn start_instance_ok() {
        let mut deployment = MockedDockerDeployment::new();
        deployment.expect_core_default_address().returning(|_| None);
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_deployment_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .once()
            .with(eq(RUNNING_INSTANCE))
            .returning(|_| Ok(InstanceStatus::Stopped));
        deployment
            .expect_start_instance()
            .once()
            .withf(|_, _, id, _| *id == Some(RUNNING_INSTANCE))
            .returning(|_, _, _, _| Ok(RUNNING_INSTANCE));
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = vault::tests::create_test_vault(
            HashMap::from([(RUNNING_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_additional_locations_proxy_config()
            .returning(|_, _| Ok(false));
        let floxy = FloxyOperation::new_arc(Arc::new(floxy));
        start_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            floxy,
            RUNNING_INSTANCE,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn start_instance_err() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_deployment_id()
            .return_const("MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .once()
            .with(eq(RUNNING_INSTANCE))
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let deployment = Deployment::Docker(Arc::new(deployment));
        let vault = vault::tests::create_test_vault(
            HashMap::from([(RUNNING_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert!(
            start_instance(
                Quest::new_synced("TestQuest".to_string()),
                vault,
                floxy,
                RUNNING_INSTANCE,
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn start_instance_not_found() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert!(
            start_instance(
                Quest::new_synced("TestQuest".to_string()),
                vault,
                floxy,
                UNKNOWN_INSTANCE_1,
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn stop_instance_not_found() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert!(
            stop_instance(
                Quest::new_synced("TestQuest".to_string()),
                vault,
                floxy,
                UNKNOWN_INSTANCE_1,
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn make_ip_reservation_test() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let network = Ipv4NetworkAccess::try_new(
            Ipv4Network::try_new(Ipv4Addr::new(10, 18, 102, 0), 24).unwrap(),
            Ipv4Addr::new(10, 18, 102, 1),
        )
        .unwrap();
        assert_eq!(
            make_ipv4_reservation(vault, network).await,
            Some(Ipv4Addr::new(10, 18, 102, 2)),
        );
    }

    #[tokio::test]
    async fn clear_ip_reservation_test() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let network = Ipv4NetworkAccess::try_new(
            Ipv4Network::try_new(Ipv4Addr::new(10, 18, 102, 0), 24).unwrap(),
            Ipv4Addr::new(10, 18, 102, 1),
        )
        .unwrap();
        assert_eq!(
            vault
                .reservation()
                .reserve_instance_pouch_mut()
                .grab()
                .await
                .instance_pouch_mut
                .as_mut()
                .unwrap()
                .reserve_free_ipv4_address(network),
            Some(Ipv4Addr::new(10, 18, 102, 2)),
        );
        assert_eq!(
            vault
                .reservation()
                .reserve_instance_pouch_mut()
                .grab()
                .await
                .instance_pouch_mut
                .as_mut()
                .unwrap()
                .reserve_free_ipv4_address(network),
            Some(Ipv4Addr::new(10, 18, 102, 3)),
        );
        clear_ip_reservation(vault.clone(), IpAddr::V4(Ipv4Addr::new(10, 18, 102, 2))).await;
        assert_eq!(
            vault
                .reservation()
                .reserve_instance_pouch_mut()
                .grab()
                .await
                .instance_pouch_mut
                .as_mut()
                .unwrap()
                .reserve_free_ipv4_address(network),
            Some(Ipv4Addr::new(10, 18, 102, 2)),
        );
    }

    #[tokio::test]
    async fn modify_instance_config_with_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(matches!(
            modify_instance_config_with(vault, UNKNOWN_INSTANCE_3, |_| true).await,
            Err(QueryInstanceConfigError::NotFound(UNKNOWN_INSTANCE_3))
        ));
    }

    #[tokio::test]
    async fn modify_instance_config_with_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let test_env_var = EnvironmentVariable {
            name: "TestVar".to_string(),
            value: None,
        };
        assert_eq!(
            modify_instance_config_with(vault.clone(), RUNNING_INSTANCE, |config| {
                config.environment_variables.push(test_env_var.clone());
                "test_value"
            })
            .await
            .unwrap(),
            "test_value"
        );
        let grab = vault.reservation().reserve_instance_pouch().grab().await;
        let Some(Instance::Docker(instance)) = grab
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .get(&RUNNING_INSTANCE)
        else {
            panic!()
        };
        assert_eq!(instance.config.environment_variables, vec![test_env_var])
    }

    #[tokio::test]
    async fn get_instance_config_part_with_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(matches!(
            get_instance_config_part_with(vault, UNKNOWN_INSTANCE_1, |_| true).await,
            Err(QueryInstanceConfigError::NotFound(UNKNOWN_INSTANCE_1))
        ));
    }

    #[tokio::test]
    async fn get_instance_config_part_with_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let test_env_var = EnvironmentVariable {
            name: "TestVar".to_string(),
            value: None,
        };
        {
            let mut grab = vault
                .reservation()
                .reserve_instance_pouch_mut()
                .grab()
                .await;
            let instance_pouch = grab.instance_pouch_mut.as_mut().unwrap();
            let Some(Instance::Docker(instance)) =
                instance_pouch.gems_mut().get_mut(&RUNNING_INSTANCE)
            else {
                panic!()
            };
            instance
                .config
                .environment_variables
                .push(test_env_var.clone());
        }
        assert_eq!(
            get_instance_config_part_with(vault.clone(), RUNNING_INSTANCE, |config| {
                config.environment_variables.clone()
            })
            .await
            .unwrap(),
            vec![test_env_var]
        );
    }

    #[tokio::test]
    async fn start_all_instances_as_desired_ok() {
        const INSTANCES_TO_START: [InstanceId; 8] = [
            RUNNING_INSTANCE,
            PORT_MAPPING_INSTANCE,
            LABEL_INSTANCE,
            ENV_INSTANCE,
            USB_DEV_INSTANCE,
            EDITOR_INSTANCE,
            NETWORK_INSTANCE,
            MOUNT_INSTANCE,
        ];
        const RUNNING_INSTANCES: [InstanceId; 3] =
            [RUNNING_INSTANCE, EDITOR_INSTANCE, NETWORK_INSTANCE];
        let instance_deployments = HashMap::from_iter(INSTANCES_TO_START.map(|instance_id| {
            let mut deployment = MockedDockerDeployment::new();
            deployment.expect_core_default_address().returning(|_| None);
            deployment
                .expect_id()
                .return_const(format!("MockDeployment-{instance_id}"));
            deployment
                .expect_deployment_id()
                .return_const(format!("MockDeployment-{instance_id}"));
            deployment
                .expect_instance_status()
                .once()
                .with(predicate::eq(instance_id))
                .returning(|instance_id| {
                    if RUNNING_INSTANCES.contains(&instance_id) {
                        Ok(InstanceStatus::Running)
                    } else {
                        Ok(InstanceStatus::Stopped)
                    }
                });
            if !RUNNING_INSTANCES.contains(&instance_id) {
                deployment
                    .expect_start_instance()
                    .once()
                    .with(
                        predicate::always(),
                        predicate::always(),
                        predicate::eq(Some(instance_id)),
                        predicate::always(),
                    )
                    .returning(move |_, _, _, _| Ok(instance_id));
            }
            let deployment = Deployment::Docker(Arc::new(deployment));
            (instance_id, deployment)
        }));
        let vault = vault::tests::create_test_vault(instance_deployments, HashMap::new(), None);
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_additional_locations_proxy_config()
            .returning(|_, _| Ok(false));
        let floxy = FloxyOperation::new_arc(Arc::new(floxy));
        start_all_instances_as_desired(Quest::new_synced("TestQuest".to_string()), vault, floxy)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn start_all_instances_as_desired_err() {
        const INSTANCES_TO_START: [InstanceId; 8] = [
            RUNNING_INSTANCE,
            PORT_MAPPING_INSTANCE,
            LABEL_INSTANCE,
            ENV_INSTANCE,
            USB_DEV_INSTANCE,
            EDITOR_INSTANCE,
            NETWORK_INSTANCE,
            MOUNT_INSTANCE,
        ];
        const RUNNING_INSTANCES: [InstanceId; 3] =
            [RUNNING_INSTANCE, EDITOR_INSTANCE, NETWORK_INSTANCE];
        const ERROR_INSTANCE: InstanceId = PORT_MAPPING_INSTANCE;
        let instance_deployments = HashMap::from_iter(INSTANCES_TO_START.map(|instance_id| {
            let mut deployment = MockedDockerDeployment::new();
            deployment.expect_core_default_address().returning(|_| None);
            deployment
                .expect_id()
                .return_const(format!("MockDeployment-{instance_id}"));
            deployment
                .expect_deployment_id()
                .return_const(format!("MockDeployment-{instance_id}"));
            deployment
                .expect_instance_status()
                .once()
                .with(predicate::eq(instance_id))
                .returning(|instance_id| {
                    if RUNNING_INSTANCES.contains(&instance_id) {
                        Ok(InstanceStatus::Running)
                    } else {
                        Ok(InstanceStatus::Stopped)
                    }
                });
            if !RUNNING_INSTANCES.contains(&instance_id) {
                deployment
                    .expect_start_instance()
                    .once()
                    .with(
                        predicate::always(),
                        predicate::always(),
                        predicate::eq(Some(instance_id)),
                        predicate::always(),
                    )
                    .returning(move |_, _, _, _| {
                        if instance_id == ERROR_INSTANCE {
                            Err(anyhow::anyhow!("TestError"))
                        } else {
                            Ok(instance_id)
                        }
                    });
            }
            let deployment = Deployment::Docker(Arc::new(deployment));
            (instance_id, deployment)
        }));
        let vault = vault::tests::create_test_vault(instance_deployments, HashMap::new(), None);
        let mut floxy = MockFloxy::new();
        floxy
            .expect_delete_additional_locations_proxy_config()
            .returning(|_, _| Ok(false));
        let floxy = FloxyOperation::new_arc(Arc::new(floxy));
        assert!(
            start_all_instances_as_desired(
                Quest::new_synced("TestQuest".to_string()),
                vault,
                floxy
            )
            .await
            .is_err()
        );
    }
}
