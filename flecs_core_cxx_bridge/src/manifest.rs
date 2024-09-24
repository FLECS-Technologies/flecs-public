use crate::get_server;
use flecs_core::vault::pouch::AppKey;
use flecs_core::*;

pub fn download_manifest(app: &str, version: &str) -> Result<String> {
    let server = get_server();
    let server = server.lock().unwrap();
    let manifest = server.runtime.block_on(async {
        let vault = lore::vault::default().await;
        sorcerer::manifesto::download_manifest(
            &vault,
            AppKey {
                name: app.into(),
                version: version.into(),
            },
        )
        .await
    })?;
    Ok(serde_json::to_string(&manifest)?)
}
