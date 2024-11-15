use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tracing::info;

struct Server {
    runtime: Runtime,
    handle: Option<JoinHandle<()>>,
}

fn get_server() -> Arc<Mutex<Server>> {
    static SERVER: OnceLock<Arc<Mutex<Server>>> = OnceLock::new();
    SERVER
        .get_or_init(|| {
            Arc::new(Mutex::new(Server {
                runtime: Runtime::new().unwrap(),
                handle: None,
            }))
        })
        .clone()
}

pub fn start_server() {
    let server = get_server();
    let mut server = server.lock().unwrap();
    assert!(server.handle.is_none());
    flecs_core::fsm::init_backtracing();
    flecs_core::fsm::init_tracing();
    info!("Executing test pull");
    server
        .runtime
        .block_on(flecs_core::relic::docker::image::test_pull());
}

fn main() {
    start_server();
}
