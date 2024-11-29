pub use super::Result;
use crate::jeweler::gem::instance::InstanceId;
use crate::quest::SyncQuest;
use crate::vault::pouch::Pouch;
use crate::vault::Vault;
use std::sync::Arc;

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
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::jeweler::deployment::Deployment;
    use crate::jeweler::gem::instance::tests::test_instance;
    use crate::jeweler::gem::instance::{InstanceId, InstanceStatus};
    use crate::quest::Quest;
    use crate::sorcerer::appraiser::tests::create_test_manifest;
    use crate::tests::prepare_test_path;
    use crate::vault::pouch::Pouch;
    use crate::vault::{Vault, VaultConfig};
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
}
