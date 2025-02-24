pub use super::{Error, Result};
use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::jeweler::deployment::Deployment;
use crate::jeweler::gem::instance::{Instance, InstanceConfig, InstanceId};
use crate::jeweler::gem::manifest::AppManifest;
use crate::quest::{State, SyncQuest};
use crate::relic::network::Ipv4NetworkAccess;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::Vault;
use futures_util::future::join_all;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use tracing::error;

pub async fn create_instance(
    quest: SyncQuest,
    deployment: Arc<dyn Deployment>,
    manifest: Arc<AppManifest>,
    name: String,
    address: IpAddr,
) -> Result<Instance> {
    Instance::create(quest, deployment, manifest, name, address).await
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
    instance.start(floxy).await
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
    instance.stop(floxy).await
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

pub async fn delete_instance<F: Floxy + 'static>(
    quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
    id: InstanceId,
) -> Result<()> {
    let mut grab = vault
        .reservation()
        .reserve_instance_pouch_mut()
        .grab()
        .await;
    let instances = grab
        .instance_pouch_mut
        .as_mut()
        .expect("Reservations should never fail")
        .gems_mut();
    match instances.remove(&id) {
        Some(instance) => {
            if let Err((e, instance)) = instance.stop_and_delete(quest, floxy).await {
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

pub async fn modify_instance_config_with<F, T>(
    vault: Arc<Vault>,
    instance_id: InstanceId,
    with: F,
) -> Option<T>
where
    F: FnOnce(&mut InstanceConfig) -> T,
{
    Some(with(
        &mut vault
            .reservation()
            .reserve_instance_pouch_mut()
            .grab()
            .await
            .instance_pouch_mut
            .as_mut()
            .expect("Reservations should never fail")
            .gems_mut()
            .get_mut(&instance_id)?
            .config,
    ))
}

pub async fn get_instance_config_part_with<F, T>(
    vault: Arc<Vault>,
    instance_id: InstanceId,
    with: F,
) -> Option<T>
where
    F: FnOnce(&InstanceConfig) -> T,
{
    Some(with(
        &vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .expect("Reservations should never fail")
            .gems()
            .get(&instance_id)?
            .config,
    ))
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::enchantment::floxy::MockFloxy;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::jeweler::gem::instance::{
        try_create_instance, InstanceDeserializable, InstanceStatus,
    };
    use crate::jeweler::gem::manifest::EnvironmentVariable;
    use crate::quest::Quest;
    use crate::relic::network::Ipv4Network;
    use crate::tests::prepare_test_path;
    use crate::vault::pouch::deployment::DeploymentId;
    use crate::vault::pouch::instance::tests::{
        create_manifest_for_instance, create_test_instances_deserializable,
    };
    use crate::vault::pouch::AppKey;
    use crate::vault::{GrabbedPouches, VaultConfig};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_deployment_for_instance(
        instance: &InstanceDeserializable,
        instance_status_return: Option<bool>,
    ) -> (DeploymentId, Arc<dyn Deployment>) {
        let mut deployment = MockedDeployment::new();
        let deployment_id = instance.deployment_id.clone();
        match instance_status_return {
            Some(true) => {
                deployment
                    .expect_instance_status()
                    .returning(|_| Ok(InstanceStatus::Running));
            }
            Some(false) => {
                deployment
                    .expect_instance_status()
                    .returning(|_| Err(anyhow::anyhow!("TestError")));
            }
            _ => {
                deployment
                    .expect_instance_status()
                    .returning(|_| Ok(InstanceStatus::Stopped));
            }
        }
        deployment.expect_default_network().returning(|| {
            Ok(bollard::models::Network {
                name: Some("flecs".to_string()),
                id: Some("flecs".to_string()),
                ..Default::default()
            })
        });
        deployment
            .expect_id()
            .returning(move || deployment_id.clone());
        (
            instance.deployment_id.clone(),
            Arc::new(deployment) as Arc<dyn Deployment>,
        )
    }

    pub async fn create_test_vault(
        path: PathBuf,
        instance_status_return: Option<bool>,
    ) -> Arc<Vault> {
        let vault = Arc::new(Vault::new(VaultConfig { path }));
        let instances = create_test_instances_deserializable();
        let manifests = instances
            .iter()
            .map(create_manifest_for_instance)
            .collect::<HashMap<AppKey, Arc<AppManifest>>>();
        let deployments = instances
            .iter()
            .map(|instance| create_deployment_for_instance(instance, instance_status_return))
            .collect::<HashMap<DeploymentId, Arc<dyn Deployment>>>();
        let instances: Vec<Instance> = instances
            .into_iter()
            .map(|instance| try_create_instance(instance, &manifests, &deployments).unwrap())
            .collect();
        if let GrabbedPouches {
            deployment_pouch_mut: Some(ref mut deployment_pouch),
            manifest_pouch_mut: Some(ref mut manifest_pouch),
            instance_pouch_mut: Some(ref mut instance_pouch),
            ..
        } = vault
            .reservation()
            .reserve_deployment_pouch_mut()
            .reserve_manifest_pouch_mut()
            .reserve_instance_pouch_mut()
            .grab()
            .await
        {
            for instance in instances {
                instance_pouch.gems_mut().insert(instance.id, instance);
            }
            *manifest_pouch.gems_mut() = manifests;
            *deployment_pouch.gems_mut() = deployments;
        } else {
            unreachable!("Vault reservations should never fail")
        };
        vault
    }

    #[tokio::test]
    async fn get_instance_info_details_ok() {
        let vault = create_test_vault(
            prepare_test_path(module_path!(), "get_instance_info_details_ok"),
            Some(true),
        )
        .await;
        assert!(get_instance_detailed_info(vault, InstanceId::new(1))
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn get_instance_info_details_not_found() {
        let vault = create_test_vault(
            prepare_test_path(module_path!(), "get_instance_info_details_not_found"),
            None,
        )
        .await;
        assert!(get_instance_detailed_info(vault, InstanceId::new(100))
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn get_instance_info_details_err() {
        let vault = create_test_vault(
            prepare_test_path(module_path!(), "get_instance_info_details_err"),
            Some(false),
        )
        .await;
        assert!(get_instance_detailed_info(vault, InstanceId::new(1))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn get_instance_info_ok() {
        let vault = create_test_vault(
            prepare_test_path(module_path!(), "get_instance_info_ok"),
            Some(true),
        )
        .await;
        assert!(get_instance_info(vault, InstanceId::new(1))
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn get_instance_info_not_found() {
        let vault = create_test_vault(
            prepare_test_path(module_path!(), "get_instance_info_not_found"),
            None,
        )
        .await;
        assert!(get_instance_info(vault, InstanceId::new(100))
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn get_instance_info_err() {
        let vault = create_test_vault(
            prepare_test_path(module_path!(), "get_instance_info_err"),
            Some(false),
        )
        .await;
        assert!(get_instance_info(vault, InstanceId::new(1)).await.is_err());
    }

    #[tokio::test]
    async fn get_instances_info_ok() {
        let vault = create_test_vault(
            prepare_test_path(module_path!(), "get_instances_info_ok"),
            Some(true),
        )
        .await;
        let instance_ids: Vec<InstanceId> = (1..=6).map(Into::into).collect();
        let instance_infos = get_instances_info(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            instance_ids.clone(),
        )
        .await;
        assert_eq!(instance_infos.len(), 6);
        for instance_id in instance_ids {
            assert!(instance_infos
                .iter()
                .any(|instance| instance.instance_id == instance_id.to_string()));
        }
    }

    #[tokio::test]
    async fn get_instances_info_part_not_found() {
        let vault = create_test_vault(
            prepare_test_path(module_path!(), "get_instances_info_part_not_found"),
            Some(true),
        )
        .await;
        let instance_ids: Vec<InstanceId> = (4..=9).map(Into::into).collect();
        let instance_infos = get_instances_info(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            instance_ids.clone(),
        )
        .await;
        assert_eq!(instance_infos.len(), 3);
        for instance_id in (4..=6).map(Into::<InstanceId>::into) {
            assert!(instance_infos
                .iter()
                .any(|instance| instance.instance_id == instance_id.to_string()));
        }
    }

    #[tokio::test]
    async fn get_instances_info_err() {
        let vault = create_test_vault(
            prepare_test_path(module_path!(), "get_instances_info_err"),
            Some(false),
        )
        .await;
        let instance_ids: Vec<InstanceId> = (4..=9).map(Into::into).collect();
        let instance_infos = get_instances_info(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            instance_ids.clone(),
        )
        .await;
        assert!(instance_infos.is_empty());
    }

    #[tokio::test]
    async fn start_instance_ok() {
        let path = prepare_test_path(module_path!(), "start_instance_ok");
        let vault = create_test_vault(path.join("vault"), Some(true)).await;
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        start_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            floxy,
            InstanceId::new(1),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn start_instance_err() {
        let path = prepare_test_path(module_path!(), "start_instance_err");
        let vault = create_test_vault(path.join("vault"), Some(false)).await;
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert!(start_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            floxy,
            InstanceId::new(1),
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn start_instance_not_found() {
        let path = prepare_test_path(module_path!(), "start_instance_not_found");
        let vault = create_test_vault(path.join("vault"), Some(true)).await;
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert!(start_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            floxy,
            InstanceId::new(10),
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn stop_instance_not_found() {
        let vault = create_test_vault(
            prepare_test_path(module_path!(), "stop_instance_not_found"),
            Some(true),
        )
        .await;
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert!(stop_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            floxy,
            InstanceId::new(10),
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn make_ip_reservation_test() {
        let vault = create_test_vault(
            prepare_test_path(module_path!(), "make_ip_reservation_test"),
            Some(true),
        )
        .await;
        let network = Ipv4NetworkAccess::try_new(
            Ipv4Network::try_new(Ipv4Addr::new(10, 18, 102, 0), 24).unwrap(),
            Ipv4Addr::new(10, 18, 102, 2),
        )
        .unwrap();
        assert_eq!(
            make_ipv4_reservation(vault, network).await,
            Some(Ipv4Addr::new(10, 18, 102, 3)),
        );
    }

    #[tokio::test]
    async fn clear_ip_reservation_test() {
        let vault = create_test_vault(
            prepare_test_path(module_path!(), "clear_ip_reservation_test"),
            Some(true),
        )
        .await;
        let network = Ipv4NetworkAccess::try_new(
            Ipv4Network::try_new(Ipv4Addr::new(10, 18, 102, 0), 24).unwrap(),
            Ipv4Addr::new(10, 18, 102, 2),
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
            Some(Ipv4Addr::new(10, 18, 102, 3)),
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
            Some(Ipv4Addr::new(10, 18, 102, 4)),
        );
        clear_ip_reservation(vault.clone(), IpAddr::V4(Ipv4Addr::new(10, 18, 102, 3))).await;
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
    }

    #[tokio::test]
    async fn modify_instance_config_with_none() {
        let vault = create_test_vault(
            prepare_test_path(module_path!(), "modify_instance_config_with_none"),
            None,
        )
        .await;
        assert!(
            modify_instance_config_with(vault, InstanceId::new(10000), |_| true)
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn modify_instance_config_with_some() {
        let vault = create_test_vault(
            prepare_test_path(module_path!(), "modify_instance_config_with_some"),
            None,
        )
        .await;
        let test_env_var = EnvironmentVariable {
            name: "TestVar".to_string(),
            value: None,
        };
        assert_eq!(
            modify_instance_config_with(vault.clone(), InstanceId::new(1), |config| {
                config.environment_variables.push(test_env_var.clone());
                "test_value"
            })
            .await,
            Some("test_value")
        );
        let grab = vault.reservation().reserve_instance_pouch().grab().await;
        assert_eq!(
            grab.instance_pouch
                .as_ref()
                .unwrap()
                .gems()
                .get(&InstanceId::new(1))
                .unwrap()
                .config
                .environment_variables,
            vec![test_env_var]
        )
    }

    #[tokio::test]
    async fn get_instance_config_part_with_none() {
        let vault = create_test_vault(
            prepare_test_path(module_path!(), "get_instance_config_part_with_none"),
            None,
        )
        .await;
        assert!(
            get_instance_config_part_with(vault, InstanceId::new(10000), |_| true)
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn get_instance_config_part_with_some() {
        let vault = create_test_vault(
            prepare_test_path(module_path!(), "get_instance_config_part_with_some"),
            None,
        )
        .await;
        let test_env_var = EnvironmentVariable {
            name: "TestVar".to_string(),
            value: None,
        };
        vault
            .reservation()
            .reserve_instance_pouch_mut()
            .grab()
            .await
            .instance_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut()
            .get_mut(&InstanceId::new(1))
            .unwrap()
            .config
            .environment_variables
            .push(test_env_var.clone());
        assert_eq!(
            get_instance_config_part_with(vault.clone(), InstanceId::new(1), |config| {
                config.environment_variables.clone()
            })
            .await,
            Some(vec![test_env_var])
        );
    }
}
