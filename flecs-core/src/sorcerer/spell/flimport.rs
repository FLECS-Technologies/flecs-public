use crate::jeweler::deployment::DeploymentId;
use crate::jeweler::gem::app::{App, AppDeserializable, try_create_app, try_create_legacy_app};
use crate::jeweler::gem::deployment::{Deployment, SerializedDeployment};
use crate::jeweler::gem::instance::compose::ComposeInstance;
use crate::jeweler::gem::instance::docker::DockerInstance;
use crate::jeweler::gem::instance::{
    CreateInstanceError, Instance, InstanceDeserializable, InstanceId,
};
use crate::jeweler::gem::manifest::AppManifest;
use crate::legacy;
use crate::lore::Lore;
use crate::quest::SyncQuest;
use crate::relic::device::usb::UsbDeviceReader;
use crate::relic::system::info::try_create_system_info;
use crate::sorcerer::exportius::manifest::{Manifest, v2, v3};
use crate::sorcerer::importius::{
    ImportAppError, ImportDeploymentError, ImportError, ImportInstanceError, ImportManifestError,
    ReadImportManifestError,
};
use crate::vault::pouch::deployment::DefaultDeployments;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::{GrabbedPouches, Vault, pouch};
use futures_util::future::{BoxFuture, join_all};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::oneshot::error::RecvError;
use tracing::{debug, error, warn};

pub async fn read_import_manifest(
    quest: SyncQuest,
    src: PathBuf,
) -> Result<Manifest, ReadImportManifestError> {
    let manifest_path = src.join("manifest.json");
    let manifest = tokio::fs::read(&manifest_path).await?;
    let manifest: Manifest = serde_json::from_slice(&manifest)?;
    let manifest = quest
        .lock()
        .await
        .create_sub_quest("Validate import", |_quest| {
            validate_import(manifest, src.clone())
        })
        .await
        .2;
    manifest.await
}

async fn merged_apps(vault: &Arc<Vault>, new_apps: &pouch::app::Gems) -> Arc<pouch::app::Gems> {
    let merged_apps: HashMap<_, _> = vault
        .reservation()
        .reserve_app_pouch()
        .grab()
        .await
        .app_pouch
        .as_ref()
        .expect("Vault reservations should never fail")
        .gems()
        .iter()
        .chain(new_apps.iter())
        .map(|(key, app)| (key.clone(), app.clone()))
        .collect();
    Arc::new(merged_apps)
}

async fn merged_deployments(
    vault: &Arc<Vault>,
    new_deployments: &pouch::deployment::Gems,
) -> Arc<pouch::deployment::Gems> {
    let merged_deployments: HashMap<_, _> = vault
        .reservation()
        .reserve_deployment_pouch()
        .grab()
        .await
        .deployment_pouch
        .as_ref()
        .expect("Vault reservations should never fail")
        .gems()
        .iter()
        .chain(new_deployments.iter())
        .map(|(id, deployment)| (id.clone(), deployment.clone()))
        .collect();
    Arc::new(merged_deployments)
}

async fn merged_manifests(
    vault: &Arc<Vault>,
    new_manifests: &pouch::manifest::Gems,
) -> Arc<pouch::manifest::Gems> {
    let merged_manifests: HashMap<_, _> = vault
        .reservation()
        .reserve_manifest_pouch()
        .grab()
        .await
        .manifest_pouch
        .as_ref()
        .expect("Vault reservations should never fail")
        .gems()
        .iter()
        .chain(new_manifests.iter())
        .map(|(key, manifest)| (key.clone(), manifest.clone()))
        .collect();
    Arc::new(merged_manifests)
}

pub async fn import_directory(
    quest: SyncQuest,
    vault: Arc<Vault>,
    lore: Arc<Lore>,
    manifest: v3::Manifest,
    src: PathBuf,
    dst: PathBuf,
) -> Result<(), ImportError> {
    let deployments = quest
        .lock()
        .await
        .create_sub_quest("Import deployments", |quest| {
            import_deployments(
                quest,
                manifest.contents.deployments.clone(),
                src.join("deployments"),
            )
        })
        .await
        .2;
    let manifests = quest
        .lock()
        .await
        .create_infallible_sub_quest("Import app manifests", |quest| {
            import_manifests(quest, manifest.contents.apps.clone(), src.join("apps"))
        })
        .await
        .2;
    let (apps_input_sender, apps_input_receiver) = tokio::sync::oneshot::channel();
    let apps = import_apps_quest(
        &quest,
        manifest.contents.apps.clone(),
        lore.clone(),
        apps_input_receiver,
        src.join("apps"),
    )
    .await;
    let (instances_input_sender, instances_input_receiver) = tokio::sync::oneshot::channel();
    let config = ImportInstanceConfig {
        lore,
        src: src.join("instances"),
        dst: dst.join("instances"),
    };
    let instances = import_instances_quest(
        &quest,
        manifest.contents.instances,
        instances_input_receiver,
        config,
    )
    .await;
    // Deployments and manifests can be imported concurrently as there are no dependencies
    let (new_deployments, new_manifests) = tokio::join!(deployments, manifests);
    let new_deployments = new_deployments?;

    // We need to take all deployments and manifests into account, not just the new ones
    // e.g. a flecsport with an app but no deployments or manifests
    let merged_deployments = merged_deployments(&vault, &new_deployments).await;
    let merged_manifests = merged_manifests(&vault, &new_manifests).await;
    _ = apps_input_sender.send((merged_manifests.clone(), merged_deployments.clone()));
    let new_apps = apps.await?;

    // We need to take all apps into account, not just the new ones
    // e.g. a flecsport with an instance but without the app
    let merged_apps = merged_apps(&vault, &new_apps).await;
    _ = instances_input_sender.send((merged_manifests, merged_deployments, merged_apps));
    let instances = instances.await?;
    import_to_vault(vault, new_deployments, new_manifests, new_apps, instances).await;
    Ok(())
}

pub async fn import_legacy_directory<U: UsbDeviceReader + 'static>(
    quest: SyncQuest,
    vault: Arc<Vault>,
    lore: Arc<Lore>,
    usb_device_reader: Arc<U>,
    manifest: v2::Manifest,
    src: PathBuf,
    dst: PathBuf,
) -> Result<(), ImportError> {
    let default_docker_deployments = vault
        .reservation()
        .reserve_deployment_pouch()
        .grab()
        .await
        .deployment_pouch
        .as_ref()
        .expect("Vault reservations should never fail")
        .default_deployments();
    let manifests = quest
        .lock()
        .await
        .create_infallible_sub_quest("Import app manifests", |quest| {
            import_legacy_manifests(quest, manifest.contents.apps.clone(), src.join("apps"))
        })
        .await
        .2;
    let (apps_input_sender, apps_input_receiver) = tokio::sync::oneshot::channel();
    let apps = import_legacy_apps_quest(
        &quest,
        manifest.contents.apps.clone(),
        lore.clone(),
        apps_input_receiver,
        default_docker_deployments.clone(),
        src.join("apps"),
    )
    .await;
    let (instances_input_sender, instances_input_receiver) = tokio::sync::oneshot::channel();
    let config = ImportInstanceConfig {
        lore,
        src: src.join("instances"),
        dst: dst.join("instances"),
    };
    let instances = import_legacy_instances_quest(
        &quest,
        usb_device_reader,
        manifest.contents.instances,
        instances_input_receiver,
        default_docker_deployments,
        config,
    )
    .await;
    let manifests = Arc::new(manifests.await);
    _ = apps_input_sender.send(manifests.clone());
    _ = instances_input_sender.send(manifests.clone());
    let apps = apps.await?;
    let instances = instances.await?;
    let Ok(manifests) = Arc::try_unwrap(manifests) else {
        return Err(ImportError::Logic("Unexpected strong references"));
    };
    import_to_vault(vault, Default::default(), manifests, apps, instances).await;
    Ok(())
}

pub async fn import_deployments(
    quest: SyncQuest,
    deployments: Vec<DeploymentId>,
    path: PathBuf,
) -> Result<pouch::deployment::Gems, ImportDeploymentError> {
    let mut results = Vec::new();
    {
        let mut quest = quest.lock().await;
        for deployment in deployments {
            let result = quest
                .create_sub_quest(format!("Import {deployment}"), |quest| {
                    import_deployment(quest, deployment, path.clone())
                })
                .await
                .2;
            results.push(result);
        }
    }
    join_all(results)
        .await
        .into_iter()
        .map(|result| result.map(|deployment| (deployment.id().clone(), deployment)))
        .collect()
}

pub async fn import_deployment(
    _quest: SyncQuest,
    deployment: DeploymentId,
    path: PathBuf,
) -> Result<Deployment, ImportDeploymentError> {
    let deployment_path = path.join(format!("{deployment}.json"));
    let deployment = tokio::fs::read(&deployment_path).await?;
    let deployment: SerializedDeployment = serde_json::from_slice(&deployment)?;
    Ok(deployment.into())
}

pub async fn import_manifests(
    quest: SyncQuest,
    app_keys: Vec<AppKey>,
    path: PathBuf,
) -> pouch::manifest::Gems {
    let mut results = Vec::new();
    {
        let mut quest = quest.lock().await;
        for app_key in app_keys.iter() {
            let result = quest
                .create_sub_quest(format!("Import manifest for {app_key}"), |quest| {
                    import_manifest(quest, app_key.clone(), path.clone())
                })
                .await
                .2;
            results.push(result);
        }
    }
    let mut manifests = pouch::manifest::Gems::new();
    for (manifest, app_key) in join_all(results).await.into_iter().zip(app_keys) {
        match manifest {
            Err(e) => error!("Failed to import manifest for {app_key}: {e}"),
            Ok(manifest) => {
                if let Some(replaced_manifest) = manifests.insert(app_key, manifest) {
                    warn!(
                        "Replaced manifest {} during import",
                        replaced_manifest.key()
                    )
                }
            }
        }
    }
    manifests
}

pub async fn import_legacy_manifests(
    quest: SyncQuest,
    app_keys: Vec<AppKey>,
    path: PathBuf,
) -> pouch::manifest::Gems {
    let mut results = Vec::new();
    {
        let mut quest = quest.lock().await;
        for app_key in app_keys.iter() {
            let result = quest
                .create_sub_quest(format!("Import manifest for {app_key}"), |quest| {
                    import_legacy_manifest(quest, app_key.clone(), path.clone())
                })
                .await
                .2;
            results.push(result);
        }
    }
    let mut manifests = pouch::manifest::Gems::new();
    for (manifest, app_key) in join_all(results).await.into_iter().zip(app_keys) {
        match manifest {
            Err(e) => error!("Failed to import manifest for {app_key}: {e}"),
            Ok(manifest) => {
                if let Some(replaced_manifest) = manifests.insert(app_key, manifest) {
                    warn!(
                        "Replaced manifest {} during import",
                        replaced_manifest.key()
                    )
                }
            }
        }
    }
    manifests
}

async fn read_manifest(manifest_path: &Path) -> Result<AppManifest, ImportManifestError> {
    let manifest = tokio::fs::read_to_string(manifest_path).await?;
    let manifest = flecs_app_manifest::AppManifestVersion::from_str(&manifest)?;
    let manifest = flecs_app_manifest::AppManifest::try_from(manifest)?;
    let manifest = AppManifest::try_from(manifest)?;
    Ok(manifest)
}

pub async fn import_manifest(
    _quest: SyncQuest,
    app_key: AppKey,
    path: PathBuf,
) -> Result<AppManifest, ImportManifestError> {
    let manifest_path = path.join(format!(
        "{}_{}/{}_{}.manifest.json",
        app_key.name, app_key.version, app_key.name, app_key.version
    ));
    read_manifest(&manifest_path).await
}

pub async fn import_legacy_manifest(
    _quest: SyncQuest,
    app_key: AppKey,
    path: PathBuf,
) -> Result<AppManifest, ImportManifestError> {
    let manifest_path = path.join(format!("{}_{}.json", app_key.name, app_key.version));
    read_manifest(&manifest_path).await
}

async fn import_apps_quest(
    quest: &SyncQuest,
    app_keys: Vec<AppKey>,
    lore: Arc<Lore>,
    input_recv: tokio::sync::oneshot::Receiver<(
        Arc<pouch::manifest::Gems>,
        Arc<pouch::deployment::Gems>,
    )>,
    path: PathBuf,
) -> BoxFuture<'static, Result<HashMap<AppKey, App>, RecvError>> {
    quest
        .lock()
        .await
        .create_sub_quest("Import apps", |quest| async move {
            let (manifests, deployments) = input_recv.await?;
            Ok(import_apps(quest, app_keys, manifests, deployments, lore, path).await)
        })
        .await
        .2
}

async fn import_legacy_apps_quest(
    quest: &SyncQuest,
    app_keys: Vec<AppKey>,
    lore: Arc<Lore>,
    input_recv: tokio::sync::oneshot::Receiver<Arc<pouch::manifest::Gems>>,
    default_deployments: DefaultDeployments,
    path: PathBuf,
) -> BoxFuture<'static, Result<pouch::app::Gems, RecvError>> {
    quest
        .lock()
        .await
        .create_sub_quest("Import apps", |quest| async move {
            let manifests = input_recv.await?;
            Ok(
                import_legacy_apps(quest, app_keys, manifests, default_deployments, lore, path)
                    .await,
            )
        })
        .await
        .2
}

pub async fn import_apps(
    quest: SyncQuest,
    app_keys: Vec<AppKey>,
    manifests: Arc<pouch::manifest::Gems>,
    deployments: Arc<pouch::deployment::Gems>,
    lore: Arc<Lore>,
    path: PathBuf,
) -> pouch::app::Gems {
    let mut results = Vec::new();
    {
        let mut quest = quest.lock().await;
        for app_key in app_keys.iter() {
            let result = quest
                .create_sub_quest(format!("Import app {app_key}"), |quest| {
                    import_app(
                        quest,
                        app_key.clone(),
                        manifests.clone(),
                        deployments.clone(),
                        lore.clone(),
                        path.clone(),
                    )
                })
                .await
                .2;
            results.push(result);
        }
    }
    let mut apps = pouch::app::Gems::new();
    for (app, app_key) in join_all(results).await.into_iter().zip(app_keys) {
        match app {
            Err(e) => error!("Failed to import app {app_key}: {e}"),
            Ok(app) => {
                if let Some(replaced_app) = apps.insert(app_key, app) {
                    warn!("Replaced app {} during import", replaced_app.key)
                }
            }
        }
    }
    apps
}

pub async fn import_app(
    quest: SyncQuest,
    app_key: AppKey,
    manifests: Arc<pouch::manifest::Gems>,
    deployments: Arc<pouch::deployment::Gems>,
    lore: Arc<Lore>,
    path: PathBuf,
) -> Result<App, ImportAppError> {
    let path = path.join(format!("{}_{}", app_key.name, app_key.version));
    let app_path = path.join(format!("{}_{}.json", app_key.name, app_key.version));
    let app = tokio::fs::read(&app_path).await?;
    let app: AppDeserializable = serde_json::from_slice(&app)?;
    let app = try_create_app(app, &manifests, &deployments)?;
    app.import(quest, lore, path).await?;
    Ok(app)
}

pub async fn import_legacy_apps(
    quest: SyncQuest,
    app_keys: Vec<AppKey>,
    manifests: Arc<pouch::manifest::Gems>,
    default_deployments: DefaultDeployments,
    lore: Arc<Lore>,
    path: PathBuf,
) -> pouch::app::Gems {
    let mut results = Vec::new();
    {
        let mut quest = quest.lock().await;
        for app_key in app_keys.iter() {
            let result = quest
                .create_sub_quest(format!("Import app {app_key}"), |quest| {
                    import_legacy_app(
                        quest,
                        app_key.clone(),
                        manifests.clone(),
                        default_deployments.clone(),
                        lore.clone(),
                        path.clone(),
                    )
                })
                .await
                .2;
            results.push(result);
        }
    }
    let mut apps = pouch::app::Gems::new();
    for (app, app_key) in join_all(results).await.into_iter().zip(app_keys) {
        match app {
            Err(e) => error!("Failed to import app {app_key}: {e}"),
            Ok(app) => {
                if let Some(replaced_app) = apps.insert(app_key, app) {
                    warn!("Replaced app {} during import", replaced_app.key)
                }
            }
        }
    }
    apps
}

pub async fn import_legacy_app(
    quest: SyncQuest,
    app_key: AppKey,
    manifests: Arc<pouch::manifest::Gems>,
    default_deployments: DefaultDeployments,
    lore: Arc<Lore>,
    path: PathBuf,
) -> Result<App, ImportAppError> {
    let app = try_create_legacy_app(app_key.clone(), &manifests, default_deployments)?;
    let app_dir = path.join(format!("{}_{}", app_key.name, app_key.version));
    tokio::fs::create_dir_all(&app_dir).await?;
    match app.manifest() {
        AppManifest::Multi(manifest) => {
            for service in manifest.services_with_image_info() {
                let name = service.name;
                let image = service.image;
                let old_path =
                    path.join(format!("{}_{}.{name}.tar", app_key.name, app_key.version));
                let new_path = app_dir.join(format!("{image}.tar"));
                tokio::fs::rename(old_path, new_path).await?;
            }
        }
        AppManifest::Single(_) => {
            let filename = format!("{}_{}.json", app_key.name, app_key.version);
            let old_path = path.join(&filename);
            let new_path = app_dir.join(filename);
            tokio::fs::rename(old_path, new_path).await?;
            let filename = format!("{}_{}.tar", app_key.name, app_key.version);
            let old_path = path.join(&filename);
            let new_path = app_dir.join(filename);
            tokio::fs::rename(old_path, new_path).await?;
        }
    }
    app.import(quest, lore, app_dir).await?;
    Ok(app)
}

#[derive(Clone)]
pub struct ImportInstanceConfig {
    pub lore: Arc<Lore>,
    pub src: PathBuf,
    pub dst: PathBuf,
}

async fn import_legacy_instances_quest<U: UsbDeviceReader + 'static>(
    quest: &SyncQuest,
    usb_device_reader: Arc<U>,
    instances: Vec<legacy::deployment::Instance>,
    input_recv: tokio::sync::oneshot::Receiver<Arc<pouch::manifest::Gems>>,
    default_docker_deployments: DefaultDeployments,
    config: ImportInstanceConfig,
) -> BoxFuture<'static, Result<pouch::instance::Gems, RecvError>> {
    quest
        .lock()
        .await
        .create_sub_quest("Import instances", |quest| async move {
            let manifests = input_recv.await?;
            Ok(import_legacy_instances(
                quest,
                usb_device_reader,
                instances,
                manifests,
                default_docker_deployments,
                config,
            )
            .await)
        })
        .await
        .2
}

pub async fn import_legacy_instances<U: UsbDeviceReader + 'static>(
    quest: SyncQuest,
    usb_device_reader: Arc<U>,
    legacy_instances: Vec<legacy::deployment::Instance>,
    manifests: Arc<pouch::manifest::Gems>,
    default_docker_deployments: DefaultDeployments,
    config: ImportInstanceConfig,
) -> pouch::instance::Gems {
    let mut results = Vec::new();
    {
        let mut quest = quest.lock().await;
        for instance in legacy_instances.clone() {
            let config = {
                ImportInstanceConfig {
                    src: config.src.join(&instance.instance_id),
                    dst: config.dst.join(&instance.instance_id),
                    lore: config.lore.clone(),
                }
            };
            let result = quest
                .create_sub_quest(
                    format!(
                        "Import instance {} from {}",
                        instance.instance_id,
                        config.src.display()
                    ),
                    |quest| {
                        import_legacy_instance(
                            quest,
                            manifests.clone(),
                            default_docker_deployments.clone(),
                            usb_device_reader.clone(),
                            instance,
                            config,
                        )
                    },
                )
                .await
                .2;
            results.push(result);
        }
    }
    let mut instances = pouch::instance::Gems::new();
    for (instance, legacy_instance) in join_all(results).await.into_iter().zip(legacy_instances) {
        match instance {
            Err(e) => error!(
                "Failed to import instance {}: {e}",
                legacy_instance.instance_id
            ),
            Ok(instance) => {
                if let Some(replaced_instance) = instances.insert(instance.id(), instance) {
                    warn!("Replaced instance {} during import", replaced_instance.id())
                }
            }
        }
    }
    instances
}

pub async fn import_legacy_instance<U: UsbDeviceReader>(
    quest: SyncQuest,
    manifests: Arc<pouch::manifest::Gems>,
    default_docker_deployments: DefaultDeployments,
    usb_device_reader: Arc<U>,
    instance: legacy::deployment::Instance,
    ImportInstanceConfig { lore, src, dst }: ImportInstanceConfig,
) -> Result<Instance, ImportInstanceError> {
    let Some(manifest) = manifests.get(&instance.app_key).cloned() else {
        return Err(CreateInstanceError::NoManifest(instance.app_key).into());
    };
    let mut instance = match (manifest, default_docker_deployments) {
        (
            AppManifest::Single(manifest),
            DefaultDeployments {
                docker: Some(Deployment::Docker(deployment)),
                ..
            },
        ) => Instance::Docker(
            DockerInstance::try_create_from_legacy(
                lore,
                instance,
                usb_device_reader.as_ref(),
                manifest,
                deployment,
            )
            .await?,
        ),
        (
            AppManifest::Multi(manifest),
            DefaultDeployments {
                compose: Some(Deployment::Compose(deployment)),
                ..
            },
        ) => Instance::Compose(
            ComposeInstance::try_create_from_legacy(lore, instance, manifest, deployment).await?,
        ),
        _ => return Err(CreateInstanceError::NoFittingDeployment.into()),
    };
    instance.import(quest, src, dst).await?;
    Ok(instance)
}

async fn import_instances_quest(
    quest: &SyncQuest,
    instances: Vec<InstanceId>,
    input_recv: tokio::sync::oneshot::Receiver<(
        Arc<pouch::manifest::Gems>,
        Arc<pouch::deployment::Gems>,
        Arc<pouch::app::Gems>,
    )>,
    config: ImportInstanceConfig,
) -> BoxFuture<'static, Result<HashMap<InstanceId, Instance>, RecvError>> {
    quest
        .lock()
        .await
        .create_sub_quest("Import instances", |quest| async move {
            let (manifests, deployments, apps) = input_recv.await?;
            Ok(import_instances(quest, instances, manifests, deployments, apps, config).await)
        })
        .await
        .2
}

pub async fn import_instances(
    quest: SyncQuest,
    instance_ids: Vec<InstanceId>,
    manifests: Arc<pouch::manifest::Gems>,
    deployments: Arc<pouch::deployment::Gems>,
    apps: Arc<pouch::app::Gems>,
    config: ImportInstanceConfig,
) -> pouch::instance::Gems {
    let mut results = Vec::new();
    {
        let mut quest = quest.lock().await;
        for instance_id in instance_ids.iter() {
            let result = quest
                .create_sub_quest(format!("Import instance {instance_id}"), |quest| {
                    import_instance(
                        quest,
                        manifests.clone(),
                        deployments.clone(),
                        apps.clone(),
                        *instance_id,
                        config.clone(),
                    )
                })
                .await
                .2;
            results.push(result);
        }
    }
    let mut instances = pouch::instance::Gems::new();
    for (instance, instance_id) in join_all(results).await.into_iter().zip(instance_ids) {
        match instance {
            Err(e) => error!("Failed to import instance {instance_id}: {e}"),
            Ok(instance) => {
                if let Some(replaced_instance) = instances.insert(instance_id, instance) {
                    warn!("Replaced instance {} during import", replaced_instance.id())
                }
            }
        }
    }
    instances
}

pub async fn import_instance(
    quest: SyncQuest,
    manifests: Arc<pouch::manifest::Gems>,
    deployments: Arc<pouch::deployment::Gems>,
    apps: Arc<pouch::app::Gems>,
    id: InstanceId,
    ImportInstanceConfig { lore, src, dst }: ImportInstanceConfig,
) -> Result<Instance, ImportInstanceError> {
    let src = src.join(id.to_string());
    let dst = dst.join(id.to_string());
    let instance_path = src.join("instance.json");
    let instance = tokio::fs::read(&instance_path).await?;
    let instance: InstanceDeserializable = serde_json::from_slice(&instance)?;
    if !apps.contains_key(instance.app_key()) {
        return Err(ImportInstanceError::AppNotPresent(
            instance.app_key().clone(),
        ));
    }
    let mut instance = Instance::try_create_with_state(lore, instance, &manifests, &deployments)?;
    instance.import(quest, src, dst).await?;
    Ok(instance)
}

pub async fn validate_import(
    manifest: Manifest,
    path: PathBuf,
) -> Result<Manifest, ReadImportManifestError> {
    Ok(match manifest {
        Manifest::V2(manifest) => validate_v2_import(manifest, path).await?.into(),
        Manifest::V3(manifest) => validate_v3_import(manifest, path).await?.into(),
    })
}

fn validate_architecture_match(arch: &str) -> Result<(), ReadImportManifestError> {
    let sys_info = try_create_system_info()?;
    if sys_info.arch != arch {
        Err(ReadImportManifestError::ArchitectureMismatch {
            device_arch: sys_info.arch,
            import_arch: arch.to_string(),
        })
    } else {
        Ok(())
    }
}

async fn validate_v2_import(
    manifest: v2::Manifest,
    path: PathBuf,
) -> Result<v2::Manifest, ReadImportManifestError> {
    validate_architecture_match(&manifest.device.sysinfo.arch)?;
    for app_key in &manifest.contents.apps {
        let manifest_path = path.join(format!("apps/{}_{}.json", app_key.name, app_key.version));
        if !tokio::fs::try_exists(&manifest_path).await? {
            return Err(ReadImportManifestError::Invalid(anyhow::anyhow!(
                "App {app_key} is listed in import manifest but no app manifest was found at {manifest_path:?}"
            )));
        }
    }
    Ok(manifest)
}

async fn validate_v3_import(
    manifest: v3::Manifest,
    path: PathBuf,
) -> Result<v3::Manifest, ReadImportManifestError> {
    // TODO: Check that everything has unique ids (manifest -> AppKey, app -> AppKey, instance -> InstanceId, deployment -> DeploymentId
    validate_architecture_match(&manifest.device.sysinfo.arch)?;
    for deployment in &manifest.contents.deployments {
        if !tokio::fs::try_exists(path.join(format!("deployments/{deployment}.json"))).await? {
            return Err(ReadImportManifestError::Invalid(anyhow::anyhow!(
                "Deployment {deployment} is listed in import manifest but was not found"
            )));
        }
    }
    for app_key in &manifest.contents.apps {
        let app_path = path.join(format!("apps/{}_{}", app_key.name, app_key.version));
        if !tokio::fs::try_exists(&app_path).await? {
            return Err(ReadImportManifestError::Invalid(anyhow::anyhow!(
                "App {app_key} is listed in import manifest but directory {app_path:?} was not found"
            )));
        }
        let app_json = app_path.join(format!("{}_{}.json", app_key.name, app_key.version));
        if !tokio::fs::try_exists(app_json).await? {
            return Err(ReadImportManifestError::Invalid(anyhow::anyhow!(
                "App {app_key} is listed in import manifest but was not found"
            )));
        }
        let manifest_path = app_path.join(format!(
            "{}_{}.manifest.json",
            app_key.name, app_key.version
        ));
        if !tokio::fs::try_exists(manifest_path).await? {
            return Err(ReadImportManifestError::Invalid(anyhow::anyhow!(
                "App {app_key} is listed in import manifest but no app manifest was found"
            )));
        }
    }
    for instance_id in &manifest.contents.instances {
        let instance_path = path
            .join("instances")
            .join(instance_id.to_string())
            .join("instance.json");
        if !tokio::fs::try_exists(instance_path).await? {
            return Err(ReadImportManifestError::Invalid(anyhow::anyhow!(
                "Instance {instance_id} is listed in import manifest but was not found"
            )));
        }
    }
    Ok(manifest)
}

pub async fn import_to_vault(
    vault: Arc<Vault>,
    deployments: pouch::deployment::Gems,
    manifests: pouch::manifest::Gems,
    apps: pouch::app::Gems,
    instances: HashMap<InstanceId, Instance>,
) {
    let GrabbedPouches {
        manifest_pouch_mut: Some(ref mut manifest_pouch),
        deployment_pouch_mut: Some(ref mut deployment_pouch),
        app_pouch_mut: Some(ref mut app_pouch),
        instance_pouch_mut: Some(ref mut instance_pouch),
        ..
    } = vault
        .reservation()
        .reserve_manifest_pouch_mut()
        .reserve_deployment_pouch_mut()
        .reserve_app_pouch_mut()
        .reserve_instance_pouch_mut()
        .grab()
        .await
    else {
        unreachable!("Vault reservations should never fail")
    };
    for (id, deployment) in deployments {
        if let Some(deployment) = deployment_pouch.gems_mut().insert(id, deployment) {
            debug!("Replaced deployment {}", deployment.id());
        }
    }
    for (key, manifest) in manifests {
        if let Some(app) = app_pouch.gems_mut().get_mut(&key) {
            app.replace_manifest(manifest.clone());
        }
        for instance in instance_pouch
            .gems_mut()
            .values_mut()
            .filter(|instance| instance.app_key() == manifest.key())
        {
            instance.replace_manifest(manifest.clone());
        }
        if let Some(manifest) = manifest_pouch.gems_mut().insert(key, manifest) {
            debug!("Replaced manifest {}", manifest.key());
        }
    }
    for (key, app) in apps {
        if let Some(app) = app_pouch.gems_mut().insert(key, app) {
            debug!("Replaced app {}", app.key);
        }
    }
    for (id, instance) in instances {
        if let Some(instance) = instance_pouch.gems_mut().insert(id, instance) {
            debug!("Replaced instance {}", instance.id());
        }
    }
}
