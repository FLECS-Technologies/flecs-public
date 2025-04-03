pub mod export_id;
use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::enchantment::quest_master::QuestMaster;
use crate::jeweler::gem::instance::InstanceId;
use crate::quest::QuestResult;
use crate::sorcerer::exportius::Exportius;
use crate::vault::pouch::AppKey;
use crate::vault::Vault;
use flecsd_axum_server::apis::flecsport::{
    ExportsGetResponse as GetResponse, ExportsPostResponse as PostResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::ExportRequest as PostRequest;
use std::str::FromStr;
use std::sync::Arc;

pub async fn get<E: Exportius>(exportius: Arc<E>) -> GetResponse {
    match exportius.get_exports().await {
        Ok(exports) => GetResponse::Status200_Success(
            exports.into_iter().map(models::ExportId::from).collect(),
        ),
        Err(e) => {
            GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

pub async fn post<E: Exportius, F: Floxy + 'static>(
    vault: Arc<Vault>,
    exportius: Arc<E>,
    floxy: Arc<F>,
    quest_master: QuestMaster,
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
                .create_export_archive(
                    quest,
                    vault,
                    FloxyOperation::new_arc(floxy),
                    apps,
                    instance_ids,
                )
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
    use crate::sorcerer::exportius::MockExportius;
    use std::io::ErrorKind;

    #[tokio::test]
    async fn get_200() {
        let mut exportius = MockExportius::new();
        exportius
            .expect_get_exports()
            .once()
            .returning(|| Ok(vec!["1234".to_string(), "5678".to_string()]));
        assert_eq!(
            get(Arc::new(exportius)).await,
            GetResponse::Status200_Success(vec![
                models::ExportId::from("1234".to_string()),
                models::ExportId::from("5678".to_string())
            ])
        );
    }

    #[tokio::test]
    async fn get_500() {
        let mut exportius = MockExportius::new();
        exportius
            .expect_get_exports()
            .once()
            .returning(|| Err(std::io::Error::from(ErrorKind::PermissionDenied)));
        assert!(matches!(
            get(Arc::new(exportius)).await,
            GetResponse::Status500_InternalServerError(_)
        ));
    }
}
