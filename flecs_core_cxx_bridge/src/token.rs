use crate::ffi::Token;
use crate::get_server;
use flecs_core::Result;
use flecs_core::{lore, sorcerer};
pub fn acquire_download_token(app: &str, version: &str) -> Result<Token> {
    let server = get_server();
    let server = server.lock().unwrap();
    let data = server.runtime.block_on(async {
        let vault = lore::vault::default().await;
        let configuration = lore::console_client_config::default().await;
        sorcerer::authmancer::acquire_download_token(configuration, &vault, app, version).await
    })?;
    Ok(Token {
        username: data.token.username,
        password: data.token.password,
    })
}
