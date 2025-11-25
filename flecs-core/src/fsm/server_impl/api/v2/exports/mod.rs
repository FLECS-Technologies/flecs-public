pub mod export_id;
use crate::enchantment::floxy::Floxy;
use crate::enchantment::quest_master::QuestMaster;
use crate::jeweler::gem::instance::InstanceId;
use crate::lore::ExportLoreRef;
use crate::quest::QuestResult;
use crate::sorcerer::exportius::Exportius;
use crate::vault::Vault;
use crate::vault::pouch::AppKey;
use flecsd_axum_server::apis::flecsport::{
    ExportsGetResponse as GetResponse, ExportsPostResponse as PostResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::ExportRequest as PostRequest;
use std::str::FromStr;
use std::sync::Arc;

pub async fn get<E: Exportius>(exportius: Arc<E>, lore: ExportLoreRef) -> GetResponse {
    match exportius.get_exports(lore).await {
        Ok(exports) => GetResponse::Status200_Success(
            exports.into_iter().map(models::ExportId::from).collect(),
        ),
        Err(e) => {
            GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

pub async fn post<E: Exportius>(
    vault: Arc<Vault>,
    exportius: Arc<E>,
    floxy: Arc<dyn Floxy>,
    quest_master: QuestMaster,
    lore: ExportLoreRef,
    request: PostRequest,
) -> PostResponse {
    let instance_ids = request
        .instances
        .unwrap_or_default()
        .into_iter()
        .map(|id| InstanceId::from_str(&id).unwrap())
        .collect();
    let apps = request.apps.into_iter().map(AppKey::from).collect();
    match quest_master
        .lock()
        .await
        .schedule_quest_with_result("Create export".to_string(), |quest| async move {
            let id = exportius
                .create_export_archive(quest, vault, floxy, lore, apps, instance_ids)
                .await?;
            Ok(QuestResult::ExportId(id))
        })
        .await
    {
        Err(e) => {
            PostResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
        Ok((id, _)) => PostResponse::Status202_Accepted(models::JobMeta::new(id.0 as i32)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lore;
    use crate::relic::var::test::MockVarReader;
    use crate::sorcerer::exportius::MockExportius;
    use std::io::ErrorKind;
    use testdir::testdir;

    #[tokio::test]
    async fn get_200() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut exportius = MockExportius::new();
        exportius
            .expect_get_exports()
            .once()
            .returning(|_| Ok(vec!["1234".to_string(), "5678".to_string()]));
        assert_eq!(
            get(Arc::new(exportius), lore).await,
            GetResponse::Status200_Success(vec![
                models::ExportId::from("1234".to_string()),
                models::ExportId::from("5678".to_string())
            ])
        );
    }

    #[tokio::test]
    async fn get_500() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let mut exportius = MockExportius::new();
        exportius
            .expect_get_exports()
            .once()
            .returning(|_| Err(std::io::Error::from(ErrorKind::PermissionDenied)));
        assert!(matches!(
            get(Arc::new(exportius), lore).await,
            GetResponse::Status500_InternalServerError(_)
        ));
    }
}
