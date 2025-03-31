pub mod bind;
pub mod volumes;

use crate::forge::vec::VecExtension;
use crate::jeweler::gem::instance::InstanceId;
use crate::sorcerer::instancius::Instancius;
use crate::vault::Vault;
pub use flecsd_axum_server::apis::instances::InstancesInstanceIdConfigMountsGetResponse as GetResponse;
use flecsd_axum_server::models;
pub use flecsd_axum_server::models::InstancesInstanceIdConfigMountsGetPathParams as GetPathParams;
use std::str::FromStr;
use std::sync::Arc;

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .get_instance_config_mounts(vault, instance_id)
        .await
    {
        None => GetResponse::Status404_InstanceNotFound,
        Some((volumes, bind_mounts)) => GetResponse::Status200_Success(models::Mounts {
            volume_mounts: volumes
                .into_iter()
                .map(models::InstanceDetailVolume::from)
                .collect::<Vec<_>>()
                .empty_to_none(),
            bind_mounts: bind_mounts
                .into_iter()
                .map(models::BindMount::from)
                .collect::<Vec<_>>()
                .empty_to_none(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jeweler::gem::manifest::{BindMount, VolumeMount};
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
            .expect_get_instance_config_mounts()
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
            .expect_get_instance_config_mounts()
            .once()
            .with(predicate::always(), predicate::eq(INSTANCE_ID))
            .returning(|_, _| {
                Some((
                    vec![
                        VolumeMount {
                            name: "volume-1".to_string(),
                            container_path: PathBuf::from("/config/v1"),
                        },
                        VolumeMount {
                            name: "volume-2".to_string(),
                            container_path: PathBuf::from("/data/v2"),
                        },
                    ],
                    vec![
                        BindMount {
                            host_path: PathBuf::from("/etc/config"),
                            container_path: PathBuf::from("/etc/config"),
                        },
                        BindMount {
                            host_path: PathBuf::from("/log/app-logs"),
                            container_path: PathBuf::from("/etc/log"),
                        },
                    ],
                ))
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
            GetResponse::Status200_Success(models::Mounts {
                volume_mounts: Some(vec![
                    models::InstanceDetailVolume {
                        name: "volume-1".to_string(),
                        path: "/config/v1".to_string()
                    },
                    models::InstanceDetailVolume {
                        name: "volume-2".to_string(),
                        path: "/data/v2".to_string()
                    }
                ]),
                bind_mounts: Some(vec![
                    models::BindMount {
                        host: "/etc/config".to_string(),
                        container: "/etc/config".to_string()
                    },
                    models::BindMount {
                        host: "/log/app-logs".to_string(),
                        container: "/etc/log".to_string(),
                    },
                ])
            })
        );
    }
}
