pub use super::{Error, Result};
use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::jeweler::deployment::Deployment;
use crate::jeweler::gem::instance::{Instance, InstanceConfig, InstanceId};
use crate::jeweler::gem::manifest::AppManifest;
use crate::jeweler::network::NetworkId;
use crate::quest::{State, SyncQuest};
use crate::relic::network::Ipv4NetworkAccess;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::Vault;
use futures_util::future::join_all;
use std::net::{IpAddr, Ipv4Addr};
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
    #[error("Failed to disconnect instance: {0}")]
    Other(String),
}

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

pub async fn halt_all_instances<F: Floxy + 'static>(
    quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
) -> Result<()> {
    let mut instances_to_halt = Vec::new();
    let mut halt_results = Vec::new();
    {
        let mut grab = vault
            .reservation()
            .reserve_instance_pouch_mut()
            .grab()
            .await;
        let instances = grab
            .instance_pouch_mut
            .as_mut()
            .expect("Vault reservations should never fail");
        let mut quest = quest.lock().await;
        for (id, instance) in instances.gems() {
            match instance.is_running().await {
                Ok(false) => {}
                _ => {
                    instances_to_halt.push(id);
                    let result = quest
                        .create_sub_quest(format!("Halt instance {id}"), |quest| {
                            halt_instance(quest, vault.clone(), floxy.clone(), *id)
                        })
                        .await
                        .2;
                    halt_results.push(result);
                }
            }
        }
    }
    join_all(halt_results).await.into_iter().collect()
}

pub async fn halt_instance<F: Floxy>(
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
    instance.halt(floxy).await
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
    match instance.disconnect_network(network_id.clone()).await {
        Ok(Some(address)) => Ok(address),
        Ok(None) => Err(DisconnectInstanceError::InstanceNotConnected {
            instance: id,
            network: network_id,
        }),
        Err(e) => Err(DisconnectInstanceError::Other(e.to_string())),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::enchantment::floxy::MockFloxy;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::jeweler::gem::instance::InstanceStatus;
    use crate::jeweler::gem::manifest::EnvironmentVariable;
    use crate::quest::Quest;
    use crate::relic::network::Ipv4Network;
    use crate::vault;
    use crate::vault::pouch::instance::tests::{
        EDITOR_INSTANCE, ENV_INSTANCE, LABEL_INSTANCE, MINIMAL_INSTANCE, PORT_MAPPING_INSTANCE,
        RUNNING_INSTANCE, UNKNOWN_INSTANCE_1, UNKNOWN_INSTANCE_2, UNKNOWN_INSTANCE_3,
        USB_DEV_INSTANCE,
    };
    use mockall::predicate::eq;
    use std::collections::HashMap;

    #[tokio::test]
    async fn get_instance_info_details_ok() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(move || "MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let vault = vault::tests::create_test_vault(
            HashMap::from([(RUNNING_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        assert!(get_instance_detailed_info(vault, RUNNING_INSTANCE)
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn get_instance_info_details_not_found() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(get_instance_detailed_info(vault, UNKNOWN_INSTANCE_2)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn get_instance_info_details_err() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(move || "MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let vault = vault::tests::create_test_vault(
            HashMap::from([(RUNNING_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        assert!(get_instance_detailed_info(vault, RUNNING_INSTANCE)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn get_instance_info_ok() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(move || "MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let vault = vault::tests::create_test_vault(
            HashMap::from([(RUNNING_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        assert!(get_instance_info(vault, RUNNING_INSTANCE)
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn get_instance_info_not_found() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(get_instance_info(vault, UNKNOWN_INSTANCE_2)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn get_instance_info_err() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(move || "MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let vault = vault::tests::create_test_vault(
            HashMap::from([(RUNNING_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        assert!(get_instance_info(vault, RUNNING_INSTANCE).await.is_err());
    }

    #[tokio::test]
    async fn get_instances_info_ok() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(move || "MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
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
            assert!(instance_infos
                .iter()
                .any(|instance| instance.instance_id == instance_id.to_string()));
        }
    }

    #[tokio::test]
    async fn get_instances_info_part_not_found() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(move || "MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
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
            assert!(instance_infos
                .iter()
                .any(|instance| instance.instance_id == known_instance_id.to_string()));
        }
        for unknown_instance_id in unknown_instance_ids {
            assert!(!instance_infos
                .iter()
                .any(|instance| instance.instance_id == unknown_instance_id.to_string()));
        }
    }

    #[tokio::test]
    async fn get_instances_info_err() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(move || "MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
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
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(move || "MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .once()
            .with(eq(RUNNING_INSTANCE))
            .returning(|_| Ok(InstanceStatus::Stopped));
        deployment
            .expect_start_instance()
            .once()
            .withf(|_, id, _| *id == Some(RUNNING_INSTANCE))
            .returning(|_, _, _| Ok(RUNNING_INSTANCE));
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let vault = vault::tests::create_test_vault(
            HashMap::from([(RUNNING_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
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
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(move || "MockedDeployment".to_string());
        deployment
            .expect_instance_status()
            .once()
            .with(eq(RUNNING_INSTANCE))
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let vault = vault::tests::create_test_vault(
            HashMap::from([(RUNNING_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert!(start_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            floxy,
            RUNNING_INSTANCE,
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn start_instance_not_found() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert!(start_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            floxy,
            UNKNOWN_INSTANCE_1,
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn stop_instance_not_found() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert!(stop_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            floxy,
            UNKNOWN_INSTANCE_1,
        )
        .await
        .is_err());
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
        assert!(
            modify_instance_config_with(vault, UNKNOWN_INSTANCE_3, |_| true)
                .await
                .is_none()
        );
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
            .await,
            Some("test_value")
        );
        let grab = vault.reservation().reserve_instance_pouch().grab().await;
        assert_eq!(
            grab.instance_pouch
                .as_ref()
                .unwrap()
                .gems()
                .get(&RUNNING_INSTANCE)
                .unwrap()
                .config
                .environment_variables,
            vec![test_env_var]
        )
    }

    #[tokio::test]
    async fn get_instance_config_part_with_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(
            get_instance_config_part_with(vault, UNKNOWN_INSTANCE_1, |_| true)
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn get_instance_config_part_with_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
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
            .get_mut(&RUNNING_INSTANCE)
            .unwrap()
            .config
            .environment_variables
            .push(test_env_var.clone());
        assert_eq!(
            get_instance_config_part_with(vault.clone(), RUNNING_INSTANCE, |config| {
                config.environment_variables.clone()
            })
            .await,
            Some(vec![test_env_var])
        );
    }
}
