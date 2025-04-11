use crate::sorcerer::instancius::{GetInstanceConfigVolumeMountError, Instancius};
use crate::vault::pouch::instance::InstanceId;
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::InstancesInstanceIdConfigMountsVolumesVolumeNameGetResponse as GetResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::InstancesInstanceIdConfigMountsVolumesVolumeNameGetPathParams as GetPathParams;
use std::str::FromStr;
use std::sync::Arc;

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .get_instance_config_volume_mount(vault, instance_id, path_params.volume_name)
        .await
    {
        Err(e @ GetInstanceConfigVolumeMountError::InstanceNotFound(_))
        | Err(e @ GetInstanceConfigVolumeMountError::VolumeMountNotFound(_)) => {
            GetResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
                additional_info: Some(e.to_string()),
            })
        }
        Err(e @ GetInstanceConfigVolumeMountError::NotSupported(_)) => {
            GetResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(volume) => GetResponse::Status200_Success(models::InstanceDetailVolume::from(volume)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::gem::manifest::single::VolumeMount;
    use crate::sorcerer::instancius::MockInstancius;
    use crate::vault::tests::create_empty_test_vault;
    use mockall::predicate;
    use std::path::PathBuf;

    #[tokio::test]
    async fn get_404_instance() {
        const INSTANCE_ID: InstanceId = InstanceId::new(0x200);
        const VOLUME: &str = "volume-1";
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_volume_mount()
            .once()
            .with(
                predicate::always(),
                predicate::eq(INSTANCE_ID),
                predicate::eq(VOLUME.to_string()),
            )
            .returning(|_, _, _| {
                Err(GetInstanceConfigVolumeMountError::InstanceNotFound(
                    INSTANCE_ID,
                ))
            });
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "00000200".to_string(),
                    volume_name: VOLUME.to_string(),
                }
            )
            .await,
            GetResponse::Status404_ResourceNotFound(_)
        ));
    }

    #[tokio::test]
    async fn get_404_volume() {
        const INSTANCE_ID: InstanceId = InstanceId::new(0x200);
        const VOLUME: &str = "unknown-volume-1";
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_volume_mount()
            .once()
            .with(
                predicate::always(),
                predicate::eq(INSTANCE_ID),
                predicate::eq(VOLUME.to_string()),
            )
            .returning(|_, _, _| {
                Err(GetInstanceConfigVolumeMountError::VolumeMountNotFound(
                    VOLUME.to_string(),
                ))
            });
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "00000200".to_string(),
                    volume_name: VOLUME.to_string(),
                }
            )
            .await,
            GetResponse::Status404_ResourceNotFound(_)
        ));
    }

    #[tokio::test]
    async fn get_200() {
        const INSTANCE_ID: InstanceId = InstanceId::new(0x200);
        const VOLUME: &str = "volume-1";
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_volume_mount()
            .once()
            .with(
                predicate::always(),
                predicate::eq(INSTANCE_ID),
                predicate::eq(VOLUME.to_string()),
            )
            .returning(|_, _, _| {
                Ok(VolumeMount {
                    name: "volume-1".to_string(),
                    container_path: PathBuf::from("/config/v1"),
                })
            });
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "00000200".to_string(),
                    volume_name: VOLUME.to_string(),
                }
            )
            .await,
            GetResponse::Status200_Success(models::InstanceDetailVolume {
                name: "volume-1".to_string(),
                path: "/config/v1".to_string(),
            })
        );
    }
}
