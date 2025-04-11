pub mod compose;
pub mod docker;

use crate::jeweler::deployment::CommonDeployment;
use crate::jeweler::gem::deployment::compose::ComposeDeploymentImpl;
use crate::jeweler::gem::deployment::docker::DockerDeploymentImpl;
use crate::jeweler::GetDeploymentId;
use crate::vault::pouch::deployment::DeploymentId;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[derive(Clone)]
pub enum Deployment {
    Compose(Arc<dyn compose::ComposeDeployment>),
    Docker(Arc<dyn docker::DockerDeployment>),
}

impl GetDeploymentId for Deployment {
    fn deployment_id(&self) -> &DeploymentId {
        self.id()
    }
}

impl Deref for Deployment {
    type Target = dyn CommonDeployment;

    fn deref(&self) -> &Self::Target {
        match self {
            Deployment::Compose(compose) => compose.as_ref(),
            Deployment::Docker(docker) => docker.as_ref(),
        }
    }
}

impl Deployment {
    pub fn id(&self) -> &DeploymentId {
        match self {
            Self::Docker(deployment) => deployment.id(),
            Self::Compose(deployment) => deployment.id(),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum SerializedDeployment {
    Docker(DockerDeploymentImpl),
    Compose(ComposeDeploymentImpl),
}

impl From<SerializedDeployment> for Deployment {
    fn from(value: SerializedDeployment) -> Deployment {
        match value {
            SerializedDeployment::Docker(docker) => Self::Docker(Arc::new(docker)),
            SerializedDeployment::Compose(compose) => Self::Compose(Arc::new(compose)),
        }
    }
}
