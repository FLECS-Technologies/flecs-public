use crate::ffi::Token;
use crate::get_server;
use flecs_core::Result;
use flecs_core::sorcerer::authmancer::Authmancer;

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
    let authmancer = server.sorcerers().authmancer.clone();
    let vault = server.vault().clone();
    let console_client = server.console_client().clone();
    let token = server.runtime.block_on(async {
        authmancer
            .acquire_download_token(console_client, &vault, app, version)
            .await
    })?;
    Ok(token.map(|token| token.into()).unwrap_or_default())
}
