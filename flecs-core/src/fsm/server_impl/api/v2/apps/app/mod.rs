use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::enchantment::quest_master::QuestMaster;
use crate::sorcerer::appraiser::AppRaiser;
use crate::vault::Vault;
use crate::vault::pouch::{AppKey, Pouch};
use flecsd_axum_server::apis::apps::{
    AppsAppDeleteResponse as DeleteResponse, AppsAppDeleteResponse,
    AppsAppGetResponse as GetResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    AppsAppDeletePathParams as DeletePathParams, AppsAppDeleteQueryParams as DeleteQueryParams,
    AppsAppGetPathParams as GetPathParams, AppsAppGetQueryParams as GetQueryParams,
};
use std::sync::Arc;

pub async fn get<A: AppRaiser>(
    vault: Arc<Vault>,
    appraiser: Arc<A>,
    path_params: GetPathParams,
    query_params: GetQueryParams,
) -> Result<GetResponse, ()> {
    let apps = appraiser
        .get_app(vault, path_params.app, query_params.version)
        .await
        // TODO: Add 500 Response to API
        .map_err(|_| ())?;
    if apps.is_empty() {
        Ok(GetResponse::Status404_NoSuchAppOrApp)
    } else {
        Ok(GetResponse::Status200_Success(apps))
    }
}

pub async fn delete<A: AppRaiser + 'static, F: Floxy + 'static>(
    vault: Arc<Vault>,
    floxy: Arc<F>,
    appraiser: Arc<A>,
    quest_master: QuestMaster,
    path_params: DeletePathParams,
    query_params: DeleteQueryParams,
) -> Result<DeleteResponse, ()> {
    match query_params.version {
        Some(app_version) => {
            let key = AppKey {
                name: path_params.app,
                version: app_version,
            };
            if !vault
                .reservation()
                .reserve_app_pouch()
                .grab()
                .await
                .app_pouch
                .as_ref()
                .expect("Vault reservations should never fail")
                .gems()
                .contains_key(&key)
            {
                return Ok(AppsAppDeleteResponse::Status404_NoSuchAppOrApp);
            }
            let vault = vault.clone();
            let floxy = FloxyOperation::new_arc(floxy);
            let (id, _) = quest_master
                .lock()
                .await
                .schedule_quest(format!("Uninstall {key}"), move |quest| async move {
                    appraiser.uninstall_app(quest, vault, floxy, key).await
                })
                .await
                // TODO: Add 500 Response to API
                .map_err(|_| ())?;
            Ok(AppsAppDeleteResponse::Status202_Accepted(
                models::JobMeta::new(id.0 as i32),
            ))
        }
        // TODO: Add 400 Response to API
        None => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enchantment::floxy::MockFloxy;
    use crate::sorcerer::appraiser::MockAppRaiser;
    use crate::vault::tests::create_empty_test_vault;
    use std::sync::Arc;

    #[tokio::test]
    async fn delete_no_version() {
        let vault = create_empty_test_vault();
        let appraiser = Arc::new(MockAppRaiser::default());
        let floxy = Arc::new(MockFloxy::default());
        assert!(
            delete(
                vault,
                floxy,
                appraiser,
                QuestMaster::default(),
                DeletePathParams {
                    app: "app".to_string(),
                },
                DeleteQueryParams { version: None },
            )
            .await
            .is_err()
        )
    }

    #[tokio::test]
    async fn delete_404() {
        let vault = create_empty_test_vault();
        let appraiser = Arc::new(MockAppRaiser::default());
        let floxy = Arc::new(MockFloxy::default());
        assert_eq!(
            Ok(AppsAppDeleteResponse::Status404_NoSuchAppOrApp),
            delete(
                vault,
                floxy,
                appraiser,
                QuestMaster::default(),
                DeletePathParams {
                    app: "app".to_string(),
                },
                DeleteQueryParams {
                    version: Some("version".to_string())
                },
            )
            .await
        )
    }
}
