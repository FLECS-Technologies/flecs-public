pub use super::Result;
use crate::jeweler::app::AppStatus;
use crate::jeweler::deployment::Deployment;
use crate::jeweler::gem::instance::InstanceId;
use crate::quest::SyncQuest;
use crate::sorcerer::spell;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::{GrabbedPouches, Vault};
use std::sync::Arc;

pub async fn get_instance(
    vault: Arc<Vault>,
    instance_id: InstanceId,
) -> Result<Option<flecsd_axum_server::models::AppInstance>> {
    spell::instance::get_instance_info(vault, instance_id).await
}

pub async fn get_instance_detailed(
    vault: Arc<Vault>,
    instance_id: InstanceId,
) -> Result<Option<flecsd_axum_server::models::InstancesInstanceIdGet200Response>> {
    spell::instance::get_instance_detailed_info(vault, instance_id).await
}

pub async fn get_instances_filtered(
    quest: SyncQuest,
    vault: Arc<Vault>,
    app_name: Option<String>,
    app_version: Option<String>,
) -> Vec<flecsd_axum_server::models::AppInstance> {
    let instance_ids = {
        let grab = vault.reservation().reserve_instance_pouch().grab().await;
        let instance_pouch = grab
            .instance_pouch
            .as_ref()
            .expect("Vault reservations should never fail");
        match (app_name, app_version) {
            (None, None) => instance_pouch.gems().keys().copied().collect(),
            (None, Some(version)) => instance_pouch.instance_ids_by_app_version(version),
            (Some(name), None) => instance_pouch.instance_ids_by_app_name(name),
            (Some(name), Some(version)) => {
                instance_pouch.instance_ids_by_app_key(AppKey { name, version })
            }
        }
    };
    spell::instance::get_instances_info(quest, vault, instance_ids).await
}

pub async fn get_all_instances(
    quest: SyncQuest,
    vault: Arc<Vault>,
) -> Vec<flecsd_axum_server::models::AppInstance> {
    let instance_ids: Vec<InstanceId> = vault
        .reservation()
        .reserve_instance_pouch()
        .grab()
        .await
        .instance_pouch
        .as_ref()
        .expect("Vault reservations should never fail")
        .gems()
        .keys()
        .copied()
        .collect();
    spell::instance::get_instances_info(quest, vault, instance_ids).await
}

pub async fn create_instance(
    quest: SyncQuest,
    vault: Arc<Vault>,
    app_key: AppKey,
    name: String,
) -> Result<InstanceId> {
    let (manifest, deployments) = quest
        .lock()
        .await
        .create_sub_quest(
            format!("Validate request for creation of instance '{name}' of {app_key}"),
            |_quest| {
                let vault = vault.clone();
                let app_key = app_key.clone();
                async move {
                    let GrabbedPouches {
                        deployment_pouch: Some(deployments),
                        manifest_pouch: Some(manifests),
                        instance_pouch: Some(instances),
                        app_pouch: Some(apps),
                        ..
                    } = &vault
                        .reservation()
                        .reserve_deployment_pouch()
                        .reserve_manifest_pouch()
                        .reserve_instance_pouch()
                        .reserve_app_pouch()
                        .grab()
                        .await
                    else {
                        unreachable!("Vault reservations should never fail")
                    };
                    let is_app_installed = match apps.gems().get(&app_key) {
                        None => false,
                        Some(app) => app.status().await? == AppStatus::Installed,
                    };
                    anyhow::ensure!(is_app_installed, "App {app_key} is not installed");

                    let manifest = manifests
                        .gems()
                        .get(&app_key)
                        .ok_or_else(|| anyhow::anyhow!("No manifest for {app_key} present"))?
                        .clone();
                    if !manifest.multi_instance()
                        && !instances
                            .instance_ids_by_app_key(app_key.clone())
                            .is_empty()
                    {
                        anyhow::bail!("Can not create multiple instances for {app_key}");
                    }
                    let deployments = deployments
                        .gems()
                        .values()
                        .cloned()
                        .collect::<Vec<Arc<dyn Deployment>>>();
                    Ok((manifest, deployments))
                }
            },
        )
        .await
        .2
        .await?;
    // TODO: In which deployment(s) should an instance be created? All?
    let deployment = deployments
        .first()
        .ok_or_else(|| anyhow::anyhow!("No deployment present to create instance in"))?
        .clone();
    let instance = quest
        .lock()
        .await
        .create_sub_quest(format!("Create instance '{name}' for {app_key}"), |quest| {
            spell::instance::create_instance(quest, deployment, manifest, name)
        })
        .await
        .2
        .await?;
    let instance_id = instance.id;
    quest
        .lock()
        .await
        .create_infallible_sub_quest(
            format!(
                "Saving new instance {} with id {}",
                instance.name, instance_id
            ),
            |_quest| async move {
                vault
                    .reservation()
                    .reserve_instance_pouch_mut()
                    .grab()
                    .await
                    .instance_pouch_mut
                    .as_mut()
                    .expect("Vault reservations should never fail")
                    .gems_mut()
                    .insert(instance_id, instance);
            },
        )
        .await
        .2
        .await;
    Ok(instance_id)
}

pub async fn does_instance_exist(vault: Arc<Vault>, id: InstanceId) -> bool {
    vault
        .reservation()
        .reserve_instance_pouch()
        .grab()
        .await
        .instance_pouch
        .as_ref()
        .expect("Reservations should never fail")
        .gems()
        .contains_key(&id)
}

pub async fn delete_instance(quest: SyncQuest, vault: Arc<Vault>, id: InstanceId) -> Result<()> {
    quest
        .lock()
        .await
        .create_sub_quest(format!("Delete instance {id}"), |quest| async move {
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
                    if let Err((e, instance)) = instance.stop_and_delete(quest).await {
                        instances.insert(id, instance);
                        Err(e)
                    } else {
                        Ok(())
                    }
                }
                None => anyhow::bail!("Instance {id} not found"),
            }
        })
        .await
        .2
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::app::AppInfo;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::jeweler::deployment::Deployment;
    use crate::jeweler::gem::app::{try_create_app, AppDataDeserializable, AppDeserializable};
    use crate::jeweler::gem::instance::tests::test_instance;
    use crate::jeweler::gem::instance::{InstanceId, InstanceStatus};
    use crate::jeweler::gem::manifest::tests::{create_test_manifest, create_test_manifest_full};
    use crate::quest::Quest;
    use crate::tests::prepare_test_path;
    use crate::vault::pouch::Pouch;
    use crate::vault::{Vault, VaultConfig};
    use std::collections::HashMap;
    use std::sync::Arc;

    async fn test_vault(
        deployment: Arc<dyn Deployment>,
        instance_count: u32,
        test_name: &str,
    ) -> Arc<Vault> {
        let path = prepare_test_path(module_path!(), test_name);
        let vault = Arc::new(Vault::new(VaultConfig { path }));
        {
            let mut grab = vault
                .reservation()
                .reserve_instance_pouch_mut()
                .grab()
                .await;
            let instances = grab.instance_pouch_mut.as_mut().unwrap();
            for i in 0..instance_count {
                let instance = test_instance(i, deployment.clone(), create_test_manifest(None));
                instances.gems_mut().insert(instance.id, instance);
            }
        }
        vault
    }

    #[tokio::test]
    async fn delete_instance_test() {
        const INSTANCE_COUNT: u32 = 4;
        const INSTANCE_TO_DELETE: u32 = 2;
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(move || "MockedDeployment".to_string());
        deployment
            .expect_stop_instance()
            .times(1)
            .returning(|_| Ok(()));
        deployment
            .expect_delete_instance()
            .times(1)
            .returning(|_| Ok(true));
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        deployment
            .expect_delete_volume()
            .withf(|_, id| id.starts_with(&format!("Instance#{INSTANCE_TO_DELETE}")))
            .times(4)
            .returning(|_, _| Ok(()));
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let vault = test_vault(deployment.clone(), INSTANCE_COUNT, "delete_instance_test").await;
        let instance_id = InstanceId::new(INSTANCE_TO_DELETE);
        assert!(delete_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            instance_id,
        )
        .await
        .is_ok());
        assert!(!vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .contains_key(&instance_id));
        assert!(delete_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            instance_id,
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn does_instance_exist_test() {
        const INSTANCE_COUNT: u32 = 4;
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(move || "MockedDeployment".to_string());
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let vault = test_vault(
            deployment.clone(),
            INSTANCE_COUNT,
            "does_instance_exist_test",
        )
        .await;
        for i in 0..INSTANCE_COUNT {
            assert!(does_instance_exist(vault.clone(), InstanceId::new(i)).await);
        }
        for i in INSTANCE_COUNT..INSTANCE_COUNT + 10 {
            assert!(!does_instance_exist(vault.clone(), InstanceId::new(i)).await);
        }
    }

    async fn create_test_vault(
        test_name: &str,
        deployment: Arc<dyn Deployment>,
        with_manifest: bool,
        with_app: bool,
        multi_instance: bool,
        insert_deployment_into_pouch: bool,
    ) -> (Arc<Vault>, AppKey) {
        let path = prepare_test_path(module_path!(), test_name);
        let vault = Arc::new(Vault::new(VaultConfig { path: path.clone() }));
        let app_key = {
            let GrabbedPouches {
                manifest_pouch_mut: Some(ref mut manifests),
                app_pouch_mut: Some(ref mut apps),
                deployment_pouch_mut: Some(ref mut deployments),
                ..
            } = vault
                .reservation()
                .reserve_deployment_pouch_mut()
                .reserve_manifest_pouch_mut()
                .reserve_app_pouch_mut()
                .grab()
                .await
            else {
                unreachable!("Vault reservations should never fail")
            };
            if insert_deployment_into_pouch {
                deployments
                    .gems_mut()
                    .insert(deployment.id(), deployment.clone());
            }
            let manifest = Arc::new(create_test_manifest_full(Some(multi_instance)));
            let app_key = manifest.key.clone();
            if with_manifest {
                manifests.gems_mut().insert(app_key.clone(), manifest);
            }
            if with_app {
                let app = AppDeserializable {
                    key: app_key.clone(),
                    deployments: vec![AppDataDeserializable {
                        desired: AppStatus::Installed,
                        id: Some("TestAppId".to_string()),
                        deployment_id: deployment.id(),
                    }],
                };
                let deployments = HashMap::from([(deployment.id(), deployment)]);
                let app = try_create_app(app, manifests.gems(), &deployments).unwrap();
                apps.gems_mut().insert(app_key.clone(), app);
            }
            app_key
        };
        (vault, app_key)
    }

    #[tokio::test]
    async fn create_instance_ok() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
        deployment
            .expect_copy_from_app_image()
            .times(3)
            .returning(|_, _, _, _, _| Ok(()));
        let (vault, app_key) = create_test_vault(
            "create_instance_ok",
            Arc::new(deployment),
            true,
            true,
            false,
            true,
        )
        .await;
        let instance_id = create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key,
            "TestInstance".to_string(),
        )
        .await
        .unwrap();

        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Vault reservations should never fail")
        };
        assert_eq!(instances.gems().len(), 1);
        assert!(instances.gems().contains_key(&instance_id));
    }
    #[tokio::test]
    async fn create_multi_instance_ok() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
        deployment
            .expect_copy_from_app_image()
            .times(6)
            .returning(|_, _, _, _, _| Ok(()));
        let (vault, app_key) = create_test_vault(
            "create_multi_instance_ok",
            Arc::new(deployment),
            true,
            true,
            true,
            true,
        )
        .await;
        let instance_id_1 = create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key.clone(),
            "TestInstance1".to_string(),
        )
        .await
        .unwrap();
        let instance_id_2 = create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key,
            "TestInstance2".to_string(),
        )
        .await
        .unwrap();

        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Vault reservations should never fail")
        };
        assert_eq!(instances.gems().len(), 2);
        assert!(instances.gems().contains_key(&instance_id_1));
        assert!(instances.gems().contains_key(&instance_id_2));
    }
    #[tokio::test]
    async fn create_instance_single_instance_but_instance_present() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
        deployment
            .expect_copy_from_app_image()
            .times(3)
            .returning(|_, _, _, _, _| Ok(()));
        let (vault, app_key) = create_test_vault(
            "create_instance_single_instance_but_instance_present",
            Arc::new(deployment),
            true,
            true,
            false,
            true,
        )
        .await;
        let instance_id = create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key.clone(),
            "TestInstance1".to_string(),
        )
        .await
        .unwrap();
        assert!(create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key,
            "TestInstance2".to_string(),
        )
        .await
        .is_err());

        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Vault reservations should never fail")
        };
        assert_eq!(instances.gems().len(), 1);
        assert!(instances.gems().contains_key(&instance_id));
    }
    #[tokio::test]
    async fn create_instance_app_not_installed() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        let (vault, app_key) = create_test_vault(
            "create_instance_app_not_installed",
            Arc::new(deployment),
            true,
            true,
            false,
            true,
        )
        .await;
        assert!(create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key,
            "TestInstance".to_string(),
        )
        .await
        .is_err());

        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Vault reservations should never fail")
        };
        assert!(instances.gems().is_empty());
    }
    #[tokio::test]
    async fn create_instance_app_not_created() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        let (vault, app_key) = create_test_vault(
            "create_instance_app_not_created",
            Arc::new(deployment),
            true,
            false,
            false,
            true,
        )
        .await;
        assert!(create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key,
            "TestInstance".to_string(),
        )
        .await
        .is_err());

        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Vault reservations should never fail")
        };
        assert!(instances.gems().is_empty());
    }
    #[tokio::test]
    async fn create_instance_manifest_not_present() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
        let (vault, app_key) = create_test_vault(
            "create_instance_manifest_not_present",
            Arc::new(deployment),
            false,
            true,
            false,
            true,
        )
        .await;
        assert!(create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key,
            "TestInstance".to_string(),
        )
        .await
        .is_err());

        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Vault reservations should never fail")
        };
        assert!(instances.gems().is_empty());
    }

    #[tokio::test]
    async fn create_instance_no_deployment() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
        let (vault, app_key) = create_test_vault(
            "create_instance_no_deployment",
            Arc::new(deployment),
            true,
            true,
            false,
            false,
        )
        .await;
        assert!(create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            app_key,
            "TestInstance".to_string(),
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn get_all_instances_ok() {
        let vault = spell::instance::tests::create_test_vault(
            module_path!(),
            "get_all_instances",
            Some(true),
        )
        .await;
        let instances_infos =
            get_all_instances(Quest::new_synced("TestQuest".to_string()), vault).await;
        assert_eq!(instances_infos.len(), 6);
    }

    #[tokio::test]
    async fn get_instances_filtered_all() {
        let vault = spell::instance::tests::create_test_vault(
            module_path!(),
            "get_instances_filtered_all",
            Some(true),
        )
        .await;
        let instances_infos = get_instances_filtered(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            None,
            None,
        )
        .await;
        assert_eq!(instances_infos.len(), 6);
    }

    #[tokio::test]
    async fn get_instances_filtered_name() {
        let vault = spell::instance::tests::create_test_vault(
            module_path!(),
            "get_instances_filtered_name",
            Some(true),
        )
        .await;
        let instances_infos = get_instances_filtered(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            Some("some.test.app-4".to_string()),
            None,
        )
        .await;
        assert_eq!(instances_infos.len(), 3);
    }

    #[tokio::test]
    async fn get_instances_filtered_version() {
        let vault = spell::instance::tests::create_test_vault(
            module_path!(),
            "get_instances_filtered_version",
            Some(true),
        )
        .await;
        let instances_infos = get_instances_filtered(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            None,
            Some("1.2.4".to_string()),
        )
        .await;
        assert_eq!(instances_infos.len(), 4);
    }

    #[tokio::test]
    async fn get_instances_filtered_key() {
        let vault = spell::instance::tests::create_test_vault(
            module_path!(),
            "get_instances_filtered_key",
            Some(true),
        )
        .await;
        let instances_infos = get_instances_filtered(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            Some("some.test.app-4".to_string()),
            Some("1.2.4".to_string()),
        )
        .await;
        assert_eq!(instances_infos.len(), 2);
    }

    #[tokio::test]
    async fn get_instance_ok() {
        let vault = spell::instance::tests::create_test_vault(
            module_path!(),
            "get_instance_ok",
            Some(true),
        )
        .await;
        assert!(get_instance(vault, 1.into()).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn get_instance_detailed_ok() {
        let vault = spell::instance::tests::create_test_vault(
            module_path!(),
            "get_instance_detailed_ok",
            Some(true),
        )
        .await;
        assert!(get_instance_detailed(vault, 1.into())
            .await
            .unwrap()
            .is_some());
    }
}
