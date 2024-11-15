use crate::jeweler::app::AppDeployment;
use crate::jeweler::instance::InstanceDeployment;
use crate::jeweler::network::NetworkDeployment;
use crate::jeweler::volume::VolumeDeployment;
use async_trait::async_trait;
use erased_serde::serialize_trait_object;
use std::fmt::{Debug, Formatter};

pub type DeploymentId = String;

#[async_trait]
pub trait Deployment:
    Send
    + Sync
    + AppDeployment
    + InstanceDeployment
    + NetworkDeployment
    + VolumeDeployment
    + erased_serde::Serialize
{
    fn id(&self) -> DeploymentId;
}

serialize_trait_object!(Deployment);

impl Debug for dyn Deployment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Deployment: {}", self.id())
    }
}
