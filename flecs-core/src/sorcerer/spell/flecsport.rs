use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::jeweler::deployment::Deployment;
use crate::jeweler::gem::instance::InstanceId;
use crate::quest::SyncQuest;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::Vault;
use futures_util::future::join_all;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum ExportInstanceError {
    #[error("Instance not found: {0}")]
    InstanceNotFound(InstanceId),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ExportAppError {
    #[error("App not found: {0}")]
    AppNotFound(AppKey),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ExportDeploymentError {
    #[error("Failed to serialize deployment with serde_json: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("IO error serializing deployment: {0}")]
    IO(#[from] std::io::Error),
}

pub async fn export_instance<F: Floxy>(
    quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
    instance_id: InstanceId,
    path: PathBuf,
) -> Result<(), ExportInstanceError> {
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
        Some(instance) => Ok(instance.export(quest, floxy, &path).await?),
        None => Err(ExportInstanceError::InstanceNotFound(instance_id)),
    }
}

pub async fn export_instances<F: Floxy + 'static>(
    quest: SyncQuest,
    vault: Arc<Vault>,
    floxy: Arc<FloxyOperation<F>>,
    instance_ids: Vec<InstanceId>,
    path: PathBuf,
) -> Result<(), ExportInstanceError> {
    let mut results = Vec::new();
    {
        let mut quest = quest.lock().await;
        for instance_id in instance_ids {
            let result = quest
                .create_sub_quest(
                    format!("Export instance {instance_id} to {path:?}"),
                    |quest| {
                        export_instance(
                            quest,
                            vault.clone(),
                            floxy.clone(),
                            instance_id,
                            path.join(instance_id.to_string()),
                        )
                    },
                )
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

pub async fn get_export(
    export_dir: &Path,
    export_id: String,
) -> Result<Option<tokio::fs::File>, std::io::Error> {
    let path = export_dir.join(format!("{export_id}.tar"));
    match tokio::fs::File::open(path).await {
        Ok(file) if file.metadata().await?.is_file() => Ok(Some(file)),
        Ok(_) => Ok(None),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

pub async fn export_apps(
    quest: SyncQuest,
    vault: Arc<Vault>,
    app_keys: Vec<AppKey>,
    path: PathBuf,
) -> Result<(), ExportAppError> {
    let mut results = Vec::new();
    {
        let mut quest = quest.lock().await;
        for app_key in app_keys {
            let result = quest
                .create_sub_quest(format!("Export app {app_key} to {path:?}"), |quest| {
                    export_app(quest, vault.clone(), app_key, path.clone())
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

pub async fn export_app(
    quest: SyncQuest,
    vault: Arc<Vault>,
    app_key: AppKey,
    path: PathBuf,
) -> Result<(), ExportAppError> {
    match vault
        .reservation()
        .reserve_app_pouch()
        .grab()
        .await
        .app_pouch
        .as_ref()
        .expect("Reservations should never fail")
        .gems()
        .get(&app_key)
    {
        Some(app) => Ok(app.export(quest, path).await?),
        None => Err(ExportAppError::AppNotFound(app_key)),
    }
}

pub async fn export_deployments(
    quest: SyncQuest,
    vault: Arc<Vault>,
    path: PathBuf,
) -> Result<(), ExportDeploymentError> {
    tokio::fs::create_dir_all(&path).await?;
    let deployments: Vec<_> = vault
        .reservation()
        .reserve_deployment_pouch()
        .grab()
        .await
        .deployment_pouch
        .as_ref()
        .expect("Vault reservations should never fail")
        .gems()
        .values()
        .cloned()
        .collect();
    let mut results = Vec::new();
    {
        let mut quest = quest.lock().await;
        for deployment in deployments {
            let result = quest
                .create_sub_quest(
                    format!("Export deployment {} to {path:?}", deployment.id()),
                    |_quest| export_deployment(deployment, path.clone()),
                )
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

pub async fn export_deployment(
    deployment: Arc<dyn Deployment>,
    path: PathBuf,
) -> Result<PathBuf, ExportDeploymentError> {
    let path = path.join(format!("{}.json", deployment.id()));
    let data = serde_json::to_vec_pretty(&deployment)?;
    tokio::fs::write(&path, &data).await?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enchantment::floxy::MockFloxy;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::jeweler::gem::instance::InstanceStatus;
    use crate::quest::Quest;
    use crate::vault::pouch::app::tests::{
        LABEL_APP_NAME, LABEL_APP_VERSION, MINIMAL_APP_NAME, MINIMAL_APP_VERSION, MOUNT_APP_NAME,
        MOUNT_APP_VERSION, NETWORK_APP_NAME, NETWORK_APP_VERSION, NO_MANIFEST_APP_NAME,
        NO_MANIFEST_APP_VERSION,
    };
    use crate::vault::pouch::instance::tests::{
        ENV_INSTANCE, MINIMAL_INSTANCE, PORT_MAPPING_INSTANCE, UNKNOWN_INSTANCE_2, USB_DEV_INSTANCE,
    };
    use crate::vault::tests::{create_empty_test_vault, create_test_vault};
    use mockall::predicate;
    use std::collections::HashMap;
    use testdir::testdir;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn export_instances_ok() {
        const INSTANCE_IDS: [InstanceId; 4] = [
            ENV_INSTANCE,
            MINIMAL_INSTANCE,
            PORT_MAPPING_INSTANCE,
            USB_DEV_INSTANCE,
        ];
        let path = testdir!().join("exports");
        let deployments = HashMap::from_iter(INSTANCE_IDS.iter().map(|instance_id| {
            let mut deployment = MockedDeployment::new();
            deployment
                .expect_id()
                .return_const(format!("MockedDeployment_{instance_id}"));
            deployment.expect_is_default().return_const(false);
            deployment
                .expect_instance_status()
                .returning(|_| Ok(InstanceStatus::Running));
            deployment
                .expect_stop_instance()
                .once()
                .returning(|_, _| Ok(()));
            let deployment: Arc<dyn Deployment> = Arc::new(deployment);
            (*instance_id, deployment)
        }));
        let vault = create_test_vault(deployments, HashMap::new(), None);
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        export_instances(
            Quest::new_synced("TestQuest"),
            vault,
            floxy,
            INSTANCE_IDS.to_vec(),
            path.clone(),
        )
        .await
        .unwrap();
        assert_eq!(std::fs::read_dir(&path).unwrap().count(), 4);
        for instance_id in INSTANCE_IDS {
            assert!(path
                .join(instance_id.to_string())
                .join("instance.json")
                .is_file());
        }
    }
    #[tokio::test]
    async fn export_instances_err() {
        const INSTANCE_ID: InstanceId = InstanceId::new(10);
        let path = testdir!().join("exports");
        let vault = create_empty_test_vault();
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert!(export_instances(
            Quest::new_synced("TestQuest"),
            vault,
            floxy,
            vec![INSTANCE_ID],
            path.clone(),
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn export_instance_ok() {
        const INSTANCE_ID: InstanceId = ENV_INSTANCE;
        let path = testdir!().join("exports");
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment.expect_is_default().return_const(false);
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        deployment
            .expect_stop_instance()
            .once()
            .returning(|_, _| Ok(()));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = create_test_vault(
            HashMap::from([(INSTANCE_ID, deployment)]),
            HashMap::new(),
            None,
        );
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        export_instance(
            Quest::new_synced("TestQuest"),
            vault,
            floxy,
            INSTANCE_ID,
            path.clone(),
        )
        .await
        .unwrap();
        assert!(path.join("instance.json").is_file())
    }

    #[tokio::test]
    async fn export_instance_err_not_found() {
        const INSTANCE_ID: InstanceId = UNKNOWN_INSTANCE_2;
        let path = testdir!().join("exports");
        let vault = create_test_vault(HashMap::new(), HashMap::new(), None);
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert!(matches!(
            export_instance(
                Quest::new_synced("TestQuest"),
                vault,
                floxy,
                INSTANCE_ID,
                path.clone(),
            )
            .await,
            Err(ExportInstanceError::InstanceNotFound(UNKNOWN_INSTANCE_2))
        ));
    }

    #[tokio::test]
    async fn export_instance_err_instance() {
        const INSTANCE_ID: InstanceId = MINIMAL_INSTANCE;
        let path = testdir!().join("exports");
        // Provoke conflict by creating directory with path of an exported instance json
        std::fs::create_dir_all(path.join("instance.json")).unwrap();
        let vault = create_test_vault(HashMap::new(), HashMap::new(), None);
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert!(export_instance(
            Quest::new_synced("TestQuest"),
            vault,
            floxy,
            INSTANCE_ID,
            path.clone(),
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn export_apps_ok() {
        let path = testdir!().join("exports");
        let app_keys = [
            AppKey {
                name: NETWORK_APP_NAME.to_string(),
                version: NETWORK_APP_VERSION.to_string(),
            },
            AppKey {
                name: MINIMAL_APP_NAME.to_string(),
                version: MINIMAL_APP_VERSION.to_string(),
            },
            AppKey {
                name: LABEL_APP_NAME.to_string(),
                version: LABEL_APP_VERSION.to_string(),
            },
            AppKey {
                name: MOUNT_APP_NAME.to_string(),
                version: MOUNT_APP_VERSION.to_string(),
            },
        ];
        let deployments = HashMap::from_iter(app_keys.clone().into_iter().map(|app_key| {
            let mut deployment = MockedDeployment::new();
            deployment.expect_id().return_const(format!(
                "MockedDeployment{}_{}",
                app_key.name, app_key.version
            ));
            deployment.expect_is_default().return_const(false);
            let expected_version = app_key.version.clone();
            deployment
                .expect_export_app()
                .once()
                .with(
                    predicate::always(),
                    predicate::function(move |image: &String| image.ends_with(&expected_version)),
                    predicate::eq(path.join(format!("{}_{}.tar", app_key.name, app_key.version))),
                )
                .returning(|_, _, _| Ok(()));
            let deployment: Arc<dyn Deployment> = Arc::new(deployment);
            (app_key.clone(), deployment)
        }));
        let vault = create_test_vault(HashMap::new(), deployments, None);
        export_apps(
            Quest::new_synced("TestQuest"),
            vault,
            app_keys.to_vec(),
            path.clone(),
        )
        .await
        .unwrap();
        let files: Vec<_> = std::fs::read_dir(&path)
            .unwrap()
            .map(Result::<_, _>::unwrap)
            .collect();
        assert_eq!(
            files
                .iter()
                .filter(|file| file.path().extension().unwrap() == "json")
                .count(),
            app_keys.len() * 2
        );
    }

    #[tokio::test]
    async fn export_apps_err() {
        let path = testdir!().join("exports");
        let app_key = AppKey {
            name: NO_MANIFEST_APP_NAME.to_string(),
            version: NO_MANIFEST_APP_VERSION.to_string(),
        };
        let vault = create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(matches!(
            export_apps(
                Quest::new_synced("TestQuest"),
                vault,
                vec![app_key],
                path.clone(),
            )
            .await,
            Err(ExportAppError::Other(_))
        ));
    }

    #[tokio::test]
    async fn export_app_ok() {
        let path = testdir!().join("exports");
        let app_key = AppKey {
            name: NETWORK_APP_NAME.to_string(),
            version: NETWORK_APP_VERSION.to_string(),
        };
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment.expect_is_default().return_const(true);
        deployment
            .expect_export_app()
            .once()
            .with(
                predicate::always(),
                predicate::eq(format!(
                    "flecs.azurecr.io/tech.flecs.network-app:{NETWORK_APP_VERSION}"
                )),
                predicate::eq(path.join(format!("{NETWORK_APP_NAME}_{NETWORK_APP_VERSION}.tar"))),
            )
            .returning(|_, _, _| Ok(()));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = create_test_vault(
            HashMap::new(),
            HashMap::from([(app_key.clone(), deployment)]),
            None,
        );
        export_app(Quest::new_synced("TestQuest"), vault, app_key, path.clone())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn export_app_err_app() {
        let path = testdir!().join("exports");
        let app_key = AppKey {
            name: NO_MANIFEST_APP_NAME.to_string(),
            version: NO_MANIFEST_APP_VERSION.to_string(),
        };
        let vault = create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(matches!(
            export_app(Quest::new_synced("TestQuest"), vault, app_key, path).await,
            Err(ExportAppError::Other(_))
        ));
    }

    #[tokio::test]
    async fn export_app_err_app_not_found() {
        let path = testdir!().join("exports");
        let app_key = AppKey {
            name: NETWORK_APP_NAME.to_string(),
            version: NETWORK_APP_VERSION.to_string(),
        };
        let vault = create_empty_test_vault();
        assert!(matches!(
            export_app(Quest::new_synced("TestQuest"), vault, app_key.clone(), path.clone())
                .await, Err(ExportAppError::AppNotFound(key)) if key == app_key
        ));
    }

    #[tokio::test]
    async fn export_deployments_ok() {
        const DEPLOYMENT_IDS: [&str; 4] = [
            "MockedDeployment_#1",
            "MockedDeployment_#3",
            "MockedDeployment_#4",
            "MockedDeployment_#5",
        ];
        let path = testdir!().join("exports");
        let vault = create_empty_test_vault();
        {
            let Some(ref mut deployments) = vault
                .reservation()
                .reserve_deployment_pouch_mut()
                .grab()
                .await
                .deployment_pouch_mut
            else {
                unreachable!("Reservation failed")
            };
            for deployment_id in DEPLOYMENT_IDS {
                let mut deployment = MockedDeployment::new();
                deployment
                    .expect_id()
                    .return_const(deployment_id.to_string());
                deployment.expect_is_default().return_const(false);
                deployments
                    .gems_mut()
                    .insert(deployment_id.to_string(), Arc::new(deployment));
            }
        }
        export_deployments(Quest::new_synced("TestQuest"), vault, path.clone())
            .await
            .unwrap();
        assert!(path.try_exists().unwrap());
        for deployment_id in DEPLOYMENT_IDS {
            assert_eq!(
                std::fs::read_to_string(path.join(format!("{deployment_id}.json"))).unwrap(),
                format!("\"{deployment_id}\"")
            );
        }
    }

    #[tokio::test]
    async fn export_deployments_err_deployment() {
        const DEPLOYMENT_IDS: [&str; 4] = [
            "MockedDeployment_#1",
            "MockedDeployment_#3",
            "MockedDeployment_#4",
            "MockedDeployment_#5",
        ];
        let path = testdir!().join("exports");
        let vault = create_empty_test_vault();
        {
            let Some(ref mut deployments) = vault
                .reservation()
                .reserve_deployment_pouch_mut()
                .grab()
                .await
                .deployment_pouch_mut
            else {
                unreachable!("Reservation failed")
            };
            for deployment_id in DEPLOYMENT_IDS {
                let mut deployment = MockedDeployment::new();
                deployment
                    .expect_id()
                    .return_const(deployment_id.to_string());
                deployment.expect_is_default().return_const(false);
                deployments
                    .gems_mut()
                    .insert(deployment_id.to_string(), Arc::new(deployment));
            }
        }
        // Provoke conflict by creating directory with path of an exported Deployment json
        std::fs::create_dir_all(path.join(format!("{}.json", DEPLOYMENT_IDS[2]))).unwrap();
        assert!(matches!(
            export_deployments(Quest::new_synced("TestQuest"), vault, path.clone()).await,
            Err(ExportDeploymentError::IO(_))
        ));
    }

    #[tokio::test]
    async fn export_deployments_err_io() {
        let path = testdir!().join("exports");
        let vault = create_empty_test_vault();
        std::fs::write(&path, "").unwrap();
        let result = export_deployments(Quest::new_synced("TestQuest"), vault, path.clone()).await;
        assert!(
            matches!(result, Err(ExportDeploymentError::IO(_))),
            "Expected Err(ExportDeploymentError::IO(_)), got '{result:?}'"
        );
    }

    #[tokio::test]
    async fn export_deployment_ok() {
        const DEPLOYMENT_ID: &str = "ExportedMockDeployment";
        let path = testdir!();
        let expected_file_path = path.join(format!("{DEPLOYMENT_ID}.json"));
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .return_const(DEPLOYMENT_ID.to_string());
        assert_eq!(
            export_deployment(Arc::new(deployment), path.clone())
                .await
                .unwrap(),
            expected_file_path
        );
        assert_eq!(
            std::fs::read_to_string(expected_file_path).unwrap(),
            format!("\"{DEPLOYMENT_ID}\"")
        );
    }

    #[tokio::test]
    async fn export_deployment_err() {
        const DEPLOYMENT_ID: &str = "ExportedMockDeployment";
        let path = testdir!();
        let expected_file_path = path.join(format!("{DEPLOYMENT_ID}.json"));
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .return_const(DEPLOYMENT_ID.to_string());
        // Provoke error by creating a directory at the file location
        std::fs::create_dir_all(expected_file_path).unwrap();
        assert!(export_deployment(Arc::new(deployment), path.clone())
            .await
            .is_err());
    }

    #[tokio::test]
    async fn get_export_ok_some() {
        const EXPORT_ID: &str = "1234tasf236zt";
        const EXPORT_DATA: &[u8; 9] = b"dataaaaaa";
        let path = testdir!();
        let expected_file_path = path.join(format!("{EXPORT_ID}.tar"));
        std::fs::write(expected_file_path, EXPORT_DATA).unwrap();
        let mut file = get_export(&path, EXPORT_ID.to_string())
            .await
            .unwrap()
            .unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, EXPORT_DATA);
    }

    #[tokio::test]
    async fn get_export_ok_none() {
        const EXPORT_ID: &str = "1234tasf236zt";
        let path = testdir!();
        assert!(matches!(
            get_export(&path, EXPORT_ID.to_string()).await,
            Ok(None)
        ));
    }

    #[tokio::test]
    async fn get_export_ok_none_dir() {
        const EXPORT_ID: &str = "1234tasf236zt";
        let path = testdir!();
        let expected_file_path = path.join(format!("{EXPORT_ID}.tar"));
        std::fs::create_dir_all(expected_file_path).unwrap();
        assert!(matches!(
            get_export(&path, EXPORT_ID.to_string()).await,
            Ok(None)
        ));
    }
}
