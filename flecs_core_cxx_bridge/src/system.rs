use crate::ffi;
use flecsd_axum_server::models::{SystemDistro, SystemInfo, SystemKernel};
pub fn read_system_info() -> ffi::SystemInfo {
    flecs_core::relic::system::info::try_create_system_info()
        .map(|info| info.into())
        .unwrap_or_default()
}

impl From<SystemKernel> for ffi::Kernel {
    fn from(value: SystemKernel) -> Self {
        Self {
            machine: value.machine,
            version: value.version,
            build: value.build,
        }
    }
}

impl From<SystemDistro> for ffi::Distro {
    fn from(value: SystemDistro) -> Self {
        Self {
            version: value.version,
            id: value.id,
            name: value.name,
            codename: value.codename,
        }
    }
}

impl From<SystemInfo> for ffi::SystemInfo {
    fn from(value: SystemInfo) -> Self {
        Self {
            arch: value.arch,
            platform: value.platform,
            kernel: value.kernel.into(),
            distro: value.distro.into(),
        }
    }
}
