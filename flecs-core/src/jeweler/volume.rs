use super::Result;
use crate::quest::SyncQuest;
use async_trait::async_trait;
// TODO: Use more generic struct as soon as the second type of deployment is implemented
pub use bollard::models::Volume;
use std::path::Path;

pub(crate) type VolumeId = String;

#[async_trait]
pub trait VolumeDeployment {
    async fn create_volume(&self, quest: SyncQuest, name: &str) -> Result<VolumeId>;
    async fn delete_volume(&self, _quest: SyncQuest, id: VolumeId) -> Result<()>;
    async fn import_volume(
        &self,
        _quest: SyncQuest,
        src: &Path,
        container_path: &Path,
        name: &str,
        image: &str,
    ) -> Result<VolumeId>;
    async fn export_volume(
        &self,
        quest: SyncQuest,
        id: VolumeId,
        export_path: &Path,
        container_path: &Path,
        image: &str,
    ) -> Result<()>;

    async fn inspect_volume(&self, id: VolumeId) -> Result<Option<Volume>>;
}
