use crate::quest::SyncQuest;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::Vault;
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
    use crate::jeweler::app::AppInfo;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::jeweler::gem::app::{App, AppData};
    use crate::quest::Quest;
    use crate::vault::VaultConfig;
    use std::path::Path;

    #[tokio::test]
    async fn uninstall_app_not_found() {
        assert!(uninstall_app(
            Quest::new_synced("TestQuest".to_string()),
            Arc::new(Vault::new(VaultConfig {
                path: Path::new("/tmp/flecs-tests/uninstall_app_not_found/").to_path_buf(),
            })),
            AppKey {
                name: "not_found".to_string(),
                version: "1.0.0".to_string(),
            }
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn uninstall_app_error() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_app_info()
            .times(1)
            .returning(|_, _| Ok(AppInfo::default()));
        deployment
            .expect_uninstall_app()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        deployment
            .expect_id()
            .returning(|| "MockedDeployment".to_string());
        let mut app_data = AppData::new(Arc::new(deployment));
        app_data.set_id("TestAppId".to_string());
        let key = AppKey {
            name: "TestApp".to_string(),
            version: "1.2.3".to_string(),
        };
        let mut app = App::new(key.clone(), Vec::new());
        app.properties
            .insert("Mocked_deployment".to_string(), app_data);
        let vault = Arc::new(Vault::new(VaultConfig {
            path: Path::new("/tmp/flecs-tests/uninstall_app_error/").to_path_buf(),
        }));
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
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_app_info()
            .times(1)
            .returning(|_, _| Ok(AppInfo::default()));
        deployment
            .expect_uninstall_app()
            .times(1)
            .returning(|_, _| Ok(()));
        deployment
            .expect_id()
            .returning(|| "MockedDeployment".to_string());
        let mut app_data = AppData::new(Arc::new(deployment));
        app_data.set_id("TestAppId".to_string());
        let key = AppKey {
            name: "TestApp".to_string(),
            version: "1.2.3".to_string(),
        };
        let mut app = App::new(key.clone(), Vec::new());
        app.properties
            .insert("Mocked_deployment".to_string(), app_data);
        let vault = Arc::new(Vault::new(VaultConfig {
            path: Path::new("/tmp/flecs-tests/uninstall_app_ok/").to_path_buf(),
        }));
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
