pub use super::Result;
use crate::relic::serde::SerdeIteratorAdapter;
use crate::vault::pouch::{Pouch, VaultPouch};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(tag = "type")]
pub enum Deployment {
    Docker(DockerDeployment),
}

impl Default for Deployment {
    fn default() -> Self {
        Self::Docker(DockerDeployment {
            id: "DefaultDockerDeployment".to_string(),
            path: PathBuf::from("/var/run/docker.sock"),
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct DockerDeployment {
    id: DeploymentId,
    path: PathBuf,
}

pub type DeploymentId = String;

#[derive(Debug, PartialEq)]
pub struct DeploymentPouch {
    deployments: HashMap<DeploymentId, Arc<Deployment>>,
    path: PathBuf,
}

impl Pouch for DeploymentPouch {
    type Gems = HashMap<DeploymentId, Arc<Deployment>>;

    fn gems(&self) -> &Self::Gems {
        &self.deployments
    }

    fn gems_mut(&mut self) -> &mut Self::Gems {
        &mut self.deployments
    }
}

impl VaultPouch for DeploymentPouch {
    fn close(&mut self) -> Result<()> {
        fs::create_dir_all(&self.path)?;
        let file = fs::File::create(self.deployments_path())?;
        serde_json::to_writer_pretty(file, &SerdeIteratorAdapter::new(self.deployments.values()))?;
        Ok(())
    }

    fn open(&mut self) -> Result<()> {
        let deployments = Self::read_deployments(&self.deployments_path())?;
        self.deployments = deployments
            .into_iter()
            .map(|d| (d.id(), Arc::new(d)))
            .collect();
        Ok(())
    }
}

impl Deployment {
    pub fn id(&self) -> DeploymentId {
        match self {
            Self::Docker(deployment) => deployment.id.clone(),
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
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::Path;

    fn test_path() -> &'static Path {
        Path::new("/tmp/flecs-tests/deployment_pouch/")
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

    fn prepare_path(path: &Path) {
        println!("Preparing {:?}", path);
        let _ = fs::remove_dir_all(path);
        assert!(!path.try_exists().unwrap());
        fs::create_dir_all(path).unwrap();
        assert!(path.try_exists().unwrap());
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
        let gems = HashMap::from([(
            TEST_DEPLOYMENT_ID.to_string(),
            Arc::new(Deployment::Docker(DockerDeployment {
                id: TEST_DEPLOYMENT_ID.to_string(),
                path: PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
            })),
        )]);
        let mut deployment_pouch = DeploymentPouch {
            deployments: gems.clone(),
            path: test_path().to_path_buf(),
        };
        assert_eq!(&gems, deployment_pouch.gems());
        assert_eq!(&gems, deployment_pouch.gems_mut());
    }

    #[test]
    fn deployment_id() {
        let deployment = Deployment::Docker(DockerDeployment {
            id: TEST_DEPLOYMENT_ID.to_string(),
            path: PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
        });
        assert_eq!(deployment.id(), TEST_DEPLOYMENT_ID);
    }

    #[test]
    fn new_deployment_pouch() {
        let path = test_path();
        let deployment_pouch = DeploymentPouch::new(path);
        assert!(deployment_pouch.deployments.is_empty());
        assert_eq!(deployment_pouch.path, path);
    }

    #[test]
    fn default_deployment() {
        let deployment = Deployment::default();
        #[allow(unreachable_patterns)]
        match deployment {
            Deployment::Docker(deployment) => {
                assert_eq!(deployment.id, "DefaultDockerDeployment");
                assert_eq!(deployment.path, PathBuf::from("/var/run/docker.sock"));
            }
            _ => panic!("Expected default deployment to be of type Docker"),
        }
    }

    #[tokio::test]
    async fn open_deployment_pouch() {
        let path = test_path().join("open");
        prepare_path(&path);
        let json = serde_json::to_string(&test_deployment_json()).unwrap();
        let mut deployment_pouch = DeploymentPouch::new(&path);
        fs::write(deployment_pouch.deployments_path(), json).unwrap();
        deployment_pouch.open().unwrap();
        assert_eq!(
            deployment_pouch.deployments,
            HashMap::from([(
                TEST_DEPLOYMENT_ID.to_string(),
                Arc::new(Deployment::Docker(DockerDeployment {
                    id: TEST_DEPLOYMENT_ID.to_string(),
                    path: PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
                }))
            )])
        );
    }

    #[tokio::test]
    async fn close_deployment_pouch() {
        let path = test_path().join("close/deployments.json");
        prepare_path(path.parent().unwrap());
        let json = test_deployment_json();
        let mut deployment_pouch = DeploymentPouch {
            deployments: HashMap::from([(
                TEST_DEPLOYMENT_ID.to_string(),
                Arc::new(Deployment::Docker(DockerDeployment {
                    id: TEST_DEPLOYMENT_ID.to_string(),
                    path: PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
                })),
            )]),
            path: path.parent().unwrap().to_path_buf(),
        };
        deployment_pouch.close().unwrap();
        let file_content: serde_json::Value =
            serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();
        assert_eq!(json, file_content);
    }

    #[tokio::test]
    async fn read_deployments() {
        let path = test_path().join("read/deployments.json");
        prepare_path(path.parent().unwrap());

        let json = serde_json::to_string(&test_deployments_json()).unwrap();
        fs::write(&path, json).unwrap();
        assert_eq!(
            DeploymentPouch::read_deployments(&path).unwrap(),
            vec![
                Deployment::Docker(DockerDeployment {
                    id: "test1".to_string(),
                    path: PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
                }),
                Deployment::Docker(DockerDeployment {
                    id: "test2".to_string(),
                    path: PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
                }),
                Deployment::Docker(DockerDeployment {
                    id: "test3".to_string(),
                    path: PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
                }),
                Deployment::Docker(DockerDeployment {
                    id: "test4".to_string(),
                    path: PathBuf::from(TEST_DEPLOYMENT_SOCK_PATH),
                })
            ]
        );
    }
}
