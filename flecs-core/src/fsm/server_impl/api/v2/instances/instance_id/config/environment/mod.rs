pub mod variable_name;
use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::manifest::single::EnvironmentVariable;
use crate::sorcerer::instancius::{Instancius, QueryInstanceConfigError};
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::{
    InstancesInstanceIdConfigEnvironmentDeleteResponse as DeleteResponse,
    InstancesInstanceIdConfigEnvironmentGetResponse as GetResponse,
    InstancesInstanceIdConfigEnvironmentPutResponse as PutResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    InstanceEnvironment as PutRequest,
    InstancesInstanceIdConfigEnvironmentDeletePathParams as DeletePathParams,
    InstancesInstanceIdConfigEnvironmentGetPathParams as GetPathParams,
    InstancesInstanceIdConfigEnvironmentPutPathParams as PutPathParams,
};
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;

pub async fn delete<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: DeletePathParams,
) -> DeleteResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .delete_instance_config_environment(vault, instance_id)
        .await
    {
        Err(QueryInstanceConfigError::NotFound(_)) => {
            DeleteResponse::Status404_NoInstanceWithThisInstance
        }
        Err(e @ QueryInstanceConfigError::NotSupported(_)) => {
            DeleteResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(_) => DeleteResponse::Status200_EnvironmentOfInstanceWithThisInstance,
    }
}

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .get_instance_config_environment(vault, instance_id)
        .await
    {
        Err(QueryInstanceConfigError::NotFound(_)) => {
            GetResponse::Status404_NoInstanceWithThisInstance
        }
        Err(e @ QueryInstanceConfigError::NotSupported(_)) => {
            GetResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(environment) => GetResponse::Status200_Success(models::InstanceEnvironment::from(
            environment
                .into_iter()
                .map(models::InstanceEnvironmentVariable::from)
                .collect::<Vec<models::InstanceEnvironmentVariable>>(),
        )),
    }
}

pub async fn put<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: PutPathParams,
    request: PutRequest,
) -> PutResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    let environment: Vec<_> = request.into_iter().map(EnvironmentVariable::from).collect();
    if let Err(errors) = validate_environment_variables(&environment) {
        return PutResponse::Status400_MalformedRequest(models::AdditionalInfo::new(
            errors.join("\n"),
        ));
    };
    match instancius
        .put_instance_config_environment(vault, instance_id, environment)
        .await
    {
        Err(QueryInstanceConfigError::NotFound(_)) => {
            PutResponse::Status404_NoInstanceWithThisInstance
        }
        Err(e @ QueryInstanceConfigError::NotSupported(_)) => {
            PutResponse::Status400_MalformedRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(previous_environment) if previous_environment.is_empty() => {
            PutResponse::Status201_EnvironmentForInstanceWithThisInstanceIdWasCreated
        }
        Ok(_) => PutResponse::Status200_EnvironmentForInstanceWithThisInstanceIdIsSet,
    }
}

fn validate_environment_variables(
    environment_variables: &[EnvironmentVariable],
) -> Result<(), Vec<String>> {
    let mut set = HashSet::new();
    let mut errors = Vec::new();
    for environment_variable in environment_variables {
        if !set.insert(environment_variable.name.as_str()) {
            errors.push(format!(
                "Duplicate environment variable name: {}",
                environment_variable.name
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

impl From<EnvironmentVariable> for models::InstanceEnvironmentVariable {
    fn from(value: EnvironmentVariable) -> Self {
        Self {
            name: value.name,
            value: value.value,
        }
    }
}

impl From<models::InstanceEnvironmentVariable> for EnvironmentVariable {
    fn from(value: models::InstanceEnvironmentVariable) -> Self {
        Self {
            name: value.name,
            value: value.value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorcerer::instancius::MockInstancius;
    use crate::vault::tests::create_empty_test_vault;

    #[tokio::test]
    async fn delete_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_environment()
            .withf(move |_, id| id.value == 0x12341234)
            .once()
            .returning(|_, _| {
                Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                    0x12341234,
                )))
            });
        let vault = create_empty_test_vault();
        assert!(matches!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: "12341234".to_string(),
                },
            )
            .await,
            DeleteResponse::Status404_NoInstanceWithThisInstance
        ));
    }

    #[tokio::test]
    async fn delete_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_delete_instance_config_environment()
            .withf(move |_, id| id.value == 6)
            .once()
            .returning(|_, _| Ok(Vec::new()));
        let vault = create_empty_test_vault();
        assert!(matches!(
            delete(
                vault,
                Arc::new(instancius),
                DeletePathParams {
                    instance_id: "00000006".to_string(),
                },
            )
            .await,
            DeleteResponse::Status200_EnvironmentOfInstanceWithThisInstance
        ));
    }

    #[tokio::test]
    async fn get_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_environment()
            .withf(move |_, id| id.value == 0x12341234)
            .once()
            .returning(|_, _| {
                Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                    0x12341234,
                )))
            });
        let vault = create_empty_test_vault();
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "12341234".to_string(),
                },
            )
            .await,
            GetResponse::Status404_NoInstanceWithThisInstance
        ));
    }

    #[tokio::test]
    async fn get_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_config_environment()
            .withf(move |_, id| id.value == 6)
            .once()
            .returning(|_, _| {
                Ok(vec![
                    EnvironmentVariable {
                        name: "VAR_1".to_string(),
                        value: None,
                    },
                    EnvironmentVariable {
                        name: "VAR_2".to_string(),
                        value: Some("value".to_string()),
                    },
                ])
            });
        let vault = create_empty_test_vault();
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "00000006".to_string(),
                },
            )
            .await,
            GetResponse::Status200_Success(models::InstanceEnvironment::from(vec![
                models::InstanceEnvironmentVariable {
                    name: "VAR_1".to_string(),
                    value: None,
                },
                models::InstanceEnvironmentVariable {
                    name: "VAR_2".to_string(),
                    value: Some("value".to_string()),
                }
            ]))
        );
    }

    #[tokio::test]
    async fn put_400_duplicate_variable_name() {
        let vault = create_empty_test_vault();
        let instancius = MockInstancius::new();
        assert!(matches!(
            put(
                vault,
                Arc::new(instancius),
                PutPathParams {
                    instance_id: "00000001".to_string(),
                },
                PutRequest::from(vec![
                    models::InstanceEnvironmentVariable {
                        name: "VAR_1".to_string(),
                        value: None,
                    },
                    models::InstanceEnvironmentVariable {
                        name: "VAR_1".to_string(),
                        value: Some("value".to_string()),
                    }
                ]),
            )
            .await,
            PutResponse::Status400_MalformedRequest(_)
        ));
    }

    #[tokio::test]
    async fn put_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_environment()
            .withf(move |_, id, envs| id.value == 0x78907890 && envs.is_empty())
            .once()
            .returning(|_, _, _| {
                Err(QueryInstanceConfigError::NotFound(InstanceId::new(
                    0x78907890,
                )))
            });
        let vault = create_empty_test_vault();
        assert!(matches!(
            put(
                vault,
                Arc::new(instancius),
                PutPathParams {
                    instance_id: "78907890".to_string(),
                },
                PutRequest::from(vec![]),
            )
            .await,
            PutResponse::Status404_NoInstanceWithThisInstance
        ));
    }

    #[tokio::test]
    async fn put_201() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_environment()
            .withf(move |_, id, envs| {
                id.value == 1
                    && *envs
                        == vec![
                            EnvironmentVariable {
                                name: "VAR_1".to_string(),
                                value: None,
                            },
                            EnvironmentVariable {
                                name: "VAR_2".to_string(),
                                value: Some("value".to_string()),
                            },
                        ]
            })
            .once()
            .returning(|_, _, _| Ok(Vec::new()));
        let vault = create_empty_test_vault();
        assert!(matches!(
            put(
                vault,
                Arc::new(instancius),
                PutPathParams {
                    instance_id: "00000001".to_string(),
                },
                PutRequest::from(vec![
                    models::InstanceEnvironmentVariable {
                        name: "VAR_1".to_string(),
                        value: None,
                    },
                    models::InstanceEnvironmentVariable {
                        name: "VAR_2".to_string(),
                        value: Some("value".to_string()),
                    }
                ]),
            )
            .await,
            PutResponse::Status201_EnvironmentForInstanceWithThisInstanceIdWasCreated
        ));
    }

    #[tokio::test]
    async fn put_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_put_instance_config_environment()
            .withf(move |_, id, envs| {
                id.value == 6
                    && *envs
                        == vec![
                            EnvironmentVariable {
                                name: "VAR_10".to_string(),
                                value: None,
                            },
                            EnvironmentVariable {
                                name: "VAR_20".to_string(),
                                value: Some("value".to_string()),
                            },
                        ]
            })
            .once()
            .returning(|_, _, _| {
                Ok(vec![EnvironmentVariable {
                    name: "previous_var".to_string(),
                    value: None,
                }])
            });
        let vault = create_empty_test_vault();
        assert!(matches!(
            put(
                vault,
                Arc::new(instancius),
                PutPathParams {
                    instance_id: "00000006".to_string(),
                },
                PutRequest::from(vec![
                    models::InstanceEnvironmentVariable {
                        name: "VAR_10".to_string(),
                        value: None,
                    },
                    models::InstanceEnvironmentVariable {
                        name: "VAR_20".to_string(),
                        value: Some("value".to_string()),
                    }
                ]),
            )
            .await,
            PutResponse::Status200_EnvironmentForInstanceWithThisInstanceIdIsSet
        ));
    }

    #[test]
    fn validate_environment_variables_empty() {
        assert!(validate_environment_variables(&[]).is_ok());
    }

    #[test]
    fn validate_environment_variables_ok() {
        assert!(validate_environment_variables(&[
            EnvironmentVariable {
                name: "Variable1".to_string(),
                value: None,
            },
            EnvironmentVariable {
                name: "Variable2".to_string(),
                value: Some("Value".to_string()),
            },
            EnvironmentVariable {
                name: "TEST_VAR".to_string(),
                value: None,
            }
        ])
        .is_ok());
    }

    #[test]
    fn validate_environment_variables_err_single() {
        let errors = validate_environment_variables(&[
            EnvironmentVariable {
                name: "Variable1".to_string(),
                value: None,
            },
            EnvironmentVariable {
                name: "TEST_VAR".to_string(),
                value: Some("Value".to_string()),
            },
            EnvironmentVariable {
                name: "Variable2".to_string(),
                value: Some("Value".to_string()),
            },
            EnvironmentVariable {
                name: "TEST_VAR".to_string(),
                value: None,
            },
        ])
        .err()
        .unwrap();
        assert_eq!(errors.len(), 1, "{errors:?}");
    }

    #[test]
    fn validate_environment_variables_err_multiple() {
        let errors = validate_environment_variables(&[
            EnvironmentVariable {
                name: "Variable1".to_string(),
                value: None,
            },
            EnvironmentVariable {
                name: "TEST_VAR".to_string(),
                value: None,
            },
            EnvironmentVariable {
                name: "TEST_VAR".to_string(),
                value: Some("Value".to_string()),
            },
            EnvironmentVariable {
                name: "Variable2".to_string(),
                value: Some("Value".to_string()),
            },
            EnvironmentVariable {
                name: "TEST_VAR".to_string(),
                value: None,
            },
            EnvironmentVariable {
                name: "Variable1".to_string(),
                value: Some("Value".to_string()),
            },
        ])
        .err()
        .unwrap();
        assert_eq!(errors.len(), 3, "{errors:?}");
    }

    #[test]
    fn from_environment_variable() {
        let name = "TEST_VAR".to_string();
        let value = Some("test-value".to_string());
        assert_eq!(
            models::InstanceEnvironmentVariable::from(EnvironmentVariable {
                name: name.clone(),
                value: value.clone()
            }),
            models::InstanceEnvironmentVariable { name, value }
        );
    }

    #[test]
    fn from_instance_environment_variable() {
        let name = "TEST_VAR".to_string();
        let value = Some("test-value".to_string());
        assert_eq!(
            EnvironmentVariable::from(models::InstanceEnvironmentVariable {
                name: name.clone(),
                value: value.clone()
            }),
            EnvironmentVariable { name, value }
        );
    }
}
