pub use super::Result;
use crate::quest::SyncQuest;
use async_trait::async_trait;
use flecs_app_manifest::AppManifest;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub(crate) type AppId = String;
// TODO: Change to a custom general type as soon as the second Deployment implementation is created
pub(crate) type AppInfo = bollard::models::ImageInspect;
#[async_trait]
pub trait AppDeployment {
    async fn install_app(
        &self,
        quest: SyncQuest,
        manifest: Arc<AppManifest>,
        username: String,
        password: String,
    ) -> Result<AppId>;
    async fn uninstall_app(&self, quest: SyncQuest, id: AppId) -> Result<()>;

    async fn is_app_installed(&self, quest: SyncQuest, id: AppId) -> Result<bool> {
        Ok(self.app_info(quest, id).await.is_ok())
    }

    async fn installed_app_size(&self, quest: SyncQuest, id: AppId) -> Result<usize> {
        Ok(self
            .app_info(quest, id)
            .await?
            .size
            .ok_or_else(|| anyhow::anyhow!("Size was not specified"))? as usize)
    }

    async fn app_info(&self, quest: SyncQuest, id: AppId) -> Result<AppInfo>;
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
    use crate::jeweler::deployment::tests::MockedDeployment;
    use crate::quest::Quest;
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

    #[tokio::test]
    async fn is_app_installed_test() {
        let mut mock = MockedDeployment::new();
        mock.expect_app_info()
            .times(1)
            .returning(|_, _| Ok(AppInfo::default()));
        assert!(mock
            .is_app_installed(Quest::new_synced("test".to_string()), "test".to_string())
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn is_app_not_installed_test() {
        let mut mock = MockedDeployment::new();
        mock.expect_app_info()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("test")));
        assert!(!mock
            .is_app_installed(Quest::new_synced("test".to_string()), "test".to_string())
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn installed_app_size_err() {
        let mut mock = MockedDeployment::new();
        mock.expect_app_info()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("test")));
        assert!(mock
            .installed_app_size(Quest::new_synced("test".to_string()), "test".to_string())
            .await
            .is_err());
    }

    #[tokio::test]
    async fn installed_app_size_ok() {
        let mut mock = MockedDeployment::new();
        mock.expect_app_info().times(1).returning(|_, _| {
            Ok(AppInfo {
                size: Some(1230),
                ..AppInfo::default()
            })
        });
        assert_eq!(
            mock.installed_app_size(Quest::new_synced("test".to_string()), "test".to_string())
                .await
                .unwrap(),
            1230
        );
    }

    #[tokio::test]
    async fn installed_app_size_unspecified() {
        let mut mock = MockedDeployment::new();
        mock.expect_app_info()
            .times(1)
            .returning(|_, _| Ok(AppInfo::default()));
        assert!(mock
            .installed_app_size(Quest::new_synced("test".to_string()), "test".to_string())
            .await
            .is_err());
    }
}
