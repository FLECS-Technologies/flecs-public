use crate::enchantment::floxy::Floxy;
use crate::fsm::server_impl::ServerImpl;
use crate::relic::device::net::NetDeviceReaderImpl;
use crate::relic::device::usb::UsbDeviceReaderImpl;
use crate::relic::network::NetworkAdapterReaderImpl;
use crate::sorcerer::appraiser::AppRaiser;
use crate::sorcerer::authmancer::Authmancer;
use crate::sorcerer::deploymento::Deploymento;
use crate::sorcerer::exportius::Exportius;
use crate::sorcerer::importius::Importius;
use crate::sorcerer::instancius::Instancius;
use crate::sorcerer::licenso::Licenso;
use crate::sorcerer::mage_quester::MageQuester;
use crate::sorcerer::manifesto::Manifesto;
use crate::sorcerer::providius::Providius;
use crate::sorcerer::systemus::Systemus;
use crate::vault::Vault;
use axum::extract::FromRef;
use std::sync::Arc;

pub struct ProvidiusState(pub Arc<dyn Providius>);

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
    F: Floxy + 'static,
>
    FromRef<
        Arc<
            ServerImpl<
                APP,
                AUTH,
                I,
                L,
                Q,
                M,
                SYS,
                D,
                E,
                IMP,
                F,
                UsbDeviceReaderImpl,
                NetworkAdapterReaderImpl,
                NetDeviceReaderImpl,
            >,
        >,
    > for ProvidiusState
{
    fn from_ref(
        input: &Arc<
            ServerImpl<
                APP,
                AUTH,
                I,
                L,
                Q,
                M,
                SYS,
                D,
                E,
                IMP,
                F,
                UsbDeviceReaderImpl,
                NetworkAdapterReaderImpl,
                NetDeviceReaderImpl,
            >,
        >,
    ) -> Self {
        Self(input.sorcerers.providius.clone())
    }
}

pub struct VaultState(pub Arc<Vault>);

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
    F: Floxy + 'static,
>
    FromRef<
        Arc<
            ServerImpl<
                APP,
                AUTH,
                I,
                L,
                Q,
                M,
                SYS,
                D,
                E,
                IMP,
                F,
                UsbDeviceReaderImpl,
                NetworkAdapterReaderImpl,
                NetDeviceReaderImpl,
            >,
        >,
    > for VaultState
{
    fn from_ref(
        input: &Arc<
            ServerImpl<
                APP,
                AUTH,
                I,
                L,
                Q,
                M,
                SYS,
                D,
                E,
                IMP,
                F,
                UsbDeviceReaderImpl,
                NetworkAdapterReaderImpl,
                NetDeviceReaderImpl,
            >,
        >,
    ) -> Self {
        Self(input.vault.clone())
    }
}

#[cfg(feature = "auth")]
pub struct LoreState(pub Arc<crate::lore::Lore>);

#[cfg(feature = "auth")]
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
    F: Floxy + 'static,
>
    FromRef<
        Arc<
            ServerImpl<
                APP,
                AUTH,
                I,
                L,
                Q,
                M,
                SYS,
                D,
                E,
                IMP,
                F,
                UsbDeviceReaderImpl,
                NetworkAdapterReaderImpl,
                NetDeviceReaderImpl,
            >,
        >,
    > for LoreState
{
    fn from_ref(
        input: &Arc<
            ServerImpl<
                APP,
                AUTH,
                I,
                L,
                Q,
                M,
                SYS,
                D,
                E,
                IMP,
                F,
                UsbDeviceReaderImpl,
                NetworkAdapterReaderImpl,
                NetDeviceReaderImpl,
            >,
        >,
    ) -> Self {
        Self(input.lore.clone())
    }
}

#[cfg(feature = "auth")]
pub struct ImportiusState<I: Importius + 'static>(pub Arc<I>);

#[cfg(feature = "auth")]
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
    F: Floxy + 'static,
>
    FromRef<
        Arc<
            ServerImpl<
                APP,
                AUTH,
                I,
                L,
                Q,
                M,
                SYS,
                D,
                E,
                IMP,
                F,
                UsbDeviceReaderImpl,
                NetworkAdapterReaderImpl,
                NetDeviceReaderImpl,
            >,
        >,
    > for ImportiusState<IMP>
{
    fn from_ref(
        input: &Arc<
            ServerImpl<
                APP,
                AUTH,
                I,
                L,
                Q,
                M,
                SYS,
                D,
                E,
                IMP,
                F,
                UsbDeviceReaderImpl,
                NetworkAdapterReaderImpl,
                NetDeviceReaderImpl,
            >,
        >,
    ) -> Self {
        Self(input.sorcerers.importius.clone())
    }
}

#[cfg(feature = "auth")]
pub struct FloxyState<F: Floxy + 'static>(pub Arc<F>);

#[cfg(feature = "auth")]
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
    F: Floxy + 'static,
>
    FromRef<
        Arc<
            ServerImpl<
                APP,
                AUTH,
                I,
                L,
                Q,
                M,
                SYS,
                D,
                E,
                IMP,
                F,
                UsbDeviceReaderImpl,
                NetworkAdapterReaderImpl,
                NetDeviceReaderImpl,
            >,
        >,
    > for FloxyState<F>
{
    fn from_ref(
        input: &Arc<
            ServerImpl<
                APP,
                AUTH,
                I,
                L,
                Q,
                M,
                SYS,
                D,
                E,
                IMP,
                F,
                UsbDeviceReaderImpl,
                NetworkAdapterReaderImpl,
                NetDeviceReaderImpl,
            >,
        >,
    ) -> Self {
        Self(input.enchantments.floxy.clone())
    }
}

#[cfg(feature = "auth")]
pub struct UsbDeviceReaderState(pub Arc<UsbDeviceReaderImpl>);

#[cfg(feature = "auth")]
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
    F: Floxy + 'static,
>
    FromRef<
        Arc<
            ServerImpl<
                APP,
                AUTH,
                I,
                L,
                Q,
                M,
                SYS,
                D,
                E,
                IMP,
                F,
                UsbDeviceReaderImpl,
                NetworkAdapterReaderImpl,
                NetDeviceReaderImpl,
            >,
        >,
    > for UsbDeviceReaderState
{
    fn from_ref(
        input: &Arc<
            ServerImpl<
                APP,
                AUTH,
                I,
                L,
                Q,
                M,
                SYS,
                D,
                E,
                IMP,
                F,
                UsbDeviceReaderImpl,
                NetworkAdapterReaderImpl,
                NetDeviceReaderImpl,
            >,
        >,
    ) -> Self {
        Self(input.usb_reader.clone())
    }
}

#[cfg(feature = "auth")]
pub struct QuestMasterState(pub crate::enchantment::quest_master::QuestMaster);

#[cfg(feature = "auth")]
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
    F: Floxy + 'static,
>
    FromRef<
        Arc<
            ServerImpl<
                APP,
                AUTH,
                I,
                L,
                Q,
                M,
                SYS,
                D,
                E,
                IMP,
                F,
                UsbDeviceReaderImpl,
                NetworkAdapterReaderImpl,
                NetDeviceReaderImpl,
            >,
        >,
    > for QuestMasterState
{
    fn from_ref(
        input: &Arc<
            ServerImpl<
                APP,
                AUTH,
                I,
                L,
                Q,
                M,
                SYS,
                D,
                E,
                IMP,
                F,
                UsbDeviceReaderImpl,
                NetworkAdapterReaderImpl,
                NetDeviceReaderImpl,
            >,
        >,
    ) -> Self {
        Self(input.enchantments.quest_master.clone())
    }
}
