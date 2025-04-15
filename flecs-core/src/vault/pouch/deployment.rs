pub use super::Result;
use crate::jeweler::gem::deployment::Deployment;
use crate::jeweler::gem::deployment::SerializedDeployment;
use crate::relic::serde::SerdeIteratorAdapter;
use crate::vault::pouch::Pouch;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::error;

pub type DeploymentId = String;
pub type Gems = HashMap<DeploymentId, Deployment>;

#[derive(Debug)]
pub struct DeploymentPouch {
    deployments: Gems,
    default_docker_deployment_id: Option<DeploymentId>,
    default_compose_deployment_id: Option<DeploymentId>,
    path: PathBuf,
}

impl Pouch for DeploymentPouch {
    type Gems = Gems;

    fn gems(&self) -> &Self::Gems {
        &self.deployments
    }

    fn gems_mut(&mut self) -> &mut Self::Gems {
        &mut self.deployments
    }
}

impl DeploymentPouch {
    pub(in super::super) fn close(&mut self) -> Result<()> {
        self.set_default_deployments();
        fs::create_dir_all(&self.path)?;
        let file = fs::File::create(self.deployments_path())?;
        serde_json::to_writer_pretty(file, &SerdeIteratorAdapter::new(self.deployments.values()))?;
        Ok(())
    }

    pub(in super::super) fn open(&mut self) -> Result<()> {
        let path = self.deployments_path();
        let deployments = match Self::read_deployments(&path) {
            Ok(deployments) => deployments,
            Err(e) => {
                error!("Failed to read deployments from {path:?}: {e}");
                return Err(e);
            }
        };
        self.deployments = deployments
            .into_iter()
            .map(|d| (d.id().clone(), d))
            .collect();
        self.set_default_deployments();
        Ok(())
    }

    pub fn new(path: &Path) -> DeploymentPouch {
        Self {
            deployments: Default::default(),
            path: path.to_path_buf(),
            default_docker_deployment_id: Default::default(),
            default_compose_deployment_id: Default::default(),
        }
    }

    pub fn default_compose_deployment(&self) -> Option<Deployment> {
        self.deployments
            .get(self.default_compose_deployment_id.as_ref()?)
            .cloned()
    }

    pub fn default_docker_deployment(&self) -> Option<Deployment> {
        self.deployments
            .get(self.default_docker_deployment_id.as_ref()?)
            .cloned()
    }

    pub fn set_default_deployments(&mut self) {
        self.set_default_docker_deployment();
        self.set_default_compose_deployment();
    }

    pub fn set_default_docker_deployment(&mut self) {
        let mut docker_deployments = self
            .deployments
            .values()
            .filter_map(|deployment| match deployment {
                Deployment::Docker(docker) => Some(docker),
                _ => None,
            })
            .peekable();
        let mut id = docker_deployments.peek().map(|deployment| deployment.id());
        if let Some(default_id) = docker_deployments.find_map(|deployment| {
            if deployment.is_default() {
                Some(deployment.id())
            } else {
                None
            }
        }) {
            id = Some(default_id);
        }
        self.default_docker_deployment_id = id.cloned();
    }

    pub fn set_default_compose_deployment(&mut self) {
        let mut compose_deployments = self
            .deployments
            .values()
            .filter_map(|deployment| match deployment {
                Deployment::Compose(compose) => Some(compose),
                _ => None,
            })
            .peekable();
        let mut id = compose_deployments.peek().map(|deployment| deployment.id());
        if let Some(default_id) = compose_deployments.find_map(|deployment| {
            if deployment.is_default() {
                Some(deployment.id())
            } else {
                None
            }
        }) {
            id = Some(default_id);
        }
        self.default_compose_deployment_id = id.cloned();
    }

    fn deployments_path(&self) -> PathBuf {
        self.path.join("deployments.json")
    }

    fn read_deployments(path: &Path) -> Result<Vec<Deployment>> {
        match path.try_exists() {
            Ok(false) => Ok(Vec::new()),
            _ => {
                let deployments: Vec<SerializedDeployment> =
                    serde_json::from_reader(fs::File::open(path)?)?;
                let deployments = deployments.into_iter().map(Into::into).collect();
                Ok(deployments)
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::jeweler::gem::deployment::docker::DockerDeploymentImpl;
    use crate::jeweler::gem::deployment::docker::tests::MockedDockerDeployment;
    use crate::tests::prepare_test_path;
    use serde_json::json;
    use std::path::Path;
    use std::sync::Arc;
    use testdir::testdir;

    pub fn test_deployment_pouch(default_deployment: Option<Deployment>) -> DeploymentPouch {
        let deployment = default_deployment.unwrap_or_else(|| {
            let mut deployment = MockedDockerDeployment::new();
            deployment
                .expect_id()
                .return_const("DefaultMockedDeploymentId".to_string());
            deployment.expect_is_default().return_const(true);
            Deployment::Docker(Arc::new(deployment))
        });
        let default_docker_deployment_id = Some(deployment.id().clone());
        let deployments: HashMap<String, Deployment> =
            HashMap::from([(deployment.id().clone(), deployment)]);
        DeploymentPouch {
            path: testdir!().join("deployments"),
            deployments,
            default_docker_deployment_id,
            default_compose_deployment_id: None,
        }
    }

    fn test_deployment_json() -> serde_json::Value {
        json!([{
                "type": "Docker",
                "id": TEST_DEPLOYMENT_ID,
                "path": TEST_DEPLOYMENT_SOCK_PATH,
                "is_default": false,
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
        let deployment = Deployment::Docker(Arc::new(DockerDeploymentImpl::new(
            TEST_DEPLOYMENT_ID.to_string(),
            PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
        )));
        let default_docker_deployment_id = Some(deployment.id().clone());
        let gems = HashMap::from([(TEST_DEPLOYMENT_ID.to_string(), deployment)]);
        let mut deployment_pouch = DeploymentPouch {
            deployments: gems,
            path: prepare_test_path(module_path!(), "gems"),
            default_docker_deployment_id,
            default_compose_deployment_id: None,
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
        let deployment = Deployment::Docker(Arc::new(DockerDeploymentImpl::new(
            TEST_DEPLOYMENT_ID.to_string(),
            PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
        )));
        let default_docker_deployment_id = Some(deployment.id().clone());
        let mut deployment_pouch = DeploymentPouch {
            deployments: HashMap::from([(TEST_DEPLOYMENT_ID.to_string(), deployment)]),
            path: path.parent().unwrap().to_path_buf(),
            default_docker_deployment_id,
            default_compose_deployment_id: None,
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
        let deployments = DeploymentPouch::read_deployments(&path).unwrap();
        assert_eq!(deployments.len(), 4);
        for (i, deployment) in deployments.iter().enumerate() {
            assert_eq!(deployment.id(), &format!("test{}", i + 1));
        }
    }

    #[test]
    fn get_default_deployment_no_id() {
        let deployment = Deployment::Docker(Arc::new(MockedDockerDeployment::new()));
        let pouch = DeploymentPouch {
            path: testdir!(),
            default_docker_deployment_id: None,
            default_compose_deployment_id: None,
            deployments: HashMap::from([("MockDeployment".to_string(), deployment)]),
        };
        assert!(pouch.default_docker_deployment().is_none());
        assert!(pouch.default_compose_deployment().is_none());
    }

    #[test]
    fn get_default_deployment_no_deployment() {
        let deployment = Deployment::Docker(Arc::new(MockedDockerDeployment::new()));
        let pouch = DeploymentPouch {
            path: testdir!(),
            default_docker_deployment_id: Some("DefaultDockerDeployment".to_string()),
            default_compose_deployment_id: Some("DefaultComposeDeployment".to_string()),
            deployments: HashMap::from([("MockDeployment".to_string(), deployment)]),
        };
        assert!(pouch.default_docker_deployment().is_none());
        assert!(pouch.default_compose_deployment().is_none());
    }

    #[test]
    fn get_default_deployment_some() {
        let deployments = HashMap::from_iter(
            [
                "MockDeployment1",
                "MockDeployment2",
                "MockDeployment3",
                "DefaultDeployment",
            ]
            .map(|name| {
                let mut deployment = MockedDockerDeployment::new();
                deployment.expect_id().return_const(name.to_string());
                let deployment = Deployment::Docker(Arc::new(deployment));
                (deployment.id().clone(), deployment)
            }),
        );
        let pouch = DeploymentPouch {
            path: testdir!(),
            default_docker_deployment_id: Some("DefaultDeployment".to_string()),
            default_compose_deployment_id: None,
            deployments,
        };
        assert_eq!(
            pouch.default_docker_deployment().unwrap().id(),
            "DefaultDeployment"
        );
    }

    #[test]
    fn set_default_deployment_no_deployment() {
        let mut pouch = DeploymentPouch {
            path: testdir!(),
            default_docker_deployment_id: Some("DefaultDeployment".to_string()),
            default_compose_deployment_id: None,
            deployments: HashMap::new(),
        };
        pouch.set_default_deployments();
        assert!(pouch.default_docker_deployment_id.is_none());
    }

    #[test]
    fn set_default_deployment_no_default_deployment() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_id()
            .return_const("MockedDeployment".to_string());
        deployment.expect_is_default().return_const(false);
        let deployment = Deployment::Docker(Arc::new(deployment));
        let mut pouch = DeploymentPouch {
            path: testdir!(),
            default_docker_deployment_id: None,
            default_compose_deployment_id: None,
            deployments: HashMap::from([("MockedDeployment".to_string(), deployment)]),
        };
        pouch.set_default_deployments();
        assert_eq!(
            pouch.default_docker_deployment_id,
            Some("MockedDeployment".to_string())
        );
    }

    #[test]
    fn set_default_deployment_some_default_deployment() {
        const DEFAULT_DEPLOYMENT_ID: &str = "DefaultDeployment";
        let deployments = HashMap::from_iter(
            [
                "MockDeployment1",
                "MockDeployment2",
                "MockDeployment3",
                DEFAULT_DEPLOYMENT_ID,
            ]
            .map(|name| {
                let mut deployment = MockedDockerDeployment::new();
                deployment.expect_id().return_const(name.to_string());
                deployment
                    .expect_is_default()
                    .return_const(name == DEFAULT_DEPLOYMENT_ID);
                let deployment = Deployment::Docker(Arc::new(deployment));
                (deployment.id().to_string(), deployment)
            }),
        );
        let mut pouch = DeploymentPouch {
            path: testdir!(),
            default_docker_deployment_id: None,
            default_compose_deployment_id: None,
            deployments,
        };
        pouch.set_default_deployments();
        assert_eq!(
            pouch.default_docker_deployment_id,
            Some(DEFAULT_DEPLOYMENT_ID.to_string())
        );
    }
}
