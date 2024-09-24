use crate::vault::pouch::Pouch;
use crate::vault::Vault;
use flecs_console_client::models::SessionId;
use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Middleware, Next, Result};
use std::sync::Arc;
use tracing::{debug, error};

struct SessionIdMiddleware {
    vault: Arc<Vault>,
}

impl SessionIdMiddleware {
    pub async fn default() -> Self {
        Self {
            vault: crate::lore::vault::default().await,
        }
    }
}

pub async fn create_default_client_with_middleware() -> ClientWithMiddleware {
    ClientBuilder::new(reqwest::Client::new())
        .with(SessionIdMiddleware::default().await)
        .build()
}

pub fn create_new_client_with_middleware(vault: Arc<Vault>) -> ClientWithMiddleware {
    ClientBuilder::new(reqwest::Client::new())
        .with(SessionIdMiddleware::new(vault))
        .build()
}

impl SessionIdMiddleware {
    fn handle_request(&self, request: &mut Request) {
        debug!("{request:?}");
    }

    async fn handle_response(&self, response: Result<Response>) -> Result<Response> {
        debug!("{response:?}");
        if let Ok(response) = response {
            if let Some(session) = response.headers().get("x-session-id") {
                let session_id: serde_json::Result<SessionId> =
                    serde_json::from_slice(session.as_bytes());
                match session_id {
                    Ok(session_id) => {
                        self.vault
                            .reservation()
                            .reserve_secret_pouch_mut()
                            .grab()
                            .await
                            .secret_pouch_mut
                            .as_mut()
                            .unwrap()
                            .gems_mut()
                            .set_session_id(session_id);
                    }
                    Err(e) => {
                        error!("Error extracting session id: {e}");
                    }
                }
            }
            Ok(response)
        } else {
            response
        }
    }

    pub(crate) fn new(vault: Arc<Vault>) -> Self {
        Self { vault }
    }
}

#[async_trait::async_trait]
impl Middleware for SessionIdMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        self.handle_request(&mut req);
        let res = next.run(req, extensions).await;
        self.handle_response(res).await
    }
}
