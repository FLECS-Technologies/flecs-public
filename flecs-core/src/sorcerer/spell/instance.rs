pub use super::Result;
use crate::jeweler::deployment::Deployment;
use crate::jeweler::gem::instance::{Instance, InstanceId};
use crate::jeweler::gem::manifest::AppManifest;
use crate::quest::SyncQuest;
use crate::vault::pouch::Pouch;
use crate::vault::Vault;
use futures_util::future::join_all;
use std::sync::Arc;
use tracing::error;

pub async fn create_instance(
    quest: SyncQuest,
    deployment: Arc<dyn Deployment>,
    manifest: Arc<AppManifest>,
    name: String,
) -> Result<Instance> {
    Instance::create(quest, deployment, manifest, name).await
}

pub async fn start_instance(
    _quest: SyncQuest,
    vault: Arc<Vault>,
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
    instance.start().await
}

pub async fn stop_instance(
    _quest: SyncQuest,
    vault: Arc<Vault>,
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
    instance.stop().await
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

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::jeweler::gem::instance::{
        try_create_instance, InstanceDeserializable, InstanceStatus,
    };
    use crate::quest::Quest;
    use crate::tests::prepare_test_path;
    use crate::vault::pouch::deployment::DeploymentId;
    use crate::vault::pouch::instance::tests::{
        create_manifest_for_instance, create_test_instances_deserializable,
    };
    use crate::vault::pouch::AppKey;
    use crate::vault::{GrabbedPouches, VaultConfig};
    use std::collections::HashMap;

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
            _ => {}
        }
        deployment
            .expect_id()
            .returning(move || deployment_id.clone());
        (
            instance.deployment_id.clone(),
            Arc::new(deployment) as Arc<dyn Deployment>,
        )
    }

    pub async fn create_test_vault(
        module: &str,
        test_name: &str,
        instance_status_return: Option<bool>,
    ) -> Arc<Vault> {
        let path = prepare_test_path(module, test_name);
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
        let vault =
            create_test_vault(module_path!(), "get_instance_info_details_ok", Some(true)).await;
        assert!(get_instance_detailed_info(vault, InstanceId::new(1))
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn get_instance_info_details_not_found() {
        let vault =
            create_test_vault(module_path!(), "get_instance_info_details_not_found", None).await;
        assert!(get_instance_detailed_info(vault, InstanceId::new(100))
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn get_instance_info_details_err() {
        let vault =
            create_test_vault(module_path!(), "get_instance_info_details_err", Some(false)).await;
        assert!(get_instance_detailed_info(vault, InstanceId::new(1))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn get_instance_info_ok() {
        let vault = create_test_vault(module_path!(), "get_instance_info_ok", Some(true)).await;
        assert!(get_instance_info(vault, InstanceId::new(1))
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn get_instance_info_not_found() {
        let vault = create_test_vault(module_path!(), "get_instance_info_not_found", None).await;
        assert!(get_instance_info(vault, InstanceId::new(100))
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn get_instance_info_err() {
        let vault = create_test_vault(module_path!(), "get_instance_info_err", Some(false)).await;
        assert!(get_instance_info(vault, InstanceId::new(1)).await.is_err());
    }

    #[tokio::test]
    async fn get_instances_info_ok() {
        let vault = create_test_vault(module_path!(), "get_instances_info_ok", Some(true)).await;
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
            module_path!(),
            "get_instances_info_part_not_found",
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
        let vault = create_test_vault(module_path!(), "get_instances_info_err", Some(false)).await;
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
        let vault = create_test_vault(module_path!(), "start_instance_ok", Some(true)).await;
        start_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            InstanceId::new(1),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn start_instance_err() {
        let vault = create_test_vault(module_path!(), "start_instance_err", Some(false)).await;
        assert!(start_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            InstanceId::new(1),
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn start_instance_not_found() {
        let vault = create_test_vault(module_path!(), "start_instance_not_found", Some(true)).await;
        assert!(start_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            InstanceId::new(10),
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn stop_instance_not_found() {
        let vault = create_test_vault(module_path!(), "stop_instance_not_found", Some(true)).await;
        assert!(stop_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            InstanceId::new(10),
        )
        .await
        .is_err());
    }
}
