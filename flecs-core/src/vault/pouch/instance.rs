use crate::jeweler;
use crate::jeweler::deployment::Deployment;
use crate::jeweler::gem::instance::{try_create_instance, Instance, InstanceDeserializable};
use crate::jeweler::gem::manifest::AppManifest;
use crate::relic::network::Ipv4NetworkAccess;
use crate::vault::pouch::deployment::DeploymentId;
use crate::vault::pouch::{AppKey, Pouch};
pub use crate::Result;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::error;

const INSTANCES_FILE_NAME: &str = "instances.json";
pub type InstanceId = jeweler::gem::instance::InstanceId;

pub struct InstancePouch {
    path: PathBuf,
    instances: HashMap<InstanceId, Instance>,
    reserved_ip_addresses: HashSet<IpAddr>,
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

    pub fn instance_ids_by_app_key(&self, app_key: AppKey) -> Vec<InstanceId> {
        self.instances
            .iter()
            .filter_map(|(id, instance)| {
                if instance.app_key() == app_key {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn instance_ids_by_app_name(&self, app_name: String) -> Vec<InstanceId> {
        self.instances
            .iter()
            .filter_map(|(id, instance)| {
                if instance.app_key().name == app_name {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn instance_ids_by_app_version(&self, app_version: String) -> Vec<InstanceId> {
        self.instances
            .iter()
            .filter_map(|(id, instance)| {
                if instance.app_key().version == app_version {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn unavailable_ipv4_addresses(&self) -> HashSet<Ipv4Addr> {
        let instance_ips = self.instances.values().flat_map(|instance| {
            instance
                .config
                .network_addresses
                .values()
                .filter_map(|ip_addr| match ip_addr {
                    IpAddr::V4(address) => Some(address),
                    _ => None,
                })
        });
        self.reserved_ip_addresses
            .iter()
            .filter_map(|address| match address {
                IpAddr::V4(address) => Some(address),
                _ => None,
            })
            .chain(instance_ips)
            .cloned()
            .collect()
    }

    pub fn reserve_free_ipv4_address(&mut self, network: Ipv4NetworkAccess) -> Option<Ipv4Addr> {
        match network.next_free_ipv4_address(self.unavailable_ipv4_addresses()) {
            None => None,
            Some(address) => {
                self.reserved_ip_addresses.insert(IpAddr::V4(address));
                Some(address)
            }
        }
    }

    pub fn clear_ip_address_reservation(&mut self, address: IpAddr) {
        self.reserved_ip_addresses.remove(&address);
    }
}

impl InstancePouch {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            instances: HashMap::default(),
            reserved_ip_addresses: HashSet::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::jeweler::gem::instance::{InstanceConfig, InstanceStatus};
    use crate::relic::network::{Ipv4Iterator, Ipv4Network};
    use crate::tests::prepare_test_path;
    use crate::vault::pouch::manifest::tests::create_test_manifest;
    use serde_json::Value;
    use std::net::Ipv6Addr;

    pub fn create_manifest_for_instance(
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

    pub fn create_test_instances_deserializable() -> Vec<InstanceDeserializable> {
        vec![
            InstanceDeserializable {
                config: InstanceConfig::default(),
                name: "test-instance-1".to_string(),
                hostname: format!("flecs-{}", InstanceId::new(1)),
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
                hostname: format!("flecs-{}", InstanceId::new(2)),
                id: InstanceId::new(2),
                desired: InstanceStatus::Running,
                app_key: AppKey {
                    name: "some.test.app-2".to_string(),
                    version: "1.2.4".to_string(),
                },
                deployment_id: "test-deployment-2".to_string(),
            },
            InstanceDeserializable {
                config: InstanceConfig::default(),
                name: "test-instance-3".to_string(),
                hostname: format!("flecs-{}", InstanceId::new(3)),
                id: InstanceId::new(3),
                desired: InstanceStatus::Running,
                app_key: AppKey {
                    name: "some.test.app-3".to_string(),
                    version: "1.2.4".to_string(),
                },
                deployment_id: "test-deployment-3".to_string(),
            },
            InstanceDeserializable {
                config: InstanceConfig::default(),
                name: "test-instance-4".to_string(),
                hostname: format!("flecs-{}", InstanceId::new(4)),
                id: InstanceId::new(4),
                desired: InstanceStatus::Running,
                app_key: AppKey {
                    name: "some.test.app-4".to_string(),
                    version: "1.2.4".to_string(),
                },
                deployment_id: "test-deployment-4".to_string(),
            },
            InstanceDeserializable {
                config: InstanceConfig::default(),
                name: "test-instance-5".to_string(),
                hostname: format!("flecs-{}", InstanceId::new(5)),
                id: InstanceId::new(5),
                desired: InstanceStatus::Running,
                app_key: AppKey {
                    name: "some.test.app-4".to_string(),
                    version: "1.2.4".to_string(),
                },
                deployment_id: "test-deployment-4".to_string(),
            },
            InstanceDeserializable {
                config: InstanceConfig::default(),
                name: "test-instance-6".to_string(),
                hostname: format!("flecs-{}", InstanceId::new(6)),
                id: InstanceId::new(6),
                desired: InstanceStatus::Running,
                app_key: AppKey {
                    name: "some.test.app-4".to_string(),
                    version: "1.2.6".to_string(),
                },
                deployment_id: "test-deployment-4".to_string(),
            },
        ]
    }

    pub type TestData = (
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
                "hostname": "flecs-00000001",
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
                "hostname": "flecs-00000002",
                "id": 2,
                "desired": "Running",
                "app_key": {
                    "name": "some.test.app-2",
                    "version": "1.2.4"
                },
                "deployment_id": "test-deployment-2",
                "config": {}
            },
            {
                "name": "test-instance-3",
                "hostname": "flecs-00000003",
                "id": 3,
                "desired": "Running",
                "app_key": {
                    "name": "some.test.app-3",
                    "version": "1.2.4"
                },
                "deployment_id": "test-deployment-3",
                "config": {}
            },
            {
                "name": "test-instance-4",
                "hostname": "flecs-00000004",
                "id": 4,
                "desired": "Running",
                "app_key": {
                    "name": "some.test.app-4",
                    "version": "1.2.4"
                },
                "deployment_id": "test-deployment-4",
                "config": {}
            },
            {
                "name": "test-instance-5",
                "hostname": "flecs-00000005",
                "id": 5,
                "desired": "Running",
                "app_key": {
                    "name": "some.test.app-4",
                    "version": "1.2.4"
                },
                "deployment_id": "test-deployment-4",
                "config": {}
            },
            {
                "name": "test-instance-6",
                "hostname": "flecs-00000006",
                "id": 6,
                "desired": "Running",
                "app_key": {
                    "name": "some.test.app-4",
                    "version": "1.2.6"
                },
                "deployment_id": "test-deployment-4",
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
            6
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
            reserved_ip_addresses: HashSet::default(),
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
            reserved_ip_addresses: HashSet::default(),
        };
        fs::write(
            path.join(INSTANCES_FILE_NAME),
            serde_json::to_string_pretty(&create_test_json()).unwrap(),
        )
        .unwrap();
        pouch.open(&manifests, &deployments).unwrap();
        assert!(pouch.reserved_ip_addresses.is_empty());
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
            reserved_ip_addresses: HashSet::default(),
        };
        for gem in gems {
            assert!(pouch.gems().contains_key(&gem.0));
            assert!(pouch.gems_mut().contains_key(&gem.0));
        }
    }

    #[test]
    fn get_instance_ids_by_app_key() {
        let path = prepare_test_path(module_path!(), "get_instance_ids_by_app_key");
        let (instances, manifests, deployments) = create_test_data();
        let pouch = InstancePouch {
            path: path.clone(),
            instances: InstancePouch::create_instances(instances, &manifests, &deployments),
            reserved_ip_addresses: HashSet::default(),
        };
        assert_eq!(pouch.instances.len(), 6);
        let instance_ids_by_app_key = pouch.instance_ids_by_app_key(AppKey {
            name: "some.test.app-4".to_string(),
            version: "1.2.4".to_string(),
        });
        assert_eq!(instance_ids_by_app_key.len(), 2);
        assert!(instance_ids_by_app_key.contains(&InstanceId::new(4)));
        assert!(instance_ids_by_app_key.contains(&InstanceId::new(5)));
    }

    #[test]
    fn get_instance_ids_by_app_name() {
        let path = prepare_test_path(module_path!(), "get_instance_ids_by_app_name");
        let (instances, manifests, deployments) = create_test_data();
        let pouch = InstancePouch {
            path: path.clone(),
            instances: InstancePouch::create_instances(instances, &manifests, &deployments),
            reserved_ip_addresses: HashSet::default(),
        };
        assert_eq!(pouch.instances.len(), 6);
        let instance_ids_by_app_name =
            pouch.instance_ids_by_app_name("some.test.app-4".to_string());
        assert_eq!(instance_ids_by_app_name.len(), 3);
        assert!(instance_ids_by_app_name.contains(&InstanceId::new(4)));
        assert!(instance_ids_by_app_name.contains(&InstanceId::new(5)));
        assert!(instance_ids_by_app_name.contains(&InstanceId::new(6)));
    }

    #[test]
    fn get_instance_ids_by_app_version() {
        let path = prepare_test_path(module_path!(), "get_instance_ids_by_app_version");
        let (instances, manifests, deployments) = create_test_data();
        let pouch = InstancePouch {
            path: path.clone(),
            instances: InstancePouch::create_instances(instances, &manifests, &deployments),
            reserved_ip_addresses: HashSet::default(),
        };
        assert_eq!(pouch.instances.len(), 6);
        let instance_ids_by_app_version = pouch.instance_ids_by_app_version("1.2.4".to_string());
        assert_eq!(instance_ids_by_app_version.len(), 4);
        assert!(instance_ids_by_app_version.contains(&InstanceId::new(2)));
        assert!(instance_ids_by_app_version.contains(&InstanceId::new(3)));
        assert!(instance_ids_by_app_version.contains(&InstanceId::new(4)));
        assert!(instance_ids_by_app_version.contains(&InstanceId::new(5)));
    }

    #[test]
    fn unavailable_ipv4_addresses_empty() {
        let path = prepare_test_path(module_path!(), "unavailable_ipv4_addresses_empty");
        let pouch = InstancePouch {
            path: path.clone(),
            instances: HashMap::default(),
            reserved_ip_addresses: HashSet::default(),
        };
        assert!(pouch.unavailable_ipv4_addresses().is_empty());
    }

    #[test]
    fn unavailable_ipv4_addresses_some_reserved() {
        let path = prepare_test_path(module_path!(), "unavailable_ipv4_addresses_some_reserved");
        let ipv4_addresses = [
            Ipv4Addr::new(5, 10, 20, 40),
            Ipv4Addr::new(1, 2, 3, 4),
            Ipv4Addr::new(56, 84, 71, 93),
        ];
        let pouch = InstancePouch {
            path: path.clone(),
            instances: HashMap::default(),
            reserved_ip_addresses: ipv4_addresses
                .iter()
                .map(|ipv4_address| (*ipv4_address).into())
                .collect(),
        };
        assert_eq!(
            pouch.unavailable_ipv4_addresses(),
            HashSet::from(ipv4_addresses)
        );
    }

    #[test]
    fn unavailable_ipv4_addresses_some_instances() {
        let path = prepare_test_path(module_path!(), "unavailable_ipv4_addresses_some_instances");
        let (instances, manifests, deployments) = create_test_data();
        let mut pouch = InstancePouch {
            path: path.clone(),
            instances: InstancePouch::create_instances(instances.clone(), &manifests, &deployments),
            reserved_ip_addresses: HashSet::default(),
        };
        for instance in pouch.instances.values_mut() {
            instance.config.network_addresses.insert(
                format!("TestNetwork-{}", instance.id),
                IpAddr::V4(Ipv4Addr::new(1, 2, 3, instance.id.value as u8)),
            );
        }
        pouch
            .instances
            .get_mut(&InstanceId::new(1))
            .unwrap()
            .config
            .network_addresses
            .insert(
                "DoubleTestNetwork".to_string(),
                IpAddr::V4(Ipv4Addr::new(10, 20, 30, 40)),
            );
        let expected_ipv4_addresses = HashSet::from([
            Ipv4Addr::new(1, 2, 3, 1),
            Ipv4Addr::new(1, 2, 3, 2),
            Ipv4Addr::new(1, 2, 3, 3),
            Ipv4Addr::new(1, 2, 3, 4),
            Ipv4Addr::new(1, 2, 3, 5),
            Ipv4Addr::new(1, 2, 3, 6),
            Ipv4Addr::new(10, 20, 30, 40),
        ]);
        assert_eq!(pouch.unavailable_ipv4_addresses(), expected_ipv4_addresses);
    }

    #[test]
    fn unavailable_ipv4_addresses_some() {
        let path = prepare_test_path(module_path!(), "unavailable_ipv4_addresses_some");
        let (instances, manifests, deployments) = create_test_data();
        let mut pouch = InstancePouch {
            path: path.clone(),
            instances: InstancePouch::create_instances(instances.clone(), &manifests, &deployments),
            reserved_ip_addresses: HashSet::from([
                IpAddr::V4(Ipv4Addr::new(5, 10, 20, 40)),
                IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)),
                IpAddr::V4(Ipv4Addr::new(56, 84, 71, 93)),
            ]),
        };
        let expected_ipv4_addresses = HashSet::from([
            Ipv4Addr::new(5, 10, 20, 40),
            Ipv4Addr::new(1, 2, 3, 4),
            Ipv4Addr::new(56, 84, 71, 93),
            Ipv4Addr::new(1, 2, 3, 1),
            Ipv4Addr::new(1, 2, 3, 2),
            Ipv4Addr::new(1, 2, 3, 3),
            Ipv4Addr::new(1, 2, 3, 4),
            Ipv4Addr::new(1, 2, 3, 5),
            Ipv4Addr::new(1, 2, 3, 6),
            Ipv4Addr::new(10, 20, 30, 40),
        ]);
        for instance in pouch.instances.values_mut() {
            instance.config.network_addresses.insert(
                format!("TestNetwork-{}", instance.id),
                IpAddr::V4(Ipv4Addr::new(1, 2, 3, instance.id.value as u8)),
            );
        }
        pouch
            .instances
            .get_mut(&InstanceId::new(1))
            .unwrap()
            .config
            .network_addresses
            .insert(
                "DoubleTestNetwork".to_string(),
                IpAddr::V4(Ipv4Addr::new(10, 20, 30, 40)),
            );
        assert_eq!(pouch.unavailable_ipv4_addresses(), expected_ipv4_addresses);
    }

    #[test]
    fn unavailable_ipv4_addresses_ipv6_skipped() {
        let path = prepare_test_path(module_path!(), "unavailable_ipv4_addresses_ipv6_skipped");
        let (instances, manifests, deployments) = create_test_data();
        let mut pouch = InstancePouch {
            path: path.clone(),
            instances: InstancePouch::create_instances(instances.clone(), &manifests, &deployments),
            reserved_ip_addresses: HashSet::from([
                IpAddr::V6(Ipv6Addr::new(
                    0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x81,
                )),
                IpAddr::V6(Ipv6Addr::new(
                    0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x82,
                )),
                IpAddr::V6(Ipv6Addr::new(
                    0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x83,
                )),
            ]),
        };
        for instance in pouch.instances.values_mut() {
            instance.config.network_addresses.insert(
                format!("TestNetwork-{}", instance.id),
                IpAddr::V6(Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, instance.id.value as u16)),
            );
        }
        pouch
            .instances
            .get_mut(&InstanceId::new(1))
            .unwrap()
            .config
            .network_addresses
            .insert(
                "DoubleTestNetwork".to_string(),
                IpAddr::V6(Ipv6Addr::new(
                    0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x84,
                )),
            );
        assert!(pouch.unavailable_ipv4_addresses().is_empty());
    }

    #[test]
    fn clear_ip_address_reservation_test() {
        let path = prepare_test_path(module_path!(), "clear_ip_address_reservation_test");
        let ip1 = IpAddr::V6(Ipv6Addr::new(
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x81,
        ));
        let ip2 = IpAddr::V6(Ipv6Addr::new(
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x82,
        ));
        let ip3 = IpAddr::V6(Ipv6Addr::new(
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x83,
        ));
        let mut pouch = InstancePouch {
            path: path.clone(),
            instances: HashMap::default(),
            reserved_ip_addresses: HashSet::from([ip1, ip2, ip3]),
        };
        pouch.clear_ip_address_reservation(ip1);
        assert_eq!(pouch.reserved_ip_addresses, HashSet::from([ip2, ip3]));
        pouch.clear_ip_address_reservation(ip3);
        assert_eq!(pouch.reserved_ip_addresses, HashSet::from([ip2]));
        pouch.clear_ip_address_reservation(ip2);
        assert!(pouch.reserved_ip_addresses.is_empty());
    }

    #[test]
    fn reserve_free_ipv4_address_some() {
        let path = prepare_test_path(module_path!(), "reserve_free_ipv4_address_some");
        let network = Ipv4NetworkAccess::try_new(
            Ipv4Network::try_new(Ipv4Addr::new(20, 30, 40, 0), 24).unwrap(),
            Ipv4Addr::new(20, 30, 40, 2),
        )
        .unwrap();
        let (instances, manifests, deployments) = create_test_data();
        let mut pouch = InstancePouch {
            path: path.clone(),
            instances: InstancePouch::create_instances(instances.clone(), &manifests, &deployments),
            reserved_ip_addresses: HashSet::from([
                IpAddr::V4(Ipv4Addr::new(20, 30, 40, 3)),
                IpAddr::V4(Ipv4Addr::new(20, 30, 40, 4)),
                IpAddr::V4(Ipv4Addr::new(20, 30, 40, 5)),
            ]),
        };
        for (i, instance) in pouch.instances.values_mut().enumerate() {
            instance.config.network_addresses.insert(
                format!("TestNetwork-{}", instance.id),
                IpAddr::V4(Ipv4Addr::new(20, 30, 40, (6 + i) as u8)),
            );
        }
        assert_eq!(
            pouch.reserve_free_ipv4_address(network),
            Some(Ipv4Addr::new(20, 30, 40, 6 + pouch.instances.len() as u8))
        )
    }

    #[test]
    fn reserve_free_ipv4_address_none() {
        let path = prepare_test_path(module_path!(), "reserve_free_ipv4_address_none");
        let network = Ipv4NetworkAccess::try_new(
            Ipv4Network::try_new(Ipv4Addr::new(20, 30, 40, 0), 24).unwrap(),
            Ipv4Addr::new(20, 30, 40, 2),
        )
        .unwrap();
        let mut pouch = InstancePouch {
            path: path.clone(),
            instances: HashMap::default(),
            reserved_ip_addresses: Ipv4Iterator::from(
                Ipv4Addr::new(20, 30, 40, 3).into()..Ipv4Addr::new(20, 30, 40, 255).into(),
            )
            .map(Into::into)
            .collect(),
        };
        assert_eq!(pouch.reserve_free_ipv4_address(network), None)
    }
}
