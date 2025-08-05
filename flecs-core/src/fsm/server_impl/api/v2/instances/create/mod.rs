use crate::enchantment::quest_master::QuestMaster;
use crate::lore::Lore;
use crate::quest::QuestResult;
use crate::sorcerer::appraiser::AppRaiser;
use crate::sorcerer::instancius::Instancius;
use crate::vault::Vault;
use crate::vault::pouch::AppKey;
use flecsd_axum_server::apis::instances::InstancesCreatePostResponse as PostResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::InstancesCreatePostRequest as PostRequest;
use std::sync::Arc;

pub async fn post<A: AppRaiser, I: Instancius + 'static>(
    vault: Arc<Vault>,
    lore: Arc<Lore>,
    appraiser: Arc<A>,
    instancius: Arc<I>,
    quest_master: QuestMaster,
    request: PostRequest,
) -> Result<PostResponse, ()> {
    let app_key: AppKey = request.app_key.into();
    if !appraiser
        .does_app_exist(vault.clone(), app_key.clone())
        .await
    {
        return Ok(PostResponse::Status400_MalformedRequest(
            models::AdditionalInfo::new(format!("App {app_key} does not exist")),
        ));
    }
    let instance_name = request.instance_name;
    let (id, _quest) = quest_master
        .lock()
        .await
        .schedule_quest_with_result(
            format!("Create instance for {app_key}"),
            |quest| async move {
                let id = instancius
                    .create_instance(
                        quest,
                        vault,
                        lore,
                        app_key,
                        instance_name.unwrap_or_default(),
                    )
                    .await?;
                Ok(QuestResult::InstanceId(id))
            },
        )
        .await
        // TODO: Add 500 Response to API
        .map_err(|_| ())?;
    Ok(PostResponse::Status202_Accepted(models::JobMeta::new(
        id.0 as i32,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsm::server_impl::await_quest_completion;
    use crate::jeweler::gem::instance::InstanceId;
    use crate::lore;
    use crate::relic::var::test::MockVarReader;
    use crate::sorcerer::appraiser::MockAppRaiser;
    use crate::sorcerer::instancius::MockInstancius;
    use flecsd_axum_server::models;
    use testdir::testdir;

    #[tokio::test]
    async fn post_400() {
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let instancius = Arc::new(MockInstancius::new());
        let mut appraiser = MockAppRaiser::new();
        appraiser.expect_does_app_exist().once().return_const(false);
        let appraiser = Arc::new(appraiser);
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            post(
                vault,
                lore,
                appraiser,
                instancius,
                QuestMaster::default(),
                PostRequest {
                    app_key: models::AppKey {
                        name: "TestName".to_string(),
                        version: "1.2.3".to_string()
                    },
                    instance_name: None,
                },
            )
            .await,
            Ok(PostResponse::Status400_MalformedRequest(_))
        ))
    }

    #[tokio::test]
    async fn create_instance_ok() {
        let test_key = models::AppKey {
            name: "TestName".to_string(),
            version: "1.2.3".to_string(),
        };
        let lore = Arc::new(lore::test_lore(testdir!(), &MockVarReader::new()));
        let expected_key = test_key.clone();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_create_instance()
            .withf(move |_, _, _, app_key, name| {
                app_key.name == expected_key.name
                    && app_key.version == expected_key.version
                    && name.is_empty()
            })
            .once()
            .returning(|_, _, _, _, _| Ok(InstanceId::new(1)));
        let mut appraiser = MockAppRaiser::new();
        appraiser
            .expect_does_app_exist()
            .once()
            .returning(|_, _| true);
        let quest_master = QuestMaster::default();
        let vault = crate::vault::tests::create_empty_test_vault();
        let result = post(
            vault,
            lore,
            Arc::new(appraiser),
            Arc::new(instancius),
            quest_master.clone(),
            PostRequest {
                app_key: test_key.clone(),
                instance_name: None,
            },
        )
        .await;
        match result {
            Ok(PostResponse::Status202_Accepted(_)) => {
                await_quest_completion(quest_master).await;
            }
            _ => panic!("Expected InstancesCreatePostResponse::Status202_Accepted"),
        }
    }
}
