use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::enchantment::quest_master::QuestMaster;
use crate::jeweler::gem::instance::InstanceId;
use crate::sorcerer::instancius::Instancius;
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::InstancesInstanceIdStopPostResponse as PostResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::InstancesInstanceIdStopPostPathParams as PostPathParams;
use std::str::FromStr;
use std::sync::Arc;

pub async fn post<I: Instancius + 'static, F: Floxy + 'static>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    floxy: Arc<F>,
    quest_master: QuestMaster,
    path_params: PostPathParams,
) -> Result<PostResponse, ()> {
    // TODO: Add 400 Response to API
    let instance_id = InstanceId::from_str(path_params.instance_id.as_str()).unwrap();
    if !instancius
        .does_instance_exist(vault.clone(), instance_id)
        .await
    {
        return Ok(PostResponse::Status404_NoInstanceWithThisInstance);
    }
    let floxy = FloxyOperation::new_arc(floxy);
    let quest_id = quest_master
        .lock()
        .await
        .schedule_quest(
            format!("Stop instance {instance_id}"),
            move |quest| async move {
                instancius
                    .stop_instance(quest, vault, floxy, instance_id)
                    .await
            },
        )
        .await
        // TODO: Add 500 Response to API
        .map_err(|_| ())?
        .0;
    Ok(PostResponse::Status202_Accepted(models::JobMeta {
        job_id: quest_id.0 as i32,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enchantment::floxy::MockFloxy;
    use crate::sorcerer::instancius::MockInstancius;

    #[tokio::test]
    async fn stop_404() {
        let floxy = MockFloxy::new();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_does_instance_exist()
            .withf(|_, id| id.value == 0x1234)
            .once()
            .returning(|_, _| false);
        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            post(
                vault,
                Arc::new(instancius),
                Arc::new(floxy),
                QuestMaster::default(),
                PostPathParams {
                    instance_id: "00001234".to_string(),
                },
            )
            .await,
            Ok(PostResponse::Status404_NoInstanceWithThisInstance)
        )
    }
}
