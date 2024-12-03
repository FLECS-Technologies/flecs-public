use crate::jeweler;
use crate::jeweler::deployment::Deployment;
use crate::jeweler::gem::instance::{try_create_instance, Instance, InstanceDeserializable};
use crate::jeweler::gem::manifest::AppManifest;
use crate::vault::pouch::{AppKey, DeploymentId, Pouch};
pub use crate::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::error;

const INSTANCES_FILE_NAME: &str = "instances.json";
pub type InstanceStatus = jeweler::gem::instance::InstanceStatus;
pub type InstanceId = jeweler::gem::instance::InstanceId;

pub struct InstancePouch {
    path: PathBuf,
    instances: HashMap<InstanceId, Instance>,
}

impl Pouch for InstancePouch {
    type Gems = HashMap<InstanceId, Instance>;

    fn gems(&self) -> &Self::Gems {
        &self.instances
    }

    fn gems_mut(&mut self) -> &mut Self::Gems {
        &mut self.instances
    }
}

impl InstancePouch {
    pub(in super::super) fn close(&mut self) -> Result<()> {
        let file = fs::File::create(self.path.join(INSTANCES_FILE_NAME))?;
        let content: Vec<_> = self.instances.values().collect();
        serde_json::to_writer_pretty(file, &content)?;
        Ok(())
    }

    pub(in super::super) fn open(
        &mut self,
        manifests: &HashMap<AppKey, Arc<AppManifest>>,
        deployments: &HashMap<DeploymentId, Arc<dyn Deployment>>,
    ) -> Result<()> {
        self.instances = Self::create_instances(self.read_instances()?, manifests, deployments);
        Ok(())
    }

    fn read_instances(&self) -> anyhow::Result<Vec<InstanceDeserializable>> {
        let file = fs::File::open(self.path.join(INSTANCES_FILE_NAME))?;
        Ok(serde_json::from_reader(file)?)
    }

    fn create_instances(
        instances: Vec<InstanceDeserializable>,
        manifests: &HashMap<AppKey, Arc<AppManifest>>,
        deployments: &HashMap<DeploymentId, Arc<dyn Deployment>>,
    ) -> HashMap<InstanceId, Instance> {
        instances
            .into_iter()
            .filter_map(|instance| {
                let id = instance.id;
                let app_key = instance.app_key.clone();
                match try_create_instance(instance, manifests, deployments) {
                    Ok(instance) => Some((id, instance)),
                    Err(e) => {
                        error!("Could not create instance {id} of {app_key}: {e}");
                        None
                    }
                }
            })
            .collect()
    }
}

impl InstancePouch {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            instances: HashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::jeweler::gem::instance::InstanceConfig;
    use crate::tests::prepare_test_path;
    use crate::vault::pouch::tests::create_test_manifest;
    use serde_json::Value;

    fn create_manifest_for_instance(
        instance: &InstanceDeserializable,
    ) -> (AppKey, Arc<AppManifest>) {
        (
            instance.app_key.clone(),
            Arc::new(create_test_manifest(
                instance.app_key.name.as_str(),
                instance.app_key.version.as_str(),
            )),
        )
    }

    fn create_deployment_for_instance(
        instance: &InstanceDeserializable,
    ) -> (DeploymentId, Arc<dyn Deployment>) {
        let mut deployment = MockedDeployment::new();
        let deployment_id = instance.deployment_id.clone();
        deployment
            .expect_id()
            .returning(move || deployment_id.clone());
        (
            instance.deployment_id.clone(),
            Arc::new(deployment) as Arc<dyn Deployment>,
        )
    }

    fn create_test_instances_deserializable() -> Vec<InstanceDeserializable> {
        vec![
            InstanceDeserializable {
                config: InstanceConfig::default(),
                name: "test-instance-1".to_string(),
                id: InstanceId::new(1),
                desired: InstanceStatus::Running,
                app_key: AppKey {
                    name: "some.test.app-1".to_string(),
                    version: "1.2.3".to_string(),
                },
                deployment_id: "test-deployment-1".to_string(),
            },
            InstanceDeserializable {
                config: InstanceConfig::default(),
                name: "test-instance-2".to_string(),
                id: InstanceId::new(2),
                desired: InstanceStatus::Running,
                app_key: AppKey {
                    name: "some.test.app-2".to_string(),
                    version: "1.2.4".to_string(),
                },
                deployment_id: "test-deployment-2".to_string(),
            },
        ]
    }

    type TestData = (
        Vec<InstanceDeserializable>,
        HashMap<AppKey, Arc<AppManifest>>,
        HashMap<DeploymentId, Arc<dyn Deployment>>,
    );

    fn create_test_data() -> TestData {
        let instances = create_test_instances_deserializable();
        let manifests = instances
            .iter()
            .map(create_manifest_for_instance)
            .collect::<HashMap<AppKey, Arc<AppManifest>>>();
        let deployments = instances
            .iter()
            .map(create_deployment_for_instance)
            .collect::<HashMap<DeploymentId, Arc<dyn Deployment>>>();
        (instances, manifests, deployments)
    }

    fn create_test_json() -> Value {
        serde_json::json!([
            {
                "name": "test-instance-1",
                "id": 1,
                "desired": "Running",
                "app_key": {
                    "name": "some.test.app-1",
                    "version": "1.2.3"
                },
                "deployment_id": "test-deployment-1",
                "config": {}
            },
            {
                "name": "test-instance-2",
                "id": 2,
                "desired": "Running",
                "app_key": {
                    "name": "some.test.app-2",
                    "version": "1.2.4"
                },
                "deployment_id": "test-deployment-2",
                "config": {}
            }
        ])
    }

    #[test]
    fn read_instances_ok() {
        let path = prepare_test_path(module_path!(), "read_instances_ok").join(INSTANCES_FILE_NAME);
        let json = create_test_json();
        let instance_pouch = InstancePouch::new(path.parent().unwrap());
        fs::write(path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
        let instances = instance_pouch.read_instances().unwrap();
        assert_eq!(instances, create_test_instances_deserializable());
    }

    #[test]
    fn read_instances_invalid_file() {
        let path = prepare_test_path(module_path!(), "read_instances_invalid_file")
            .join(INSTANCES_FILE_NAME);
        let instance_pouch = InstancePouch::new(path.parent().unwrap());
        fs::write(path, "random_data").unwrap();
        assert!(instance_pouch.read_instances().is_err());
    }

    #[test]
    fn read_instances_file_missing() {
        let path = prepare_test_path(module_path!(), "read_instances_file_missing")
            .join(INSTANCES_FILE_NAME);
        let instance_pouch = InstancePouch::new(path.parent().unwrap());
        assert!(instance_pouch.read_instances().is_err());
    }

    #[test]
    fn create_instances_ok() {
        let (instances, manifests, deployments) = create_test_data();
        assert_eq!(
            InstancePouch::create_instances(instances, &manifests, &deployments).len(),
            2
        );
    }

    #[test]
    fn create_instances_error() {
        let instances = create_test_instances_deserializable();
        let manifests = instances
            .iter()
            .take(1)
            .map(create_manifest_for_instance)
            .collect::<HashMap<AppKey, Arc<AppManifest>>>();
        let deployments = instances
            .iter()
            .take(1)
            .map(create_deployment_for_instance)
            .collect::<HashMap<DeploymentId, Arc<dyn Deployment>>>();
        assert_eq!(
            InstancePouch::create_instances(instances, &manifests, &deployments).len(),
            1
        );
    }

    #[test]
    fn close_pouch() {
        let path = prepare_test_path(module_path!(), "close_pouch");
        let (instances, manifests, deployments) = create_test_data();
        let mut pouch = InstancePouch {
            path: path.clone(),
            instances: InstancePouch::create_instances(instances, &manifests, &deployments),
        };
        pouch.close().unwrap();
        let data = fs::read_to_string(path.join(INSTANCES_FILE_NAME)).unwrap();
        let test_json = create_test_json();
        let test_json = test_json.as_array().unwrap();
        let result_json = serde_json::from_str::<Value>(data.as_str()).unwrap();
        let result_json = result_json.as_array().unwrap();
        for json in test_json {
            result_json
                .iter()
                .find(|result| *result == json)
                .unwrap_or_else(|| panic!("Expected to find {json:#?}"));
        }
    }

    #[test]
    fn open_pouch() {
        let path = prepare_test_path(module_path!(), "open_pouch");
        let (instances, manifests, deployments) = create_test_data();
        let mut pouch = InstancePouch {
            path: path.clone(),
            instances: HashMap::new(),
        };
        fs::write(
            path.join(INSTANCES_FILE_NAME),
            serde_json::to_string_pretty(&create_test_json()).unwrap(),
        )
        .unwrap();
        pouch.open(&manifests, &deployments).unwrap();
        assert_eq!(pouch.instances.len(), instances.len());
        for instance in instances {
            assert!(pouch.instances.contains_key(&instance.id));
        }
    }

    #[test]
    fn gems() {
        let path = prepare_test_path(module_path!(), "gems");
        let (instances, manifests, deployments) = create_test_data();
        let gems = InstancePouch::create_instances(instances.clone(), &manifests, &deployments);
        let mut pouch = InstancePouch {
            path: path.clone(),
            instances: InstancePouch::create_instances(instances, &manifests, &deployments),
        };
        for gem in gems {
            assert!(pouch.gems().contains_key(&gem.0));
            assert!(pouch.gems_mut().contains_key(&gem.0));
        }
    }
}
