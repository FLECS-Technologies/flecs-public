use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::manifest::VolumeMount;
use crate::sorcerer::instancius::Instancius;
use crate::vault::Vault;
pub use flecsd_axum_server::apis::instances::InstancesInstanceIdConfigMountsVolumesGetResponse as GetResponse;
use flecsd_axum_server::models;
pub use flecsd_axum_server::models::InstancesInstanceIdConfigMountsVolumesGetPathParams as GetPathParams;
use std::str::FromStr;
use std::sync::Arc;

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .get_instance_config_volume_mounts(vault, instance_id)
        .await
    {
        None => GetResponse::Status404_InstanceNotFound,
        Some(volumes) => GetResponse::Status200_Success(
            volumes
                .into_iter()
                .map(models::InstanceDetailVolume::from)
                .collect(),
        ),
    }
}

impl From<VolumeMount> for models::InstanceDetailVolume {
    fn from(value: VolumeMount) -> Self {
        Self {
            name: value.name,
            path: value.container_path.to_string_lossy().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorcerer::instancius::MockInstancius;
    use crate::vault::tests::create_empty_test_vault;
    use mockall::predicate;
    use std::path::PathBuf;

    #[tokio::test]
    async fn get_404() {
        const INSTANCE_ID: InstanceId = InstanceId::new(0x200);
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_volume_mounts()
            .once()
            .with(predicate::always(), predicate::eq(INSTANCE_ID))
            .returning(|_, _| None);
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "00000200".to_string()
                }
            )
            .await,
            GetResponse::Status404_InstanceNotFound
        );
    }

    #[tokio::test]
    async fn get_200() {
        const INSTANCE_ID: InstanceId = InstanceId::new(0x200);
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_volume_mounts()
            .once()
            .with(predicate::always(), predicate::eq(INSTANCE_ID))
            .returning(|_, _| {
                Some(vec![
                    VolumeMount {
                        name: "volume-1".to_string(),
                        container_path: PathBuf::from("/config/v1"),
                    },
                    VolumeMount {
                        name: "volume-2".to_string(),
                        container_path: PathBuf::from("/data/v2"),
                    },
                ])
            });
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "00000200".to_string()
                }
            )
            .await,
            GetResponse::Status200_Success(vec![
                models::InstanceDetailVolume {
                    name: "volume-1".to_string(),
                    path: "/config/v1".to_string()
                },
                models::InstanceDetailVolume {
                    name: "volume-2".to_string(),
                    path: "/data/v2".to_string()
                }
            ])
        );
    }
}
