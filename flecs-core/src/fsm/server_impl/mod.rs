mod apps;
mod console;
mod device;
mod instances;
mod jobs;
mod system;
use crate::vault::Vault;
use anyhow::Error;
use axum::async_trait;
use axum::extract::Host;
use axum_extra::extract::CookieJar;
use flecs_console_client::models::SessionId;
use flecsd_axum_server::apis::flunder::{Flunder, FlunderBrowseGetResponse};
use flecsd_axum_server::models::{AdditionalInfo, FlunderBrowseGetQueryParams};
use http::Method;
use std::sync::Arc;

fn additional_info_from_error(error: Error) -> AdditionalInfo {
    AdditionalInfo {
        additional_info: format!("{error:#}"),
    }
}

fn ok() -> AdditionalInfo {
    AdditionalInfo {
        additional_info: "OK".to_string(),
    }
}

pub struct ServerImpl {
    vault: Arc<Vault>,
}

impl ServerImpl {
    pub async fn default() -> Self {
        Self {
            vault: crate::lore::vault::default().await,
        }
    }
}

#[async_trait]
impl Flunder for ServerImpl {
    async fn flunder_browse_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        _query_params: FlunderBrowseGetQueryParams,
    ) -> Result<FlunderBrowseGetResponse, ()> {
        todo!()
    }
}

fn console_session_id_to_core_session_id(
    session_id: SessionId,
) -> flecsd_axum_server::models::SessionId {
    flecsd_axum_server::models::SessionId {
        id: session_id.id,
        timestamp: session_id.timestamp,
    }
}

fn invalid_instance_id_additional_info(instance_id: &str) -> AdditionalInfo {
    AdditionalInfo {
        additional_info: format!("Invalid instance_id: {}", instance_id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_instance_id_info() {
        assert_eq!(
            invalid_instance_id_additional_info("test_instance_id"),
            AdditionalInfo {
                additional_info: "Invalid instance_id: test_instance_id".to_string()
            }
        );
    }
}
