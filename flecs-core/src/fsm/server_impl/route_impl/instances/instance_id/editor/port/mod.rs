use crate::enchantment::floxy::{Floxy, FloxyOperation};
use crate::jeweler::gem::instance::InstanceId;
use crate::sorcerer::instancius::RedirectEditorRequestResult::*;
use crate::vault::Vault;
use axum::extract::Host;
use flecsd_axum_server::apis::instances::InstancesInstanceIdEditorPortGetResponse as GetResponse;
use flecsd_axum_server::models::AdditionalInfo;
use std::num::NonZeroU16;
use std::sync::Arc;

pub async fn get<F: Floxy>(
    vault: Arc<Vault>,
    floxy: Arc<F>,
    host: Host,
    instance_id: InstanceId,
    port: NonZeroU16,
) -> Result<GetResponse, ()> {
    match crate::sorcerer::instancius::redirect_editor_request(
        vault,
        FloxyOperation::new_arc(floxy),
        instance_id,
        port,
    )
    .await
    {
        Err(e) => Ok(GetResponse::Status500_InternalServerError(
            AdditionalInfo::new(e.to_string()),
        )),
        Ok(Redirected(host_port)) => Ok(GetResponse::Status302_Found {
            location: format!("http://{}:{host_port}", host.0),
        }),
        Ok(InstanceNotFound) => Ok(GetResponse::Status404_InstanceIdOrPortNotFound(
            AdditionalInfo::new(format!("Instance {instance_id} not found")),
        )),
        Ok(UnknownPort) => Ok(GetResponse::Status404_InstanceIdOrPortNotFound(
            AdditionalInfo::new(format!("Unknown port {port}")),
        )),
        Ok(EditorSupportsReverseProxy) => Ok(GetResponse::Status400_MalformedRequest(
            AdditionalInfo::new("Editor supports reverse proxy -> use floxy".to_string()),
        )),
        Ok(InstanceNotRunning) => Ok(GetResponse::Status400_MalformedRequest(
            AdditionalInfo::new(format!("Instance {instance_id} not running")),
        )),
        Ok(InstanceNotConnectedToNetwork) => Ok(GetResponse::Status400_MalformedRequest(
            AdditionalInfo::new(format!("Instance {instance_id} not connected to network")),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enchantment::floxy::MockFloxy;
    use crate::jeweler::gem::instance::InstanceId;
    use crate::sorcerer::instancius::tests::spell_test_vault;
    use crate::tests::prepare_test_path;
    use std::num::NonZeroU16;
    use std::sync::Arc;

    #[tokio::test]
    async fn get_500() {
        let vault =
            spell_test_vault(prepare_test_path(module_path!(), "get_500"), Some(false)).await;
        assert!(matches!(
            get(
                vault,
                Arc::new(MockFloxy::new()),
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
        let vault =
            spell_test_vault(prepare_test_path(module_path!(), "get_302"), Some(true)).await;
        let mut floxy = MockFloxy::new();
        floxy
            .expect_add_instance_editor_redirect_to_free_port()
            .times(1)
            .returning(|_, _, _, _| Ok((false, 125)));
        assert_eq!(
            get(
                vault,
                Arc::new(floxy),
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
        let vault = spell_test_vault(
            prepare_test_path(module_path!(), "get_404_instance_not_found"),
            None,
        )
        .await;
        assert!(matches!(
            get(
                vault,
                Arc::new(MockFloxy::new()),
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
        let vault = spell_test_vault(
            prepare_test_path(module_path!(), "get_404_unknown_port"),
            None,
        )
        .await;
        assert!(matches!(
            get(
                vault,
                Arc::new(MockFloxy::new()),
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
        let vault = spell_test_vault(
            prepare_test_path(module_path!(), "get_400_reverse_proxy_support"),
            Some(true),
        )
        .await;
        let result = get(
            vault,
            Arc::new(MockFloxy::new()),
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
        let vault = spell_test_vault(
            prepare_test_path(module_path!(), "get_400_instance_stopped"),
            None,
        )
        .await;
        assert!(matches!(
            get(
                vault,
                Arc::new(MockFloxy::new()),
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
        let vault = spell_test_vault(
            prepare_test_path(module_path!(), "get_400_not_connected_to_network"),
            Some(true),
        )
        .await;
        assert!(matches!(
            get(
                vault,
                Arc::new(MockFloxy::new()),
                Host("host".to_string()),
                InstanceId::new(1),
                NonZeroU16::new(1234).unwrap()
            )
            .await,
            Ok(GetResponse::Status400_MalformedRequest(_))
        ));
    }
}
