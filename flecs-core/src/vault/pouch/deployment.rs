pub use super::Result;
use crate::jeweler;
use crate::jeweler::deployment::Deployment as DeploymentTrait;
use crate::jeweler::gem::deployment::docker::DockerDeployment;
use crate::relic::serde::SerdeIteratorAdapter;
use crate::vault::pouch::Pouch;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(tag = "type")]
pub enum Deployment {
    Docker(DockerDeployment),
}

impl Default for Deployment {
    fn default() -> Self {
        Self::Docker(DockerDeployment::new(
            "DefaultDockerDeployment".to_string(),
            PathBuf::from("/var/run/docker.sock"),
        ))
    }
}

impl From<Deployment> for Arc<dyn DeploymentTrait> {
    fn from(value: Deployment) -> Self {
        match value {
            Deployment::Docker(d) => Arc::new(d),
        }
    }
}

pub type DeploymentId = String;

#[derive(Debug)]
pub struct DeploymentPouch {
    deployments: HashMap<DeploymentId, Arc<dyn DeploymentTrait>>,
    path: PathBuf,
}

impl Pouch for DeploymentPouch {
    type Gems = HashMap<DeploymentId, Arc<dyn DeploymentTrait>>;

    fn gems(&self) -> &Self::Gems {
        &self.deployments
    }

    fn gems_mut(&mut self) -> &mut Self::Gems {
        &mut self.deployments
    }
}

impl DeploymentPouch {
    pub(in super::super) fn close(&mut self) -> Result<()> {
        fs::create_dir_all(&self.path)?;
        let file = fs::File::create(self.deployments_path())?;
        serde_json::to_writer_pretty(file, &SerdeIteratorAdapter::new(self.deployments.values()))?;
        Ok(())
    }

    pub(in super::super) fn open(&mut self) -> Result<()> {
        let deployments = Self::read_deployments(&self.deployments_path())?;
        self.deployments = deployments
            .into_iter()
            .map(|d| {
                let id = d.id();
                let deployment: Arc<dyn jeweler::deployment::Deployment> = Arc::new(match d {
                    Deployment::Docker(d) => d,
                });
                (id, deployment)
            })
            .collect();
        Ok(())
    }
}

impl Deployment {
    pub fn id(&self) -> DeploymentId {
        match self {
            Self::Docker(deployment) => deployment.id().clone(),
        }
    }
}

impl DeploymentPouch {
    pub fn new(path: &Path) -> DeploymentPouch {
        Self {
            deployments: Default::default(),
            path: path.to_path_buf(),
        }
    }

    fn deployments_path(&self) -> PathBuf {
        self.path.join("deployments.json")
    }

    fn read_deployments(path: &Path) -> Result<Vec<Deployment>> {
        match path.try_exists() {
            Ok(false) => Ok(Vec::new()),
            _ => Ok(serde_json::from_reader(fs::File::open(path)?)?),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::tests::prepare_test_path;
    use serde_json::json;
    use std::path::Path;
    use testdir::testdir;

    pub fn test_deployment_pouch(
        default_deployment: Option<Arc<dyn crate::jeweler::deployment::Deployment>>,
    ) -> DeploymentPouch {
        let deployment = default_deployment.unwrap_or_else(|| {
            let mut deployment = MockedDeployment::new();
            deployment
                .expect_id()
                .returning(|| "DefaultMockedDeploymentId".to_string());
            Arc::new(deployment)
        });
        let deployments: HashMap<String, Arc<dyn crate::jeweler::deployment::Deployment>> =
            HashMap::from([(deployment.id(), deployment)]);
        DeploymentPouch {
            path: testdir!().join("deployments"),
            deployments,
        }
    }

    fn test_deployment_json() -> serde_json::Value {
        json!([{
                "type": "Docker",
                "id": TEST_DEPLOYMENT_ID,
                "path": TEST_DEPLOYMENT_SOCK_PATH
        }])
    }

    fn test_deployments_json() -> serde_json::Value {
        json!([{
                "type": "Docker",
                "id": "test1",
                "path": TEST_DEPLOYMENT_SOCK_PATH
        },{
                "type": "Docker",
                "id": "test2",
                "path": TEST_DEPLOYMENT_SOCK_PATH
        },{
                "type": "Docker",
                "id": "test3",
                "path": TEST_DEPLOYMENT_SOCK_PATH
        },{
                "type": "Docker",
                "id": "test4",
                "path": TEST_DEPLOYMENT_SOCK_PATH
        }])
    }

    const TEST_DEPLOYMENT_ID: &str = "some-deployment-id";
    const TEST_DEPLOYMENT_SOCK_PATH: &str = "/path/to/docker.sock";

    #[test]
    fn deployments_path() {
        let path = Path::new("/some/random/path/");
        let deployment_pouch = DeploymentPouch::new(path);
        assert_eq!(
            deployment_pouch.deployments_path(),
            PathBuf::from("/some/random/path/deployments.json")
        )
    }

    #[test]
    fn deployment_gems() {
        let deployment: Arc<dyn jeweler::deployment::Deployment> = Arc::new(DockerDeployment::new(
            TEST_DEPLOYMENT_ID.to_string(),
            PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
        ));
        let gems = HashMap::from([(TEST_DEPLOYMENT_ID.to_string(), deployment)]);
        let mut deployment_pouch = DeploymentPouch {
            deployments: gems,
            path: prepare_test_path(module_path!(), "gems"),
        };
        assert_eq!(deployment_pouch.gems().len(), 1);
        assert_eq!(
            deployment_pouch
                .gems()
                .get(TEST_DEPLOYMENT_ID)
                .unwrap()
                .id(),
            TEST_DEPLOYMENT_ID
        );
        assert_eq!(deployment_pouch.gems_mut().len(), 1);
        assert_eq!(
            deployment_pouch
                .gems_mut()
                .get(TEST_DEPLOYMENT_ID)
                .unwrap()
                .id(),
            TEST_DEPLOYMENT_ID
        );
    }

    #[test]
    fn new_deployment_pouch() {
        let path = prepare_test_path(module_path!(), "new_pouch");
        let deployment_pouch = DeploymentPouch::new(&path);
        assert!(deployment_pouch.deployments.is_empty());
        assert_eq!(deployment_pouch.path, path);
    }

    #[tokio::test]
    async fn open_deployment_pouch() {
        let path = prepare_test_path(module_path!(), "open_pouch");
        let json = serde_json::to_string(&test_deployment_json()).unwrap();
        let mut deployment_pouch = DeploymentPouch::new(&path);
        fs::write(deployment_pouch.deployments_path(), json).unwrap();
        deployment_pouch.open().unwrap();
        assert_eq!(deployment_pouch.deployments.len(), 1);
        assert_eq!(
            deployment_pouch
                .deployments
                .get(TEST_DEPLOYMENT_ID)
                .unwrap()
                .id(),
            TEST_DEPLOYMENT_ID
        );
    }

    #[tokio::test]
    async fn close_deployment_pouch() {
        let path = prepare_test_path(module_path!(), "close_pouch").join("deployments.json");
        let json = test_deployment_json();
        let deployment: Arc<dyn jeweler::deployment::Deployment> = Arc::new(DockerDeployment::new(
            TEST_DEPLOYMENT_ID.to_string(),
            PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
        ));
        let mut deployment_pouch = DeploymentPouch {
            deployments: HashMap::from([(TEST_DEPLOYMENT_ID.to_string(), deployment)]),
            path: path.parent().unwrap().to_path_buf(),
        };
        deployment_pouch.close().unwrap();
        let file_content: serde_json::Value =
            serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();
        assert_eq!(json, file_content);
    }

    #[tokio::test]
    async fn read_deployments() {
        let path = prepare_test_path(module_path!(), "read").join("deployments.json");

        let json = serde_json::to_string(&test_deployments_json()).unwrap();
        fs::write(&path, json).unwrap();
        assert_eq!(
            DeploymentPouch::read_deployments(&path).unwrap(),
            vec![
                Deployment::Docker(DockerDeployment::new(
                    "test1".to_string(),
                    PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
                )),
                Deployment::Docker(DockerDeployment::new(
                    "test2".to_string(),
                    PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
                )),
                Deployment::Docker(DockerDeployment::new(
                    "test3".to_string(),
                    PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
                )),
                Deployment::Docker(DockerDeployment::new(
                    "test4".to_string(),
                    PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
                ))
            ]
        );
    }
}
