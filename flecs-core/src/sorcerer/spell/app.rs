use crate::quest::SyncQuest;
use crate::vault::Vault;
use crate::vault::pouch::{AppKey, Pouch};
use std::sync::Arc;

pub async fn uninstall_app(quest: SyncQuest, vault: Arc<Vault>, key: AppKey) -> crate::Result<()> {
    let app = vault
        .reservation()
        .reserve_app_pouch_mut()
        .grab()
        .await
        .app_pouch_mut
        .as_mut()
        .expect("Vault reservations should never fail")
        .gems_mut()
        .remove(&key)
        .ok_or_else(|| anyhow::anyhow!("Can not uninstall {key}, which is not installed"))?;
    match app.uninstall(quest).await {
        Ok(()) => Ok(()),
        Err((e, app)) => {
            vault
                .reservation()
                .reserve_app_pouch_mut()
                .grab()
                .await
                .app_pouch_mut
                .as_mut()
                .expect("Vault reservations should never fail")
                .gems_mut()
                .insert(key, app);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::gem::app::{App, AppData};
    use crate::jeweler::gem::deployment::Deployment;
    use crate::jeweler::gem::deployment::docker::tests::MockedDockerDeployment;
    use crate::quest::Quest;
    use crate::vault::pouch::manifest::tests::min_app_1_0_0_manifest;
    use crate::vault::tests::create_empty_test_vault;

    #[tokio::test]
    async fn uninstall_app_not_found() {
        assert!(
            uninstall_app(
                Quest::new_synced("TestQuest".to_string()),
                create_empty_test_vault(),
                AppKey {
                    name: "not_found".to_string(),
                    version: "1.0.0".to_string(),
                }
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn uninstall_app_error() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_is_app_installed()
            .once()
            .returning(|_, _, _| Ok(true));
        deployment
            .expect_uninstall_app()
            .times(1)
            .returning(|_, _, _| Err(anyhow::anyhow!("TestError")));
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        let deployment = Deployment::Docker(Arc::new(deployment));
        let manifest = min_app_1_0_0_manifest();
        let mut app_data = AppData::new(deployment);
        app_data.set_id("TestAppId".to_string());
        let key = manifest.key().clone();
        let mut app = App::new(key.clone(), Vec::new(), manifest);
        app.deployments
            .insert("Mocked_deployment".to_string(), app_data);
        let vault = create_empty_test_vault();
        vault
            .reservation()
            .reserve_app_pouch_mut()
            .grab()
            .await
            .app_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut()
            .insert(key.clone(), app);
        let result = uninstall_app(Quest::new_synced("TestQuest".to_string()), vault, key).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn uninstall_app_ok() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_is_app_installed()
            .once()
            .returning(|_, _, _| Ok(true));
        deployment
            .expect_uninstall_app()
            .times(1)
            .returning(|_, _, _| Ok(()));
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        let deployment = Deployment::Docker(Arc::new(deployment));
        let manifest = min_app_1_0_0_manifest();
        let mut app_data = AppData::new(deployment);
        app_data.set_id("TestAppId".to_string());
        let key = manifest.key().clone();
        let mut app = App::new(key.clone(), Vec::new(), manifest);
        app.deployments
            .insert("Mocked_deployment".to_string(), app_data);
        let vault = create_empty_test_vault();
        vault
            .reservation()
            .reserve_app_pouch_mut()
            .grab()
            .await
            .app_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut()
            .insert(key.clone(), app);
        uninstall_app(Quest::new_synced("TestQuest".to_string()), vault, key)
            .await
            .unwrap();
    }
}
