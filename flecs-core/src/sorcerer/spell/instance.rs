pub use super::Result;
use crate::jeweler::deployment::Deployment;
use crate::jeweler::gem::instance::Instance;
use crate::jeweler::gem::manifest::AppManifest;
use crate::quest::SyncQuest;
use std::sync::Arc;

pub async fn create_instance(
    quest: SyncQuest,
    deployment: Arc<dyn Deployment>,
    manifest: Arc<AppManifest>,
    name: String,
) -> Result<Instance> {
    Instance::create(quest, deployment, manifest, name).await
}
