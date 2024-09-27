use crate::jeweler::app::AppDeployment;
use crate::jeweler::instance::InstanceDeployment;
use crate::jeweler::network::NetworkDeployment;
use async_trait::async_trait;

pub type DeploymentId = String;

#[async_trait]
pub trait Deployment: Send + Sync + AppDeployment + InstanceDeployment + NetworkDeployment {
    async fn id(&self) -> DeploymentId;
}
