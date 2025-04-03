mod exportius_impl;
use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::jeweler::gem::instance::InstanceId;
use crate::quest::SyncQuest;
use crate::relic::async_flecstract::archive_to_file;
use crate::sorcerer::spell::flecsport::{
    ExportAppError, ExportDeploymentError, ExportInstanceError,
};
use crate::sorcerer::Sorcerer;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::Vault;
use async_trait::async_trait;
pub use exportius_impl::*;
#[cfg(test)]
use mockall::automock;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

pub mod manifest {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "_schemaVersion")]
    pub enum Manifest {
        #[serde(rename = "3.0.0")]
        V3(v3::Manifest),
    }

    /// Schema version 3.0.0
    pub mod v3 {
        use crate::jeweler::deployment::DeploymentId;
        use crate::vault::pouch::instance::InstanceId;
        use crate::vault::pouch::AppKey;
        use flecsd_axum_server::models::SystemInfo;
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug, Clone)]
        pub struct Manifest {
            pub time: std::time::SystemTime,
            pub contents: Contents,
            pub device: Device,
            pub version: Version,
        }

        #[derive(Serialize, Deserialize, Debug, Clone)]
        pub struct Contents {
            pub apps: Vec<AppKey>,
            pub instances: Vec<InstanceId>,
            pub deployments: Vec<DeploymentId>,
        }

        #[derive(Serialize, Deserialize, Debug, Clone)]
        pub struct Device {
            pub sysinfo: SystemInfo,
            pub hostname: Option<String>,
        }

        #[derive(Serialize, Deserialize, Debug, Clone)]
        pub struct Version {
            pub core: String,
            pub api: String,
        }

        impl Default for Version {
            fn default() -> Self {
                Self {
                    core: crate::lore::CORE_VERSION.to_string(),
                    api: crate::lore::API_VERSION.to_string(),
                }
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CreateExportError {
    #[error("Failed to export apps: {0}")]
    App(#[from] ExportAppError),
    #[error("Failed to export instances: {0}")]
    Instance(#[from] ExportInstanceError),
    #[error("Failed to export deployments: {0}")]
    Deployment(#[from] ExportDeploymentError),
    #[error("Failed to create export manifest: {0}")]
    Manifest(String),
    #[error("Failed to archive export: {0}")]
    Archive(#[from] ArchiveError),
    #[error("I/O Error during export: {0}")]
    IO(#[from] std::io::Error),
    #[error("Failed to get system info: {0}")]
    SysInfo(String),
    #[error("Unexpected error: {0}")]
    Other(String),
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ArchiveError {
    #[error("Failed to archive export {path:?}: {error}")]
    Any { path: PathBuf, error: String },
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Exportius: Sorcerer + 'static {
    /// Creates an export as a tar archive at the exports base path (default /var/lib/flecs/exports)
    /// with the current time as the filename.
    async fn create_export_archive<F: Floxy + 'static>(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        apps: Vec<AppKey>,
        instances: Vec<InstanceId>,
    ) -> Result<String, CreateExportError> {
        let path = quest
            .lock()
            .await
            .create_sub_quest("Create export".to_string(), |quest| {
                Self::create_export(quest, vault, floxy, apps, instances)
            })
            .await
            .2
            .await?;
        let mut archive_path = path.clone();
        archive_path.set_extension("tar");
        let result = archive_path
            .file_stem()
            .ok_or_else(|| {
                CreateExportError::Other(format!("Could not get file name from {archive_path:?}"))
            })?
            .to_string_lossy()
            .to_string();
        let archive_result = quest
            .lock()
            .await
            .create_sub_quest("Archive export".to_string(), |quest| {
                Self::archive_export(quest, path.clone(), archive_path)
            })
            .await
            .2
            .await;
        tokio::fs::remove_dir_all(path).await?;
        archive_result?;
        Ok(result)
    }

    /// Creates an export in a directory at the exports base path (default /var/lib/flecs/exports)
    /// with the current time as the directory name. The export consists of an export manifest and
    /// the specified content. See [Exportius::export_content] for details.
    /// Structure:
    ///     /var/lib/flecs/exports
    ///         /{timestamp}
    ///             /manifest.json
    ///             /apps
    ///             /instances
    ///             /deployments
    async fn create_export<F: Floxy + 'static>(
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        apps: Vec<AppKey>,
        instances: Vec<InstanceId>,
    ) -> Result<PathBuf, CreateExportError> {
        let now = std::time::SystemTime::now();
        let export_dir = PathBuf::from(crate::lore::flecsport::BASE_PATH).join(
            now.duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis()
                .to_string(),
        );
        quest
            .lock()
            .await
            .create_sub_quest(
                format!("Prepare directory {export_dir:?} for export"),
                |_quest| {
                    let export_dir = export_dir.clone();
                    async move {
                        match tokio::fs::remove_dir_all(&export_dir).await {
                            Ok(_) => {}
                            Err(e) => match e.kind() {
                                std::io::ErrorKind::NotFound => {}
                                _ => return Err(e),
                            },
                        }
                        tokio::fs::create_dir_all(&export_dir).await
                    }
                },
            )
            .await
            .2
            .await?;
        let deployments: Vec<_> = vault
            .reservation()
            .reserve_deployment_pouch()
            .grab()
            .await
            .deployment_pouch
            .as_ref()
            .expect("Vault reservations should never fail")
            .gems()
            .keys()
            .cloned()
            .collect();

        let manifest = manifest::Manifest::V3(manifest::v3::Manifest {
            time: now,
            contents: manifest::v3::Contents {
                apps: apps.clone(),
                instances: instances.clone(),
                deployments,
            },
            device: manifest::v3::Device {
                sysinfo: crate::relic::system::info::try_create_system_info()
                    .map_err(|e| CreateExportError::SysInfo(e.to_string()))?,
                hostname: crate::relic::system::hostname().ok(),
            },
            version: Default::default(),
        });

        if let Err(e) = tokio::fs::write(
            export_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).expect("Manifest should always be serializable"),
        )
        .await
        {
            _ = tokio::fs::remove_dir_all(&export_dir).await;
            return Err(e.into());
        };

        if let Err(e) = quest
            .lock()
            .await
            .create_sub_quest(format!("Export content to {export_dir:?}"), |quest| {
                Self::export_content(
                    quest,
                    vault.clone(),
                    floxy,
                    apps,
                    instances,
                    export_dir.clone(),
                )
            })
            .await
            .2
            .await
        {
            _ = tokio::fs::remove_dir_all(&export_dir).await;
            return Err(e);
        };
        Ok(export_dir)
    }

    /// Exports the specified content to the given 'export_dir'. The content consists of apps,
    /// instances and deployments taken from the 'vault'. See [Exportius::export_apps],
    /// [Exportius::export_instances] and [Exportius::export_deployments] for details.
    /// Structure:
    ///     export_dir
    ///         /apps
    ///         /instances
    ///         /deployments
    async fn export_content<F: Floxy + 'static>(
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        apps: Vec<AppKey>,
        instances: Vec<InstanceId>,
        export_dir: PathBuf,
    ) -> Result<(), CreateExportError> {
        let apps_result = quest
            .lock()
            .await
            .create_sub_quest(format!("Export apps to {export_dir:?}"), |quest| {
                Self::export_apps(quest, vault.clone(), apps, export_dir.join("apps"))
            })
            .await
            .2;
        let instances_result = quest
            .lock()
            .await
            .create_sub_quest(format!("Export instances to {export_dir:?}"), |quest| {
                Self::export_instances(
                    quest,
                    vault.clone(),
                    floxy,
                    instances,
                    export_dir.join("instances"),
                )
            })
            .await
            .2;
        let deployments_result = quest
            .lock()
            .await
            .create_sub_quest(format!("Export deployments to {export_dir:?}"), |quest| {
                Self::export_deployments(quest, vault.clone(), export_dir.join("deployments"))
            })
            .await
            .2;
        let (apps_result, instances_result, deployments_result) =
            futures::join!(apps_result, instances_result, deployments_result);
        apps_result?;
        instances_result?;
        deployments_result?;
        Ok(())
    }

    async fn archive_export(
        _quest: SyncQuest,
        src: PathBuf,
        dst: PathBuf,
    ) -> Result<(), ArchiveError> {
        archive_to_file(&src, &dst, true)
            .await
            .map_err(|e| ArchiveError::Any {
                path: src,
                error: e.to_string(),
            })
    }

    async fn export_apps(
        quest: SyncQuest,
        vault: Arc<Vault>,
        apps: Vec<AppKey>,
        path: PathBuf,
    ) -> Result<(), ExportAppError>;

    async fn export_instances<F: Floxy + 'static>(
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        instances: Vec<InstanceId>,
        path: PathBuf,
    ) -> Result<(), ExportInstanceError>;

    async fn export_deployments(
        quest: SyncQuest,
        vault: Arc<Vault>,
        path_buf: PathBuf,
    ) -> Result<(), ExportDeploymentError>;
}

#[cfg(test)]
impl Sorcerer for MockExportius {}
