use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::forge::vec::VecExtension;
use crate::jeweler::app::AppStatus;
use crate::jeweler::deployment::Deployment;
use crate::jeweler::gem::instance::{
    InstanceConfig, InstanceId, InstancePortMapping, TransportProtocol, UsbPathConfig,
};
use crate::jeweler::gem::manifest::{EnvironmentVariable, Label, PortMapping, PortRange};
use crate::jeweler::instance::Logs;
use crate::quest::SyncQuest;
use crate::relic::device::usb::{UsbDevice, UsbDeviceReader};
use crate::relic::network::Ipv4NetworkAccess;
use crate::sorcerer::instancius::{
    GetInstanceUsbDeviceResult, Instancius, PutInstanceUsbDeviceResult, RedirectEditorRequestResult,
};
use crate::sorcerer::{spell, Sorcerer};
use crate::vault::pouch::{AppKey, Pouch};
use crate::vault::{GrabbedPouches, Vault};
use async_trait::async_trait;
use flecsd_axum_server::models::{AppInstance, InstancesInstanceIdGet200Response};
use std::collections::HashMap;
use std::net::IpAddr;
use std::num::NonZeroU16;
use std::sync::Arc;

#[derive(Default)]
pub struct InstanciusImpl {}

impl Sorcerer for InstanciusImpl {}

#[async_trait]
impl Instancius for InstanciusImpl {
    async fn start_instance<F: Floxy>(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        instance_id: InstanceId,
    ) -> anyhow::Result<()> {
        spell::instance::start_instance(quest, vault, floxy, instance_id).await
    }

    async fn stop_instance<F: Floxy>(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        instance_id: InstanceId,
    ) -> anyhow::Result<()> {
        spell::instance::stop_instance(quest, vault, floxy, instance_id).await
    }

    async fn get_instance(
        &self,
        vault: Arc<Vault>,
        instance_id: InstanceId,
    ) -> anyhow::Result<Option<AppInstance>> {
        spell::instance::get_instance_info(vault, instance_id).await
    }

    async fn get_instance_detailed(
        &self,
        vault: Arc<Vault>,
        instance_id: InstanceId,
    ) -> anyhow::Result<Option<InstancesInstanceIdGet200Response>> {
        spell::instance::get_instance_detailed_info(vault, instance_id).await
    }

    async fn get_instances_filtered(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        app_name: Option<String>,
        app_version: Option<String>,
    ) -> Vec<AppInstance> {
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

    async fn get_all_instances(&self, quest: SyncQuest, vault: Arc<Vault>) -> Vec<AppInstance> {
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

    async fn create_instance(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        app_key: AppKey,
        name: String,
    ) -> anyhow::Result<InstanceId> {
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
                        let network =
                            Ipv4NetworkAccess::try_from(deployment.default_network().await?)?;
                        let address = spell::instance::make_ipv4_reservation(vault, network)
                            .await
                            .ok_or_else(|| {
                                anyhow::anyhow!("No free ip address in default network")
                            })?;
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

    async fn does_instance_exist(&self, vault: Arc<Vault>, id: InstanceId) -> bool {
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

    async fn get_instance_config(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Option<InstanceConfig> {
        spell::instance::get_instance_config_part_with(vault, id, |config| config.clone()).await
    }

    async fn get_instance_usb_devices<U: UsbDeviceReader>(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        usb_reader: Arc<U>,
    ) -> anyhow::Result<Option<Vec<(UsbPathConfig, Option<UsbDevice>)>>> {
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

    async fn delete_instance_usb_devices(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Option<HashMap<String, UsbPathConfig>> {
        spell::instance::modify_instance_config_with(vault, id, |config| {
            std::mem::take(&mut config.usb_devices)
        })
        .await
    }

    async fn delete_instance_usb_device(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        port: String,
    ) -> Option<Option<UsbPathConfig>> {
        spell::instance::modify_instance_config_with(vault, id, |config| {
            config.usb_devices.remove(&port)
        })
        .await
    }

    async fn get_instance_usb_device<U: UsbDeviceReader>(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        port: String,
        usb_reader: Arc<U>,
    ) -> anyhow::Result<GetInstanceUsbDeviceResult> {
        let Some(mapped_device) =
            spell::instance::get_instance_config_part_with(vault, id, |config| {
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

    async fn put_instance_usb_device<U: UsbDeviceReader>(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        port: String,
        usb_reader: Arc<U>,
    ) -> anyhow::Result<PutInstanceUsbDeviceResult> {
        let Some(usb_device) = usb_reader.read_usb_devices()?.remove(&port) else {
            return Ok(PutInstanceUsbDeviceResult::DeviceNotFound);
        };
        let result = spell::instance::modify_instance_config_with(vault, id, |config| {
            Ok(config
                .usb_devices
                .insert(port, UsbPathConfig::try_from((&usb_device, &*usb_reader))?))
        })
        .await;
        match result {
            None => Ok(PutInstanceUsbDeviceResult::InstanceNotFound),
            Some(Ok(Some(previous_mapping))) => Ok(
                PutInstanceUsbDeviceResult::DeviceMappingUpdated(previous_mapping),
            ),
            Some(Ok(None)) => Ok(PutInstanceUsbDeviceResult::DeviceMappingCreated),
            Some(Err(e)) => Err(e),
        }
    }

    async fn get_instance_config_port_mapping(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        host_port: u16,
        transport_protocol: TransportProtocol,
    ) -> Option<Option<PortMapping>> {
        self.get_instance_config_port_mapping_range(
            vault,
            id,
            PortRange::new(host_port..=host_port),
            transport_protocol,
        )
        .await
    }

    async fn get_instance_config_port_mappings(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Option<InstancePortMapping> {
        spell::instance::get_instance_config_part_with(vault, id, |config| {
            config.port_mapping.clone()
        })
        .await
    }

    async fn get_instance_config_protocol_port_mappings(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        transport_protocol: TransportProtocol,
    ) -> Option<Vec<PortMapping>> {
        spell::instance::get_instance_config_part_with(vault, id, |config| match transport_protocol
        {
            TransportProtocol::Tcp => config.port_mapping.tcp.clone(),
            TransportProtocol::Udp => config.port_mapping.udp.clone(),
            TransportProtocol::Sctp => config.port_mapping.sctp.clone(),
        })
        .await
    }

    async fn delete_instance_config_protocol_port_mappings(
        &self,
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

    async fn delete_instance_config_port_mapping(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        host_port: u16,
        transport_protocol: TransportProtocol,
    ) -> Option<bool> {
        self.delete_instance_config_port_mapping_range(
            vault,
            id,
            PortRange::new(host_port..=host_port),
            transport_protocol,
        )
        .await
    }

    async fn delete_instance_config_port_mapping_range(
        &self,
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

    async fn get_instance_config_port_mapping_range(
        &self,
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

    async fn put_instance_config_port_mapping(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
        port_mapping: PortMapping,
        transport_protocol: TransportProtocol,
    ) -> anyhow::Result<Option<bool>> {
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

    async fn put_instance_config_protocol_port_mappings(
        &self,
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

    async fn delete_instance_config_port_mappings(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> bool {
        spell::instance::modify_instance_config_with(vault, id, |config| {
            config.port_mapping.clear();
        })
        .await
        .is_some()
    }

    async fn get_instance_config_environment_variable_value(
        &self,
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

    async fn put_instance_config_environment_variable_value(
        &self,
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

    async fn delete_instance_config_environment_variable_value(
        &self,
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

    async fn get_instance_config_environment(
        &self,
        vault: Arc<Vault>,
        id: InstanceId,
    ) -> Option<Vec<EnvironmentVariable>> {
        spell::instance::get_instance_config_part_with(vault, id, |config| {
            config.environment_variables.clone()
        })
        .await
    }

    async fn put_instance_config_environment(
        &self,
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

    async fn delete_instance_config_environment(
        &self,
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

    async fn delete_instance<F: Floxy + 'static>(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        id: InstanceId,
    ) -> anyhow::Result<()> {
        spell::instance::delete_instance(quest, vault, floxy, id).await
    }

    async fn get_instance_logs(&self, vault: Arc<Vault>, id: InstanceId) -> anyhow::Result<Logs> {
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

    async fn get_instance_labels(&self, vault: Arc<Vault>, id: InstanceId) -> Option<Vec<Label>> {
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

    async fn get_instance_label_value(
        &self,
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

    async fn redirect_editor_request<F: Floxy>(
        &self,
        vault: Arc<Vault>,
        floxy: Arc<FloxyOperation<F>>,
        instance_id: InstanceId,
        port: NonZeroU16,
    ) -> anyhow::Result<RedirectEditorRequestResult> {
        let mut grab = vault
            .reservation()
            .reserve_instance_pouch_mut()
            .grab()
            .await;
        let Some(instance) = grab
            .instance_pouch_mut
            .as_mut()
            .expect("Reservations should never fail")
            .gems_mut()
            .get_mut(&instance_id)
        else {
            return Ok(RedirectEditorRequestResult::InstanceNotFound);
        };
        match instance
            .manifest
            .editors()
            .iter()
            .find(|editor| editor.port == port)
        {
            None => return Ok(RedirectEditorRequestResult::UnknownPort),
            Some(editor) if editor.supports_reverse_proxy => {
                return Ok(RedirectEditorRequestResult::EditorSupportsReverseProxy);
            }
            _ => {}
        }
        if let Some(host_port) = instance.config.mapped_editor_ports.get(&port.get()) {
            return Ok(RedirectEditorRequestResult::Redirected(*host_port));
        }
        if !instance.is_running().await? {
            return Ok(RedirectEditorRequestResult::InstanceNotRunning);
        }
        let Some(network_address) = instance.get_default_network_address().await? else {
            return Ok(RedirectEditorRequestResult::InstanceNotConnectedToNetwork);
        };
        let host_port = floxy.add_instance_editor_redirect_to_free_port(
            &instance.app_key().name,
            instance_id,
            network_address,
            port.get(),
        )?;
        instance
            .config
            .mapped_editor_ports
            .insert(host_port, port.get());
        Ok(RedirectEditorRequestResult::Redirected(host_port))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::enchantment::floxy::MockFloxy;
    use crate::jeweler::app::AppInfo;
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::jeweler::deployment::Deployment;
    use crate::jeweler::gem::instance::{InstanceId, InstanceStatus};
    use crate::quest::Quest;
    use crate::relic::device::usb::{Error, MockUsbDeviceReader};
    use crate::vault;
    use crate::vault::pouch::app::tests::{
        MINIMAL_APP_NAME, MINIMAL_APP_VERSION, MINIMAL_APP_WITH_INSTANCE_NAME,
        MINIMAL_APP_WITH_INSTANCE_VERSION, MULTI_INSTANCE_APP_NAME, MULTI_INSTANCE_APP_VERSION,
        NO_MANIFEST_APP_NAME, NO_MANIFEST_APP_VERSION, SINGLE_INSTANCE_APP_NAME,
        SINGLE_INSTANCE_APP_VERSION, UNKNOWN_APP_NAME, UNKNOWN_APP_VERSION,
    };
    use crate::vault::pouch::instance::tests::{
        test_instances, test_port_mapping, EDITOR_INSTANCE, ENV_INSTANCE, LABEL_INSTANCE,
        PORT_MAPPING_INSTANCE, RUNNING_INSTANCE, UNKNOWN_INSTANCE_1, UNKNOWN_INSTANCE_2,
        UNKNOWN_INSTANCE_3, USB_DEV_INSTANCE,
    };
    use crate::vault::pouch::Pouch;
    use bollard::models::{Ipam, IpamConfig, Network};
    use std::collections::{HashMap, HashSet};
    use std::io::ErrorKind;
    use std::net::Ipv4Addr;
    use std::sync::Arc;

    #[tokio::test]
    async fn delete_instance_test() {
        const INSTANCE_TO_DELETE: InstanceId = RUNNING_INSTANCE;
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
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        let vault = vault::tests::create_test_vault(
            HashMap::from([(INSTANCE_TO_DELETE, deployment)]),
            HashMap::new(),
            None,
        );
        InstanciusImpl::default()
            .delete_instance(
                Quest::new_synced("TestQuest".to_string()),
                vault.clone(),
                floxy.clone(),
                INSTANCE_TO_DELETE,
            )
            .await
            .unwrap();
        assert!(!vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .contains_key(&INSTANCE_TO_DELETE));
        assert!(InstanciusImpl::default()
            .delete_instance(
                Quest::new_synced("TestQuest".to_string()),
                vault.clone(),
                floxy,
                INSTANCE_TO_DELETE,
            )
            .await
            .is_err());
    }

    #[tokio::test]
    async fn stop_instance_test() {
        const INSTANCE_TO_STOP: InstanceId = vault::pouch::instance::tests::RUNNING_INSTANCE;
        let floxy = MockFloxy::new();
        let floxy = FloxyOperation::new_arc(Arc::new(floxy));
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(move || "MockedDeployment".to_string());
        deployment
            .expect_stop_instance()
            .withf(|id, _| *id == INSTANCE_TO_STOP)
            .times(2)
            .returning(|_, _| Ok(()));
        deployment
            .expect_instance_status()
            .withf(|id| *id == INSTANCE_TO_STOP)
            .returning(|_| Ok(InstanceStatus::Running));
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let vault = vault::tests::create_test_vault(
            HashMap::from([(INSTANCE_TO_STOP, deployment)]),
            HashMap::new(),
            None,
        );
        InstanciusImpl::default()
            .stop_instance(
                Quest::new_synced("TestQuest".to_string()),
                vault.clone(),
                floxy.clone(),
                INSTANCE_TO_STOP,
            )
            .await
            .unwrap();
        InstanciusImpl::default()
            .stop_instance(
                Quest::new_synced("TestQuest".to_string()),
                vault.clone(),
                floxy.clone(),
                INSTANCE_TO_STOP,
            )
            .await
            .unwrap();
        assert!(InstanciusImpl::default()
            .stop_instance(
                Quest::new_synced("TestQuest".to_string()),
                vault.clone(),
                floxy.clone(),
                vault::pouch::instance::tests::UNKNOWN_INSTANCE_1,
            )
            .await
            .is_err());
    }

    #[tokio::test]
    async fn does_instance_exist_test() {
        let known_instance_ids = [vault::pouch::instance::tests::MINIMAL_INSTANCE];
        let unknown_instance_ids = [
            vault::pouch::instance::tests::UNKNOWN_INSTANCE_1,
            vault::pouch::instance::tests::UNKNOWN_INSTANCE_2,
            vault::pouch::instance::tests::UNKNOWN_INSTANCE_3,
        ];
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        for id in known_instance_ids {
            assert!(
                InstanciusImpl::default()
                    .does_instance_exist(vault.clone(), id)
                    .await
            );
        }
        for id in unknown_instance_ids {
            assert!(
                !InstanciusImpl::default()
                    .does_instance_exist(vault.clone(), id)
                    .await
            );
        }
    }

    #[tokio::test]
    async fn instance_logs_ok() {
        const ID: InstanceId = vault::pouch::instance::tests::MINIMAL_INSTANCE;
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(|| "MockedDeployment".to_string());
        deployment
            .expect_instance_logs()
            .withf(|_, id| *id == ID)
            .once()
            .returning(|_, _| {
                Ok(Logs {
                    stdout: "TestOutput".to_string(),
                    stderr: "TestError".to_string(),
                })
            });
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let vault = vault::tests::create_test_vault(
            HashMap::from([(ID, deployment)]),
            HashMap::new(),
            None,
        );
        let logs = InstanciusImpl::default()
            .get_instance_logs(vault, ID)
            .await
            .unwrap();
        assert_eq!(logs.stderr, "TestError");
        assert_eq!(logs.stdout, "TestOutput");
    }

    #[tokio::test]
    async fn instance_logs_err() {
        const ID: InstanceId = vault::pouch::instance::tests::MINIMAL_INSTANCE;
        let mut deployment = MockedDeployment::new();
        deployment
            .expect_id()
            .returning(|| "MockedDeployment".to_string());
        deployment
            .expect_instance_logs()
            .once()
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        let deployment = Arc::new(deployment) as Arc<dyn Deployment>;
        let vault = vault::tests::create_test_vault(
            HashMap::from([(ID, deployment)]),
            HashMap::new(),
            None,
        );
        assert!(InstanciusImpl::default()
            .get_instance_logs(vault, ID)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn create_instance_ok() {
        let app_key = AppKey {
            name: MINIMAL_APP_NAME.to_string(),
            version: MINIMAL_APP_VERSION.to_string(),
        };
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
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
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = vault::tests::create_test_vault(
            HashMap::new(),
            HashMap::from([(app_key.clone(), deployment.clone())]),
            Some(deployment.clone()),
        );
        let instance_id = InstanciusImpl::default()
            .create_instance(
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
        assert!(instances.gems().contains_key(&instance_id));
        assert_eq!(
            instances
                .gems()
                .get(&instance_id)
                .unwrap()
                .config
                .connected_networks,
            HashMap::from([(
                "DefaultTestNetworkId".to_string(),
                IpAddr::V4(Ipv4Addr::new(10, 18, 0, 1))
            )])
        );
    }

    #[tokio::test]
    async fn create_instance_err() {
        let app_key = AppKey {
            name: MINIMAL_APP_NAME.to_string(),
            version: MINIMAL_APP_VERSION.to_string(),
        };
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
        deployment
            .expect_default_network()
            .times(1)
            .returning(|| Err(anyhow::anyhow!("TestError")));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = vault::tests::create_test_vault(
            HashMap::new(),
            HashMap::from([(app_key.clone(), deployment.clone())]),
            Some(deployment.clone()),
        );
        let previous_unavailable_ips: HashSet<_> = vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .unavailable_ipv4_addresses();
        let previous_instances: HashSet<_> = vault
            .reservation()
            .reserve_instance_pouch()
            .grab()
            .await
            .instance_pouch
            .as_ref()
            .unwrap()
            .gems()
            .keys()
            .cloned()
            .collect();
        assert!(InstanciusImpl::default()
            .create_instance(
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
        assert_eq!(
            previous_unavailable_ips,
            instances.unavailable_ipv4_addresses()
        );
        let current_instances: HashSet<_> = instances.gems().keys().cloned().collect();
        assert_eq!(previous_instances, current_instances);
    }

    #[tokio::test]
    async fn create_multi_instance_ok() {
        let app_key = AppKey {
            name: MULTI_INSTANCE_APP_NAME.to_string(),
            version: MULTI_INSTANCE_APP_VERSION.to_string(),
        };
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
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
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = vault::tests::create_test_vault(
            HashMap::new(),
            HashMap::from([(app_key.clone(), deployment.clone())]),
            Some(deployment.clone()),
        );
        let instance_id_1 = InstanciusImpl::default()
            .create_instance(
                Quest::new_synced("TestQuest".to_string()),
                vault.clone(),
                app_key.clone(),
                "TestInstance1".to_string(),
            )
            .await
            .unwrap();
        let instance_id_2 = InstanciusImpl::default()
            .create_instance(
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
        assert!(instances.gems().contains_key(&instance_id_1));
        assert!(instances.gems().contains_key(&instance_id_2));
        assert_eq!(
            instances
                .gems()
                .get(&instance_id_1)
                .unwrap()
                .config
                .connected_networks,
            HashMap::from([(
                "DefaultTestNetworkId".to_string(),
                IpAddr::V4(Ipv4Addr::new(10, 18, 0, 1))
            )])
        );
        assert_eq!(
            instances
                .gems()
                .get(&instance_id_2)
                .unwrap()
                .config
                .connected_networks,
            HashMap::from([(
                "DefaultTestNetworkId".to_string(),
                IpAddr::V4(Ipv4Addr::new(10, 18, 0, 2))
            )])
        );
    }
    #[tokio::test]
    async fn create_instance_single_instance_but_instance_present() {
        let app_key = AppKey {
            name: SINGLE_INSTANCE_APP_NAME.to_string(),
            version: SINGLE_INSTANCE_APP_VERSION.to_string(),
        };
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
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
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = vault::tests::create_test_vault(
            HashMap::new(),
            HashMap::from([(app_key.clone(), deployment.clone())]),
            Some(deployment.clone()),
        );
        let instance_id = InstanciusImpl::default()
            .create_instance(
                Quest::new_synced("TestQuest".to_string()),
                vault.clone(),
                app_key.clone(),
                "TestInstance1".to_string(),
            )
            .await
            .unwrap();
        assert!(InstanciusImpl::default()
            .create_instance(
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
        assert!(instances.gems().contains_key(&instance_id));
        assert_eq!(
            instances
                .gems()
                .get(&instance_id)
                .unwrap()
                .config
                .connected_networks,
            HashMap::from([(
                "DefaultTestNetworkId".to_string(),
                IpAddr::V4(Ipv4Addr::new(10, 18, 0, 1))
            )])
        );
    }
    #[tokio::test]
    async fn create_instance_app_not_installed() {
        let app_key = AppKey {
            name: MINIMAL_APP_NAME.to_string(),
            version: MINIMAL_APP_VERSION.to_string(),
        };
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = vault::tests::create_test_vault(
            HashMap::new(),
            HashMap::from([(app_key.clone(), deployment.clone())]),
            Some(deployment.clone()),
        );
        assert!(InstanciusImpl::default()
            .create_instance(
                Quest::new_synced("TestQuest".to_string()),
                vault.clone(),
                app_key,
                "TestInstance".to_string(),
            )
            .await
            .is_err());
    }
    #[tokio::test]
    async fn create_instance_app_not_created() {
        let app_key = AppKey {
            name: UNKNOWN_APP_NAME.to_string(),
            version: UNKNOWN_APP_VERSION.to_string(),
        };
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = vault::tests::create_test_vault(
            HashMap::new(),
            HashMap::from([(app_key.clone(), deployment.clone())]),
            Some(deployment.clone()),
        );
        assert!(InstanciusImpl::default()
            .create_instance(
                Quest::new_synced("TestQuest".to_string()),
                vault.clone(),
                app_key,
                "TestInstance".to_string(),
            )
            .await
            .is_err());
    }

    #[tokio::test]
    async fn create_instance_manifest_not_present() {
        let app_key = AppKey {
            name: NO_MANIFEST_APP_NAME.to_string(),
            version: NO_MANIFEST_APP_VERSION.to_string(),
        };
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = vault::tests::create_test_vault(
            HashMap::new(),
            HashMap::from([(app_key.clone(), deployment.clone())]),
            Some(deployment.clone()),
        );
        assert!(InstanciusImpl::default()
            .create_instance(
                Quest::new_synced("TestQuest".to_string()),
                vault.clone(),
                app_key,
                "TestInstance".to_string(),
            )
            .await
            .is_err());
    }

    #[tokio::test]
    async fn create_instance_no_deployment() {
        let app_key = AppKey {
            name: MINIMAL_APP_NAME.to_string(),
            version: MINIMAL_APP_VERSION.to_string(),
        };
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_app_info()
            .returning(|_, _| Ok(AppInfo::default()));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = vault::tests::create_test_vault(
            HashMap::new(),
            HashMap::from([(app_key.clone(), deployment.clone())]),
            None,
        );
        vault
            .reservation()
            .reserve_deployment_pouch_mut()
            .grab()
            .await
            .deployment_pouch_mut
            .as_mut()
            .unwrap()
            .gems_mut()
            .clear();
        assert!(InstanciusImpl::default()
            .create_instance(
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
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        let vault = vault::tests::create_test_vault(
            HashMap::new(),
            HashMap::new(),
            Some(Arc::new(deployment)),
        );
        let instances_infos = InstanciusImpl::default()
            .get_all_instances(Quest::new_synced("TestQuest".to_string()), vault)
            .await;
        assert_eq!(instances_infos.len(), test_instances().len());
    }

    #[tokio::test]
    async fn get_instances_filtered_all() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        let vault = vault::tests::create_test_vault(
            HashMap::new(),
            HashMap::new(),
            Some(Arc::new(deployment)),
        );
        let instances_infos = InstanciusImpl::default()
            .get_instances_filtered(
                Quest::new_synced("TestQuest".to_string()),
                vault,
                None,
                None,
            )
            .await;
        assert_eq!(instances_infos.len(), test_instances().len());
    }

    #[tokio::test]
    async fn get_instances_filtered_name() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        let vault = vault::tests::create_test_vault(
            HashMap::new(),
            HashMap::new(),
            Some(Arc::new(deployment)),
        );
        let instances_infos = InstanciusImpl::default()
            .get_instances_filtered(
                Quest::new_synced("TestQuest".to_string()),
                vault,
                Some(MINIMAL_APP_NAME.to_string()),
                None,
            )
            .await;
        assert_eq!(instances_infos.len(), 5);
    }

    #[tokio::test]
    async fn get_instances_filtered_version() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        let vault = vault::tests::create_test_vault(
            HashMap::new(),
            HashMap::new(),
            Some(Arc::new(deployment)),
        );
        let instances_infos = InstanciusImpl::default()
            .get_instances_filtered(
                Quest::new_synced("TestQuest".to_string()),
                vault,
                None,
                Some(MINIMAL_APP_WITH_INSTANCE_VERSION.to_string()),
            )
            .await;
        assert_eq!(instances_infos.len(), 1);
    }

    #[tokio::test]
    async fn get_instances_filtered_key() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        let vault = vault::tests::create_test_vault(
            HashMap::new(),
            HashMap::new(),
            Some(Arc::new(deployment)),
        );
        let instances_infos = InstanciusImpl::default()
            .get_instances_filtered(
                Quest::new_synced("TestQuest".to_string()),
                vault,
                Some(MINIMAL_APP_WITH_INSTANCE_NAME.to_string()),
                Some(MINIMAL_APP_WITH_INSTANCE_VERSION.to_string()),
            )
            .await;
        assert_eq!(instances_infos.len(), 1);
    }

    #[tokio::test]
    async fn get_instance_ok() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = vault::tests::create_test_vault(
            HashMap::from([(RUNNING_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        assert!(InstanciusImpl::default()
            .get_instance(vault, RUNNING_INSTANCE)
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn get_instance_detailed_ok() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_instance_status()
            .returning(|_| Ok(InstanceStatus::Running));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = vault::tests::create_test_vault(
            HashMap::from([(RUNNING_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        assert!(InstanciusImpl::default()
            .get_instance_detailed(vault, RUNNING_INSTANCE)
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn start_instance_ok() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_instance_status()
            .once()
            .returning(|_| Ok(InstanceStatus::Stopped));
        deployment
            .expect_start_instance()
            .once()
            .withf(|_, id, _| *id == Some(RUNNING_INSTANCE))
            .returning(|_, _, _| Ok(RUNNING_INSTANCE));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = vault::tests::create_test_vault(
            HashMap::from([(RUNNING_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        InstanciusImpl::default()
            .start_instance(
                Quest::new_synced("TestQuest".to_string()),
                vault,
                floxy,
                RUNNING_INSTANCE,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn start_instance_err() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_instance_status()
            .once()
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = vault::tests::create_test_vault(
            HashMap::from([(RUNNING_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert!(InstanciusImpl::default()
            .start_instance(
                Quest::new_synced("TestQuest".to_string()),
                vault,
                floxy,
                RUNNING_INSTANCE,
            )
            .await
            .is_err());
    }

    #[tokio::test]
    async fn get_instance_config_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .get_instance_config(vault, RUNNING_INSTANCE)
            .await
            .is_some());
    }

    #[tokio::test]
    async fn get_instance_config_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .get_instance_config(vault, UNKNOWN_INSTANCE_1)
            .await
            .is_none());
    }

    #[tokio::test]
    async fn get_instance_config_port_mapping_some_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let expected_port_mapping = test_port_mapping().tcp[0].clone();
        assert_eq!(
            InstanciusImpl::default()
                .get_instance_config_port_mapping(
                    vault,
                    PORT_MAPPING_INSTANCE,
                    80,
                    TransportProtocol::Tcp
                )
                .await,
            Some(Some(expected_port_mapping))
        );
    }

    #[tokio::test]
    async fn get_instance_config_port_mapping_some_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert_eq!(
            InstanciusImpl::default()
                .get_instance_config_port_mapping(
                    vault,
                    PORT_MAPPING_INSTANCE,
                    1,
                    TransportProtocol::Sctp
                )
                .await,
            Some(None)
        );
    }

    #[tokio::test]
    async fn get_instance_config_port_mapping_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .get_instance_config_port_mapping(vault, UNKNOWN_INSTANCE_3, 1, TransportProtocol::Udp)
            .await
            .is_none());
    }

    #[tokio::test]
    async fn get_instance_config_port_mappings_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let expected_port_mappings = test_port_mapping();
        assert_eq!(
            InstanciusImpl::default()
                .get_instance_config_port_mappings(vault, PORT_MAPPING_INSTANCE)
                .await,
            Some(expected_port_mappings)
        );
    }

    #[tokio::test]
    async fn get_instance_config_port_mappings_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .get_instance_config_port_mappings(vault, UNKNOWN_INSTANCE_3)
            .await
            .is_none());
    }

    #[tokio::test]
    async fn get_instance_config_protocol_port_mappings_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let expected_port_mappings = test_port_mapping();
        assert_eq!(
            InstanciusImpl::default()
                .get_instance_config_protocol_port_mappings(
                    vault.clone(),
                    PORT_MAPPING_INSTANCE,
                    TransportProtocol::Tcp
                )
                .await,
            Some(expected_port_mappings.tcp)
        );
        assert_eq!(
            InstanciusImpl::default()
                .get_instance_config_protocol_port_mappings(
                    vault.clone(),
                    PORT_MAPPING_INSTANCE,
                    TransportProtocol::Udp
                )
                .await,
            Some(expected_port_mappings.udp)
        );
        assert_eq!(
            InstanciusImpl::default()
                .get_instance_config_protocol_port_mappings(
                    vault,
                    PORT_MAPPING_INSTANCE,
                    TransportProtocol::Sctp
                )
                .await,
            Some(expected_port_mappings.sctp)
        );
    }

    #[tokio::test]
    async fn get_instance_config_protocol_port_mappings_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .get_instance_config_protocol_port_mappings(
                vault.clone(),
                UNKNOWN_INSTANCE_1,
                TransportProtocol::Tcp
            )
            .await
            .is_none());
        assert!(InstanciusImpl::default()
            .get_instance_config_protocol_port_mappings(
                vault.clone(),
                UNKNOWN_INSTANCE_2,
                TransportProtocol::Udp
            )
            .await
            .is_none());
        assert!(InstanciusImpl::default()
            .get_instance_config_protocol_port_mappings(
                vault.clone(),
                UNKNOWN_INSTANCE_3,
                TransportProtocol::Sctp
            )
            .await
            .is_none());
    }

    #[tokio::test]
    async fn delete_instance_config_protocol_port_mappings_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let expected_port_mappings = test_port_mapping();
        assert_eq!(
            InstanciusImpl::default()
                .delete_instance_config_protocol_port_mappings(
                    vault.clone(),
                    PORT_MAPPING_INSTANCE,
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
            .get(&PORT_MAPPING_INSTANCE)
            .unwrap()
            .config
            .port_mapping
            .clone();
        assert!(port_mappings.sctp.is_empty());
        assert!(!port_mappings.udp.is_empty());
        assert!(!port_mappings.tcp.is_empty());
        assert_eq!(
            InstanciusImpl::default()
                .delete_instance_config_protocol_port_mappings(
                    vault.clone(),
                    PORT_MAPPING_INSTANCE,
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
            .get(&PORT_MAPPING_INSTANCE)
            .unwrap()
            .config
            .port_mapping
            .clone();
        assert!(port_mappings.sctp.is_empty());
        assert!(!port_mappings.udp.is_empty());
        assert!(port_mappings.tcp.is_empty());
        assert_eq!(
            InstanciusImpl::default()
                .delete_instance_config_protocol_port_mappings(
                    vault.clone(),
                    PORT_MAPPING_INSTANCE,
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
            .get(&PORT_MAPPING_INSTANCE)
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
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .delete_instance_config_protocol_port_mappings(
                vault.clone(),
                UNKNOWN_INSTANCE_1,
                TransportProtocol::Tcp
            )
            .await
            .is_none());
        assert!(InstanciusImpl::default()
            .delete_instance_config_protocol_port_mappings(
                vault.clone(),
                UNKNOWN_INSTANCE_2,
                TransportProtocol::Udp
            )
            .await
            .is_none());
        assert!(InstanciusImpl::default()
            .delete_instance_config_protocol_port_mappings(
                vault.clone(),
                UNKNOWN_INSTANCE_3,
                TransportProtocol::Sctp
            )
            .await
            .is_none());
    }

    #[tokio::test]
    async fn delete_instance_config_port_mapping_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .delete_instance_config_port_mapping(
                vault.clone(),
                UNKNOWN_INSTANCE_3,
                10,
                TransportProtocol::Sctp
            )
            .await
            .is_none());
    }

    #[tokio::test]
    async fn delete_instance_config_port_mapping_true() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert_eq!(
            InstanciusImpl::default()
                .delete_instance_config_port_mapping(
                    vault.clone(),
                    PORT_MAPPING_INSTANCE,
                    80,
                    TransportProtocol::Tcp
                )
                .await,
            Some(true)
        );
    }

    #[tokio::test]
    async fn delete_instance_config_port_mapping_false() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert_eq!(
            InstanciusImpl::default()
                .delete_instance_config_port_mapping(
                    vault.clone(),
                    PORT_MAPPING_INSTANCE,
                    80,
                    TransportProtocol::Udp
                )
                .await,
            Some(false)
        );
    }

    #[tokio::test]
    async fn delete_instance_config_port_mapping_range_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .delete_instance_config_port_mapping_range(
                vault.clone(),
                UNKNOWN_INSTANCE_3,
                PortRange::new(20..=30),
                TransportProtocol::Sctp
            )
            .await
            .is_none());
    }

    #[tokio::test]
    async fn delete_instance_config_port_mapping_range_true() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert_eq!(
            InstanciusImpl::default()
                .delete_instance_config_port_mapping_range(
                    vault.clone(),
                    PORT_MAPPING_INSTANCE,
                    PortRange::new(50..=100),
                    TransportProtocol::Udp
                )
                .await,
            Some(true)
        );
    }

    #[tokio::test]
    async fn delete_instance_config_port_mapping_range_false() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert_eq!(
            InstanciusImpl::default()
                .delete_instance_config_port_mapping_range(
                    vault.clone(),
                    PORT_MAPPING_INSTANCE,
                    PortRange::new(50..=60),
                    TransportProtocol::Udp
                )
                .await,
            Some(false)
        );
    }

    #[tokio::test]
    async fn get_instance_config_port_mapping_range_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .get_instance_config_port_mapping_range(
                vault.clone(),
                UNKNOWN_INSTANCE_2,
                PortRange::new(20..=30),
                TransportProtocol::Sctp
            )
            .await
            .is_none());
    }

    #[tokio::test]
    async fn get_instance_config_port_mapping_range_some_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert_eq!(
            InstanciusImpl::default()
                .get_instance_config_port_mapping_range(
                    vault.clone(),
                    PORT_MAPPING_INSTANCE,
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
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert_eq!(
            InstanciusImpl::default()
                .get_instance_config_port_mapping_range(
                    vault.clone(),
                    PORT_MAPPING_INSTANCE,
                    PortRange::new(50..=60),
                    TransportProtocol::Udp
                )
                .await,
            Some(None)
        );
    }

    #[tokio::test]
    async fn put_instance_config_port_mapping_ok_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(matches!(
            InstanciusImpl::default()
                .put_instance_config_port_mapping(
                    vault.clone(),
                    UNKNOWN_INSTANCE_1,
                    PortMapping::Single(1, 2),
                    TransportProtocol::Udp
                )
                .await,
            Ok(None)
        ));
    }

    #[tokio::test]
    async fn put_instance_config_port_mapping_ok_some_true() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(matches!(
            InstanciusImpl::default()
                .put_instance_config_port_mapping(
                    vault.clone(),
                    PORT_MAPPING_INSTANCE,
                    PortMapping::Single(80, 2),
                    TransportProtocol::Tcp
                )
                .await,
            Ok(Some(true))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_port_mapping_ok_some_false() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(matches!(
            InstanciusImpl::default()
                .put_instance_config_port_mapping(
                    vault.clone(),
                    PORT_MAPPING_INSTANCE,
                    PortMapping::Single(99, 2),
                    TransportProtocol::Sctp
                )
                .await,
            Ok(Some(false))
        ));
    }

    #[tokio::test]
    async fn put_instance_config_port_mapping_err() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .put_instance_config_port_mapping(
                vault.clone(),
                PORT_MAPPING_INSTANCE,
                PortMapping::Single(60, 2),
                TransportProtocol::Udp
            )
            .await
            .is_err());
    }

    #[tokio::test]
    async fn put_instance_config_protocol_port_mappings_true() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let mappings = vec![PortMapping::Single(60, 2)];
        assert!(
            InstanciusImpl::default()
                .put_instance_config_protocol_port_mappings(
                    vault.clone(),
                    PORT_MAPPING_INSTANCE,
                    mappings.clone(),
                    TransportProtocol::Tcp
                )
                .await
        );
        assert!(
            InstanciusImpl::default()
                .put_instance_config_protocol_port_mappings(
                    vault.clone(),
                    PORT_MAPPING_INSTANCE,
                    mappings.clone(),
                    TransportProtocol::Udp
                )
                .await
        );
        assert!(
            InstanciusImpl::default()
                .put_instance_config_protocol_port_mappings(
                    vault.clone(),
                    PORT_MAPPING_INSTANCE,
                    mappings.clone(),
                    TransportProtocol::Sctp
                )
                .await
        );
    }

    #[tokio::test]
    async fn put_instance_config_protocol_port_mappings_false() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let mappings = vec![PortMapping::Single(60, 2)];
        assert!(
            !InstanciusImpl::default()
                .put_instance_config_protocol_port_mappings(
                    vault.clone(),
                    UNKNOWN_INSTANCE_1,
                    mappings.clone(),
                    TransportProtocol::Tcp
                )
                .await
        );
        assert!(
            !InstanciusImpl::default()
                .put_instance_config_protocol_port_mappings(
                    vault.clone(),
                    UNKNOWN_INSTANCE_2,
                    mappings.clone(),
                    TransportProtocol::Udp
                )
                .await
        );
        assert!(
            !InstanciusImpl::default()
                .put_instance_config_protocol_port_mappings(
                    vault.clone(),
                    UNKNOWN_INSTANCE_3,
                    mappings.clone(),
                    TransportProtocol::Sctp
                )
                .await
        );
    }

    #[tokio::test]
    async fn delete_instance_config_port_mappings_false() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(
            !InstanciusImpl::default()
                .delete_instance_config_port_mappings(vault, UNKNOWN_INSTANCE_2)
                .await
        )
    }

    #[tokio::test]
    async fn delete_instance_config_port_mappings_true() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(
            InstanciusImpl::default()
                .delete_instance_config_port_mappings(vault.clone(), PORT_MAPPING_INSTANCE)
                .await
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
            .get(&PORT_MAPPING_INSTANCE)
            .unwrap()
            .config
            .port_mapping
            .is_empty())
    }

    #[tokio::test]
    async fn get_instance_config_environment_variable_value_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .get_instance_config_environment_variable_value(
                vault,
                UNKNOWN_INSTANCE_3,
                "".to_string()
            )
            .await
            .is_none());
    }

    #[tokio::test]
    async fn get_instance_config_environment_variable_value_some_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(matches!(
            InstanciusImpl::default()
                .get_instance_config_environment_variable_value(
                    vault,
                    ENV_INSTANCE,
                    "VAR_3".to_string()
                )
                .await,
            Some(None)
        ));
    }

    #[tokio::test]
    async fn get_instance_config_environment_variable_value_some_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert_eq!(
            InstanciusImpl::default()
                .get_instance_config_environment_variable_value(
                    vault.clone(),
                    ENV_INSTANCE,
                    "VAR_2".to_string()
                )
                .await,
            Some(Some(Some("value".to_string())))
        );
        assert_eq!(
            InstanciusImpl::default()
                .get_instance_config_environment_variable_value(
                    vault,
                    ENV_INSTANCE,
                    "VAR_1".to_string()
                )
                .await,
            Some(Some(None))
        );
    }

    #[tokio::test]
    async fn put_instance_config_environment_variable_value_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .put_instance_config_environment_variable_value(
                vault.clone(),
                UNKNOWN_INSTANCE_1,
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
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let new_environment_variable = EnvironmentVariable {
            name: "VAR_3".to_string(),
            value: Some("test-value".to_string()),
        };
        assert!(matches!(
            InstanciusImpl::default()
                .put_instance_config_environment_variable_value(
                    vault.clone(),
                    ENV_INSTANCE,
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
                .get(&ENV_INSTANCE)
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
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let new_environment_variable = EnvironmentVariable {
            name: "VAR_2".to_string(),
            value: Some("test-value".to_string()),
        };
        assert_eq!(
            InstanciusImpl::default()
                .put_instance_config_environment_variable_value(
                    vault.clone(),
                    ENV_INSTANCE,
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
                .get(&ENV_INSTANCE)
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
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .delete_instance_config_environment_variable_value(
                vault,
                UNKNOWN_INSTANCE_1,
                "".to_string()
            )
            .await
            .is_none());
    }

    #[tokio::test]
    async fn delete_instance_config_environment_variable_value_some_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(matches!(
            InstanciusImpl::default()
                .delete_instance_config_environment_variable_value(
                    vault,
                    ENV_INSTANCE,
                    "VAR_3".to_string()
                )
                .await,
            Some(None)
        ));
    }

    #[tokio::test]
    async fn delete_instance_config_environment_variable_value_some_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let expected_environment_variable = EnvironmentVariable {
            name: "VAR_2".to_string(),
            value: Some("value".to_string()),
        };
        assert_eq!(
            InstanciusImpl::default()
                .delete_instance_config_environment_variable_value(
                    vault.clone(),
                    ENV_INSTANCE,
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
            InstanciusImpl::default()
                .delete_instance_config_environment_variable_value(
                    vault.clone(),
                    ENV_INSTANCE,
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
            .get(&ENV_INSTANCE)
            .unwrap()
            .config
            .environment_variables
            .is_empty());
    }

    #[tokio::test]
    async fn get_instance_config_environment_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .get_instance_config_environment(vault.clone(), UNKNOWN_INSTANCE_1)
            .await
            .is_none());
    }

    #[tokio::test]
    async fn get_instance_config_environment_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let result = InstanciusImpl::default()
            .get_instance_config_environment(vault.clone(), ENV_INSTANCE)
            .await;
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
                    .get(&ENV_INSTANCE)
                    .unwrap()
                    .config
                    .environment_variables
                    .clone()
            )
        );
    }

    #[tokio::test]
    async fn put_instance_config_environment_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .put_instance_config_environment(vault.clone(), UNKNOWN_INSTANCE_1, Vec::new())
            .await
            .is_none());
    }

    #[tokio::test]
    async fn put_instance_config_environment_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
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
                .get(&ENV_INSTANCE)
                .unwrap()
                .config
                .environment_variables
                .clone(),
        );
        assert_eq!(
            InstanciusImpl::default()
                .put_instance_config_environment(
                    vault.clone(),
                    ENV_INSTANCE,
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
                .get(&ENV_INSTANCE)
                .unwrap()
                .config
                .environment_variables,
            new_environment
        );
    }

    #[tokio::test]
    async fn delete_instance_config_environment_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .delete_instance_config_environment(vault.clone(), UNKNOWN_INSTANCE_1)
            .await
            .is_none());
    }

    #[tokio::test]
    async fn delete_instance_config_environment_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
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
                .get(&ENV_INSTANCE)
                .unwrap()
                .config
                .environment_variables
                .clone(),
        );
        assert_eq!(
            InstanciusImpl::default()
                .delete_instance_config_environment(vault.clone(), ENV_INSTANCE)
                .await,
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
            .get(&ENV_INSTANCE)
            .unwrap()
            .config
            .environment_variables
            .is_empty());
    }

    #[tokio::test]
    async fn get_instance_labels_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .get_instance_labels(vault, UNKNOWN_INSTANCE_2)
            .await
            .is_none());
    }

    #[tokio::test]
    async fn get_instance_labels_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert_eq!(
            InstanciusImpl::default()
                .get_instance_labels(vault, LABEL_INSTANCE)
                .await,
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
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .get_instance_label_value(vault, UNKNOWN_INSTANCE_3, "label".to_string())
            .await
            .is_none());
    }

    #[tokio::test]
    async fn get_instance_label_value_some_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(matches!(
            InstanciusImpl::default()
                .get_instance_label_value(vault, LABEL_INSTANCE, "label".to_string())
                .await,
            Some(None)
        ));
    }

    #[tokio::test]
    async fn get_instance_label_value_some_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert_eq!(
            InstanciusImpl::default()
                .get_instance_label_value(vault.clone(), LABEL_INSTANCE, "tech.flecs".to_string())
                .await,
            Some(Some(None))
        );
        assert_eq!(
            InstanciusImpl::default()
                .get_instance_label_value(
                    vault,
                    LABEL_INSTANCE,
                    "tech.flecs.some-label".to_string()
                )
                .await,
            Some(Some(Some("Some custom label value".to_string())))
        );
    }

    #[tokio::test]
    async fn get_instance_usb_devices_err() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let mut usb_reader = MockUsbDeviceReader::new();
        usb_reader.expect_read_usb_devices().times(1).returning(|| {
            Err(Error::Io(std::io::Error::new(
                ErrorKind::Other,
                "test error",
            )))
        });
        assert!(InstanciusImpl::default()
            .get_instance_usb_devices(vault, USB_DEV_INSTANCE, Arc::new(usb_reader))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn get_instance_usb_devices_ok_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(matches!(
            InstanciusImpl::default()
                .get_instance_usb_devices(
                    vault,
                    UNKNOWN_INSTANCE_1,
                    Arc::new(MockUsbDeviceReader::default())
                )
                .await,
            Ok(None)
        ));
    }

    #[tokio::test]
    async fn get_instance_usb_devices_ok_inactive() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let mut usb_reader = MockUsbDeviceReader::new();
        usb_reader
            .expect_read_usb_devices()
            .times(1)
            .returning(|| Ok(HashMap::default()));
        assert_eq!(
            InstanciusImpl::default()
                .get_instance_usb_devices(vault, USB_DEV_INSTANCE, Arc::new(usb_reader))
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
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let mut usb_reader = MockUsbDeviceReader::new();
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
        usb_reader
            .expect_read_usb_devices()
            .times(1)
            .returning(move || {
                Ok(HashMap::from([(
                    "test_port".to_string(),
                    expected_device.clone(),
                )]))
            });
        assert_eq!(
            InstanciusImpl::default()
                .get_instance_usb_devices(vault, USB_DEV_INSTANCE, Arc::new(usb_reader))
                .await
                .unwrap()
                .unwrap(),
            expected_result
        );
    }

    #[tokio::test]
    async fn delete_instance_usb_devices_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .delete_instance_usb_devices(vault, UNKNOWN_INSTANCE_1)
            .await
            .is_none(),);
    }

    #[tokio::test]
    async fn delete_instance_usb_devices_some_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(matches!(
            InstanciusImpl::default()
                .delete_instance_usb_device(vault, USB_DEV_INSTANCE, "unknown_port".to_string())
                .await,
            Some(None)
        ));
    }

    #[tokio::test]
    async fn delete_instance_usb_devices_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert_eq!(
            InstanciusImpl::default()
                .delete_instance_usb_devices(vault.clone(), USB_DEV_INSTANCE)
                .await,
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
            .get(&USB_DEV_INSTANCE)
            .unwrap()
            .config
            .usb_devices
            .is_empty());
    }

    #[tokio::test]
    async fn delete_instance_usb_device_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(InstanciusImpl::default()
            .delete_instance_usb_device(vault, UNKNOWN_INSTANCE_1, "test_port".to_string())
            .await
            .is_none(),);
    }

    #[tokio::test]
    async fn delete_instance_usb_device_some_none() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert!(matches!(
            InstanciusImpl::default()
                .delete_instance_usb_device(vault, USB_DEV_INSTANCE, "unknown_port".to_string())
                .await,
            Some(None)
        ));
    }

    #[tokio::test]
    async fn delete_instance_usb_device_some() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        assert_eq!(
            InstanciusImpl::default()
                .delete_instance_usb_device(
                    vault.clone(),
                    USB_DEV_INSTANCE,
                    "test_port".to_string()
                )
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
            .get(&USB_DEV_INSTANCE)
            .unwrap()
            .config
            .usb_devices
            .is_empty());
    }

    #[tokio::test]
    async fn get_instance_usb_device_ok_instance_not_found() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let reader = MockUsbDeviceReader::new();
        assert!(matches!(
            InstanciusImpl::default()
                .get_instance_usb_device(
                    vault,
                    UNKNOWN_INSTANCE_1,
                    "test_port".to_string(),
                    Arc::new(reader)
                )
                .await,
            Ok(GetInstanceUsbDeviceResult::InstanceNotFound),
        ));
    }

    #[tokio::test]
    async fn get_instance_usb_device_ok_device_not_mapped() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
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
            InstanciusImpl::default()
                .get_instance_usb_device(
                    vault,
                    USB_DEV_INSTANCE,
                    "unmapped_port".to_string(),
                    Arc::new(reader)
                )
                .await,
            Ok(GetInstanceUsbDeviceResult::DeviceNotMapped)
        ));
    }

    #[tokio::test]
    async fn get_instance_usb_device_ok_unknown_device() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_read_usb_devices()
            .times(1)
            .return_once(|| Ok(HashMap::default()));
        assert!(matches!(
            InstanciusImpl::default()
                .get_instance_usb_device(
                    vault,
                    USB_DEV_INSTANCE,
                    "unknown_port".to_string(),
                    Arc::new(reader)
                )
                .await,
            Ok(GetInstanceUsbDeviceResult::UnknownDevice)
        ));
    }

    #[tokio::test]
    async fn get_instance_usb_device_ok_inactive() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_read_usb_devices()
            .times(1)
            .returning(|| Ok(HashMap::default()));
        assert_eq!(
            InstanciusImpl::default()
                .get_instance_usb_device(
                    vault,
                    USB_DEV_INSTANCE,
                    "test_port".to_string(),
                    Arc::new(reader)
                )
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
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
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
            InstanciusImpl::default()
                .get_instance_usb_device(
                    vault,
                    USB_DEV_INSTANCE,
                    "test_port".to_string(),
                    Arc::new(reader)
                )
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
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let mut reader = MockUsbDeviceReader::new();
        reader.expect_read_usb_devices().times(1).return_once(|| {
            Err(Error::Io(std::io::Error::new(
                ErrorKind::Other,
                "test error",
            )))
        });
        assert!(InstanciusImpl::default()
            .get_instance_usb_device(
                vault,
                USB_DEV_INSTANCE,
                "test_port".to_string(),
                Arc::new(reader)
            )
            .await
            .is_err());
    }

    #[tokio::test]
    async fn put_instance_usb_device_err_devices() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let mut reader = MockUsbDeviceReader::new();
        reader.expect_read_usb_devices().times(1).return_once(|| {
            Err(Error::Io(std::io::Error::new(
                ErrorKind::Other,
                "test error",
            )))
        });
        assert!(InstanciusImpl::default()
            .put_instance_usb_device(
                vault,
                UNKNOWN_INSTANCE_1,
                "test_port".to_string(),
                Arc::new(reader)
            )
            .await
            .is_err(),);
    }

    #[tokio::test]
    async fn put_instance_usb_device_err_devnum() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
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
        assert!(InstanciusImpl::default()
            .put_instance_usb_device(
                vault.clone(),
                USB_DEV_INSTANCE,
                "test_port".to_string(),
                Arc::new(reader)
            )
            .await
            .is_err());
    }

    #[tokio::test]
    async fn put_instance_usb_device_ok_instance_not_found() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
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
            InstanciusImpl::default()
                .put_instance_usb_device(
                    vault,
                    UNKNOWN_INSTANCE_1,
                    "test_port".to_string(),
                    Arc::new(reader)
                )
                .await,
            Ok(PutInstanceUsbDeviceResult::InstanceNotFound),
        ));
    }

    #[tokio::test]
    async fn put_instance_usb_device_ok_device_not_found() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let mut reader = MockUsbDeviceReader::new();
        reader
            .expect_read_usb_devices()
            .times(1)
            .return_once(|| Ok(HashMap::default()));
        assert!(matches!(
            InstanciusImpl::default()
                .put_instance_usb_device(
                    vault,
                    USB_DEV_INSTANCE,
                    "unmapped_port".to_string(),
                    Arc::new(reader)
                )
                .await,
            Ok(PutInstanceUsbDeviceResult::DeviceNotFound)
        ));
    }

    #[tokio::test]
    async fn put_instance_usb_device_ok_mapping_created() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let mut reader = MockUsbDeviceReader::new();
        let device = UsbDevice {
            vid: 10,
            pid: 100,
            port: "new_port".to_string(),
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
            .withf(|value_name, port| value_name == "devnum" && port == "new_port")
            .returning(|_, _| Ok("120".to_string()));
        reader
            .expect_get_usb_value()
            .times(1)
            .withf(|value_name, port| value_name == "busnum" && port == "new_port")
            .returning(|_, _| Ok("10".to_string()));
        assert_eq!(
            InstanciusImpl::default()
                .put_instance_usb_device(
                    vault.clone(),
                    USB_DEV_INSTANCE,
                    "new_port".to_string(),
                    Arc::new(reader)
                )
                .await
                .unwrap(),
            PutInstanceUsbDeviceResult::DeviceMappingCreated
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
                .get(&USB_DEV_INSTANCE)
                .unwrap()
                .config
                .usb_devices
                .get(&"new_port".to_string())
                .cloned()
                .unwrap(),
            UsbPathConfig {
                port: "new_port".to_string(),
                bus_num: 10,
                dev_num: 120
            }
        );
    }

    #[tokio::test]
    async fn put_instance_usb_device_ok_mapping_updated() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
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
            InstanciusImpl::default()
                .put_instance_usb_device(
                    vault.clone(),
                    USB_DEV_INSTANCE,
                    "test_port".to_string(),
                    Arc::new(reader)
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
                .get(&USB_DEV_INSTANCE)
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

    #[tokio::test]
    async fn redirect_editor_request_instance_not_found() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert!(matches!(
            InstanciusImpl::default()
                .redirect_editor_request(
                    vault,
                    floxy,
                    UNKNOWN_INSTANCE_1,
                    NonZeroU16::new(100).unwrap()
                )
                .await,
            Ok(RedirectEditorRequestResult::InstanceNotFound)
        ));
    }

    #[tokio::test]
    async fn redirect_editor_request_unknown_port() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert!(matches!(
            InstanciusImpl::default()
                .redirect_editor_request(
                    vault,
                    floxy,
                    EDITOR_INSTANCE,
                    NonZeroU16::new(60).unwrap()
                )
                .await,
            Ok(RedirectEditorRequestResult::UnknownPort)
        ));
    }

    #[tokio::test]
    async fn redirect_editor_request_no_reverse_proxy_support() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_instance_status()
            .once()
            .returning(|_| Ok(InstanceStatus::Running));
        deployment.expect_default_network().once().returning(|| {
            Ok(Network {
                id: Some("flecs".to_string()),
                ..Network::default()
            })
        });
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = vault::tests::create_test_vault(
            HashMap::from([(EDITOR_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        let mut floxy = MockFloxy::new();
        floxy
            .expect_add_instance_editor_redirect_to_free_port()
            .times(1)
            .returning(|_, _, _, _| Ok((false, 125)));
        let floxy = FloxyOperation::new_arc(Arc::new(floxy));
        assert_eq!(
            InstanciusImpl::default()
                .redirect_editor_request(
                    vault,
                    floxy,
                    EDITOR_INSTANCE,
                    NonZeroU16::new(1234).unwrap()
                )
                .await
                .unwrap(),
            RedirectEditorRequestResult::Redirected(125)
        );
    }

    #[tokio::test]
    async fn redirect_editor_request_reverse_proxy_support() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert_eq!(
            InstanciusImpl::default()
                .redirect_editor_request(
                    vault,
                    floxy,
                    EDITOR_INSTANCE,
                    NonZeroU16::new(5678).unwrap()
                )
                .await
                .unwrap(),
            RedirectEditorRequestResult::EditorSupportsReverseProxy
        );
    }

    #[tokio::test]
    async fn redirect_editor_request_instance_stopped() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_instance_status()
            .once()
            .returning(|_| Ok(InstanceStatus::Stopped));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = vault::tests::create_test_vault(
            HashMap::from([(EDITOR_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert_eq!(
            InstanciusImpl::default()
                .redirect_editor_request(
                    vault,
                    floxy,
                    EDITOR_INSTANCE,
                    NonZeroU16::new(1234).unwrap()
                )
                .await
                .unwrap(),
            RedirectEditorRequestResult::InstanceNotRunning
        );
    }

    #[tokio::test]
    async fn redirect_editor_request_not_connected_to_network() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_instance_status()
            .once()
            .returning(|_| Ok(InstanceStatus::Running));
        deployment.expect_default_network().once().returning(|| {
            Ok(Network {
                id: Some("unknown".to_string()),
                ..Network::default()
            })
        });
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = vault::tests::create_test_vault(
            HashMap::from([(EDITOR_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert_eq!(
            InstanciusImpl::default()
                .redirect_editor_request(
                    vault,
                    floxy,
                    EDITOR_INSTANCE,
                    NonZeroU16::new(1234).unwrap()
                )
                .await
                .unwrap(),
            RedirectEditorRequestResult::InstanceNotConnectedToNetwork
        );
    }

    #[tokio::test]
    async fn redirect_editor_request_existing_redirect() {
        let vault = vault::tests::create_test_vault(HashMap::new(), HashMap::new(), None);
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert_eq!(
            InstanciusImpl::default()
                .redirect_editor_request(
                    vault,
                    floxy,
                    EDITOR_INSTANCE,
                    NonZeroU16::new(3000).unwrap()
                )
                .await
                .unwrap(),
            RedirectEditorRequestResult::Redirected(4000)
        );
    }

    #[tokio::test]
    async fn redirect_editor_request_err() {
        let mut deployment = MockedDeployment::new();
        deployment.expect_id().return_const("MockedDeployment");
        deployment
            .expect_instance_status()
            .once()
            .returning(|_| Err(anyhow::anyhow!("TestError")));
        let deployment: Arc<dyn Deployment> = Arc::new(deployment);
        let vault = vault::tests::create_test_vault(
            HashMap::from([(EDITOR_INSTANCE, deployment)]),
            HashMap::new(),
            None,
        );
        let floxy = FloxyOperation::new_arc(Arc::new(MockFloxy::new()));
        assert!(InstanciusImpl::default()
            .redirect_editor_request(
                vault,
                floxy,
                EDITOR_INSTANCE,
                NonZeroU16::new(1234).unwrap()
            )
            .await
            .is_err());
    }
}
