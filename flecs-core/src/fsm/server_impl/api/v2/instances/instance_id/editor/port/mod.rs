use crate::enchantment::floxy::Floxy;
use crate::jeweler::gem::instance::InstanceId;
use crate::sorcerer::instancius::Instancius;
use crate::sorcerer::instancius::RedirectEditorRequestResult::*;
use crate::vault::Vault;
use axum::extract::Host;
use flecsd_axum_server::apis::instances::InstancesInstanceIdEditorPortGetResponse as GetResponse;
use flecsd_axum_server::models;
use std::num::NonZeroU16;
use std::sync::Arc;

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    floxy: Arc<dyn Floxy>,
    instancius: Arc<I>,
    host: Host,
    instance_id: InstanceId,
    port: NonZeroU16,
) -> Result<GetResponse, ()> {
    match instancius
        .redirect_editor_request(vault, floxy, instance_id, port)
        .await
    {
        Err(e) => Ok(GetResponse::Status500_InternalServerError(
            models::AdditionalInfo::new(e.to_string()),
        )),
        Ok(Redirected(host_port)) => Ok(GetResponse::Status302_Found {
            location: format!("http://{}:{host_port}", host.0),
        }),
        Ok(InstanceNotFound) => Ok(GetResponse::Status404_InstanceIdOrPortNotFound(
            models::AdditionalInfo::new(format!("Instance {instance_id} not found")),
        )),
        Ok(UnknownPort) => Ok(GetResponse::Status404_InstanceIdOrPortNotFound(
            models::AdditionalInfo::new(format!("Unknown port {port}")),
        )),
        Ok(EditorSupportsReverseProxy) => Ok(GetResponse::Status400_MalformedRequest(
            models::AdditionalInfo::new("Editor supports reverse proxy -> use floxy".to_string()),
        )),
        Ok(InstanceNotRunning) => Ok(GetResponse::Status400_MalformedRequest(
            models::AdditionalInfo::new(format!("Instance {instance_id} not running")),
        )),
        Ok(InstanceNotConnectedToNetwork) => Ok(GetResponse::Status400_MalformedRequest(
            models::AdditionalInfo::new(format!("Instance {instance_id} not connected to network")),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enchantment::floxy::MockFloxy;
    use crate::jeweler::gem::instance::InstanceId;
    use crate::sorcerer::instancius::MockInstancius;
    use std::num::NonZeroU16;
    use std::sync::Arc;

    #[tokio::test]
    async fn get_500() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_redirect_editor_request()
            .once()
            .returning(|_, _, _, _| Err(anyhow::anyhow!("TestError")));
        assert!(matches!(
            get(
                crate::vault::tests::create_empty_test_vault(),
                Arc::new(MockFloxy::new()),
                Arc::new(instancius),
                Host("host".to_string()),
                InstanceId::new(6),
                NonZeroU16::new(1234).unwrap()
            )
            .await,
            Ok(GetResponse::Status500_InternalServerError(_))
        ));
    }

    #[tokio::test]
    async fn get_302() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_redirect_editor_request()
            .withf(|_, _, id, port| id.value == 6 && port.get() == 1234)
            .once()
            .returning(|_, _, _, _| Ok(Redirected(125)));
        assert_eq!(
            get(
                crate::vault::tests::create_empty_test_vault(),
                Arc::new(MockFloxy::new()),
                Arc::new(instancius),
                Host("host".to_string()),
                InstanceId::new(6),
                NonZeroU16::new(1234).unwrap()
            )
            .await,
            Ok(GetResponse::Status302_Found {
                location: "http://host:125".to_string()
            })
        );
    }

    #[tokio::test]
    async fn get_404_instance_not_found() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_redirect_editor_request()
            .withf(|_, _, id, port| id.value == 80 && port.get() == 100)
            .once()
            .returning(|_, _, _, _| Ok(InstanceNotFound));
        assert!(matches!(
            get(
                crate::vault::tests::create_empty_test_vault(),
                Arc::new(MockFloxy::new()),
                Arc::new(instancius),
                Host("host".to_string()),
                InstanceId::new(80),
                NonZeroU16::new(100).unwrap()
            )
            .await,
            Ok(GetResponse::Status404_InstanceIdOrPortNotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_404_unknown_port() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_redirect_editor_request()
            .withf(|_, _, id, port| id.value == 1 && port.get() == 60)
            .once()
            .returning(|_, _, _, _| Ok(UnknownPort));
        assert!(matches!(
            get(
                crate::vault::tests::create_empty_test_vault(),
                Arc::new(MockFloxy::new()),
                Arc::new(instancius),
                Host("host".to_string()),
                InstanceId::new(1),
                NonZeroU16::new(60).unwrap()
            )
            .await,
            Ok(GetResponse::Status404_InstanceIdOrPortNotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_400_reverse_proxy_support() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_redirect_editor_request()
            .withf(|_, _, id, port| id.value == 6 && port.get() == 5678)
            .once()
            .returning(|_, _, _, _| Ok(EditorSupportsReverseProxy));
        let result = get(
            crate::vault::tests::create_empty_test_vault(),
            Arc::new(MockFloxy::new()),
            Arc::new(instancius),
            Host("host".to_string()),
            InstanceId::new(6),
            NonZeroU16::new(5678).unwrap(),
        )
        .await;
        assert!(
            matches!(result, Ok(GetResponse::Status400_MalformedRequest(_))),
            "{:#?}",
            result
        );
    }

    #[tokio::test]
    async fn get_400_instance_stopped() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_redirect_editor_request()
            .withf(|_, _, id, port| id.value == 6 && port.get() == 1234)
            .once()
            .returning(|_, _, _, _| Ok(InstanceNotRunning));
        assert!(matches!(
            get(
                crate::vault::tests::create_empty_test_vault(),
                Arc::new(MockFloxy::new()),
                Arc::new(instancius),
                Host("host".to_string()),
                InstanceId::new(6),
                NonZeroU16::new(1234).unwrap()
            )
            .await,
            Ok(GetResponse::Status400_MalformedRequest(_))
        ));
    }

    #[tokio::test]
    async fn get_400_not_connected_to_network() {
        let mut instancius = MockInstancius::new();
        instancius
            .expect_redirect_editor_request()
            .withf(|_, _, id, port| id.value == 1 && port.get() == 1234)
            .once()
            .returning(|_, _, _, _| Ok(InstanceNotConnectedToNetwork));
        assert!(matches!(
            get(
                crate::vault::tests::create_empty_test_vault(),
                Arc::new(MockFloxy::new()),
                Arc::new(instancius),
                Host("host".to_string()),
                InstanceId::new(1),
                NonZeroU16::new(1234).unwrap()
            )
            .await,
            Ok(GetResponse::Status400_MalformedRequest(_))
        ));
    }
}
