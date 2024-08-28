use crate::ffi::Token;
use crate::get_server;
use flecs_core::Result;
use flecs_core::{lore, sorcerer};
pub fn acquire_download_token(app: &str, version: &str) -> Result<Token> {
    let server = get_server();
    let server = server.lock().unwrap();
    let configuration = lore::console_client_config::default();
    let vault = lore::vault::default();
    let data = server
        .runtime
        .block_on(sorcerer::authmancer::acquire_download_token(
            configuration,
            &vault,
            app,
            version,
        ))?;
    Ok(Token {
        username: data.token.username,
        password: data.token.password,
    })
}
