use super::instance::InstanceId;
use super::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

type VolumeId = String;
#[derive(Default)]
pub struct VolumeConfig {
    // TBD
}

#[async_trait]
pub trait VolumeDeployment {
    async fn create_volume(&self, config: VolumeConfig) -> Result<VolumeId>;
    async fn delete_volume(&self, id: VolumeId) -> Result<()>;
    async fn import_volume(&self, path: &Path) -> Result<VolumeId>;
    async fn export_volume(&self, id: VolumeId, path: &Path) -> Result<()>; // TODO: Arguments, return type
    async fn volumes(&self, instance_id: InstanceId) -> Result<HashMap<VolumeId, VolumeConfig>>;
    async fn export_volumes(&self, instance_id: InstanceId, path: &Path) -> Result<()> {
        // TODO: more logic
        for volume_id in self.volumes(instance_id).await?.keys() {
            self.export_volume(volume_id.clone(), path).await?;
        }
        Ok(())
    }
}
