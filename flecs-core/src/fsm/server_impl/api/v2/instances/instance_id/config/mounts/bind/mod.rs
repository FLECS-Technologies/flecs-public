use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::manifest::BindMount;
use crate::sorcerer::instancius::Instancius;
use crate::vault::Vault;
pub use flecsd_axum_server::apis::instances::InstancesInstanceIdConfigMountsBindGetResponse as GetResponse;
use flecsd_axum_server::models;
pub use flecsd_axum_server::models::InstancesInstanceIdConfigMountsBindGetPathParams as GetPathParams;
use std::str::FromStr;
use std::sync::Arc;

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .get_instance_config_bind_mounts(vault, instance_id)
        .await
    {
        None => GetResponse::Status404_InstanceNotFound,
        Some(bind_mounts) => GetResponse::Status200_Success(
            bind_mounts
                .into_iter()
                .map(models::BindMount::from)
                .collect(),
        ),
    }
}

impl From<BindMount> for models::BindMount {
    fn from(value: BindMount) -> Self {
        Self {
            container: value.container_path.to_string_lossy().to_string(),
            host: value.host_path.to_string_lossy().to_string(),
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
            .expect_get_instance_config_bind_mounts()
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
            .expect_get_instance_config_bind_mounts()
            .once()
            .with(predicate::always(), predicate::eq(INSTANCE_ID))
            .returning(|_, _| {
                Some(vec![
                    BindMount {
                        host_path: PathBuf::from("/etc/config"),
                        container_path: PathBuf::from("/etc/config"),
                    },
                    BindMount {
                        host_path: PathBuf::from("/log/app-logs"),
                        container_path: PathBuf::from("/etc/log"),
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
                models::BindMount {
                    host: "/etc/config".to_string(),
                    container: "/etc/config".to_string()
                },
                models::BindMount {
                    host: "/log/app-logs".to_string(),
                    container: "/etc/log".to_string(),
                }
            ])
        );
    }
}
