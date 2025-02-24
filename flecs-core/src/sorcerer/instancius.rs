pub use super::Result;
use crate::forge::vec::VecExtension;
use crate::jeweler::app::AppStatus;
use crate::jeweler::deployment::Deployment;
use crate::jeweler::gem;
use crate::jeweler::gem::instance::{InstanceId, TransportProtocol, UsbPathConfig};
use crate::jeweler::gem::manifest::{EnvironmentVariable, Label, PortMapping, PortRange};
use crate::jeweler::instance::Logs;
use crate::quest::SyncQuest;
use crate::relic::device::usb::{UsbDevice, UsbDeviceReader};
use crate::relic::network::Ipv4NetworkAccess;
use crate::sorcerer::spell;
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::{GrabbedPouches, Vault};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;

pub async fn start_instance(
    quest: SyncQuest,
    vault: Arc<Vault>,
    instance_id: InstanceId,
) -> Result<()> {
    spell::instance::start_instance(quest, vault, instance_id).await
}

pub async fn stop_instance(
    quest: SyncQuest,
    vault: Arc<Vault>,
    instance_id: InstanceId,
) -> Result<()> {
    spell::instance::stop_instance(quest, vault, instance_id).await
}

pub async fn get_instance(
    vault: Arc<Vault>,
    instance_id: InstanceId,
) -> Result<Option<flecsd_axum_server::models::AppInstance>> {
    spell::instance::get_instance_info(vault, instance_id).await
}

pub async fn get_instance_detailed(
    vault: Arc<Vault>,
    instance_id: InstanceId,
) -> Result<Option<flecsd_axum_server::models::InstancesInstanceIdGet200Response>> {
    spell::instance::get_instance_detailed_info(vault, instance_id).await
}

pub async fn get_instances_filtered(
    quest: SyncQuest,
    vault: Arc<Vault>,
    app_name: Option<String>,
    app_version: Option<String>,
) -> Vec<flecsd_axum_server::models::AppInstance> {
    let instance_ids = {
        let grab = vault.reservation().reserve_instance_pouch().grab().await;
        let instance_pouch = grab
            .instance_pouch
            .as_ref()
            .expect("Vault reservations should never fail");
        match (app_name, app_version) {
            (None, None) => instance_pouch.gems().keys().copied().collect(),
            (None, Some(version)) => instance_pouch.instance_ids_by_app_version(version),
            (Some(name), None) => instance_pouch.instance_ids_by_app_name(name),
            (Some(name), Some(version)) => {
                instance_pouch.instance_ids_by_app_key(AppKey { name, version })
            }
        }
    };
    spell::instance::get_instances_info(quest, vault, instance_ids).await
}

pub async fn get_all_instances(
    quest: SyncQuest,
    vault: Arc<Vault>,
) -> Vec<flecsd_axum_server::models::AppInstance> {
    let instance_ids: Vec<InstanceId> = vault
        .reservation()
        .reserve_instance_pouch()
        .grab()
        .await
        .instance_pouch
        .as_ref()
        .expect("Vault reservations should never fail")
        .gems()
        .keys()
        .copied()
        .collect();
    spell::instance::get_instances_info(quest, vault, instance_ids).await
}

pub async fn create_instance(
    quest: SyncQuest,
    vault: Arc<Vault>,
    app_key: AppKey,
    name: String,
) -> Result<InstanceId> {
    let (manifest, deployments) = quest
        .lock()
        .await
        .create_sub_quest(
            format!("Validate request for creation of instance '{name}' of {app_key}"),
            |_quest| {
                let vault = vault.clone();
                let app_key = app_key.clone();
                async move {
                    let GrabbedPouches {
                        deployment_pouch: Some(deployments),
                        manifest_pouch: Some(manifests),
                        instance_pouch: Some(instances),
                        app_pouch: Some(apps),
                        ..
                    } = &vault
                        .reservation()
                        .reserve_deployment_pouch()
                        .reserve_manifest_pouch()
                        .reserve_instance_pouch()
                        .reserve_app_pouch()
                        .grab()
                        .await
                    else {
                        unreachable!("Vault reservations should never fail")
                    };
                    let is_app_installed = match apps.gems().get(&app_key) {
                        None => false,
                        Some(app) => app.status().await? == AppStatus::Installed,
                    };
                    anyhow::ensure!(is_app_installed, "App {app_key} is not installed");

                    let manifest = manifests
                        .gems()
                        .get(&app_key)
                        .ok_or_else(|| anyhow::anyhow!("No manifest for {app_key} present"))?
                        .clone();
                    if !manifest.multi_instance()
                        && !instances
                            .instance_ids_by_app_key(app_key.clone())
                            .is_empty()
                    {
                        anyhow::bail!("Can not create multiple instances for {app_key}");
                    }
                    let deployments = deployments
                        .gems()
                        .values()
                        .cloned()
                        .collect::<Vec<Arc<dyn Deployment>>>();
                    Ok((manifest, deployments))
                }
            },
        )
        .await
        .2
        .await?;
    // TODO: In which deployment(s) should an instance be created? All?
    let deployment = deployments
        .first()
        .ok_or_else(|| anyhow::anyhow!("No deployment present to create instance in"))?
        .clone();
    let address = quest
        .lock()
        .await
        .create_sub_quest(
            format!(
                "Reserve ip address in default network of deployment {}",
                deployment.id()
            ),
            |quest| {
                let (vault, deployment) = (vault.clone(), deployment.clone());
                async move {
                    let network = Ipv4NetworkAccess::try_from(deployment.default_network().await?)?;
                    let address = spell::instance::make_ipv4_reservation(vault, network)
                        .await
                        .ok_or_else(|| anyhow::anyhow!("No free ip address in default network"))?;
                    quest.lock().await.detail = Some(format!("Reserved {}", address));
                    Ok(address)
                }
            },
        )
        .await
        .2
        .await?;
    let address = IpAddr::V4(address);
    let instance = quest
        .lock()
        .await
        .create_sub_quest(format!("Create instance '{name}' for {app_key}"), |quest| {
            spell::instance::create_instance(quest, deployment, manifest, name, address)
        })
        .await
        .2
        .await;
    if instance.is_err() {
        spell::instance::clear_ip_reservation(vault.clone(), address).await;
    }
    let instance = instance?;
    let instance_id = instance.id;
    quest
        .lock()
        .await
        .create_infallible_sub_quest(
            format!(
                "Saving new instance {} with id {}",
                instance.name, instance_id
            ),
            |_quest| async move {
                let mut grab = vault
                    .reservation()
                    .reserve_instance_pouch_mut()
                    .grab()
                    .await;
                let pouch = grab
                    .instance_pouch_mut
                    .as_mut()
                    .expect("Vault reservations should never fail");
                pouch.gems_mut().insert(instance_id, instance);
                pouch.clear_ip_address_reservation(address)
            },
        )
        .await
        .2
        .await;
    Ok(instance_id)
}

pub async fn does_instance_exist(vault: Arc<Vault>, id: InstanceId) -> bool {
    vault
        .reservation()
        .reserve_instance_pouch()
        .grab()
        .await
        .instance_pouch
        .as_ref()
        .expect("Reservations should never fail")
        .gems()
        .contains_key(&id)
}

pub async fn get_instance_config(
    vault: Arc<Vault>,
    id: InstanceId,
) -> Option<gem::instance::config::InstanceConfig> {
    spell::instance::get_instance_config_part_with(vault, id, |config| config.clone()).await
}

pub async fn get_instance_usb_devices(
    vault: Arc<Vault>,
    id: InstanceId,
    usb_reader: &impl UsbDeviceReader,
) -> Result<Option<Vec<(UsbPathConfig, Option<UsbDevice>)>>> {
    let Some(mapped_devices) =
        spell::instance::get_instance_config_part_with(vault, id, |config| {
            config.usb_devices.clone()
        })
        .await
    else {
        return Ok(None);
    };
    let existing_devices = usb_reader.read_usb_devices()?;
    Ok(Some(
        mapped_devices
            .into_iter()
            .map(|(port, device)| {
                let existing_device = existing_devices.get(&port).cloned();
                (device, existing_device)
            })
            .collect(),
    ))
}

pub async fn delete_instance_usb_devices(
    vault: Arc<Vault>,
    id: InstanceId,
) -> Option<HashMap<String, UsbPathConfig>> {
    spell::instance::modify_instance_config_with(vault, id, |config| {
        std::mem::take(&mut config.usb_devices)
    })
    .await
}

pub async fn delete_instance_usb_device(
    vault: Arc<Vault>,
    id: InstanceId,
    port: String,
) -> Option<Option<UsbPathConfig>> {
    spell::instance::modify_instance_config_with(vault, id, |config| {
        config.usb_devices.remove(&port)
    })
    .await
}

#[derive(Eq, PartialEq, Debug)]
pub enum GetInstanceUsbDeviceResult {
    UnknownDevice,
    DeviceNotMapped,
    InstanceNotFound,
    DeviceInactive(UsbPathConfig),
    DeviceActive(UsbPathConfig, UsbDevice),
}

pub async fn get_instance_usb_device(
    vault: Arc<Vault>,
    id: InstanceId,
    port: String,
    usb_reader: &impl UsbDeviceReader,
) -> Result<GetInstanceUsbDeviceResult> {
    let Some(mapped_device) = spell::instance::get_instance_config_part_with(vault, id, |config| {
        config.usb_devices.get(&port).cloned()
    })
    .await
    else {
        return Ok(GetInstanceUsbDeviceResult::InstanceNotFound);
    };
    match (mapped_device, usb_reader.read_usb_devices()?.remove(&port)) {
        (Some(config), Some(usb_device)) => {
            Ok(GetInstanceUsbDeviceResult::DeviceActive(config, usb_device))
        }
        (Some(config), None) => Ok(GetInstanceUsbDeviceResult::DeviceInactive(config)),
        (None, Some(_)) => Ok(GetInstanceUsbDeviceResult::DeviceNotMapped),
        (None, None) => Ok(GetInstanceUsbDeviceResult::UnknownDevice),
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum PutInstanceUsbDeviceResult {
    DeviceNotFound,
    InstanceNotFound,
    DeviceMappingCreated,
    DeviceMappingUpdated(UsbPathConfig),
}

pub async fn put_instance_usb_device(
    vault: Arc<Vault>,
    id: InstanceId,
    port: String,
    usb_reader: &impl UsbDeviceReader,
) -> Result<PutInstanceUsbDeviceResult> {
    let Some(usb_device) = usb_reader.read_usb_devices()?.remove(&port) else {
        return Ok(PutInstanceUsbDeviceResult::DeviceNotFound);
    };
    let result = spell::instance::modify_instance_config_with(vault, id, |config| {
        Ok(config
            .usb_devices
            .insert(port, UsbPathConfig::try_from((&usb_device, usb_reader))?))
    })
    .await;
    match result {
        None => Ok(PutInstanceUsbDeviceResult::InstanceNotFound),
        Some(Ok(Some(previous_mapping))) => Ok(PutInstanceUsbDeviceResult::DeviceMappingUpdated(
            previous_mapping,
        )),
        Some(Ok(None)) => Ok(PutInstanceUsbDeviceResult::DeviceMappingCreated),
        Some(Err(e)) => Err(e),
    }
}

pub async fn get_instance_config_port_mapping(
    vault: Arc<Vault>,
    id: InstanceId,
    host_port: u16,
    transport_protocol: TransportProtocol,
) -> Option<Option<PortMapping>> {
    get_instance_config_port_mapping_range(
        vault,
        id,
        PortRange::new(host_port..=host_port),
        transport_protocol,
    )
    .await
}

pub async fn get_instance_config_port_mappings(
    vault: Arc<Vault>,
    id: InstanceId,
) -> Option<gem::instance::config::InstancePortMapping> {
    spell::instance::get_instance_config_part_with(vault, id, |config| config.port_mapping.clone())
        .await
}

pub async fn get_instance_config_protocol_port_mappings(
    vault: Arc<Vault>,
    id: InstanceId,
    transport_protocol: TransportProtocol,
) -> Option<Vec<PortMapping>> {
    spell::instance::get_instance_config_part_with(vault, id, |config| match transport_protocol {
        TransportProtocol::Tcp => config.port_mapping.tcp.clone(),
        TransportProtocol::Udp => config.port_mapping.udp.clone(),
        TransportProtocol::Sctp => config.port_mapping.sctp.clone(),
    })
    .await
}

pub async fn delete_instance_config_protocol_port_mappings(
    vault: Arc<Vault>,
    id: InstanceId,
    transport_protocol: TransportProtocol,
) -> Option<Vec<PortMapping>> {
    spell::instance::modify_instance_config_with(vault, id, |config| match transport_protocol {
        TransportProtocol::Tcp => std::mem::take(&mut config.port_mapping.tcp),
        TransportProtocol::Udp => std::mem::take(&mut config.port_mapping.udp),
        TransportProtocol::Sctp => std::mem::take(&mut config.port_mapping.sctp),
    })
    .await
}

pub async fn delete_instance_config_port_mapping(
    vault: Arc<Vault>,
    id: InstanceId,
    host_port: u16,
    transport_protocol: TransportProtocol,
) -> Option<bool> {
    delete_instance_config_port_mapping_range(
        vault,
        id,
        PortRange::new(host_port..=host_port),
        transport_protocol,
    )
    .await
}

pub async fn delete_instance_config_port_mapping_range(
    vault: Arc<Vault>,
    id: InstanceId,
    host_port_range: PortRange,
    transport_protocol: TransportProtocol,
) -> Option<bool> {
    spell::instance::modify_instance_config_with(vault, id, |config| {
        config
            .port_mapping
            .delete_port_mapping_range(host_port_range, transport_protocol)
            .is_some()
    })
    .await
}

pub async fn get_instance_config_port_mapping_range(
    vault: Arc<Vault>,
    id: InstanceId,
    host_port_range: PortRange,
    transport_protocol: TransportProtocol,
) -> Option<Option<PortMapping>> {
    spell::instance::get_instance_config_part_with(vault, id, |config| {
        config
            .port_mapping
            .get_port_mapping_range(host_port_range, transport_protocol)
    })
    .await
}

pub async fn put_instance_config_port_mapping(
    vault: Arc<Vault>,
    id: InstanceId,
    port_mapping: PortMapping,
    transport_protocol: TransportProtocol,
) -> Result<Option<bool>> {
    match spell::instance::modify_instance_config_with(vault, id, |config| {
        config
            .port_mapping
            .update_port_mapping(port_mapping, transport_protocol)
    })
    .await
    {
        None => Ok(None),
        Some(result) => Ok(Some(result?)),
    }
}

pub async fn put_instance_config_protocol_port_mappings(
    vault: Arc<Vault>,
    id: InstanceId,
    port_mapping: Vec<PortMapping>,
    transport_protocol: TransportProtocol,
) -> bool {
    spell::instance::modify_instance_config_with(vault, id, |config| match transport_protocol {
        TransportProtocol::Tcp => config.port_mapping.tcp = port_mapping,
        TransportProtocol::Udp => config.port_mapping.udp = port_mapping,
        TransportProtocol::Sctp => config.port_mapping.sctp = port_mapping,
    })
    .await
    .is_some()
}

pub async fn delete_instance_config_port_mappings(vault: Arc<Vault>, id: InstanceId) -> bool {
    spell::instance::modify_instance_config_with(vault, id, |config| {
        config.port_mapping.clear();
    })
    .await
    .is_some()
}

pub async fn get_instance_config_environment_variable_value(
    vault: Arc<Vault>,
    id: InstanceId,
    variable_name: String,
) -> Option<Option<Option<String>>> {
    spell::instance::get_instance_config_part_with(vault, id, |config| {
        config
            .environment_variables
            .iter()
            .find(|env| env.name == variable_name)
            .map(|env| env.value.clone())
    })
    .await
}

pub async fn put_instance_config_environment_variable_value(
    vault: Arc<Vault>,
    id: InstanceId,
    mut environment_variable: EnvironmentVariable,
) -> Option<Option<String>> {
    spell::instance::modify_instance_config_with(vault, id, |config| {
        for existing_environment_variable in config.environment_variables.iter_mut() {
            if existing_environment_variable.name == environment_variable.name {
                std::mem::swap(
                    &mut existing_environment_variable.value,
                    &mut environment_variable.value,
                );
                return environment_variable.value;
            }
        }
        config.environment_variables.push(environment_variable);
        None
    })
    .await
}

pub async fn delete_instance_config_environment_variable_value(
    vault: Arc<Vault>,
    id: InstanceId,
    variable_name: String,
) -> Option<Option<EnvironmentVariable>> {
    spell::instance::modify_instance_config_with(vault, id, |config| {
        config
            .environment_variables
            .extract_first_element_with(|env| env.name == variable_name)
    })
    .await
}

pub async fn get_instance_config_environment(
    vault: Arc<Vault>,
    id: InstanceId,
) -> Option<Vec<EnvironmentVariable>> {
    spell::instance::get_instance_config_part_with(vault, id, |config| {
        config.environment_variables.clone()
    })
    .await
}

pub async fn put_instance_config_environment(
    vault: Arc<Vault>,
    id: InstanceId,
    mut environment: Vec<EnvironmentVariable>,
) -> Option<Vec<EnvironmentVariable>> {
    spell::instance::modify_instance_config_with(vault, id, |config| {
        std::mem::swap(&mut config.environment_variables, &mut environment);
        environment
    })
    .await
}

pub async fn delete_instance_config_environment(
    vault: Arc<Vault>,
    id: InstanceId,
) -> Option<Vec<EnvironmentVariable>> {
    spell::instance::modify_instance_config_with(vault, id, |config| {
        let mut environment = Vec::default();
        std::mem::swap(&mut config.environment_variables, &mut environment);
        environment
    })
    .await
}

pub async fn delete_instance(quest: SyncQuest, vault: Arc<Vault>, id: InstanceId) -> Result<()> {
    spell::instance::delete_instance(quest, vault, id).await
}

pub async fn get_instance_logs(vault: Arc<Vault>, id: InstanceId) -> Result<Logs> {
    match vault
        .reservation()
        .reserve_instance_pouch()
        .grab()
        .await
        .instance_pouch
        .as_ref()
        .expect("Reservations should never fail")
        .gems()
        .get(&id)
    {
        Some(instance) => instance.get_logs().await,
        None => anyhow::bail!("Instance {id} not found"),
    }
}

pub async fn get_instance_labels(vault: Arc<Vault>, id: InstanceId) -> Option<Vec<Label>> {
    let labels = vault
        .reservation()
        .reserve_instance_pouch()
        .grab()
        .await
        .instance_pouch
        .as_ref()
        .expect("Reservations should never fail")
        .gems()
        .get(&id)?
        .manifest
        .labels
        .clone();
    Some(labels)
}

pub async fn get_instance_label_value(
    vault: Arc<Vault>,
    id: InstanceId,
    label_name: String,
) -> Option<Option<Option<String>>> {
    Some(
        vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .expect("Reservations should never fail")
            .gems()
            .get(&id)?
            .manifest
            .labels
            .iter()
            .find(|label| label.label == label_name)
            .map(|label| label.value.clone()),
    )
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::jeweler::app::AppInfo;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::jeweler::deployment::Deployment;
    use crate::jeweler::gem::app::{try_create_app, AppDataDeserializable, AppDeserializable};
    use crate::jeweler::gem::instance::tests::test_instance;
    use crate::jeweler::gem::instance::{InstanceId, InstanceStatus};
    use crate::jeweler::gem::manifest::tests::{create_test_manifest, create_test_manifest_full};
    use crate::quest::Quest;
    use crate::relic::device::usb::{Error, MockUsbDeviceReader};
    use crate::tests::prepare_test_path;
    use crate::vault;
    use crate::vault::pouch::Pouch;
    use crate::vault::{Vault, VaultConfig};
    use bollard::models::{Ipam, IpamConfig, Network};
    pub use spell::instance::tests::create_test_vault as spell_test_vault;
    use std::collections::HashMap;
    use std::io::ErrorKind;
    use std::net::Ipv4Addr;
    use std::sync::Arc;

    async fn test_vault(
        deployment: Arc<dyn Deployment>,
        instance_count: u32,
        test_name: &str,
    ) -> Arc<Vault> {
        let path = prepare_test_path(module_path!(), test_name);
        let vault = Arc::new(Vault::new(VaultConfig { path }));
        {
            let mut grab = vault
                .reservation()
                .reserve_instance_pouch_mut()
                .grab()
                .await;
            let instances = grab.instance_pouch_mut.as_mut().unwrap();
            for i in 0..instance_count {
                let instance = test_instance(i, deployment.clone(), create_test_manifest(None));
                instances.gems_mut().insert(instance.id, instance);
            }
        }
        vault
    }

    #[tokio::test]
    async fn delete_instance_test() {
        const INSTANCE_COUNT: u32 = 4;
        const INSTANCE_TO_DELETE: u32 = 2;
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(move || "MockedDeployment".to_string());
        deployment
            .expect_stop_instance()
            .times(1)
            .returning(|_, _| Ok(()));
        deployment
            .expect_delete_instance()
            .times(1)
            .returning(|_| Ok(true));
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        deployment
            .expect_delete_volume()
            .withf(|_, id| id.starts_with(&format!("Instance#{INSTANCE_TO_DELETE}")))
            .times(4)
            .returning(|_, _| Ok(()));
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let vault = test_vault(deployment.clone(), INSTANCE_COUNT, "delete_instance_test").await;
        let instance_id = InstanceId::new(INSTANCE_TO_DELETE);
        assert!(delete_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            instance_id,
        )
        .await
        .is_ok());
        assert!(!vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .contains_key(&instance_id));
        assert!(delete_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            instance_id,
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn stop_instance_test() {
        const INSTANCE_COUNT: u32 = 4;
        const INSTANCE_TO_STOP: u32 = 2;
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(move || "MockedDeployment".to_string());
        deployment
            .expect_stop_instance()
            .times(2)
            .returning(|_, _| Ok(()));
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let vault = test_vault(deployment.clone(), INSTANCE_COUNT, "stop_instance_test").await;
        let instance_id = InstanceId::new(INSTANCE_TO_STOP);
        assert!(stop_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            instance_id,
        )
        .await
        .is_ok());
        assert!(stop_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            instance_id,
        )
        .await
        .is_ok());
        assert!(stop_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            InstanceId::new(100),
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn does_instance_exist_test() {
        const INSTANCE_COUNT: u32 = 4;
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(move || "MockedDeployment".to_string());
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let vault = test_vault(
            deployment.clone(),
            INSTANCE_COUNT,
            "does_instance_exist_test",
        )
        .await;
        for i in 0..INSTANCE_COUNT {
            assert!(does_instance_exist(vault.clone(), InstanceId::new(i)).await);
        }
        for i in INSTANCE_COUNT..INSTANCE_COUNT + 10 {
            assert!(!does_instance_exist(vault.clone(), InstanceId::new(i)).await);
        }
    }

    #[tokio::test]
    async fn instance_logs_ok() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_instance_logs()
            .times(1)
            .returning(|_, _| {
                Ok(Logs {
                    stdout: "TestOutput".to_string(),
                    stderr: "TestError".to_string(),
                })
            });
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let vault = test_vault(deployment.clone(), 1, "instance_logs_ok").await;
        let logs = get_instance_logs(vault, InstanceId::new(0)).await.unwrap();
        assert_eq!(logs.stderr, "TestError");
        assert_eq!(logs.stdout, "TestOutput");
    }

    #[tokio::test]
    async fn instance_logs_err() {
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_instance_logs()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let vault = test_vault(deployment.clone(), 1, "instance_logs_err").await;
        assert!(get_instance_logs(vault, InstanceId::new(0)).await.is_err());
    }

    async fn create_test_vault(
        test_name: &str,
        deployment: Arc<dyn Deployment>,
        with_manifest: bool,
        with_app: bool,
        multi_instance: bool,
        insert_deployment_into_pouch: bool,
    ) -> (Arc<Vault>, AppKey) {
        let path = prepare_test_path(module_path!(), test_name);
        let vault = Arc::new(Vault::new(VaultConfig { path: path.clone() }));
        let app_key = {
            let GrabbedPouches {
                manifest_pouch_mut: Some(ref mut manifests),
                app_pouch_mut: Some(ref mut apps),
                deployment_pouch_mut: Some(ref mut deployments),
                ..
            } = vault
                .reservation()
                .reserve_deployment_pouch_mut()
                .reserve_manifest_pouch_mut()
                .reserve_app_pouch_mut()
                .grab()
                .await
            else {
                unreachable!("Vault reservations should never fail")
            };
            if insert_deployment_into_pouch {
                deployments
                    .gems_mut()
                    .insert(deployment.id(), deployment.clone());
            }
            let manifest = Arc::new(create_test_manifest_full(Some(multi_instance)));
            let app_key = manifest.key.clone();
            if with_manifest {
                manifests.gems_mut().insert(app_key.clone(), manifest);
            }
            if with_app {
                let app = AppDeserializable {
                    key: app_key.clone(),
                    deployments: vec![AppDataDeserializable {
                        desired: AppStatus::Installed,
                        id: Some("TestAppId".to_string()),
                        deployment_id: deployment.id(),
                    }],
                };
                let deployments = HashMap::from([(deployment.id(), deployment)]);
                let app = try_create_app(app, manifests.gems(), &deployments).unwrap();
                apps.gems_mut().insert(app_key.clone(), app);
            }
            app_key
        };
        (vault, app_key)
    }

    #[tokio::test]
    async fn create_instance_ok() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
        deployment
            .expect_copy_from_app_image()
            .times(3)
            .returning(|_, _, _, _, _| Ok(()));
        deployment.expect_default_network().times(2).returning(|| {
            Ok(Network {
                id: Some("DefaultTestNetworkId".to_string()),
                ipam: Some(Ipam {
                    config: Some(vec![IpamConfig {
                        subnet: Some("10.18.0.0/16".to_string()),
                        gateway: Some("10.18.0.100".to_string()),
                        ..IpamConfig::default()
                    }]),
                    ..Ipam::default()
                }),
                ..Network::default()
            })
        });
        deployment
            .expect_create_volume()
            .times(1)
            .returning(|_, id| Ok(format!("TestVolumeIdFor{id}")));
        let (vault, app_key) = create_test_vault(
            "create_instance_ok",
            Arc::new(deployment),
            true,
            true,
            false,
            true,
        )
        .await;
        let instance_id = create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key,
            "TestInstance".to_string(),
        )
        .await
        .unwrap();

        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Vault reservations should never fail")
        };
        assert_eq!(instances.gems().len(), 1);
        assert!(instances.gems().contains_key(&instance_id));
        assert_eq!(
            instances
                .gems()
                .get(&instance_id)
                .unwrap()
                .config
                .network_addresses,
            HashMap::from([(
                "DefaultTestNetworkId".to_string(),
                IpAddr::V4(Ipv4Addr::new(10, 18, 0, 2))
            )])
        );
    }

    #[tokio::test]
    async fn create_instance_volume_err() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
        deployment
            .expect_copy_from_app_image()
            .times(3)
            .returning(|_, _, _, _, _| Ok(()));
        deployment.expect_default_network().times(2).returning(|| {
            Ok(Network {
                id: Some("DefaultTestNetworkId".to_string()),
                ipam: Some(Ipam {
                    config: Some(vec![IpamConfig {
                        subnet: Some("10.18.0.0/16".to_string()),
                        gateway: Some("10.18.0.100".to_string()),
                        ..IpamConfig::default()
                    }]),
                    ..Ipam::default()
                }),
                ..Network::default()
            })
        });
        deployment
            .expect_create_volume()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        let (vault, app_key) = create_test_vault(
            "create_instance_volume_err",
            Arc::new(deployment),
            true,
            true,
            false,
            true,
        )
        .await;
        assert!(create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key,
            "TestInstance".to_string(),
        )
        .await
        .is_err());

        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Vault reservations should never fail")
        };
        assert!(instances.gems().is_empty());
        assert!(instances.unavailable_ipv4_addresses().is_empty());
    }

    #[tokio::test]
    async fn create_multi_instance_ok() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
        deployment
            .expect_copy_from_app_image()
            .times(6)
            .returning(|_, _, _, _, _| Ok(()));
        deployment.expect_default_network().times(4).returning(|| {
            Ok(Network {
                id: Some("DefaultTestNetworkId".to_string()),
                ipam: Some(Ipam {
                    config: Some(vec![IpamConfig {
                        subnet: Some("10.18.0.0/16".to_string()),
                        gateway: Some("10.18.0.100".to_string()),
                        ..IpamConfig::default()
                    }]),
                    ..Ipam::default()
                }),
                ..Network::default()
            })
        });
        deployment
            .expect_create_volume()
            .times(2)
            .returning(|_, id| Ok(format!("TestVolumeIdFor{id}")));
        let (vault, app_key) = create_test_vault(
            "create_multi_instance_ok",
            Arc::new(deployment),
            true,
            true,
            true,
            true,
        )
        .await;
        let instance_id_1 = create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key.clone(),
            "TestInstance1".to_string(),
        )
        .await
        .unwrap();
        let instance_id_2 = create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key,
            "TestInstance2".to_string(),
        )
        .await
        .unwrap();

        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Vault reservations should never fail")
        };
        assert_eq!(instances.gems().len(), 2);
        assert!(instances.gems().contains_key(&instance_id_1));
        assert!(instances.gems().contains_key(&instance_id_2));
        assert_eq!(
            instances
                .gems()
                .get(&instance_id_1)
                .unwrap()
                .config
                .network_addresses,
            HashMap::from([(
                "DefaultTestNetworkId".to_string(),
                IpAddr::V4(Ipv4Addr::new(10, 18, 0, 2))
            )])
        );
        assert_eq!(
            instances
                .gems()
                .get(&instance_id_2)
                .unwrap()
                .config
                .network_addresses,
            HashMap::from([(
                "DefaultTestNetworkId".to_string(),
                IpAddr::V4(Ipv4Addr::new(10, 18, 0, 3))
            )])
        );
    }
    #[tokio::test]
    async fn create_instance_single_instance_but_instance_present() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
        deployment
            .expect_copy_from_app_image()
            .times(3)
            .returning(|_, _, _, _, _| Ok(()));
        deployment.expect_default_network().times(2).returning(|| {
            Ok(Network {
                id: Some("DefaultTestNetworkId".to_string()),
                ipam: Some(Ipam {
                    config: Some(vec![IpamConfig {
                        subnet: Some("10.18.0.0/16".to_string()),
                        gateway: Some("10.18.0.100".to_string()),
                        ..IpamConfig::default()
                    }]),
                    ..Ipam::default()
                }),
                ..Network::default()
            })
        });
        deployment
            .expect_create_volume()
            .times(1)
            .returning(|_, id| Ok(format!("TestVolumeIdFor{id}")));
        let (vault, app_key) = create_test_vault(
            "create_instance_single_instance_but_instance_present",
            Arc::new(deployment),
            true,
            true,
            false,
            true,
        )
        .await;
        let instance_id = create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key.clone(),
            "TestInstance1".to_string(),
        )
        .await
        .unwrap();
        assert!(create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key,
            "TestInstance2".to_string(),
        )
        .await
        .is_err());

        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Vault reservations should never fail")
        };
        assert_eq!(instances.gems().len(), 1);
        assert!(instances.gems().contains_key(&instance_id));
        assert_eq!(
            instances
                .gems()
                .get(&instance_id)
                .unwrap()
                .config
                .network_addresses,
            HashMap::from([(
                "DefaultTestNetworkId".to_string(),
                IpAddr::V4(Ipv4Addr::new(10, 18, 0, 2))
            )])
        );
    }
    #[tokio::test]
    async fn create_instance_app_not_installed() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        let (vault, app_key) = create_test_vault(
            "create_instance_app_not_installed",
            Arc::new(deployment),
            true,
            true,
            false,
            true,
        )
        .await;
        assert!(create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key,
            "TestInstance".to_string(),
        )
        .await
        .is_err());

        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Vault reservations should never fail")
        };
        assert!(instances.gems().is_empty());
    }
    #[tokio::test]
    async fn create_instance_app_not_created() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        let (vault, app_key) = create_test_vault(
            "create_instance_app_not_created",
            Arc::new(deployment),
            true,
            false,
            false,
            true,
        )
        .await;
        assert!(create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key,
            "TestInstance".to_string(),
        )
        .await
        .is_err());

        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Vault reservations should never fail")
        };
        assert!(instances.gems().is_empty());
    }
    #[tokio::test]
    async fn create_instance_manifest_not_present() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
        let (vault, app_key) = create_test_vault(
            "create_instance_manifest_not_present",
            Arc::new(deployment),
            false,
            true,
            false,
            true,
        )
        .await;
        assert!(create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault.clone(),
            app_key,
            "TestInstance".to_string(),
        )
        .await
        .is_err());

        let GrabbedPouches {
            instance_pouch: Some(ref instances),
            ..
        } = vault.reservation().reserve_instance_pouch().grab().await
        else {
            unreachable!("Vault reservations should never fail")
        };
        assert!(instances.gems().is_empty());
    }

    #[tokio::test]
    async fn create_instance_no_deployment() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
        let (vault, app_key) = create_test_vault(
            "create_instance_no_deployment",
            Arc::new(deployment),
            true,
            true,
            false,
            false,
        )
        .await;
        assert!(create_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            app_key,
            "TestInstance".to_string(),
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn get_all_instances_ok() {
        let vault = spell_test_vault(module_path!(), "get_all_instances", Some(true)).await;
        let instances_infos =
            get_all_instances(Quest::new_synced("TestQuest".to_string()), vault).await;
        assert_eq!(instances_infos.len(), 6);
    }

    #[tokio::test]
    async fn get_instances_filtered_all() {
        let vault =
            spell_test_vault(module_path!(), "get_instances_filtered_all", Some(true)).await;
        let instances_infos = get_instances_filtered(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            None,
            None,
        )
        .await;
        assert_eq!(instances_infos.len(), 6);
    }

    #[tokio::test]
    async fn get_instances_filtered_name() {
        let vault =
            spell_test_vault(module_path!(), "get_instances_filtered_name", Some(true)).await;
        let instances_infos = get_instances_filtered(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            Some("some.test.app-4".to_string()),
            None,
        )
        .await;
        assert_eq!(instances_infos.len(), 3);
    }

    #[tokio::test]
    async fn get_instances_filtered_version() {
        let vault =
            spell_test_vault(module_path!(), "get_instances_filtered_version", Some(true)).await;
        let instances_infos = get_instances_filtered(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            None,
            Some("1.2.4".to_string()),
        )
        .await;
        assert_eq!(instances_infos.len(), 4);
    }

    #[tokio::test]
    async fn get_instances_filtered_key() {
        let vault =
            spell_test_vault(module_path!(), "get_instances_filtered_key", Some(true)).await;
        let instances_infos = get_instances_filtered(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            Some("some.test.app-4".to_string()),
            Some("1.2.4".to_string()),
        )
        .await;
        assert_eq!(instances_infos.len(), 2);
    }

    #[tokio::test]
    async fn get_instance_ok() {
        let vault = spell_test_vault(module_path!(), "get_instance_ok", Some(true)).await;
        assert!(get_instance(vault, 1.into()).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn get_instance_detailed_ok() {
        let vault = spell_test_vault(module_path!(), "get_instance_detailed_ok", Some(true)).await;
        assert!(get_instance_detailed(vault, 1.into())
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn start_instance_ok() {
        let vault = spell_test_vault(module_path!(), "start_instance_ok", Some(true)).await;
        start_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            InstanceId::new(1),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn start_instance_err() {
        let vault = spell_test_vault(module_path!(), "start_instance_err", Some(false)).await;
        assert!(start_instance(
            Quest::new_synced("TestQuest".to_string()),
            vault,
            InstanceId::new(1),
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn get_instance_config_some() {
        let vault = spell_test_vault(module_path!(), "get_instance_config_some", None).await;
        let expected_config = vault::pouch::instance::tests::test_config();
        assert_eq!(
            get_instance_config(vault, InstanceId::new(6)).await,
            Some(expected_config)
        );
    }

    #[tokio::test]
    async fn get_instance_config_none() {
        let vault = spell_test_vault(module_path!(), "get_instance_config_none", None).await;
        assert!(get_instance_config(vault, InstanceId::new(80))
            .await
            .is_none());
    }

    #[tokio::test]
    async fn get_instance_config_port_mapping_some_some() {
        let vault = spell_test_vault(
            module_path!(),
            "get_instance_config_port_mapping_some_some",
            None,
        )
        .await;
        let expected_port_mapping = vault::pouch::instance::tests::test_config()
            .port_mapping
            .tcp[0]
            .clone();
        assert_eq!(
            get_instance_config_port_mapping(vault, InstanceId::new(6), 80, TransportProtocol::Tcp)
                .await,
            Some(Some(expected_port_mapping))
        );
    }

    #[tokio::test]
    async fn get_instance_config_port_mapping_some_none() {
        let vault = spell_test_vault(
            module_path!(),
            "get_instance_config_port_mapping_some_none",
            None,
        )
        .await;
        assert_eq!(
            get_instance_config_port_mapping(vault, InstanceId::new(6), 1, TransportProtocol::Sctp)
                .await,
            Some(None)
        );
    }

    #[tokio::test]
    async fn get_instance_config_port_mapping_none() {
        let vault = spell_test_vault(
            module_path!(),
            "get_instance_config_port_mapping_none",
            None,
        )
        .await;
        assert!(get_instance_config_port_mapping(
            vault,
            InstanceId::new(9),
            1,
            TransportProtocol::Udp
        )
        .await
        .is_none());
    }

    #[tokio::test]
    async fn get_instance_config_port_mappings_some() {
        let vault = spell_test_vault(
            module_path!(),
            "get_instance_config_port_mappings_some",
            None,
        )
        .await;
        let expected_port_mappings = vault::pouch::instance::tests::test_config().port_mapping;
        assert_eq!(
            get_instance_config_port_mappings(vault, InstanceId::new(6)).await,
            Some(expected_port_mappings)
        );
    }

    #[tokio::test]
    async fn get_instance_config_port_mappings_none() {
        let vault = spell_test_vault(
            module_path!(),
            "get_instance_config_port_mappings_none",
            None,
        )
        .await;
        assert!(
            get_instance_config_port_mappings(vault, InstanceId::new(20))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn get_instance_config_protocol_port_mappings_some() {
        let vault = spell_test_vault(
            module_path!(),
            "get_instance_config_protocol_port_mappings_some",
            None,
        )
        .await;
        let expected_port_mappings = vault::pouch::instance::tests::test_config().port_mapping;
        assert_eq!(
            get_instance_config_protocol_port_mappings(
                vault.clone(),
                InstanceId::new(6),
                TransportProtocol::Tcp
            )
            .await,
            Some(expected_port_mappings.tcp)
        );
        assert_eq!(
            get_instance_config_protocol_port_mappings(
                vault.clone(),
                InstanceId::new(6),
                TransportProtocol::Udp
            )
            .await,
            Some(expected_port_mappings.udp)
        );
        assert_eq!(
            get_instance_config_protocol_port_mappings(
                vault,
                InstanceId::new(6),
                TransportProtocol::Sctp
            )
            .await,
            Some(expected_port_mappings.sctp)
        );
    }

    #[tokio::test]
    async fn get_instance_config_protocol_port_mappings_none() {
        let vault = spell_test_vault(
            module_path!(),
            "get_instance_config_protocol_port_mappings_none",
            None,
        )
        .await;
        assert!(get_instance_config_protocol_port_mappings(
            vault.clone(),
            InstanceId::new(10),
            TransportProtocol::Tcp
        )
        .await
        .is_none());
        assert!(get_instance_config_protocol_port_mappings(
            vault.clone(),
            InstanceId::new(11),
            TransportProtocol::Udp
        )
        .await
        .is_none());
        assert!(get_instance_config_protocol_port_mappings(
            vault.clone(),
            InstanceId::new(12),
            TransportProtocol::Sctp
        )
        .await
        .is_none());
    }

    #[tokio::test]
    async fn delete_instance_config_protocol_port_mappings_some() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_config_protocol_port_mappings_some",
            None,
        )
        .await;
        let expected_port_mappings = vault::pouch::instance::tests::test_config().port_mapping;
        assert_eq!(
            delete_instance_config_protocol_port_mappings(
                vault.clone(),
                InstanceId::new(6),
                TransportProtocol::Sctp
            )
            .await,
            Some(expected_port_mappings.sctp)
        );
        let port_mappings = vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .get(&InstanceId::new(6))
            .unwrap()
            .config
            .port_mapping
            .clone();
        assert!(port_mappings.sctp.is_empty());
        assert!(!port_mappings.udp.is_empty());
        assert!(!port_mappings.tcp.is_empty());
        assert_eq!(
            delete_instance_config_protocol_port_mappings(
                vault.clone(),
                InstanceId::new(6),
                TransportProtocol::Tcp
            )
            .await,
            Some(expected_port_mappings.tcp)
        );
        let port_mappings = vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .get(&InstanceId::new(6))
            .unwrap()
            .config
            .port_mapping
            .clone();
        assert!(port_mappings.sctp.is_empty());
        assert!(!port_mappings.udp.is_empty());
        assert!(port_mappings.tcp.is_empty());
        assert_eq!(
            delete_instance_config_protocol_port_mappings(
                vault.clone(),
                InstanceId::new(6),
                TransportProtocol::Udp
            )
            .await,
            Some(expected_port_mappings.udp)
        );
        let port_mappings = vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .get(&InstanceId::new(6))
            .unwrap()
            .config
            .port_mapping
            .clone();
        assert!(port_mappings.sctp.is_empty());
        assert!(port_mappings.udp.is_empty());
        assert!(port_mappings.tcp.is_empty());
    }

    #[tokio::test]
    async fn delete_instance_config_protocol_port_mappings_none() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_config_protocol_port_mappings_none",
            None,
        )
        .await;
        assert!(delete_instance_config_protocol_port_mappings(
            vault.clone(),
            InstanceId::new(10),
            TransportProtocol::Tcp
        )
        .await
        .is_none());
        assert!(delete_instance_config_protocol_port_mappings(
            vault.clone(),
            InstanceId::new(11),
            TransportProtocol::Udp
        )
        .await
        .is_none());
        assert!(delete_instance_config_protocol_port_mappings(
            vault.clone(),
            InstanceId::new(12),
            TransportProtocol::Sctp
        )
        .await
        .is_none());
    }

    #[tokio::test]
    async fn delete_instance_config_port_mapping_none() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_config_port_mapping_none",
            None,
        )
        .await;
        assert!(delete_instance_config_port_mapping(
            vault.clone(),
            InstanceId::new(12),
            10,
            TransportProtocol::Sctp
        )
        .await
        .is_none());
    }

    #[tokio::test]
    async fn delete_instance_config_port_mapping_true() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_config_port_mapping_true",
            None,
        )
        .await;
        assert_eq!(
            delete_instance_config_port_mapping(
                vault.clone(),
                InstanceId::new(6),
                80,
                TransportProtocol::Tcp
            )
            .await,
            Some(true)
        );
    }

    #[tokio::test]
    async fn delete_instance_config_port_mapping_false() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_config_port_mapping_false",
            None,
        )
        .await;
        assert_eq!(
            delete_instance_config_port_mapping(
                vault.clone(),
                InstanceId::new(6),
                80,
                TransportProtocol::Udp
            )
            .await,
            Some(false)
        );
    }

    #[tokio::test]
    async fn delete_instance_config_port_mapping_range_none() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_config_port_mapping_range_none",
            None,
        )
        .await;
        assert!(delete_instance_config_port_mapping_range(
            vault.clone(),
            InstanceId::new(12),
            PortRange::new(20..=30),
            TransportProtocol::Sctp
        )
        .await
        .is_none());
    }

    #[tokio::test]
    async fn delete_instance_config_port_mapping_range_true() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_config_port_mapping_range_true",
            None,
        )
        .await;
        assert_eq!(
            delete_instance_config_port_mapping_range(
                vault.clone(),
                InstanceId::new(6),
                PortRange::new(50..=100),
                TransportProtocol::Udp
            )
            .await,
            Some(true)
        );
    }

    #[tokio::test]
    async fn delete_instance_config_port_mapping_range_false() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_config_port_mapping_range_false",
            None,
        )
        .await;
        assert_eq!(
            delete_instance_config_port_mapping_range(
                vault.clone(),
                InstanceId::new(6),
                PortRange::new(50..=60),
                TransportProtocol::Udp
            )
            .await,
            Some(false)
        );
    }

    #[tokio::test]
    async fn get_instance_config_port_mapping_range_none() {
        let vault = spell_test_vault(
            module_path!(),
            "get_instance_config_port_mapping_range_none",
            None,
        )
        .await;
        assert!(get_instance_config_port_mapping_range(
            vault.clone(),
            InstanceId::new(12),
            PortRange::new(20..=30),
            TransportProtocol::Sctp
        )
        .await
        .is_none());
    }

    #[tokio::test]
    async fn get_instance_config_port_mapping_range_some_some() {
        let vault = spell_test_vault(
            module_path!(),
            "get_instance_config_port_mapping_range_some_some",
            None,
        )
        .await;
        assert_eq!(
            get_instance_config_port_mapping_range(
                vault.clone(),
                InstanceId::new(6),
                PortRange::new(50..=100),
                TransportProtocol::Udp
            )
            .await,
            Some(Some(PortMapping::Range {
                from: PortRange::new(50..=100),
                to: PortRange::new(150..=200)
            }))
        );
    }

    #[tokio::test]
    async fn get_instance_config_port_mapping_range_some_none() {
        let vault = spell_test_vault(
            module_path!(),
            "get_instance_config_port_mapping_range_some_none",
            None,
        )
        .await;
        assert_eq!(
            get_instance_config_port_mapping_range(
                vault.clone(),
                InstanceId::new(6),
                PortRange::new(50..=60),
                TransportProtocol::Udp
            )
            .await,
            Some(None)
        );
    }

    #[tokio::test]
    async fn put_instance_config_port_mapping_ok_none() {
        let vault = spell_test_vault(
            module_path!(),
            "put_instance_config_port_mapping_ok_none",
            None,
        )
        .await;
        assert!(matches!(
            put_instance_config_port_mapping(
                vault.clone(),
                InstanceId::new(9),
                PortMapping::Single(1, 2),
                TransportProtocol::Udp
            )
            .await,
            Ok(None)
        ));
    }

    #[tokio::test]
    async fn put_instance_config_port_mapping_ok_some_true() {
        let vault = spell_test_vault(
            module_path!(),
            "put_instance_config_port_mapping_ok_some_true",
            None,
        )
        .await;
        assert!(matches!(
            put_instance_config_port_mapping(
                vault.clone(),
                InstanceId::new(6),
                PortMapping::Single(80, 2),
                TransportProtocol::Tcp
            )
            .await,
            Ok(Some(true))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_port_mapping_ok_some_false() {
        let vault = spell_test_vault(
            module_path!(),
            "put_instance_config_port_mapping_ok_some_false",
            None,
        )
        .await;
        assert!(matches!(
            put_instance_config_port_mapping(
                vault.clone(),
                InstanceId::new(6),
                PortMapping::Single(99, 2),
                TransportProtocol::Sctp
            )
            .await,
            Ok(Some(false))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_port_mapping_err() {
        let vault =
            spell_test_vault(module_path!(), "put_instance_config_port_mapping_err", None).await;
        assert!(put_instance_config_port_mapping(
            vault.clone(),
            InstanceId::new(6),
            PortMapping::Single(60, 2),
            TransportProtocol::Udp
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn put_instance_config_protocol_port_mappings_true() {
        let vault = spell_test_vault(
            module_path!(),
            "put_instance_config_protocol_port_mappings_true",
            None,
        )
        .await;
        let mappings = vec![PortMapping::Single(60, 2)];
        assert!(
            put_instance_config_protocol_port_mappings(
                vault.clone(),
                InstanceId::new(2),
                mappings.clone(),
                TransportProtocol::Tcp
            )
            .await
        );
        assert!(
            put_instance_config_protocol_port_mappings(
                vault.clone(),
                InstanceId::new(2),
                mappings.clone(),
                TransportProtocol::Udp
            )
            .await
        );
        assert!(
            put_instance_config_protocol_port_mappings(
                vault.clone(),
                InstanceId::new(2),
                mappings.clone(),
                TransportProtocol::Sctp
            )
            .await
        );
    }

    #[tokio::test]
    async fn put_instance_config_protocol_port_mappings_false() {
        let vault = spell_test_vault(
            module_path!(),
            "put_instance_config_protocol_port_mappings_false",
            None,
        )
        .await;
        let mappings = vec![PortMapping::Single(60, 2)];
        assert!(
            !put_instance_config_protocol_port_mappings(
                vault.clone(),
                InstanceId::new(9),
                mappings.clone(),
                TransportProtocol::Tcp
            )
            .await
        );
        assert!(
            !put_instance_config_protocol_port_mappings(
                vault.clone(),
                InstanceId::new(9),
                mappings.clone(),
                TransportProtocol::Udp
            )
            .await
        );
        assert!(
            !put_instance_config_protocol_port_mappings(
                vault.clone(),
                InstanceId::new(9),
                mappings.clone(),
                TransportProtocol::Sctp
            )
            .await
        );
    }

    #[tokio::test]
    async fn delete_instance_config_port_mappings_false() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_config_port_mappings_false",
            None,
        )
        .await;
        assert!(!delete_instance_config_port_mappings(vault, InstanceId::new(9)).await)
    }

    #[tokio::test]
    async fn delete_instance_config_port_mappings_true() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_config_port_mappings_true",
            None,
        )
        .await;
        assert!(delete_instance_config_port_mappings(vault.clone(), InstanceId::new(6)).await);
        assert!(vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .get(&InstanceId::new(6))
            .unwrap()
            .config
            .port_mapping
            .is_empty())
    }

    #[tokio::test]
    async fn get_instance_config_environment_variable_value_none() {
        let vault = spell_test_vault(
            module_path!(),
            "get_instance_config_environment_variable_value_none",
            None,
        )
        .await;
        assert!(get_instance_config_environment_variable_value(
            vault,
            InstanceId::new(200),
            "".to_string()
        )
        .await
        .is_none());
    }

    #[tokio::test]
    async fn get_instance_config_environment_variable_value_some_none() {
        let vault = spell_test_vault(
            module_path!(),
            "get_instance_config_environment_variable_value_some_none",
            None,
        )
        .await;
        assert!(matches!(
            get_instance_config_environment_variable_value(
                vault,
                InstanceId::new(6),
                "VAR_3".to_string()
            )
            .await,
            Some(None)
        ));
    }

    #[tokio::test]
    async fn get_instance_config_environment_variable_value_some_some() {
        let vault = spell_test_vault(
            module_path!(),
            "get_instance_config_environment_variable_value_some_some",
            None,
        )
        .await;
        assert_eq!(
            get_instance_config_environment_variable_value(
                vault.clone(),
                InstanceId::new(6),
                "VAR_2".to_string()
            )
            .await,
            Some(Some(Some("value".to_string())))
        );
        assert_eq!(
            get_instance_config_environment_variable_value(
                vault,
                InstanceId::new(6),
                "VAR_1".to_string()
            )
            .await,
            Some(Some(None))
        );
    }

    #[tokio::test]
    async fn put_instance_config_environment_variable_value_none() {
        let vault = spell_test_vault(
            module_path!(),
            "put_instance_config_environment_variable_value_none",
            None,
        )
        .await;
        assert!(put_instance_config_environment_variable_value(
            vault.clone(),
            InstanceId::new(600),
            EnvironmentVariable {
                name: "VAR_3".to_string(),
                value: None
            }
        )
        .await
        .is_none());
    }

    #[tokio::test]
    async fn put_instance_config_environment_variable_value_some_new() {
        let vault = spell_test_vault(
            module_path!(),
            "put_instance_config_environment_variable_value_some_new",
            None,
        )
        .await;
        let new_environment_variable = EnvironmentVariable {
            name: "VAR_3".to_string(),
            value: Some("test-value".to_string()),
        };
        assert!(matches!(
            put_instance_config_environment_variable_value(
                vault.clone(),
                InstanceId::new(6),
                new_environment_variable.clone(),
            )
            .await,
            Some(None)
        ));
        assert_eq!(
            vault
                .reservation()
                .reserve_instance_pouch()
                .grab()
                .await
                .instance_pouch
                .as_ref()
                .unwrap()
                .gems()
                .get(&InstanceId::new(6))
                .unwrap()
                .config
                .environment_variables
                .get(2)
                .cloned(),
            Some(new_environment_variable),
        );
    }

    #[tokio::test]
    async fn put_instance_config_environment_variable_value_some_replace() {
        let vault = spell_test_vault(
            module_path!(),
            "put_instance_config_environment_variable_value_some_replace",
            None,
        )
        .await;
        let new_environment_variable = EnvironmentVariable {
            name: "VAR_2".to_string(),
            value: Some("test-value".to_string()),
        };
        assert_eq!(
            put_instance_config_environment_variable_value(
                vault.clone(),
                InstanceId::new(6),
                new_environment_variable.clone(),
            )
            .await,
            Some(Some("value".to_string()))
        );
        assert_eq!(
            vault
                .reservation()
                .reserve_instance_pouch()
                .grab()
                .await
                .instance_pouch
                .as_ref()
                .unwrap()
                .gems()
                .get(&InstanceId::new(6))
                .unwrap()
                .config
                .environment_variables
                .get(1)
                .cloned(),
            Some(new_environment_variable),
        );
    }

    #[tokio::test]
    async fn delete_instance_config_environment_variable_value_none() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_config_environment_variable_value_none",
            None,
        )
        .await;
        assert!(delete_instance_config_environment_variable_value(
            vault,
            InstanceId::new(200),
            "".to_string()
        )
        .await
        .is_none());
    }

    #[tokio::test]
    async fn delete_instance_config_environment_variable_value_some_none() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_config_environment_variable_value_some_none",
            None,
        )
        .await;
        assert!(matches!(
            delete_instance_config_environment_variable_value(
                vault,
                InstanceId::new(6),
                "VAR_3".to_string()
            )
            .await,
            Some(None)
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_environment_variable_value_some_some() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_config_environment_variable_value_some_some",
            None,
        )
        .await;
        let expected_environment_variable = EnvironmentVariable {
            name: "VAR_2".to_string(),
            value: Some("value".to_string()),
        };
        assert_eq!(
            delete_instance_config_environment_variable_value(
                vault.clone(),
                InstanceId::new(6),
                "VAR_2".to_string()
            )
            .await,
            Some(Some(expected_environment_variable))
        );
        let expected_environment_variable = EnvironmentVariable {
            name: "VAR_1".to_string(),
            value: None,
        };
        assert_eq!(
            delete_instance_config_environment_variable_value(
                vault.clone(),
                InstanceId::new(6),
                "VAR_1".to_string()
            )
            .await,
            Some(Some(expected_environment_variable))
        );
        assert!(vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .get(&InstanceId::new(6))
            .unwrap()
            .config
            .environment_variables
            .is_empty());
    }

    #[tokio::test]
    async fn get_instance_config_environment_none() {
        let vault =
            spell_test_vault(module_path!(), "get_instance_config_environment_none", None).await;
        assert!(
            get_instance_config_environment(vault.clone(), InstanceId::new(80))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn get_instance_config_environment_some() {
        let vault =
            spell_test_vault(module_path!(), "get_instance_config_environment_some", None).await;
        let result = get_instance_config_environment(vault.clone(), InstanceId::new(6)).await;
        assert_eq!(
            result,
            Some(
                vault
                    .reservation()
                    .reserve_instance_pouch()
                    .grab()
                    .await
                    .instance_pouch
                    .as_ref()
                    .unwrap()
                    .gems()
                    .get(&InstanceId::new(6))
                    .unwrap()
                    .config
                    .environment_variables
                    .clone()
            )
        );
    }

    #[tokio::test]
    async fn put_instance_config_environment_none() {
        let vault =
            spell_test_vault(module_path!(), "put_instance_config_environment_none", None).await;
        assert!(
            put_instance_config_environment(vault.clone(), InstanceId::new(80), Vec::new())
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn put_instance_config_environment_some() {
        let vault =
            spell_test_vault(module_path!(), "put_instance_config_environment_some", None).await;
        let new_environment = vec![EnvironmentVariable {
            name: "Test".to_string(),
            value: None,
        }];
        let expected_result = Some(
            vault
                .reservation()
                .reserve_instance_pouch()
                .grab()
                .await
                .instance_pouch
                .as_ref()
                .unwrap()
                .gems()
                .get(&InstanceId::new(6))
                .unwrap()
                .config
                .environment_variables
                .clone(),
        );
        assert_eq!(
            put_instance_config_environment(
                vault.clone(),
                InstanceId::new(6),
                new_environment.clone()
            )
            .await,
            expected_result
        );
        assert_eq!(
            vault
                .reservation()
                .reserve_instance_pouch()
                .grab()
                .await
                .instance_pouch
                .as_ref()
                .unwrap()
                .gems()
                .get(&InstanceId::new(6))
                .unwrap()
                .config
                .environment_variables,
            new_environment
        );
    }

    #[tokio::test]
    async fn delete_instance_config_environment_none() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_config_environment_none",
            None,
        )
        .await;
        assert!(
            delete_instance_config_environment(vault.clone(), InstanceId::new(80))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn delete_instance_config_environment_some() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_config_environment_some",
            None,
        )
        .await;
        let expected_result = Some(
            vault
                .reservation()
                .reserve_instance_pouch()
                .grab()
                .await
                .instance_pouch
                .as_ref()
                .unwrap()
                .gems()
                .get(&InstanceId::new(6))
                .unwrap()
                .config
                .environment_variables
                .clone(),
        );
        assert_eq!(
            delete_instance_config_environment(vault.clone(), InstanceId::new(6)).await,
            expected_result
        );
        assert!(vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .get(&InstanceId::new(6))
            .unwrap()
            .config
            .environment_variables
            .is_empty());
    }

    #[tokio::test]
    async fn get_instance_labels_none() {
        let vault = spell_test_vault(module_path!(), "get_instance_labels_none", None).await;
        assert!(get_instance_labels(vault, InstanceId::new(80))
            .await
            .is_none());
    }

    #[tokio::test]
    async fn get_instance_labels_some() {
        let vault = spell_test_vault(module_path!(), "get_instance_labels_some", None).await;
        assert_eq!(
            get_instance_labels(vault, InstanceId::new(1)).await,
            Some(vec![
                Label {
                    label: "tech.flecs".to_string(),
                    value: None,
                },
                Label {
                    label: "tech.flecs.some-label".to_string(),
                    value: Some("Some custom label value".to_string()),
                }
            ])
        );
    }

    #[tokio::test]
    async fn get_instance_label_value_none() {
        let vault = spell_test_vault(module_path!(), "get_instance_label_value_none", None).await;
        assert!(
            get_instance_label_value(vault, InstanceId::new(80), "label".to_string())
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn get_instance_label_value_some_none() {
        let vault =
            spell_test_vault(module_path!(), "get_instance_label_value_some_none", None).await;
        assert!(matches!(
            get_instance_label_value(vault, InstanceId::new(1), "label".to_string()).await,
            Some(None)
        ));
    }

    #[tokio::test]
    async fn get_instance_label_value_some_some() {
        let vault =
            spell_test_vault(module_path!(), "get_instance_label_value_some_some", None).await;
        assert_eq!(
            get_instance_label_value(vault.clone(), InstanceId::new(1), "tech.flecs".to_string())
                .await,
            Some(Some(None))
        );
        assert_eq!(
            get_instance_label_value(
                vault,
                InstanceId::new(1),
                "tech.flecs.some-label".to_string()
            )
            .await,
            Some(Some(Some("Some custom label value".to_string())))
        );
    }

    #[tokio::test]
    async fn get_instance_usb_devices_err() {
        let vault = spell_test_vault(module_path!(), "get_instance_usb_devices_err", None).await;
        let mut mock_reader = MockUsbDeviceReader::new();
        mock_reader
            .expect_read_usb_devices()
            .times(1)
            .returning(|| {
                Err(Error::Io(std::io::Error::new(
                    ErrorKind::Other,
                    "test error",
                )))
            });
        assert!(
            get_instance_usb_devices(vault, InstanceId::new(1), &mock_reader)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn get_instance_usb_devices_ok_none() {
        let vault =
            spell_test_vault(module_path!(), "get_instance_usb_devices_ok_none", None).await;
        assert!(matches!(
            get_instance_usb_devices(vault, InstanceId::new(20), &MockUsbDeviceReader::new()).await,
            Ok(None)
        ));
    }

    #[tokio::test]
    async fn get_instance_usb_devices_ok_inactive() {
        let vault =
            spell_test_vault(module_path!(), "get_instance_usb_devices_ok_inactive", None).await;
        let mut mock_reader = MockUsbDeviceReader::new();
        mock_reader
            .expect_read_usb_devices()
            .times(1)
            .returning(|| Ok(HashMap::default()));
        assert_eq!(
            get_instance_usb_devices(vault, InstanceId::new(6), &mock_reader)
                .await
                .unwrap()
                .unwrap(),
            vec![(
                UsbPathConfig {
                    port: "test_port".to_string(),
                    bus_num: 10,
                    dev_num: 20,
                },
                None
            )]
        );
    }

    #[tokio::test]
    async fn get_instance_usb_devices_ok_active() {
        let vault =
            spell_test_vault(module_path!(), "get_instance_usb_devices_ok_active", None).await;
        let mut mock_reader = MockUsbDeviceReader::new();
        let expected_device = UsbDevice {
            vid: 10,
            pid: 100,
            port: "test_port".to_string(),
            device: "test-dev".to_string(),
            vendor: "test-vendor".to_string(),
        };
        let expected_result = vec![(
            UsbPathConfig {
                port: "test_port".to_string(),
                bus_num: 10,
                dev_num: 20,
            },
            Some(expected_device.clone()),
        )];
        mock_reader
            .expect_read_usb_devices()
            .times(1)
            .returning(move || {
                Ok(HashMap::from([(
                    "test_port".to_string(),
                    expected_device.clone(),
                )]))
            });
        assert_eq!(
            get_instance_usb_devices(vault, InstanceId::new(6), &mock_reader)
                .await
                .unwrap()
                .unwrap(),
            expected_result
        );
    }

    #[tokio::test]
    async fn delete_instance_usb_devices_none() {
        let vault =
            spell_test_vault(module_path!(), "delete_instance_usb_devices_none", None).await;
        assert!(delete_instance_usb_devices(vault, InstanceId::new(20))
            .await
            .is_none(),);
    }

    #[tokio::test]
    async fn delete_instance_usb_devices_some_none() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_usb_devices_some_none",
            None,
        )
        .await;
        assert!(matches!(
            delete_instance_usb_device(vault, InstanceId::new(1), "test_port".to_string()).await,
            Some(None)
        ));
    }

    #[tokio::test]
    async fn delete_instance_usb_devices_some() {
        let vault =
            spell_test_vault(module_path!(), "delete_instance_usb_devices_some", None).await;
        assert_eq!(
            delete_instance_usb_devices(vault.clone(), InstanceId::new(6)).await,
            Some(HashMap::from([(
                "test_port".to_string(),
                UsbPathConfig {
                    port: "test_port".to_string(),
                    bus_num: 10,
                    dev_num: 20,
                }
            )]))
        );
        assert!(vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .get(&InstanceId::new(6))
            .unwrap()
            .config
            .usb_devices
            .is_empty());
    }

    #[tokio::test]
    async fn delete_instance_usb_device_none() {
        let vault = spell_test_vault(module_path!(), "delete_instance_usb_device_none", None).await;
        assert!(
            delete_instance_usb_device(vault, InstanceId::new(20), "test_port".to_string())
                .await
                .is_none(),
        );
    }

    #[tokio::test]
    async fn delete_instance_usb_device_some_none() {
        let vault = spell_test_vault(
            module_path!(),
            "delete_instance_usb_device_some_empty",
            None,
        )
        .await;
        assert!(matches!(
            delete_instance_usb_device(vault, InstanceId::new(6), "unknown_port".to_string()).await,
            Some(None)
        ));
    }

    #[tokio::test]
    async fn delete_instance_usb_device_some() {
        let vault = spell_test_vault(module_path!(), "delete_instance_usb_device_some", None).await;
        assert_eq!(
            delete_instance_usb_device(vault.clone(), InstanceId::new(6), "test_port".to_string())
                .await,
            Some(Some(UsbPathConfig {
                port: "test_port".to_string(),
                bus_num: 10,
                dev_num: 20,
            }))
        );
        assert!(vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .get(&InstanceId::new(6))
            .unwrap()
            .config
            .usb_devices
            .is_empty());
    }

    #[tokio::test]
    async fn get_instance_usb_device_ok_instance_not_found() {
        let vault = spell_test_vault(
            module_path!(),
            "get_instance_usb_device_ok_instance_not_found",
            None,
        )
        .await;
        let reader = MockUsbDeviceReader::new();
        assert!(matches!(
            get_instance_usb_device(vault, InstanceId::new(20), "test_port".to_string(), &reader)
                .await,
            Ok(GetInstanceUsbDeviceResult::InstanceNotFound),
        ));
    }

    #[tokio::test]
    async fn get_instance_usb_device_ok_device_not_mapped() {
        let vault = spell_test_vault(
            module_path!(),
            "get_instance_usb_device_ok_device_not_mapped",
            None,
        )
        .await;
        let mut reader = MockUsbDeviceReader::new();
        let expected_device = UsbDevice {
            vid: 10,
            pid: 100,
            port: "unmapped_port".to_string(),
            device: "test-dev".to_string(),
            vendor: "test-vendor".to_string(),
        };
        let returned_device = expected_device.clone();
        reader.expect_read_usb_devices().times(1).return_once(|| {
            Ok(HashMap::from([(
                "unmapped_port".to_string(),
                returned_device,
            )]))
        });
        assert!(matches!(
            get_instance_usb_device(
                vault,
                InstanceId::new(6),
                "unmapped_port".to_string(),
                &reader
            )
            .await,
            Ok(GetInstanceUsbDeviceResult::DeviceNotMapped)
        ));
    }

    #[tokio::test]
    async fn get_instance_usb_device_ok_unknown_device() {
        let vault = spell_test_vault(
            module_path!(),
            "get_instance_usb_device_ok_unknown_device",
            None,
        )
        .await;
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_read_usb_devices()
            .times(1)
            .return_once(|| Ok(HashMap::default()));
        assert!(matches!(
            get_instance_usb_device(
                vault,
                InstanceId::new(6),
                "unknown_port".to_string(),
                &reader
            )
            .await,
            Ok(GetInstanceUsbDeviceResult::UnknownDevice)
        ));
    }

    #[tokio::test]
    async fn get_instance_usb_device_ok_inactive() {
        let vault =
            spell_test_vault(module_path!(), "get_instance_usb_device_ok_inactive", None).await;
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_read_usb_devices()
            .times(1)
            .returning(|| Ok(HashMap::default()));
        assert_eq!(
            get_instance_usb_device(vault, InstanceId::new(6), "test_port".to_string(), &reader)
                .await
                .unwrap(),
            GetInstanceUsbDeviceResult::DeviceInactive(UsbPathConfig {
                port: "test_port".to_string(),
                bus_num: 10,
                dev_num: 20,
            })
        );
    }

    #[tokio::test]
    async fn get_instance_usb_device_ok_active() {
        let vault =
            spell_test_vault(module_path!(), "get_instance_usb_device_ok_active", None).await;
        let mut reader = MockUsbDeviceReader::new();
        let expected_device = UsbDevice {
            vid: 10,
            pid: 100,
            port: "test_port".to_string(),
            device: "test-dev".to_string(),
            vendor: "test-vendor".to_string(),
        };
        let returned_device = expected_device.clone();
        reader
            .expect_read_usb_devices()
            .times(1)
            .return_once(|| Ok(HashMap::from([("test_port".to_string(), returned_device)])));
        assert_eq!(
            get_instance_usb_device(vault, InstanceId::new(6), "test_port".to_string(), &reader)
                .await
                .unwrap(),
            GetInstanceUsbDeviceResult::DeviceActive(
                UsbPathConfig {
                    port: "test_port".to_string(),
                    bus_num: 10,
                    dev_num: 20,
                },
                expected_device
            )
        );
    }

    #[tokio::test]
    async fn get_instance_usb_device_err() {
        let vault = spell_test_vault(module_path!(), "get_instance_usb_device_err", None).await;
        let mut reader = MockUsbDeviceReader::new();
        reader.expect_read_usb_devices().times(1).return_once(|| {
            Err(Error::Io(std::io::Error::new(
                ErrorKind::Other,
                "test error",
            )))
        });
        assert!(get_instance_usb_device(
            vault,
            InstanceId::new(6),
            "test_port".to_string(),
            &reader
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn put_instance_usb_device_err_devices() {
        let vault =
            spell_test_vault(module_path!(), "put_instance_usb_device_err_devices", None).await;
        let mut reader = MockUsbDeviceReader::new();
        reader.expect_read_usb_devices().times(1).return_once(|| {
            Err(Error::Io(std::io::Error::new(
                ErrorKind::Other,
                "test error",
            )))
        });
        assert!(put_instance_usb_device(
            vault,
            InstanceId::new(20),
            "test_port".to_string(),
            &reader
        )
        .await
        .is_err(),);
    }

    #[tokio::test]
    async fn put_instance_usb_device_err_devnum() {
        let vault =
            spell_test_vault(module_path!(), "put_instance_usb_device_err_devnum", None).await;
        let mut reader = MockUsbDeviceReader::new();
        let device = UsbDevice {
            vid: 10,
            pid: 100,
            port: "test_port".to_string(),
            device: "test-dev".to_string(),
            vendor: "test-vendor".to_string(),
        };
        reader
            .expect_read_usb_devices()
            .times(1)
            .return_once(|| Ok(HashMap::from([(device.port.clone(), device)])));
        reader
            .expect_get_usb_value()
            .times(1)
            .withf(|value_name, port| value_name == "devnum" && port == "test_port")
            .returning(|_, _| {
                Err(Error::Io(std::io::Error::new(
                    ErrorKind::Other,
                    "test error",
                )))
            });
        reader
            .expect_get_usb_value()
            .withf(|value_name, port| value_name == "busnum" && port == "test_port")
            .returning(|_, _| Ok("10".to_string()));
        assert!(put_instance_usb_device(
            vault.clone(),
            InstanceId::new(3),
            "test_port".to_string(),
            &reader
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn put_instance_usb_device_ok_instance_not_found() {
        let vault = spell_test_vault(
            module_path!(),
            "put_instance_usb_device_ok_instance_not_found",
            None,
        )
        .await;
        let device = UsbDevice {
            vid: 10,
            pid: 100,
            port: "test_port".to_string(),
            device: "test-dev".to_string(),
            vendor: "test-vendor".to_string(),
        };
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_read_usb_devices()
            .times(1)
            .return_once(|| Ok(HashMap::from([(device.port.clone(), device)])));
        assert!(matches!(
            put_instance_usb_device(vault, InstanceId::new(20), "test_port".to_string(), &reader)
                .await,
            Ok(PutInstanceUsbDeviceResult::InstanceNotFound),
        ));
    }

    #[tokio::test]
    async fn put_instance_usb_device_ok_device_not_found() {
        let vault = spell_test_vault(
            module_path!(),
            "put_instance_usb_device_ok_device_not_found",
            None,
        )
        .await;
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_read_usb_devices()
            .times(1)
            .return_once(|| Ok(HashMap::default()));
        assert!(matches!(
            put_instance_usb_device(
                vault,
                InstanceId::new(6),
                "unmapped_port".to_string(),
                &reader
            )
            .await,
            Ok(PutInstanceUsbDeviceResult::DeviceNotFound)
        ));
    }

    #[tokio::test]
    async fn put_instance_usb_device_ok_mapping_created() {
        let vault = spell_test_vault(
            module_path!(),
            "put_instance_usb_device_ok_mapping_created",
            None,
        )
        .await;
        let mut reader = MockUsbDeviceReader::new();
        let device = UsbDevice {
            vid: 10,
            pid: 100,
            port: "test_port".to_string(),
            device: "test-dev".to_string(),
            vendor: "test-vendor".to_string(),
        };
        reader
            .expect_read_usb_devices()
            .times(1)
            .return_once(|| Ok(HashMap::from([(device.port.clone(), device)])));
        reader
            .expect_get_usb_value()
            .times(1)
            .withf(|value_name, port| value_name == "devnum" && port == "test_port")
            .returning(|_, _| Ok("120".to_string()));
        reader
            .expect_get_usb_value()
            .times(1)
            .withf(|value_name, port| value_name == "busnum" && port == "test_port")
            .returning(|_, _| Ok("10".to_string()));
        assert!(matches!(
            put_instance_usb_device(
                vault.clone(),
                InstanceId::new(3),
                "test_port".to_string(),
                &reader
            )
            .await,
            Ok(PutInstanceUsbDeviceResult::DeviceMappingCreated)
        ));

        assert_eq!(
            vault
                .reservation()
                .reserve_instance_pouch()
                .grab()
                .await
                .instance_pouch
                .as_ref()
                .unwrap()
                .gems()
                .get(&InstanceId::new(3))
                .unwrap()
                .config
                .usb_devices
                .clone(),
            HashMap::from([(
                "test_port".to_string(),
                UsbPathConfig {
                    port: "test_port".to_string(),
                    bus_num: 10,
                    dev_num: 120
                }
            )])
        );
    }

    #[tokio::test]
    async fn put_instance_usb_device_ok_mapping_updated() {
        let vault = spell_test_vault(
            module_path!(),
            "put_instance_usb_device_ok_mapping_updated",
            None,
        )
        .await;
        let mut reader = MockUsbDeviceReader::new();
        let device = UsbDevice {
            vid: 10,
            pid: 100,
            port: "test_port".to_string(),
            device: "test-dev".to_string(),
            vendor: "test-vendor".to_string(),
        };
        reader
            .expect_read_usb_devices()
            .times(1)
            .return_once(|| Ok(HashMap::from([(device.port.clone(), device)])));
        reader
            .expect_get_usb_value()
            .times(1)
            .withf(|value_name, port| value_name == "devnum" && port == "test_port")
            .returning(|_, _| Ok("200".to_string()));
        reader
            .expect_get_usb_value()
            .times(1)
            .withf(|value_name, port| value_name == "busnum" && port == "test_port")
            .returning(|_, _| Ok("99".to_string()));
        assert_eq!(
            put_instance_usb_device(
                vault.clone(),
                InstanceId::new(6),
                "test_port".to_string(),
                &reader
            )
            .await
            .unwrap(),
            PutInstanceUsbDeviceResult::DeviceMappingUpdated(UsbPathConfig {
                port: "test_port".to_string(),
                bus_num: 10,
                dev_num: 20
            })
        );
        assert_eq!(
            vault
                .reservation()
                .reserve_instance_pouch()
                .grab()
                .await
                .instance_pouch
                .as_ref()
                .unwrap()
                .gems()
                .get(&InstanceId::new(6))
                .unwrap()
                .config
                .usb_devices
                .clone(),
            HashMap::from([(
                "test_port".to_string(),
                UsbPathConfig {
                    port: "test_port".to_string(),
                    bus_num: 99,
                    dev_num: 200
                }
            )])
        );
    }
}
