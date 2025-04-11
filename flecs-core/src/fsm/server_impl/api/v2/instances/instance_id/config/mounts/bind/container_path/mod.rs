use crate::sorcerer::instancius::{GetInstanceConfigBindMountError, Instancius};
use crate::vault::pouch::instance::InstanceId;
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::InstancesInstanceIdConfigMountsBindContainerPathGetResponse as GetResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::InstancesInstanceIdConfigMountsBindContainerPathGetPathParams as GetPathParams;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .get_instance_config_bind_mount(
            vault,
            instance_id,
            PathBuf::from(path_params.container_path),
        )
        .await
    {
        Err(e @ GetInstanceConfigBindMountError::InstanceNotFound(_))
        | Err(e @ GetInstanceConfigBindMountError::BindMountNotFound(_)) => {
            GetResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
                additional_info: Some(e.to_string()),
            })
        }
        Ok(bind_mount) => GetResponse::Status200_Success(models::BindMount::from(bind_mount)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::gem::manifest::single::BindMount;
    use crate::sorcerer::instancius::MockInstancius;
    use crate::vault::tests::create_empty_test_vault;
    use mockall::predicate;
    use std::path::PathBuf;

    #[tokio::test]
    async fn get_404_instance() {
        const INSTANCE_ID: InstanceId = InstanceId::new(0x200);
        const CONTAINER_PATH: &str = "/log";
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_bind_mount()
            .once()
            .with(
                predicate::always(),
                predicate::eq(INSTANCE_ID),
                predicate::eq(PathBuf::from(CONTAINER_PATH)),
            )
            .returning(|_, _, _| {
                Err(GetInstanceConfigBindMountError::InstanceNotFound(
                    INSTANCE_ID,
                ))
            });
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "00000200".to_string(),
                    container_path: CONTAINER_PATH.to_string(),
                }
            )
            .await,
            GetResponse::Status404_ResourceNotFound(_)
        ));
    }

    #[tokio::test]
    async fn get_404_volume() {
        const INSTANCE_ID: InstanceId = InstanceId::new(0x200);
        const CONTAINER_PATH: &str = "/log";
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_bind_mount()
            .once()
            .with(
                predicate::always(),
                predicate::eq(INSTANCE_ID),
                predicate::eq(PathBuf::from(CONTAINER_PATH)),
            )
            .returning(|_, _, _| {
                Err(GetInstanceConfigBindMountError::BindMountNotFound(
                    PathBuf::from(CONTAINER_PATH),
                ))
            });
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "00000200".to_string(),
                    container_path: CONTAINER_PATH.to_string(),
                }
            )
            .await,
            GetResponse::Status404_ResourceNotFound(_)
        ));
    }

    #[tokio::test]
    async fn get_200() {
        const INSTANCE_ID: InstanceId = InstanceId::new(0x200);
        const CONTAINER_PATH: &str = "/log";
        const HOST_PATH: &str = "/log/app";
        let vault = create_empty_test_vault();
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_bind_mount()
            .once()
            .with(
                predicate::always(),
                predicate::eq(INSTANCE_ID),
                predicate::eq(PathBuf::from(CONTAINER_PATH)),
            )
            .returning(|_, _, _| {
                Ok(BindMount {
                    host_path: PathBuf::from(HOST_PATH),
                    container_path: PathBuf::from(CONTAINER_PATH),
                })
            });
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "00000200".to_string(),
                    container_path: CONTAINER_PATH.to_string(),
                }
            )
            .await,
            GetResponse::Status200_Success(models::BindMount {
                host: HOST_PATH.to_string(),
                container: CONTAINER_PATH.to_string(),
            })
        );
    }
}
