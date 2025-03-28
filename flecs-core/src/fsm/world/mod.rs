mod config;
use crate::enchantment::floxy::{Floxy, FloxyImpl, FloxyOperation};
use crate::enchantment::quest_master::QuestMaster;
use crate::enchantment::Enchantments;
use crate::fsm::ServerHandle;
use crate::quest::QuestResult;
use crate::relic::device::net::{NetDeviceReader, NetDeviceReaderImpl};
use crate::relic::device::usb::{UsbDeviceReader, UsbDeviceReaderImpl};
use crate::relic::network::{NetworkAdapterReader, NetworkAdapterReaderImpl};
use crate::relic::{FlecsRelics, Relics};
use crate::sorcerer::appraiser::{AppRaiser, AppraiserImpl};
use crate::sorcerer::authmancer::{Authmancer, AuthmancerImpl};
use crate::sorcerer::deploymento::{Deploymento, DeploymentoImpl};
use crate::sorcerer::instancius::{Instancius, InstanciusImpl};
use crate::sorcerer::licenso::{Licenso, LicensoImpl};
use crate::sorcerer::mage_quester::{MageQuester, MageQuesterImpl};
use crate::sorcerer::manifesto::{Manifesto, ManifestoImpl};
use crate::sorcerer::systemus::{Systemus, SystemusImpl};
use crate::sorcerer::{FlecsSorcerers, Sorcerers};
use crate::vault::Vault;
pub use config::*;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};

pub struct World<
    APP: AppRaiser + ?Sized,
    AUTH: Authmancer + ?Sized,
    I: Instancius + ?Sized,
    L: Licenso + ?Sized,
    Q: MageQuester + ?Sized,
    M: Manifesto + ?Sized,
    SYS: Systemus + ?Sized,
    D: Deploymento + ?Sized,
    F: Floxy,
    UDR: UsbDeviceReader,
    NAR: NetworkAdapterReader,
    NDR: NetDeviceReader,
> {
    pub sorcerers: Sorcerers<APP, AUTH, I, L, Q, M, SYS, D>,
    pub enchantments: Enchantments<F>,
    pub relics: Relics<UDR, NAR, NDR>,
    pub vault: Arc<Vault>,
    pub server: ServerHandle,
}

pub type FlecsWorld = World<
    AppraiserImpl,
    AuthmancerImpl,
    InstanciusImpl,
    LicensoImpl,
    MageQuesterImpl,
    ManifestoImpl,
    SystemusImpl,
    DeploymentoImpl,
    FloxyImpl,
    UsbDeviceReaderImpl,
    NetworkAdapterReaderImpl,
    NetDeviceReaderImpl,
>;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum CreateError {
    #[error("Failed to start floxy: {0}.")]
    FloxyStartup(String),
    #[error("Failed to create floxy: {0}.")]
    FloxyCreation(String),
    #[error("Failed to spin world up: {0}.")]
    SpinUp(String),
}

impl FlecsWorld {
    pub async fn create_default() -> Result<Self, CreateError> {
        Self::create_from_config(Config::default()).await
    }

    pub async fn create_from_config(config: Config) -> Result<Self, CreateError> {
        Self::create(
            FlecsSorcerers::default(),
            Enchantments {
                floxy: Arc::new(
                    FloxyImpl::from_config(config.floxy_base_path, config.floxy_config_path)
                        .map_err(|e| CreateError::FloxyCreation(e.to_string()))?,
                ),
                quest_master: QuestMaster::default(),
            },
            FlecsRelics::default(),
            Arc::new(Vault::new(config.vault_config)),
            config.socket_path,
        )
        .await
    }
}

impl<
        APP: AppRaiser + 'static,
        AUTH: Authmancer + 'static,
        I: Instancius + 'static,
        L: Licenso + 'static,
        Q: MageQuester + 'static,
        M: Manifesto + 'static,
        SYS: Systemus + 'static,
        D: Deploymento + 'static,
        F: Floxy + 'static,
        UDR: UsbDeviceReader,
        NAR: NetworkAdapterReader,
        NDR: NetDeviceReader,
    > World<APP, AUTH, I, L, Q, M, SYS, D, F, UDR, NAR, NDR>
{
    pub async fn halt(self) {
        self.server.shutdown().await;
        let instancius = self.sorcerers.instancius;
        let vault = self.vault;
        let floxy = self.enchantments.floxy.clone();
        match self
            .enchantments
            .quest_master
            .lock()
            .await
            .shutdown_with(|quest| async move {
                instancius
                    .halt_all_instances(quest, vault, FloxyOperation::new_arc(floxy))
                    .await?;
                Ok(QuestResult::None)
            })
            .await
        {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => error!("Failed to halt all instances: {e}"),
            Err(e) => error!("Failed to shutdown QuestMaster: {e}"),
        }
        match self.enchantments.floxy.stop() {
            Ok(_) => info!("Floxy was stopped"),
            Err(e) => error!("Failed to stop floxy: {e}"),
        }
    }

    async fn spin_up(&self) -> crate::Result<()> {
        let instancius = self.sorcerers.instancius.clone();
        let floxy = FloxyOperation::new_arc(self.enchantments.floxy.clone());
        let vault = self.vault.clone();
        self.enchantments
            .quest_master
            .lock()
            .await
            .schedule_quest("Flecs startup sequence".to_string(), |quest| async move {
                instancius
                    .start_all_instances_as_desired(quest, vault, floxy)
                    .await
            })
            .await?;
        Ok(())
    }

    pub async fn create(
        sorcerers: Sorcerers<APP, AUTH, I, L, Q, M, SYS, D>,
        enchantments: Enchantments<F>,
        relics: Relics<UDR, NAR, NDR>,
        vault: Arc<Vault>,
        socket_path: PathBuf,
    ) -> Result<Self, CreateError> {
        enchantments
            .floxy
            .start()
            .map_err(|e| CreateError::FloxyStartup(e.to_string()))?;
        vault.open().await;
        let world = Self {
            server: crate::fsm::spawn_server(
                sorcerers.clone(),
                socket_path,
                enchantments.clone(),
                vault.clone(),
            ),
            sorcerers,
            enchantments,
            relics,
            vault,
        };
        world
            .spin_up()
            .await
            .map_err(|e| CreateError::SpinUp(e.to_string()))?;
        Ok(world)
    }
}
