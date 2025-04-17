mod importius_impl;

use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::jeweler::gem::instance::CreateInstanceError;
use crate::jeweler::gem::instance::docker::TransferIpError;
use crate::quest::SyncQuest;
use crate::sorcerer::Sorcerer;
use crate::vault::Vault;
use async_trait::async_trait;
pub use importius_impl::*;
#[cfg(test)]
use mockall::automock;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot::error::RecvError;

#[derive(thiserror::Error, Debug)]
pub enum ReadImportManifestError {
    #[error("Error reading manifest: {0}")]
    Parse(#[from] flecs_app_manifest::ManifestError),
    #[error(transparent)]
    Invalid(#[from] anyhow::Error),
    #[error("IO error during import: {0}")]
    IO(#[from] std::io::Error),
    #[error("Error during deserialization: {0}")]
    Ser(#[from] serde_json::Error),
    #[error("Import has different architecture than device ({device_arch}): {import_arch}")]
    ArchitectureMismatch {
        device_arch: String,
        import_arch: String,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum ImportDeploymentError {
    #[error(transparent)]
    Invalid(#[from] anyhow::Error),
    #[error("IO error during import: {0}")]
    IO(#[from] std::io::Error),
    #[error("Error during deserialization: {0}")]
    Ser(#[from] serde_json::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ImportAppError {
    #[error(transparent)]
    Invalid(#[from] anyhow::Error),
    #[error("IO error during import: {0}")]
    IO(#[from] std::io::Error),
    #[error("Error during deserialization: {0}")]
    Ser(#[from] serde_json::Error),
    #[error("Internal logic error: {0}")]
    RecvError(#[from] RecvError),
}

#[derive(thiserror::Error, Debug)]
pub enum ImportManifestError {
    #[error("Error reading manifest: {0}")]
    Parse(#[from] flecs_app_manifest::ManifestError),
    #[error("IO error during import: {0}")]
    IO(#[from] std::io::Error),
    #[error("Error during deserialization: {0}")]
    Ser(#[from] serde_json::Error),
    #[error(transparent)]
    Invalid(#[from] anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ImportInstanceError {
    #[error("Error reading manifest: {0}")]
    Parse(#[from] flecs_app_manifest::ManifestError),
    #[error("IO error during import: {0}")]
    IO(#[from] std::io::Error),
    #[error("Error during deserialization: {0}")]
    Ser(#[from] serde_json::Error),
    #[error(transparent)]
    Invalid(#[from] anyhow::Error),
    #[error("Internal logic error: {0}")]
    RecvError(#[from] RecvError),
    #[error(transparent)]
    TransferIp(#[from] TransferIpError),
    #[error(transparent)]
    Create(#[from] CreateInstanceError),
}

#[derive(thiserror::Error, Debug)]
pub enum ImportError {
    #[error("Failed to extract import {import:?}: {error}")]
    Extract {
        import: PathBuf,
        error: anyhow::Error,
    },
    #[error(transparent)]
    Deployment(#[from] ImportDeploymentError),
    #[error(transparent)]
    Manifest(#[from] ImportManifestError),
    #[error(transparent)]
    ImportManifest(#[from] ReadImportManifestError),
    #[error(transparent)]
    App(#[from] ImportAppError),
    #[error(transparent)]
    Instance(#[from] ImportInstanceError),
    #[error("Internal logic error {0}")]
    Logic(&'static str),
    #[error("Failed to stop instances before import")]
    InstanceStop(anyhow::Error),
    #[error("Failed to start instances after import")]
    InstanceStart(anyhow::Error),
    #[error("IO error during import: {0}")]
    IO(#[from] std::io::Error),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Importius: Sorcerer + 'static {
    async fn import_archive<F: Floxy + 'static>(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        archive_path: PathBuf,
        temp_path: PathBuf,
        base_path: PathBuf,
    ) -> Result<(), ImportError>;
}

#[cfg(test)]
impl Sorcerer for MockImportius {}
