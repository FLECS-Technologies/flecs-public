pub mod compose;
pub mod docker;
mod id;
pub mod status;

use crate::jeweler::deployment::DeploymentId;
use crate::jeweler::gem::instance::status::InstanceStatus;
use crate::jeweler::gem::manifest::AppManifest;
use crate::vault::pouch;
use crate::vault::pouch::AppKey;
use async_trait::async_trait;
pub use id::*;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Instance {
    Compose(compose::ComposeInstance),
    Docker(docker::DockerInstance),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(clippy::large_enum_variant)]
#[serde(tag = "type")]
pub enum InstanceDeserializable {
    Compose(compose::ComposeInstanceDeserializable),
    Docker(docker::DockerInstanceDeserializable),
}

impl InstanceDeserializable {
    pub fn id(&self) -> InstanceId {
        match self {
            InstanceDeserializable::Compose(compose) => compose.id,
            InstanceDeserializable::Docker(docker) => docker.id,
        }
    }
    pub fn deployment_id(&self) -> &DeploymentId {
        match self {
            InstanceDeserializable::Compose(compose) => &compose.deployment_id,
            InstanceDeserializable::Docker(docker) => &docker.deployment_id,
        }
    }
    pub fn app_key(&self) -> &AppKey {
        match self {
            InstanceDeserializable::Compose(compose) => &compose.app_key,
            InstanceDeserializable::Docker(docker) => &docker.app_key,
        }
    }
}

#[async_trait]
pub trait InstanceCommon {
    fn id(&self) -> InstanceId;
    fn app_key(&self) -> &AppKey;
    fn manifest(&self) -> AppManifest;
    fn replace_manifest(&mut self, manifest: AppManifest) -> AppManifest;
    async fn generate_info(&self) -> crate::Result<flecsd_axum_server::models::AppInstance>;
    async fn generate_detailed_info(
        &self,
    ) -> crate::Result<flecsd_axum_server::models::InstancesInstanceIdGet200Response>;
    async fn status(&self) -> anyhow::Result<InstanceStatus>;
    fn desired_status(&self) -> InstanceStatus;
    fn taken_ipv4_addresses(&self) -> Vec<Ipv4Addr>;
}

impl Deref for Instance {
    type Target = dyn InstanceCommon;

    fn deref(&self) -> &Self::Target {
        match self {
            Instance::Compose(compose) => compose,
            Instance::Docker(docker) => docker,
        }
    }
}

impl DerefMut for Instance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Instance::Compose(compose) => compose,
            Instance::Docker(docker) => docker,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CreateInstanceError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Instance {
    pub fn try_create_with_state(
        instance: InstanceDeserializable,
        manifests: &pouch::manifest::Gems,
        deployments: &pouch::deployment::Gems,
    ) -> Result<Instance, CreateInstanceError> {
        match instance {
            InstanceDeserializable::Compose(compose) => {
                let instance = compose::ComposeInstance::try_create_with_state(
                    compose,
                    manifests,
                    deployments,
                )?;
                Ok(Self::Compose(instance))
            }
            InstanceDeserializable::Docker(docker) => {
                let instance =
                    docker::DockerInstance::try_create_with_state(docker, manifests, deployments)?;
                Ok(Self::Docker(instance))
            }
        }
    }
}
