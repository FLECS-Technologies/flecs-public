use std::path::PathBuf;
use tracing::info;

const FLECSD_SOCKET: &str = "/run/flecs/flecsd.sock";

#[tokio::main]
async fn main() -> flecs_core::fsm::Result<()> {
    flecs_core::fsm::init_backtracing();
    flecs_core::fsm::init_tracing();
    info!("Starting rust server listening on {FLECSD_SOCKET}");
    flecs_core::fsm::server(PathBuf::from(FLECSD_SOCKET)).await
}
