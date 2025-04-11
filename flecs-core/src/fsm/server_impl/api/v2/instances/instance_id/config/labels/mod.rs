pub mod label_name;
use crate::jeweler::gem::instance::InstanceId;
use crate::jeweler::gem::manifest::single::Label;
use crate::sorcerer::instancius::Instancius;
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::InstancesInstanceIdConfigLabelsGetResponse as GetResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::InstancesInstanceIdConfigLabelsGetPathParams as GetPathParams;
use std::str::FromStr;
use std::sync::Arc;

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match instancius.get_instance_labels(vault, instance_id).await {
        None => GetResponse::Status404_NoInstanceWithThisInstance,
        Some(labels) => GetResponse::Status200_Success(
            labels
                .into_iter()
                .map(models::InstanceLabel::from)
                .collect(),
        ),
    }
}

impl From<Label> for models::InstanceLabel {
    fn from(value: Label) -> Self {
        Self {
            name: value.label,
            value: value.value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorcerer::instancius::MockInstancius;

    #[tokio::test]
    async fn get_404() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_labels()
            .withf(move |_, id| id.value == 0x66229933)
            .once()
            .returning(|_, _| None);
        let vault = crate::vault::tests::create_empty_test_vault();
        assert!(matches!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "66229933".to_string(),
                }
            )
            .await,
            GetResponse::Status404_NoInstanceWithThisInstance
        ));
    }

    #[tokio::test]
    async fn get_200() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_get_instance_labels()
            .withf(move |_, id| id.value == 1)
            .once()
            .returning(|_, _| {
                Some(vec![
                    Label {
                        label: "tech.flecs".to_string(),
                        value: None,
                    },
                    Label {
                        label: "tech.flecs.some-label".to_string(),
                        value: Some("Some custom label value".to_string()),
                    },
                ])
            });
        let vault = crate::vault::tests::create_empty_test_vault();
        assert_eq!(
            get(
                vault,
                Arc::new(instancius),
                GetPathParams {
                    instance_id: "00000001".to_string(),
                }
            )
            .await,
            GetResponse::Status200_Success(vec![
                models::InstanceLabel {
                    name: "tech.flecs".to_string(),
                    value: None,
                },
                models::InstanceLabel {
                    name: "tech.flecs.some-label".to_string(),
                    value: Some("Some custom label value".to_string()),
                }
            ])
        );
    }

    #[test]
    fn from_label() {
        assert_eq!(
            models::InstanceLabel::from(Label {
                label: "org.some".to_string(),
                value: Some("value".to_string()),
            }),
            models::InstanceLabel {
                name: "org.some".to_string(),
                value: Some("value".to_string()),
            }
        )
    }
}
