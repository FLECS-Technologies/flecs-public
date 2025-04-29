use crate::jeweler::gem::instance::InstanceId;
use crate::sorcerer::instancius::Instancius;
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::InstancesInstanceIdLogsGetResponse as GetResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    InstancesInstanceIdLogsGet200Response as GetResponse200,
    InstancesInstanceIdLogsGetPathParams as GetPathParams,
};
use std::str::FromStr;
use std::sync::Arc;

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(path_params.instance_id.as_str()).unwrap();
    if !instancius
        .does_instance_exist(vault.clone(), instance_id)
        .await
    {
        return GetResponse::Status404_NoInstanceWithThisInstance;
    }
    match instancius.get_instance_logs(vault, instance_id).await {
        Err(e) => {
            GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
        Ok(logs) => GetResponse::Status200_Success(GetResponse200 {
            stdout: logs.stdout,
            stderr: logs.stderr,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorcerer::instancius::MockInstancius;

    #[tokio::test]
    async fn logs_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_does_instance_exist()
            .withf(|_, id| id.value == 0x1234)
            .once()
            .returning(|_, _| false);
        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "00001234".to_string(),
                },
            )
            .await,
            GetResponse::Status404_NoInstanceWithThisInstance
        )
    }
}
