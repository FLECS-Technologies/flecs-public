mod compose_impl;

use crate::jeweler::deployment::CommonDeployment;
use crate::jeweler::gem::instance::Logs;
use crate::jeweler::gem::instance::status::InstanceStatus;
use crate::jeweler::gem::manifest::multi::AppManifestMulti;
use crate::lore::NetworkLoreRef;
use async_trait::async_trait;
pub use compose_impl::*;
use erased_serde::serialize_trait_object;
use std::path::Path;

#[async_trait]
pub trait ComposeDeployment: CommonDeployment {
    async fn start_instance(
        &self,
        manifest: &AppManifestMulti,
        lore: NetworkLoreRef,
        workdir: &Path,
    ) -> Result<(), ExecuteCompose>;
    async fn stop_instance(
        &self,
        manifest: &AppManifestMulti,
        lore: NetworkLoreRef,
    ) -> Result<(), ExecuteCompose>;
    async fn instance_status(
        &self,
        manifest: &AppManifestMulti,
        lore: NetworkLoreRef,
    ) -> anyhow::Result<Vec<InstanceStatus>>;
    async fn instance_logs(
        &self,
        manifest: &AppManifestMulti,
        lore: NetworkLoreRef,
    ) -> anyhow::Result<Logs>;
}

serialize_trait_object!(ComposeDeployment);
