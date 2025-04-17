use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::manifest::single::EnvironmentVariable;
use crate::sorcerer::instancius::{Instancius, QueryInstanceConfigError};
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::{
    InstancesInstanceIdConfigEnvironmentVariableNameDeleteResponse as DeleteResponse,
    InstancesInstanceIdConfigEnvironmentVariableNameGetResponse as GetResponse,
    InstancesInstanceIdConfigEnvironmentVariableNamePutResponse as PutResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    InstancesInstanceIdConfigEnvironmentVariableNameDeletePathParams as DeletePathParams,
    InstancesInstanceIdConfigEnvironmentVariableNameGet200Response as GetResponse200,
    InstancesInstanceIdConfigEnvironmentVariableNameGet200Response as PutRequest,
    InstancesInstanceIdConfigEnvironmentVariableNameGetPathParams as GetPathParams,
    InstancesInstanceIdConfigEnvironmentVariableNamePutPathParams as PutPathParams,
};
use std::str::FromStr;
use std::sync::Arc;

pub async fn delete<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: DeletePathParams,
) -> DeleteResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .delete_instance_config_environment_variable_value(
            vault,
            instance_id,
            path_params.variable_name.clone(),
        )
        .await
    {
        Err(QueryInstanceConfigError::NotFound(_)) => {
            DeleteResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
                additional_info: Some(format!("No instance with id {instance_id}")),
            })
        }
        Err(e @ QueryInstanceConfigError::NotSupported(_)) => {
            DeleteResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(None) => DeleteResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
            additional_info: Some(format!(
                "No environment variable with name {}",
                path_params.variable_name
            )),
        }),
        Ok(Some(_)) => DeleteResponse::Status200_EnvironmentVariableOfInstanceWithThisInstance,
    }
}

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .get_instance_config_environment_variable_value(
            vault,
            instance_id,
            path_params.variable_name.clone(),
        )
        .await
    {
        Err(QueryInstanceConfigError::NotFound(_)) => {
            GetResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
                additional_info: Some(format!("No instance with id {instance_id}")),
            })
        }
        Err(e @ QueryInstanceConfigError::NotSupported(_)) => {
            GetResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(None) => GetResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
            additional_info: Some(format!(
                "No environment variable with name {}",
                path_params.variable_name
            )),
        }),
        Ok(Some(value)) => GetResponse::Status200_Success(GetResponse200 { value }),
    }
}

pub async fn put<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: PutPathParams,
    request: PutRequest,
) -> PutResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .put_instance_config_environment_variable_value(
            vault,
            instance_id,
            EnvironmentVariable {
                name: path_params.variable_name,
                value: request.value,
            },
        )
        .await
    {
        Err(QueryInstanceConfigError::NotFound(_)) => {
            PutResponse::Status404_NoInstanceWithThisInstance
        }
        Err(e @ QueryInstanceConfigError::NotSupported(_)) => {
            PutResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(None) => PutResponse::Status201_EnvironmentForInstanceWithThisInstanceIdWasCreated,
        Ok(Some(_)) => PutResponse::Status200_EnvironmentForInstanceWithThisInstanceIdIsSet,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorcerer::instancius::MockInstancius;

    #[tokio::test]
    async fn get_404_instance() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_environment_variable_value()
            .withf(move |_, id, name| id.value == 0x99887766 && name == "variable_name")
            .once()
            .returning(|_, _, _| {
                Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                    0x99887766,
                )))
            });
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "99887766".to_string(),
                    variable_name: "variable_name".to_string(),
                },
            )
            .await,
            GetResponse::Status404_ResourceNotFound(_)
        ));
    }

    #[tokio::test]
    async fn get_404_variable() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_environment_variable_value()
            .withf(move |_, id, name| id.value == 6 && name == "variable_name")
            .once()
            .returning(|_, _, _| Ok(None));

        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "00000006".to_string(),
                    variable_name: "variable_name".to_string(),
                },
            )
            .await,
            GetResponse::Status404_ResourceNotFound(_)
        ));
    }

    #[tokio::test]
    async fn get_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_environment_variable_value()
            .withf(move |_, id, name| id.value == 6 && name == "VAR_1")
            .once()
            .returning(|_, _, _| Ok(Some(None)));
        instancius
            .expect_get_instance_config_environment_variable_value()
            .withf(move |_, id, name| id.value == 6 && name == "VAR_2")
            .once()
            .returning(|_, _, _| Ok(Some(Some("value".to_string()))));

        let vault = crate::vault::tests::create_empty_test_vault();
        let instancius = Arc::new(instancius);
        assert_eq!(
            get(
                vault.clone(),
                instancius.clone(),
                GetPathParams {
                    instance_id: "00000006".to_string(),
                    variable_name: "VAR_1".to_string(),
                },
            )
            .await,
            GetResponse::Status200_Success(GetResponse200 { value: None })
        );
        assert_eq!(
            get(
                vault,
                instancius,
                GetPathParams {
                    instance_id: "00000006".to_string(),
                    variable_name: "VAR_2".to_string(),
                },
            )
            .await,
            GetResponse::Status200_Success(GetResponse200 {
                value: Some("value".to_string())
            })
        );
    }

    #[tokio::test]
    async fn delete_404_instance() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_environment_variable_value()
            .withf(move |_, id, name| id.value == 0x99887766 && name == "variable_name")
            .once()
            .returning(|_, _, _| {
                Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                    0x99887766,
                )))
            });

        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: "99887766".to_string(),
                    variable_name: "variable_name".to_string(),
                },
            )
            .await,
            DeleteResponse::Status404_ResourceNotFound(_)
        ));
    }

    #[tokio::test]
    async fn delete_404_variable() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_environment_variable_value()
            .withf(move |_, id, name| id.value == 6 && name == "variable_name")
            .once()
            .returning(|_, _, _| Ok(None));

        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: "00000006".to_string(),
                    variable_name: "variable_name".to_string(),
                },
            )
            .await,
            DeleteResponse::Status404_ResourceNotFound(_)
        ));
    }

    #[tokio::test]
    async fn delete_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_environment_variable_value()
            .withf(move |_, id, name| id.value == 6 && name == "VAR_1")
            .once()
            .returning(|_, _, _| {
                Ok(Some(EnvironmentVariable {
                    name: "VAR_1".to_string(),
                    value: Some("value".to_string()),
                }))
            });

        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: "00000006".to_string(),
                    variable_name: "VAR_1".to_string(),
                },
            )
            .await,
            DeleteResponse::Status200_EnvironmentVariableOfInstanceWithThisInstance
        );
    }

    #[tokio::test]
    async fn put_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_environment_variable_value()
            .withf(move |_, id, var| {
                id.value == 0x12341234
                    && *var
                        == EnvironmentVariable {
                            name: "VAR_3".to_string(),
                            value: Some("new value".to_string()),
                        }
            })
            .once()
            .returning(|_, _, _| {
                Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                    0x12341234,
                )))
            });

        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            put(
                vault,
                Arc::new(instancius),
                PutPathParams {
                    instance_id: "12341234".to_string(),
                    variable_name: "VAR_3".to_string(),
                },
                PutRequest {
                    value: Some("new value".to_string())
                }
            )
            .await,
            PutResponse::Status404_NoInstanceWithThisInstance
        );
    }

    #[tokio::test]
    async fn put_201() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_environment_variable_value()
            .withf(move |_, id, var| {
                id.value == 6
                    && *var
                        == EnvironmentVariable {
                            name: "VAR_3".to_string(),
                            value: Some("new value".to_string()),
                        }
            })
            .once()
            .returning(|_, _, _| Ok(None));

        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            put(
                vault,
                Arc::new(instancius),
                PutPathParams {
                    instance_id: "00000006".to_string(),
                    variable_name: "VAR_3".to_string(),
                },
                PutRequest {
                    value: Some("new value".to_string())
                }
            )
            .await,
            PutResponse::Status201_EnvironmentForInstanceWithThisInstanceIdWasCreated
        );
    }

    #[tokio::test]
    async fn put_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_environment_variable_value()
            .withf(move |_, id, var| {
                id.value == 6
                    && *var
                        == EnvironmentVariable {
                            name: "VAR_2".to_string(),
                            value: Some("new value".to_string()),
                        }
            })
            .once()
            .returning(|_, _, _| Ok(Some("previous_value".to_string())));

        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            put(
                vault,
                Arc::new(instancius),
                PutPathParams {
                    instance_id: "00000006".to_string(),
                    variable_name: "VAR_2".to_string(),
                },
                PutRequest {
                    value: Some("new value".to_string())
                }
            )
            .await,
            PutResponse::Status200_EnvironmentForInstanceWithThisInstanceIdIsSet
        );
    }
}
