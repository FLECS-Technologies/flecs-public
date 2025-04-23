use crate::jeweler::gem;
use crate::jeweler::gem::instance::compose::ComposeInstance;
use crate::jeweler::gem::instance::docker::DockerInstance;
use crate::jeweler::gem::instance::{CreateInstanceError, Instance};
use crate::jeweler::gem::manifest::AppManifest;
use crate::relic::device::usb::UsbDeviceReader;
use crate::vault::pouch::Pouch;
use crate::vault::{GrabbedPouches, Vault};
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info, warn};

pub mod app;
pub mod deployment;

pub const LEGACY_DEPLOYMENT_PATH: &str = "/var/lib/flecs/deployment";
pub const LEGACY_APPS_PATH: &str = "/var/lib/flecs/apps/apps.json";

#[derive(thiserror::Error, Debug)]
pub enum MigrateError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Deserialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("No default deployment available to migrate instances to")]
    NoDefaultDeployment,
    #[error(transparent)]
    CreateInstance(#[from] CreateInstanceError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub async fn migrate_docker_instances<U: UsbDeviceReader>(
    vault: Arc<Vault>,
    usb_device_reader: &U,
) -> Result<(), MigrateError> {
    let path = Path::new(LEGACY_DEPLOYMENT_PATH).join("docker.json");
    info!("Migrating docker instances from {path:?}");
    let docker_instances = tokio::fs::read_to_string(&path)
        .await
        .map_err(MigrateError::from)?;
    let docker_instances: deployment::Deployment =
        serde_json::from_str(&docker_instances).map_err(MigrateError::from)?;
    let mut grabbed_pouches = vault
        .reservation()
        .reserve_deployment_pouch()
        .reserve_manifest_pouch()
        .reserve_instance_pouch_mut()
        .grab()
        .await;
    let GrabbedPouches {
        manifest_pouch: Some(ref manifest_pouch),
        deployment_pouch: Some(ref deployment_pouch),
        instance_pouch_mut: Some(ref mut instance_pouch),
        ..
    } = grabbed_pouches
    else {
        unreachable!("Vault reservations should never fail")
    };
    let default_docker_deployment_id = deployment_pouch
        .default_docker_deployment()
        .ok_or(MigrateError::NoDefaultDeployment)?
        .id()
        .clone();
    let docker_instances = deployment::migrate_docker_deployment(
        docker_instances,
        usb_device_reader,
        &default_docker_deployment_id,
    )?;
    for instance in docker_instances {
        let instance_id = instance.id;
        match DockerInstance::try_create_with_state(
            instance,
            manifest_pouch.gems(),
            deployment_pouch.gems(),
        ) {
            Ok(instance) => {
                if let Some(old_instance) = instance_pouch
                    .gems_mut()
                    .insert(instance.id, Instance::Docker(instance))
                {
                    warn!("Replaced instance {} during migration", old_instance.id());
                };
                info!("Migrated docker instance {instance_id}");
            }
            Err(e) => {
                error!("Failed to migrate instance {instance_id}: {e}")
            }
        }
    }
    Ok(())
}

pub async fn migrate_compose_instances(vault: Arc<Vault>) -> Result<(), MigrateError> {
    let path = Path::new(LEGACY_DEPLOYMENT_PATH).join("compose.json");
    info!("Migrating compose instances from {path:?}");
    let compose_instances = tokio::fs::read_to_string(&path)
        .await
        .map_err(MigrateError::from)?;
    let compose_instances: deployment::Deployment =
        serde_json::from_str(&compose_instances).map_err(MigrateError::from)?;
    let mut grabbed_pouches = vault
        .reservation()
        .reserve_deployment_pouch()
        .reserve_manifest_pouch()
        .reserve_instance_pouch_mut()
        .grab()
        .await;
    let GrabbedPouches {
        manifest_pouch: Some(ref manifest_pouch),
        deployment_pouch: Some(ref deployment_pouch),
        instance_pouch_mut: Some(ref mut instance_pouch),
        ..
    } = grabbed_pouches
    else {
        unreachable!("Vault reservations should never fail")
    };
    let default_compose_deployment_id = deployment_pouch
        .default_compose_deployment()
        .ok_or(MigrateError::NoDefaultDeployment)?
        .id()
        .clone();
    let compose_instances =
        deployment::migrate_compose_deployment(compose_instances, &default_compose_deployment_id)?;
    for instance in compose_instances {
        let instance_id = instance.id;
        match ComposeInstance::try_create_with_state(
            instance,
            manifest_pouch.gems(),
            deployment_pouch.gems(),
        ) {
            Ok(instance) => {
                if let Some(old_instance) = instance_pouch
                    .gems_mut()
                    .insert(instance.id, Instance::Compose(instance))
                {
                    warn!("Replaced instance {} during migration", old_instance.id());
                };
                info!("Migrated compose instance {instance_id}");
            }
            Err(e) => {
                error!("Failed to migrate instance {instance_id}: {e}")
            }
        }
    }
    Ok(())
}

pub async fn read_legacy_apps() -> Result<Vec<app::App>, MigrateError> {
    let path = Path::new(LEGACY_APPS_PATH);
    info!("Migrating apps from {path:?}");
    let apps = tokio::fs::read_to_string(&path)
        .await
        .map_err(MigrateError::from)?;
    serde_json::from_str(&apps).map_err(MigrateError::from)
}

pub async fn migrate_apps(
    vault: Arc<Vault>,
    legacy_apps: Vec<app::App>,
) -> Result<(), MigrateError> {
    let mut grabbed_pouches = vault
        .reservation()
        .reserve_deployment_pouch()
        .reserve_manifest_pouch()
        .reserve_app_pouch_mut()
        .grab()
        .await;
    let GrabbedPouches {
        manifest_pouch: Some(ref manifest_pouch),
        deployment_pouch: Some(ref deployment_pouch),
        app_pouch_mut: Some(ref mut app_pouch),
        ..
    } = grabbed_pouches
    else {
        unreachable!("Vault reservations should never fail")
    };
    let default_compose_deployment = deployment_pouch
        .default_compose_deployment()
        .ok_or(MigrateError::NoDefaultDeployment)?;
    let default_docker_deployment = deployment_pouch
        .default_docker_deployment()
        .ok_or(MigrateError::NoDefaultDeployment)?;
    for app in legacy_apps {
        let desired = app::app_status_from_legacy(&app.desired);
        let (manifest, deployment) = match manifest_pouch.gems().get(&app.app_key) {
            None => {
                error!("Could not migrate {}, no manifest found", app.app_key);
                continue;
            }
            Some(manifest @ AppManifest::Single(_)) => {
                (manifest.clone(), default_docker_deployment.clone())
            }
            Some(manifest @ AppManifest::Multi(_)) => {
                (manifest.clone(), default_compose_deployment.clone())
            }
        };
        let mut app = gem::app::App::new(app.app_key, vec![deployment], manifest);
        app.set_desired(desired);
        let app_key = app.key.clone();
        if let Some(app) = app_pouch.gems_mut().insert(app.key.clone(), app) {
            warn!("Replaced app {} during migration", app.key);
        }
        info!("Migrated app {app_key}");
    }
    Ok(())
}
