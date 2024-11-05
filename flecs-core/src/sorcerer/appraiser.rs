pub use super::Result;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::Vault;
use anyhow::anyhow;
use flecs_app_manifest::AppManifest;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Assumptions:
/// Vault provides
/// - Apps that contain
///     - a reference to deployments
///     - a state for every referenced deployment which includes all instances
/// - Immutable manifests in an Arc<>
/// - Immutable deployments in an Arc<>
pub async fn install_app(vault: Arc<Vault>, key: AppKey) -> Result<i32> {
    let session_id = vault.get_secrets().await.get_session_id();
    let mut grab = vault
        .reservation()
        .reserve_app_pouch_mut()
        .reserve_manifest_pouch_mut()
        .grab()
        .await;
    // TODO: Get deployments from vault
    let deployment = DockerDeployment {
        id: "default_deployment".to_string(),
        socket: PathBuf::from("/run/docker.sock".to_string()),
    };
    let deployment: Arc<dyn Deployment> = Arc::new(deployment);
    let deployments = HashMap::from([("default_deployment".to_string(), deployment)]);
    // TODO: Download manifest only if app has none / there is no app yet
    let manifests = grab.manifest_pouch_mut.as_mut().unwrap();
    let manifest = match manifests.gems_mut().entry(key.clone()) {
        Entry::Vacant(x) => {
            let config = crate::lore::console_client_config::default().await;
            let manifest = crate::sorcerer::spell::manifest::download_manifest(
                config,
                &session_id.id.unwrap_or_default(),
                &key.name,
                &key.version,
            )
            .await?
            .try_into()?;
            x.insert(Arc::new(manifest))
        }
        Entry::Occupied(x) => x.into_mut(),
    }
    .clone();
    let apps = grab.app_pouch_mut.as_mut().unwrap();

    // Create struct "App" if necessary and put into vault
    let app = apps
        .gems_mut()
        .entry(key.clone())
        .or_insert_with(|| todo!());
    // TODO: Current struct "App" in vault does not fit new concept
    let mut app = App {
        key,
        deployments: HashMap::new(),
        manifest: Some(manifest),
    };

    for (id, deployment) in deployments {
        app.deployments.insert(id, (AppData::default(), deployment));
    }

    app.install_all()?;

    // TODO: job id
    Ok(0)
}

trait Deployment: Send + Sync {
    fn install_app(&self, manifest: &AppManifest) -> Result<()>;
    fn id(&self) -> String;
    // fn uninstall_app(key: AppKey) -> Result<()>;
    // ...
}

struct DockerDeployment {
    id: String,
    socket: PathBuf,
}

impl Deployment for DockerDeployment {
    fn install_app(&self, manifest: &AppManifest) -> Result<()> {
        // Check image status
        // Acquire download token if necessary
        // Pull image if necessary
        Ok(())
    }
    fn id(&self) -> String {
        self.id.clone()
    }
}

#[derive(Default)]
enum AppStatus {
    #[default]
    None,
    Installed,
    NotInstalled,
}

struct Instance {}

#[derive(Default)]
struct AppData {
    desired: AppStatus,
    instances: Vec<Instance>,
}

struct App {
    key: AppKey,
    deployments: HashMap<String, (AppData, Arc<dyn Deployment>)>,
    manifest: Option<Arc<AppManifest>>,
}

impl App {
    /// Install to all registered deployments
    fn install_all(&mut self) -> Result<()> {
        let manifest = self
            .manifest
            .as_ref()
            .ok_or_else(|| anyhow!("no manifest found"))?;
        for (id, (state, deployment)) in &mut self.deployments {
            state.desired = AppStatus::Installed;
            // Check if app is already installed
            // Create future that installs the app to 'deployment' and returns the install-state and id
            // Collect all futures
        }
        // Await on all futures in parallel
        Ok(())
    }
}
