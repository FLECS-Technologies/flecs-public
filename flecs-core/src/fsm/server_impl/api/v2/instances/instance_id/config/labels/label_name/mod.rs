use crate::jeweler::gem::instance::InstanceId;
use crate::sorcerer::instancius::Instancius;
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::InstancesInstanceIdConfigLabelsLabelNameGetResponse as GetResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    InstancesInstanceIdConfigLabelsLabelNameGet200Response as GetResponse200,
    InstancesInstanceIdConfigLabelsLabelNameGetPathParams as GetPathParams,
};
use std::str::FromStr;
use std::sync::Arc;

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius
        .get_instance_label_value(vault, instance_id, path_params.label_name.clone())
        .await
    {
        None => GetResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
            additional_info: Some(format!("No instance with id {}", instance_id)),
        }),
        Some(None) => GetResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
            additional_info: Some(format!(
                "No environment label with name {}",
                path_params.label_name
            )),
        }),
        Some(Some(value)) => GetResponse::Status200_Success(GetResponse200 { value }),
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
            .expect_get_instance_label_value()
            .withf(move |_, id, name| id.value == 0x12345678 && name == "flecs.tech")
            .once()
            .returning(|_, _, _| None);
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "12345678".to_string(),
                    label_name: "flecs.tech".to_string(),
                }
            )
            .await,
            GetResponse::Status404_ResourceNotFound(_)
        ));
    }

    #[tokio::test]
    async fn get_404_label() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_label_value()
            .withf(move |_, id, name| id.value == 2 && name == "not.existing.label")
            .once()
            .returning(|_, _, _| Some(None));
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "00000002".to_string(),
                    label_name: "not.existing.label".to_string(),
                }
            )
            .await,
            GetResponse::Status404_ResourceNotFound(_)
        ));
    }

    #[tokio::test]
    async fn get_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_label_value()
            .withf(move |_, id, name| id.value == 2 && name == "tech.flecs")
            .once()
            .returning(|_, _, _| Some(Some(None)));
        instancius
            .expect_get_instance_label_value()
            .withf(move |_, id, name| id.value == 2 && name == "tech.flecs.some-label")
            .once()
            .returning(|_, _, _| Some(Some(Some("Some custom label value".to_string()))));
        let vault = crate::vault::tests::create_empty_test_vault();
        let instancius = Arc::new(instancius);
        assert_eq!(
            get(
                vault.clone(),
                instancius.clone(),
                GetPathParams {
                    instance_id: "00000002".to_string(),
                    label_name: "tech.flecs".to_string(),
                }
            )
            .await,
            GetResponse::Status200_Success(GetResponse200 { value: None })
        );
        assert_eq!(
            get(
                vault,
                instancius,
                GetPathParams {
                    instance_id: "00000002".to_string(),
                    label_name: "tech.flecs.some-label".to_string(),
                }
            )
            .await,
            GetResponse::Status200_Success(GetResponse200 {
                value: Some("Some custom label value".to_string())
            })
        );
    }
}
