use crate::jeweler::GetDeploymentId;
use crate::jeweler::app::AppDeployment;
use crate::jeweler::network::NetworkDeployment;
use crate::jeweler::volume::VolumeDeployment;
use crate::lore::NetworkLoreRef;
use async_trait::async_trait;
use erased_serde::serialize_trait_object;
use std::fmt::Debug;
use std::net::IpAddr;

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
    async fn core_default_address(&self, lore: NetworkLoreRef) -> Option<IpAddr>;
}

serialize_trait_object!(CommonDeployment);
