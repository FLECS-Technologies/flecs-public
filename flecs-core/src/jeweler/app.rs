pub use super::Result;
use crate::jeweler::gem::manifest::AppManifest;
use crate::quest::SyncQuest;
use async_trait::async_trait;
use flecs_console_client::models::{
    PostApiV2Tokens200ResponseData, PostApiV2Tokens200ResponseDataToken,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub(crate) type AppId = String;
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token {
    pub username: String,
    pub password: String,
}

impl From<PostApiV2Tokens200ResponseDataToken> for Token {
    fn from(value: PostApiV2Tokens200ResponseDataToken) -> Self {
        Self {
            username: value.username,
            password: value.password,
        }
    }
}
impl From<PostApiV2Tokens200ResponseData> for Token {
    fn from(value: PostApiV2Tokens200ResponseData) -> Self {
        (*value.token).into()
    }
}

#[async_trait]
pub trait AppDeployment {
    async fn install_app(
        &self,
        quest: SyncQuest,
        manifest: AppManifest,
        token: Option<Token>,
    ) -> Result<AppId>;
    async fn uninstall_app(&self, quest: SyncQuest, manifest: AppManifest, id: AppId)
    -> Result<()>;

    async fn is_app_installed(
        &self,
        quest: SyncQuest,
        manifest: AppManifest,
        id: AppId,
    ) -> Result<bool>;

    async fn installed_app_size(
        &self,
        quest: SyncQuest,
        manifest: AppManifest,
        id: AppId,
    ) -> Result<usize>;

    async fn export_app(
        &self,
        quest: SyncQuest,
        manifest: AppManifest,
        path: PathBuf,
    ) -> Result<()>;

    async fn import_app(
        &self,
        quest: SyncQuest,
        manifest: AppManifest,
        path: PathBuf,
    ) -> Result<()>;
}

#[derive(Debug, Default, Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
pub enum AppStatus {
    #[default]
    None,
    Installed,
    NotInstalled,
}

impl From<AppStatus> for flecsd_axum_server::models::AppStatus {
    fn from(value: AppStatus) -> Self {
        match value {
            AppStatus::None => flecsd_axum_server::models::AppStatus::Unknown,
            AppStatus::Installed => flecsd_axum_server::models::AppStatus::Installed,
            AppStatus::NotInstalled => flecsd_axum_server::models::AppStatus::NotInstalled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::test_case;

    #[test_case(AppStatus::None, flecsd_axum_server::models::AppStatus::Unknown)]
    #[test_case(AppStatus::Installed, flecsd_axum_server::models::AppStatus::Installed)]
    #[test_case(
        AppStatus::NotInstalled,
        flecsd_axum_server::models::AppStatus::NotInstalled
    )]
    fn test_app_status_from(input: AppStatus, output: flecsd_axum_server::models::AppStatus) {
        assert_eq!(output, input.into());
    }
}
