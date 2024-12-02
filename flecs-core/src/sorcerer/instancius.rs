pub use super::Result;
use crate::jeweler::deployment::DeploymentId;
use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::instance::Logs;
use crate::quest::SyncQuest;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::Vault;
use std::sync::Arc;

#[derive(Debug)]
pub enum InstanceError {
    InstanceNotFound,
    Other(anyhow::Error),
}

pub async fn find_instance(
    vault: Arc<Vault>,
    id: InstanceId,
) -> Result<(AppKey, DeploymentId), InstanceError> {
    for (app_key, app) in vault
        .reservation()
        .reserve_app_pouch()
        .grab()
        .await
        .app_pouch
        .as_ref()
        .expect("Reservations should never fail")
        .gems()
    {
        for (deployment_id, data) in app.properties.iter() {
            if data.instances.contains_key(&id) {
                return Ok((app_key.clone(), deployment_id.clone()));
            }
        }
    }
    Err(InstanceError::InstanceNotFound)
}

pub async fn delete_instance(
    quest: SyncQuest,
    vault: Arc<Vault>,
    id: InstanceId,
    app_key: AppKey,
    deployment_id: DeploymentId,
) -> Result<()> {
    quest
        .lock()
        .await
        .create_sub_quest(format!("Delete instance {id}"), |quest| async move {
            match vault
                .reservation()
                .reserve_app_pouch_mut()
                .grab()
                .await
                .app_pouch_mut
                .as_mut()
                .expect("Reservations should never fail")
                .gems_mut()
                .get_mut(&app_key)
            {
                Some(app) => app.delete_instance(quest, id, deployment_id).await,
                None => anyhow::bail!("App {app_key} not found"),
            }
        })
        .await
        .2
        .await
}

pub async fn get_instance_logs(
    vault: Arc<Vault>,
    id: InstanceId,
    app_key: AppKey,
    deployment_id: DeploymentId,
) -> Result<Logs> {
    match vault
        .reservation()
        .reserve_app_pouch_mut()
        .grab()
        .await
        .app_pouch_mut
        .as_mut()
        .expect("Reservations should never fail")
        .gems_mut()
        .get_mut(&app_key)
    {
        Some(app) => app.get_instance_logs(id, deployment_id).await,
        None => anyhow::bail!("App {app_key} not found"),
    }
}
#[cfg(test)]
mod tests {
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::jeweler::deployment::Deployment;
    use crate::jeweler::gem::app::tests::{test_instance, test_key_numbered};
    use crate::jeweler::gem::app::App;
    use crate::jeweler::gem::instance::{InstanceId, InstanceStatus};
    use crate::quest::Quest;
    use crate::sorcerer::instancius::{delete_instance, find_instance, InstanceError};
    use crate::vault::pouch::Pouch;
    use crate::vault::{Vault, VaultConfig};
    use std::path::Path;
    use std::sync::Arc;

    const TEST_PATH: &str = "/tmp/flecs-test/instancius/";

    fn calculate_instance_id(deployment_number: usize, app_number: u8, instance_number: u8) -> u32 {
        deployment_number as u32 * 100 + 10 * app_number as u32 + instance_number as u32
    }

    fn test_app_numbered(
        deployments: Vec<Arc<dyn Deployment>>,
        number: u8,
        instance_count: u8,
    ) -> App {
        let mut app = App::new(test_key_numbered(number, number), deployments.clone());
        for (i, deployment) in deployments.iter().enumerate() {
            for j in 0..instance_count {
                let instance =
                    test_instance(calculate_instance_id(i, number, j), deployment.clone());
                app.properties
                    .get_mut(&deployment.id())
                    .unwrap()
                    .instances
                    .insert(instance.id, instance);
            }
        }
        app
    }

    async fn test_vault(
        deployments: Vec<Arc<dyn Deployment>>,
        instance_count: u8,
        app_count: u8,
    ) -> Arc<Vault> {
        let vault = Arc::new(Vault::new(VaultConfig {
            path: Path::new(TEST_PATH).to_path_buf(),
        }));
        {
            let mut grab = vault.reservation().reserve_app_pouch_mut().grab().await;
            let apps = grab.app_pouch_mut.as_mut().unwrap();
            for i in 0..app_count {
                let app = test_app_numbered(deployments.clone(), i, instance_count);
                apps.gems_mut().insert(app.key.clone(), app);
            }
        }
        vault
    }

    #[tokio::test]
    async fn delete_instance_test() {
        const DEPLOYMENT_COUNT: usize = 4;
        const INSTANCE_COUNT: u8 = 4;
        const APP_COUNT: u8 = 6;
        const INSTANCE_TO_DELETE: u8 = 2;
        let deployments = (0..DEPLOYMENT_COUNT)
            .map(|i| {
                let mut deployment = MockedDeployment::new();
                deployment
                    .expect_id()
                    .returning(move || format!("MockedDeployment#{i}"));
                if i == INSTANCE_TO_DELETE as usize {
                    deployment.expect_stop_instance().returning(|_| Ok(()));
                    deployment.expect_delete_instance().returning(|_| Ok(()));
                    deployment
                        .expect_instance_status()
                        .returning(|_| Ok(InstanceStatus::Running));
                }
                Arc::new(deployment) as Arc<dyn Deployment>
            })
            .collect::<Vec<Arc<dyn Deployment>>>();
        let vault = test_vault(deployments.clone(), INSTANCE_COUNT, APP_COUNT).await;
        let instance_id = InstanceId::new(calculate_instance_id(
            INSTANCE_TO_DELETE as usize,
            INSTANCE_TO_DELETE,
            INSTANCE_TO_DELETE,
        ));
        let app_key = test_key_numbered(INSTANCE_TO_DELETE, INSTANCE_TO_DELETE);
        let deployment_id = deployments[INSTANCE_TO_DELETE as usize].id();
        assert!(delete_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            instance_id,
            app_key.clone(),
            deployment_id.clone()
        )
        .await
        .is_ok());
        assert!(delete_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            instance_id,
            test_key_numbered(APP_COUNT, APP_COUNT),
            deployment_id.clone()
        )
        .await
        .is_err());
        let grab = vault.reservation().reserve_app_pouch().grab().await;
        let apps = grab.app_pouch.as_ref().unwrap().gems();
        assert!(!apps
            .get(&app_key)
            .unwrap()
            .properties
            .get(&deployment_id)
            .unwrap()
            .instances
            .contains_key(&instance_id));
        for (deployment_number, deployment) in deployments.iter().enumerate() {
            for instance_number in 0..INSTANCE_COUNT {
                for app_number in 0..APP_COUNT {
                    if deployment_number == INSTANCE_TO_DELETE as usize
                        && instance_number == INSTANCE_TO_DELETE
                        && app_number == INSTANCE_TO_DELETE
                    {
                        continue;
                    }
                    let app_key = test_key_numbered(app_number, app_number);
                    let deployment_id = deployment.id();
                    let instance_id = InstanceId::new(calculate_instance_id(
                        deployment_number,
                        app_number,
                        instance_number,
                    ));
                    assert!(apps
                        .get(&app_key)
                        .unwrap()
                        .properties
                        .get(&deployment_id)
                        .unwrap()
                        .instances
                        .contains_key(&instance_id));
                }
            }
        }
    }

    #[tokio::test]
    async fn find_instance_ok() {
        const DEPLOYMENT_COUNT: usize = 4;
        let deployments = (0..DEPLOYMENT_COUNT)
            .map(|i| {
                let mut deployment = MockedDeployment::new();
                deployment
                    .expect_id()
                    .returning(move || format!("MockedDeployment#{i}"));
                Arc::new(deployment) as Arc<dyn Deployment>
            })
            .collect();
        let vault = test_vault(deployments, 4, 6).await;
        for deployment_number in 0..DEPLOYMENT_COUNT {
            for instance_number in 0..4 {
                for app_number in 0..6 {
                    assert_eq!(
                        find_instance(
                            vault.clone(),
                            InstanceId::new(calculate_instance_id(
                                deployment_number,
                                app_number,
                                instance_number
                            ))
                        )
                        .await
                        .unwrap(),
                        (
                            test_key_numbered(app_number, app_number),
                            format!("MockedDeployment#{deployment_number}")
                        )
                    );
                }
            }
        }

        matches!(
            find_instance(
                vault.clone(),
                InstanceId::new(calculate_instance_id(5, 1, 2))
            )
            .await
            .err()
            .unwrap(),
            InstanceError::InstanceNotFound
        );
    }
}
