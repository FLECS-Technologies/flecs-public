mod api;
mod apps;
mod console;
mod deployments;
mod device;
mod flecsport;
mod instances;
mod jobs;
mod system;
use crate::enchantment::floxy::Floxy;
use crate::enchantment::Enchantments;
use crate::fsm::console_client::ConsoleClient;
use crate::relic::device::net::NetDeviceReader;
use crate::relic::device::usb::UsbDeviceReader;
use crate::relic::network::NetworkAdapterReader;
use crate::sorcerer::appraiser::AppRaiser;
use crate::sorcerer::authmancer::Authmancer;
use crate::sorcerer::deploymento::Deploymento;
use crate::sorcerer::exportius::Exportius;
use crate::sorcerer::importius::Importius;
use crate::sorcerer::instancius::Instancius;
use crate::sorcerer::licenso::Licenso;
use crate::sorcerer::mage_quester::MageQuester;
use crate::sorcerer::manifesto::Manifesto;
use crate::sorcerer::systemus::Systemus;
use crate::sorcerer::Sorcerers;
use crate::vault::Vault;
use anyhow::Error;
use axum::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecs_console_client::models::SessionId;
use flecsd_axum_server::apis::flunder::{Flunder, FlunderBrowseGetResponse};
use flecsd_axum_server::models::{AdditionalInfo, FlunderBrowseGetQueryParams};
use http::Method;
use std::sync::Arc;

fn additional_info_from_error(error: Error) -> AdditionalInfo {
    AdditionalInfo {
        additional_info: format!("{error:#}"),
    }
}

fn ok() -> AdditionalInfo {
    AdditionalInfo {
        additional_info: "OK".to_string(),
    }
}

pub struct ServerImpl<
    APP: AppRaiser,
    AUTH: Authmancer,
    I: Instancius,
    L: Licenso,
    Q: MageQuester,
    M: Manifesto,
    SYS: Systemus,
    D: Deploymento,
    E: Exportius,
    IMP: Importius,
    F: Floxy,
    T: UsbDeviceReader,
    NET: NetworkAdapterReader,
    NetDev: NetDeviceReader,
> {
    vault: Arc<Vault>,
    enchantments: Enchantments<F>,
    usb_reader: Arc<T>,
    network_adapter_reader: Arc<NET>,
    net_device_reader: Arc<NetDev>,
    console_client: ConsoleClient,
    sorcerers: Sorcerers<APP, AUTH, I, L, Q, M, SYS, D, E, IMP>,
}

impl<
        APP: AppRaiser,
        AUTH: Authmancer,
        I: Instancius,
        L: Licenso,
        Q: MageQuester,
        M: Manifesto,
        SYS: Systemus,
        D: Deploymento,
        E: Exportius,
        IMP: Importius,
        F: Floxy,
        T: UsbDeviceReader,
        NET: NetworkAdapterReader,
        NetDev: NetDeviceReader,
    > ServerImpl<APP, AUTH, I, L, Q, M, SYS, D, E, IMP, F, T, NET, NetDev>
{
    pub async fn new(
        vault: Arc<Vault>,
        sorcerers: Sorcerers<APP, AUTH, I, L, Q, M, SYS, D, E, IMP>,
        enchantments: Enchantments<F>,
        usb_reader: T,
        network_adapter_reader: NET,
        net_device_reader: NetDev,
    ) -> Self {
        Self {
            console_client: crate::fsm::console_client::create_default(vault.clone()),
            vault,
            enchantments,
            usb_reader: Arc::new(usb_reader),
            net_device_reader: Arc::new(net_device_reader),
            network_adapter_reader: Arc::new(network_adapter_reader),
            sorcerers,
        }
    }
}

#[async_trait]
impl<
        APP: AppRaiser,
        AUTH: Authmancer,
        I: Instancius,
        L: Licenso,
        Q: MageQuester,
        M: Manifesto,
        SYS: Systemus,
        D: Deploymento,
        E: Exportius,
        IMP: Importius + 'static,
        F: Floxy,
        T: UsbDeviceReader,
        NET: NetworkAdapterReader,
        NetDev: NetDeviceReader,
    > Flunder for ServerImpl<APP, AUTH, I, L, Q, M, SYS, D, E, IMP, F, T, NET, NetDev>
{
    async fn flunder_browse_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _query_params: FlunderBrowseGetQueryParams,
    ) -> Result<FlunderBrowseGetResponse, ()> {
        todo!()
    }
}

fn console_session_id_to_core_session_id(
    session_id: SessionId,
) -> flecsd_axum_server::models::SessionId {
    flecsd_axum_server::models::SessionId {
        id: session_id.id,
        timestamp: session_id.timestamp,
    }
}

#[cfg(test)]
async fn await_quest_completion(quest_master: crate::enchantment::quest_master::QuestMaster) {
    let quest = quest_master
        .lock()
        .await
        .schedule_quest("Wait for quests to complete".to_string(), |_quest| async {
            Ok(())
        })
        .await
        .unwrap()
        .1;
    quest
        .lock()
        .await
        .create_infallible_sub_quest(
            "Subquest: Wait for quests to complete".to_string(),
            |_quest| async {},
        )
        .await
        .2
        .await;
}
