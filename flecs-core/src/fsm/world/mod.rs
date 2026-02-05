use crate::enchantment::Enchantments;
use crate::enchantment::quest_master::QuestMaster;
use crate::fsm::ServerHandle;
use crate::legacy::MigrateError;
use crate::lore::Lore;
use crate::quest::{QuestResult, SyncQuest};
use crate::relic::device::usb::{UsbDeviceReader, UsbDeviceReaderImpl};
use crate::relic::floxy::Floxy;
use crate::relic::var::EnvReader;
use crate::relic::{FlecsRelics, Relics};
use crate::sorcerer::appraiser::{AppRaiser, AppraiserImpl};
use crate::sorcerer::authmancer::{Authmancer, AuthmancerImpl};
use crate::sorcerer::deploymento::{Deploymento, DeploymentoImpl};
use crate::sorcerer::exportius::{Exportius, ExportiusImpl};
use crate::sorcerer::importius::{Importius, ImportiusImpl};
use crate::sorcerer::instancius::{Instancius, InstanciusImpl};
use crate::sorcerer::licenso::{Licenso, LicensoImpl};
use crate::sorcerer::mage_quester::{MageQuester, MageQuesterImpl};
use crate::sorcerer::manifesto::{Manifesto, ManifestoImpl};
#[cfg(feature = "auth")]
use crate::sorcerer::providius::Providius;
use crate::sorcerer::systemus::{Systemus, SystemusImpl};
use crate::sorcerer::{FlecsSorcerers, Sorcerers};
use crate::vault::Vault;
#[cfg(feature = "auth")]
use crate::wall::{Wall, enforcer::Enforcer, watch, watch::Watch};
use crate::{legacy, lore};
use net_spider::net_device::{NetDeviceReader, NetDeviceReaderImpl};
use net_spider::network_adapter::{NetworkAdapterReader, NetworkAdapterReaderImpl};
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct World<
    APP: AppRaiser + ?Sized,
    AUTH: Authmancer + ?Sized,
    I: Instancius + ?Sized,
    L: Licenso + ?Sized,
    Q: MageQuester + ?Sized,
    M: Manifesto + ?Sized,
    SYS: Systemus + ?Sized,
    D: Deploymento + ?Sized,
    E: Exportius + ?Sized,
    IMP: Importius + ?Sized,
    UDR: UsbDeviceReader,
    NAR: NetworkAdapterReader,
    NDR: NetDeviceReader,
> {
    pub sorcerers: Sorcerers<APP, AUTH, I, L, Q, M, SYS, D, E, IMP>,
    pub enchantments: Enchantments,
    pub relics: Relics<UDR, NAR, NDR>,
    pub vault: Arc<Vault>,
    pub server: ServerHandle,
    #[cfg(feature = "auth")]
    pub wall: Wall,
    pub lore: Arc<Lore>,
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
    ExportiusImpl,
    ImportiusImpl,
    UsbDeviceReaderImpl,
    NetworkAdapterReaderImpl,
    NetDeviceReaderImpl,
>;

#[derive(thiserror::Error, Debug)]
pub enum CreateError {
    #[error("Failed to start floxy: {0}.")]
    FloxyStartup(String),
    #[error("Failed to create floxy: {0}.")]
    FloxyCreation(String),
    #[error("Failed to spin world up: {0}.")]
    SpinUp(String),
    #[error("Migration failed: {0}")]
    Migration(#[from] MigrateError),
    #[error("Failed to load config: {0}")]
    Lore(#[from] lore::Error),
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[cfg(feature = "auth")]
    #[error(transparent)]
    Casbin(#[from] casbin::Error),
    #[cfg(feature = "auth")]
    #[error(transparent)]
    Watch(#[from] watch::Error),
}

impl FlecsWorld {
    pub async fn migration_necessary() -> bool {
        matches!(
            (
                tokio::fs::try_exists(
                    Path::new(legacy::LEGACY_DEPLOYMENT_PATH).join("compose.json")
                )
                .await,
                &&tokio::fs::try_exists(
                    Path::new(legacy::LEGACY_DEPLOYMENT_PATH).join("docker.json")
                )
                .await
            ),
            (Ok(true), _) | (_, Ok(true))
        )
    }

    async fn migration_backup(base_path: &Path) -> Result<(), MigrateError> {
        let backup_path = base_path.join("migration").join("3.x");
        info!("Creating backup at {}", backup_path.to_string_lossy());
        let deployment_path = backup_path.join("deployment");
        tokio::fs::create_dir_all(&deployment_path).await?;
        if let Err(e) = tokio::fs::copy(
            Path::new(legacy::LEGACY_DEPLOYMENT_PATH).join("docker.json"),
            deployment_path.join("docker.json"),
        )
        .await
        {
            warn!("Failed to backup docker.json: {e}")
        };
        if let Err(e) = tokio::fs::copy(
            Path::new(legacy::LEGACY_DEPLOYMENT_PATH).join("compose.json"),
            deployment_path.join("compose.json"),
        )
        .await
        {
            warn!("Failed to backup compose.json: {e}")
        };
        let apps_path = backup_path.join("apps");
        tokio::fs::create_dir_all(&apps_path).await?;
        if let Err(e) = tokio::fs::copy(legacy::LEGACY_APPS_PATH, apps_path.join("apps.json")).await
        {
            warn!("Failed to backup apps.json: {e}")
        };
        Ok(())
    }

    async fn delete_legacy_files() {
        if let Err(e) =
            tokio::fs::remove_file(Path::new(legacy::LEGACY_DEPLOYMENT_PATH).join("docker.json"))
                .await
        {
            warn!("Failed to remove docker.json: {e}")
        };
        if let Err(e) =
            tokio::fs::remove_file(Path::new(legacy::LEGACY_DEPLOYMENT_PATH).join("compose.json"))
                .await
        {
            warn!("Failed to remove compose.json: {e}")
        };
    }

    pub async fn migrate(lore: Arc<Lore>) -> Result<Self, CreateError> {
        info!("Migrating from 3.x to {}", lore::CORE_VERSION);
        Self::migration_backup(&lore.base_path).await?;
        let legacy_apps = legacy::read_legacy_apps().await;
        let world = Self::new_from_config(lore.clone()).await?;
        match legacy_apps {
            Err(e) => error!("Failed to migrate apps: {e}"),
            Ok(legacy_apps) => {
                if let Err(e) = legacy::migrate_apps(world.vault.clone(), legacy_apps).await {
                    error!("Failed to migrate apps: {e}")
                }
            }
        }
        if let Err(e) = legacy::migrate_docker_instances(
            world.vault.clone(),
            world.relics.usb_device_reader.as_ref(),
            lore.clone(),
        )
        .await
        {
            error!("Failed to migrate docker instances: {e}")
        }
        if let Err(e) = legacy::migrate_compose_instances(world.vault.clone(), lore).await {
            error!("Failed to migrate compose instances: {e}")
        }
        Self::delete_legacy_files().await;
        info!("Migration from 3.x to {} complete", lore::CORE_VERSION);
        world
            .spin_up()
            .await
            .map_err(|e| CreateError::SpinUp(e.to_string()))?;
        Ok(world)
    }

    pub async fn new_from_config(lore: Arc<Lore>) -> Result<Self, CreateError> {
        Self::new(
            FlecsSorcerers::default(),
            Enchantments {
                quest_master: QuestMaster::default(),
            },
            FlecsRelics::default(),
            Arc::new(Vault::new(lore.clone())),
            lore,
        )
        .await
    }

    pub async fn create_from_config(lore: Arc<Lore>) -> Result<Self, CreateError> {
        Self::create(
            FlecsSorcerers::default(),
            Enchantments {
                quest_master: QuestMaster::default(),
            },
            FlecsRelics::default(),
            Arc::new(Vault::new(lore.clone())),
            lore,
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
    E: Exportius + 'static,
    IMP: Importius + 'static,
    UDR: UsbDeviceReader + 'static,
    NAR: NetworkAdapterReader + 'static,
    NDR: NetDeviceReader + 'static,
> World<APP, AUTH, I, L, Q, M, SYS, D, E, IMP, UDR, NAR, NDR>
{
    pub async fn halt(self) {
        self.server.shutdown().await;
        let instancius = self.sorcerers.instancius;
        let vault = self.vault;
        let floxy = self.relics.floxy.clone();
        match self
            .enchantments
            .quest_master
            .lock()
            .await
            .shutdown_with(|quest| async move {
                instancius.shutdown_instances(quest, vault, floxy).await?;
                Ok(QuestResult::None)
            })
            .await
        {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => error!("Failed to shutdown instances: {e}"),
            Err(e) => error!("Failed to shutdown QuestMaster: {e}"),
        }
    }

    async fn startup_quest(
        quest: SyncQuest,
        vault: Arc<Vault>,
        floxy: Arc<dyn Floxy>,
        instancius: Arc<I>,
        #[cfg(feature = "auth")] providius: Arc<dyn Providius>,
        #[cfg(feature = "auth")] watch: Arc<Watch>,
    ) -> crate::Result<()> {
        #[cfg(feature = "auth")]
        let auth_setup = {
            let vault = vault.clone();
            quest
                .lock()
                .await
                .create_sub_quest("Setup core auth provider", |quest| async move {
                    providius
                        .setup_core_auth_provider(quest, vault, watch)
                        .await
                })
                .await
                .2
        };
        let start = quest
            .lock()
            .await
            .create_sub_quest("Start instances", |quest| async move {
                instancius
                    .start_all_instances_as_desired(quest, vault, floxy)
                    .await
            })
            .await
            .2;
        let start = start.await;
        #[cfg(feature = "auth")]
        auth_setup.await?;
        start
    }
    async fn spin_up(&self) -> crate::Result<()> {
        let instancius = self.sorcerers.instancius.clone();
        #[cfg(feature = "auth")]
        let providius = self.sorcerers.providius.clone();
        let floxy = self.relics.floxy.clone();
        let vault = self.vault.clone();
        #[cfg(feature = "auth")]
        let watch = self.wall.watch.clone();
        self.enchantments
            .quest_master
            .lock()
            .await
            .schedule_quest("Flecs startup sequence".to_string(), |quest| {
                Self::startup_quest(
                    quest,
                    vault,
                    floxy,
                    instancius,
                    #[cfg(feature = "auth")]
                    providius,
                    #[cfg(feature = "auth")]
                    watch,
                )
            })
            .await?;
        Ok(())
    }

    pub async fn create(
        sorcerers: Sorcerers<APP, AUTH, I, L, Q, M, SYS, D, E, IMP>,
        enchantments: Enchantments,
        relics: Relics<UDR, NAR, NDR>,
        vault: Arc<Vault>,
        lore: Arc<Lore>,
    ) -> Result<Self, CreateError> {
        let world = Self::new(sorcerers, enchantments, relics, vault, lore).await?;
        world
            .spin_up()
            .await
            .map_err(|e| CreateError::SpinUp(e.to_string()))?;
        Ok(world)
    }

    pub async fn new(
        sorcerers: Sorcerers<APP, AUTH, I, L, Q, M, SYS, D, E, IMP>,
        enchantments: Enchantments,
        relics: Relics<UDR, NAR, NDR>,
        vault: Arc<Vault>,
        lore: Arc<Lore>,
    ) -> Result<Self, CreateError> {
        vault.open().await;
        #[cfg(feature = "auth")]
        let wall = Self::build_wall(lore.clone()).await?;
        let world = Self {
            server: crate::fsm::spawn_server(
                sorcerers.clone(),
                enchantments.clone(),
                vault.clone(),
                lore.clone(),
                #[cfg(feature = "auth")]
                wall.clone(),
            )
            .await
            .map_err(|e| CreateError::SpinUp(e.to_string()))?,
            sorcerers,
            enchantments,
            relics,
            vault,
            #[cfg(feature = "auth")]
            wall,
            lore,
        };
        Ok(world)
    }

    #[cfg(feature = "auth")]
    pub async fn build_wall(lore: Arc<Lore>) -> Result<Wall, CreateError> {
        let enforcer = Arc::new(Enforcer::new_with_lore(lore.clone()).await?);
        let watch = Arc::new(Watch::new_with_lore(lore).await?);
        Ok(Wall { enforcer, watch })
    }

    pub async fn read_lore() -> Result<Lore, lore::Error> {
        let reader = &EnvReader;
        let config_path = lore::config_path(reader);
        let config_exists = config_path.try_exists().map_err(lore::conf::Error::from)?;
        if config_exists {
            let file_config = lore::conf::FlecsConfig::from_path(&config_path).await?;
            let env_config = lore::conf::FlecsConfig::from_var_reader(reader)?;
            let lore = lore::Lore::from_confs_with_defaults([env_config, file_config])?;
            Ok(lore)
        } else {
            let lore = lore::Lore::from_conf_with_defaults(
                lore::conf::FlecsConfig::from_var_reader(reader)?,
            )?;
            let file = lore::conf::FlecsConfig::from(&lore);
            if let Err(e) = file.to_path(&config_path).await {
                error!("Could not write config to {}: {e}", config_path.display())
            }
            Ok(lore)
        }
    }
}
