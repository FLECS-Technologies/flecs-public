use crate::ffi::Token;
use crate::get_server;
use flecs_core::Result;
use flecs_core::{lore, sorcerer};

impl From<flecs_core::jeweler::app::Token> for Token {
    fn from(value: flecs_core::jeweler::app::Token) -> Self {
        Self {
            username: value.username,
            password: value.password,
        }
    }
}
pub fn acquire_download_token(app: &str, version: &str) -> Result<Token> {
    let server = get_server();
    let server = server.lock().unwrap();
    let token = server.runtime.block_on(async {
        let vault = lore::vault::default().await;
        let configuration = lore::console_client_config::default().await;
        sorcerer::authmancer::acquire_download_token(configuration, &vault, app, version).await
    })?;
    Ok(token.map(|token| token.into()).unwrap_or_default())
}
