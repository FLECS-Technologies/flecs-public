use crate::jeweler::GetDeploymentId;
use crate::jeweler::app::AppDeployment;
use crate::jeweler::network::NetworkDeployment;
use crate::jeweler::volume::VolumeDeployment;
use async_trait::async_trait;
use erased_serde::serialize_trait_object;
use std::fmt::Debug;

pub type DeploymentId = String;

#[async_trait]
pub trait CommonDeployment:
    Send
    + Sync
    + AppDeployment
    + NetworkDeployment
    + VolumeDeployment
    + GetDeploymentId
    + Debug
    + erased_serde::Serialize
{
    fn id(&self) -> &DeploymentId;
    fn is_default(&self) -> bool;
}

serialize_trait_object!(CommonDeployment);
