use crate::jeweler::deployment::CommonDeployment;
use crate::jeweler::volume::VolumeId;
use crate::quest::SyncQuest;
use futures_util::future::join_all;
use std::sync::Arc;

pub async fn delete_volumes<T: CommonDeployment + 'static + ?Sized>(
    quest: SyncQuest,
    deployment: Arc<T>,
    volume_ids: Vec<VolumeId>,
) -> anyhow::Result<()> {
    let mut results = Vec::new();
    for volume_id in volume_ids {
        let result = quest
            .lock()
            .await
            .create_sub_quest(format!("Delete volume {volume_id}"), |quest| {
                let deployment = deployment.clone();
                async move { deployment.delete_volume(quest, volume_id).await }
            })
            .await
            .2;
        results.push(result)
    }
    join_all(results).await.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::gem::deployment::docker::tests::MockedDockerDeployment;
    use crate::quest::Quest;
    use std::sync::Arc;

    #[tokio::test]
    async fn delete_volumes_ok() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_delete_volume()
            .times(3)
            .returning(|_, _| Ok(()));
        let deployment = Arc::new(deployment);
        delete_volumes(
            Quest::new_synced("TestQuest".to_string()),
            deployment,
            vec![
                "TestVolumeId1".to_string(),
                "TestVolumeId2".to_string(),
                "TestVolumeId3".to_string(),
            ],
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn delete_volumes_err() {
        let mut deployment = MockedDockerDeployment::new();
        deployment
            .expect_delete_volume()
            .withf(|_, id| id == "TestVolumeId1")
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        deployment
            .expect_delete_volume()
            .times(2)
            .returning(|_, _| Ok(()));
        let deployment = Arc::new(deployment);
        assert!(delete_volumes(
            Quest::new_synced("TestQuest".to_string()),
            deployment,
            vec![
                "TestVolumeId1".to_string(),
                "TestVolumeId2".to_string(),
                "TestVolumeId3".to_string(),
            ],
        )
        .await
        .is_err())
    }
}
