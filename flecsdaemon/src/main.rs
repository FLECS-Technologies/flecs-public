use flecs_core::enchantment::floxy::{Floxy, FloxyImpl};
use flecs_core::enchantment::Enchantments;
use flecs_core::fsm::StartupError;
use flecs_core::sorcerer::Sorcerers;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

const FLECSD_SOCKET: &str = "/run/flecs/flecsd.sock";

#[tokio::main]
async fn main() -> flecs_core::fsm::Result<()> {
    flecs_core::fsm::init_backtracing();
    flecs_core::fsm::init_tracing();
    info!("Starting enchantments");
    let enchantments = Enchantments {
        floxy: Arc::new(
            FloxyImpl::from_config(
                PathBuf::from("/var/lib/flecs/floxy"),
                PathBuf::from("/etc/nginx/floxy.conf"),
            )
            .unwrap(),
        ),
    };
    enchantments
        .floxy
        .start()
        .map_err(|e| StartupError(e.to_string()))?;

    info!("Starting rust server listening on {FLECSD_SOCKET}");
    flecs_core::fsm::server(
        Sorcerers::default(),
        PathBuf::from(FLECSD_SOCKET),
        enchantments,
    )
    .await
}
