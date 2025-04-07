use crate::jeweler::deployment::{Deployment, DeploymentId};
use crate::jeweler::gem::app::{try_create_app, App, AppDeserializable};
use crate::jeweler::gem::instance::{
    try_create_instance, Instance, InstanceDeserializable, InstanceId,
};
use crate::jeweler::gem::manifest::AppManifest;
use crate::quest::SyncQuest;
use crate::relic::system::info::try_create_system_info;
use crate::sorcerer::exportius::manifest::{v3, Manifest};
use crate::sorcerer::importius::{
    ImportAppError, ImportDeploymentError, ImportError, ImportInstanceError, ImportManifestError,
    ReadImportManifestError,
};
use crate::vault::pouch::deployment::Deployment as PouchDeployment;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::{pouch, GrabbedPouches, Vault};
use futures_util::future::{join_all, BoxFuture};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tracing::debug;

pub async fn read_import_manifest(
    quest: SyncQuest,
    src: PathBuf,
) -> Result<v3::Manifest, ReadImportManifestError> {
    let manifest_path = src.join("manifest.json");
    let manifest = tokio::fs::read(&manifest_path).await?;
    let Manifest::V3(manifest) = serde_json::from_slice(&manifest)?;
    quest
        .lock()
        .await
        .create_sub_quest("Validate import", |_quest| {
            validate_import(manifest, src.clone())
        })
        .await
        .2
        .await
}

pub async fn import_directory(
    quest: SyncQuest,
    vault: Arc<Vault>,
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
        .create_sub_quest("Import app manifests", |quest| {
            import_manifests(quest, manifest.contents.apps.clone(), src.join("apps"))
        })
        .await
        .2;
    let (apps_input_sender, apps_input_receiver) = tokio::sync::oneshot::channel();
    let apps = import_apps_quest(
        &quest,
        manifest.contents.apps.clone(),
        apps_input_receiver,
        src.join("apps"),
    )
    .await;
    let (instances_input_sender, instances_input_receiver) = tokio::sync::oneshot::channel();
    let instances = import_instances_quest(
        &quest,
        manifest.contents.instances,
        instances_input_receiver,
        src.join("instances"),
        dst.join("instances"),
    )
    .await;
    // Deployments and manifests can be imported concurrently as there are no dependencies
    let (deployments, manifests) = tokio::join!(deployments, manifests);
    let (deployments, manifests) = (Arc::new(deployments?), Arc::new(manifests?));
    _ = apps_input_sender.send((manifests.clone(), deployments.clone()));
    _ = instances_input_sender.send((manifests.clone(), deployments.clone()));
    let apps = apps.await?;
    let instances = instances.await?;
    let Ok(deployments) = Arc::try_unwrap(deployments) else {
        return Err(ImportError::Logic("Unexpected strong references"));
    };
    let Ok(manifests) = Arc::try_unwrap(manifests) else {
        return Err(ImportError::Logic("Unexpected strong references"));
    };
    import_to_vault(vault, deployments, manifests, apps, instances).await;
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
        join_all(results)
            .await
            .into_iter()
            .map(|result| result.map(|deployment| (deployment.id(), deployment)))
            .collect()
    }
}

pub async fn import_deployment(
    _quest: SyncQuest,
    deployment: DeploymentId,
    path: PathBuf,
) -> Result<Arc<dyn Deployment>, ImportDeploymentError> {
    let deployment_path = path.join(format!("{deployment}.json"));
    let deployment = tokio::fs::read(&deployment_path).await?;
    let PouchDeployment::Docker(deployment) = serde_json::from_slice(&deployment)?;
    Ok(Arc::new(deployment))
}

pub async fn import_manifests(
    quest: SyncQuest,
    app_keys: Vec<AppKey>,
    path: PathBuf,
) -> Result<pouch::manifest::Gems, ImportManifestError> {
    let mut results = Vec::new();
    {
        let mut quest = quest.lock().await;
        for app_key in app_keys {
            let result = quest
                .create_sub_quest(format!("Import manifest for {app_key}"), |quest| {
                    import_manifest(quest, app_key, path.clone())
                })
                .await
                .2;
            results.push(result);
        }
        join_all(results)
            .await
            .into_iter()
            .map(|result| result.map(|manifest| (manifest.key.clone(), manifest)))
            .collect()
    }
}

pub async fn import_manifest(
    _quest: SyncQuest,
    app_key: AppKey,
    path: PathBuf,
) -> Result<Arc<AppManifest>, ImportManifestError> {
    let manifest_path = path.join(format!(
        "{}_{}.manifest.json",
        app_key.name, app_key.version
    ));
    let manifest = tokio::fs::read_to_string(&manifest_path).await?;
    let manifest = flecs_app_manifest::AppManifestVersion::from_str(&manifest)?;
    let manifest = flecs_app_manifest::AppManifest::try_from(manifest)?;
    let manifest = AppManifest::try_from(manifest)?;
    Ok(Arc::new(manifest))
}

async fn import_apps_quest(
    quest: &SyncQuest,
    app_keys: Vec<AppKey>,
    input_recv: tokio::sync::oneshot::Receiver<(
        Arc<pouch::manifest::Gems>,
        Arc<pouch::deployment::Gems>,
    )>,
    path: PathBuf,
) -> BoxFuture<'static, Result<HashMap<AppKey, App>, ImportAppError>> {
    quest
        .lock()
        .await
        .create_sub_quest("Import apps", |quest| async move {
            let (manifests, deployments) = input_recv.await?;
            import_apps(quest, app_keys, manifests, deployments, path).await
        })
        .await
        .2
}

pub async fn import_apps(
    quest: SyncQuest,
    app_keys: Vec<AppKey>,
    manifests: Arc<pouch::manifest::Gems>,
    deployments: Arc<pouch::deployment::Gems>,
    path: PathBuf,
) -> Result<pouch::app::Gems, ImportAppError> {
    let mut results = Vec::new();
    {
        let mut quest = quest.lock().await;
        for app_key in app_keys {
            let result = quest
                .create_sub_quest(format!("Import app {app_key}"), |quest| {
                    import_app(
                        quest,
                        app_key,
                        manifests.clone(),
                        deployments.clone(),
                        path.clone(),
                    )
                })
                .await
                .2;
            results.push(result);
        }
        join_all(results)
            .await
            .into_iter()
            .map(|result| result.map(|app| (app.key.clone(), app)))
            .collect()
    }
}

pub async fn import_app(
    quest: SyncQuest,
    app_key: AppKey,
    manifests: Arc<pouch::manifest::Gems>,
    deployments: Arc<pouch::deployment::Gems>,
    path: PathBuf,
) -> Result<App, ImportAppError> {
    let app_path = path.join(format!("{}_{}.json", app_key.name, app_key.version));
    let app = tokio::fs::read(&app_path).await?;
    let app: AppDeserializable = serde_json::from_slice(&app)?;
    let app = try_create_app(app, &manifests, &deployments)?;
    let mut results = Vec::new();
    let path = path.join(format!("{}_{}.tar", app_key.name, app_key.version));
    let mut quest = quest.lock().await;
    for data in app.deployments.values() {
        let deployment = data.deployment().clone();
        let path = path.clone();
        let result = quest
            .create_sub_quest(
                format!("Import {app_key} to {}", deployment.id()),
                move |quest| async move { deployment.import_app(quest, path).await },
            )
            .await
            .2;
        results.push(result);
    }
    for result in join_all(results).await {
        result?;
    }
    Ok(app)
}

async fn import_instances_quest(
    quest: &SyncQuest,
    instances: Vec<InstanceId>,
    input_recv: tokio::sync::oneshot::Receiver<(
        Arc<pouch::manifest::Gems>,
        Arc<pouch::deployment::Gems>,
    )>,
    src: PathBuf,
    dst: PathBuf,
) -> BoxFuture<'static, Result<HashMap<InstanceId, Instance>, ImportInstanceError>> {
    quest
        .lock()
        .await
        .create_sub_quest("Import instances", |quest| async move {
            let (manifests, deployments) = input_recv.await?;
            import_instances(quest, instances, manifests, deployments, src, dst).await
        })
        .await
        .2
}

pub async fn import_instances(
    quest: SyncQuest,
    instance_ids: Vec<InstanceId>,
    manifests: Arc<pouch::manifest::Gems>,
    deployments: Arc<pouch::deployment::Gems>,
    src: PathBuf,
    dst: PathBuf,
) -> Result<HashMap<InstanceId, Instance>, ImportInstanceError> {
    let mut results = Vec::new();
    {
        let mut quest = quest.lock().await;
        for instance_id in instance_ids {
            let result = quest
                .create_sub_quest(format!("Import instance {instance_id}"), |quest| {
                    import_instance(
                        quest,
                        manifests.clone(),
                        deployments.clone(),
                        src.join(instance_id.to_string()),
                        dst.join(instance_id.to_string()),
                    )
                })
                .await
                .2;
            results.push(result);
        }
        join_all(results)
            .await
            .into_iter()
            .map(|result| result.map(|instance| (instance.id, instance)))
            .collect()
    }
}

pub async fn import_instance(
    quest: SyncQuest,
    manifests: Arc<pouch::manifest::Gems>,
    deployments: Arc<pouch::deployment::Gems>,
    src: PathBuf,
    dst: PathBuf,
) -> Result<Instance, ImportInstanceError> {
    let instance_path = src.join("instance.json");
    let instance = tokio::fs::read(&instance_path).await?;
    let instance: InstanceDeserializable = serde_json::from_slice(&instance)?;
    let instance = try_create_instance(instance, &manifests, &deployments)?;
    let mut results = Vec::new();
    {
        for volume in instance.config.volume_mounts.values() {
            let path = src.join(format!("{}.tar", volume.name));
            results.push(
                instance
                    .import_volume_quest(&quest, path.clone(), volume.name.clone())
                    .await,
            )
        }
    }
    for result in join_all(results).await {
        result?;
    }
    for config_file in &instance.manifest.config_files {
        let src = src.join(&config_file.host_file_name);
        let dst = dst.join("conf");
        tokio::fs::create_dir_all(&dst).await?;
        let dst = dst.join(&config_file.host_file_name);
        tokio::fs::copy(src, dst).await?;
    }
    Ok(instance)
}

pub async fn validate_import(
    manifest: v3::Manifest,
    path: PathBuf,
) -> Result<v3::Manifest, ReadImportManifestError> {
    let sys_info = try_create_system_info()?;
    if sys_info.arch != manifest.device.sysinfo.arch {
        return Err(ReadImportManifestError::ArchitectureMismatch {
            device_arch: sys_info.arch,
            import_arch: manifest.device.sysinfo.arch,
        });
    }
    for deployment in &manifest.contents.deployments {
        if !tokio::fs::try_exists(path.join(format!("deployments/{deployment}.json"))).await? {
            return Err(ReadImportManifestError::Invalid(anyhow::anyhow!(
                "Deployment {deployment} is listed in import manifest but was not found"
            )));
        }
    }
    for app_key in &manifest.contents.apps {
        let app_path = path.join(format!("apps/{}_{}.json", app_key.name, app_key.version));
        if !tokio::fs::try_exists(app_path).await? {
            return Err(ReadImportManifestError::Invalid(anyhow::anyhow!(
                "App {app_key} is listed in import manifest but was not found"
            )));
        }
        let image_path = path.join(format!("apps/{}_{}.tar", app_key.name, app_key.version));
        if !tokio::fs::try_exists(image_path).await? {
            return Err(ReadImportManifestError::Invalid(anyhow::anyhow!(
                "App {app_key} is listed in import manifest but no image was found"
            )));
        }
        let manifest_path = path.join(format!(
            "apps/{}_{}.manifest.json",
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
            app.set_manifest(manifest.clone())
        }
        for instance in instance_pouch
            .gems_mut()
            .values_mut()
            .filter(|instance| instance.app_key() == manifest.key)
        {
            instance.replace_manifest(manifest.clone());
        }
        if let Some(manifest) = manifest_pouch.gems_mut().insert(key, manifest) {
            debug!("Replaced manifest {}", manifest.key);
        }
    }
    for (key, app) in apps {
        if let Some(app) = app_pouch.gems_mut().insert(key, app) {
            debug!("Replaced app {}", app.key);
        }
    }
    for (id, instance) in instances {
        if let Some(instance) = instance_pouch.gems_mut().insert(id, instance) {
            debug!("Replaced instance {}", instance.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
