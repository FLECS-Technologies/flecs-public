use flecs_core::*;
use std::sync::OnceLock;
use tokio::runtime::Runtime;
pub fn download_manifest(x_session_id: &str, app: &str, version: &str) -> anyhow::Result<String> {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    let runtime = RUNTIME.get_or_init(|| Runtime::new().unwrap());
    Ok(serde_json::to_string(&runtime.block_on(
        spell::download_manifest(
            lore::console_client_config::default(),
            x_session_id,
            app,
            version,
        ),
    )?)?)
}
