use crate::sorcerer::deploymento::{Deploymento, ReserveIpv4AddressError};
use crate::vault::Vault;
use flecsd_axum_server::apis::deployments::DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4PostResponse as PostResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4Post200Response as PostResponse200,
    DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4PostPathParams as PostPathParams,
};
use std::sync::Arc;

pub async fn post<T: Deploymento>(
    vault: Arc<Vault>,
    deploymento: Arc<T>,
    path_params: PostPathParams,
) -> PostResponse {
    match deploymento
        .reserve_ipv4_address(vault, path_params.deployment_id, path_params.network_id)
        .await
    {
        Ok(address) => PostResponse::Status200_Success(PostResponse200 {
            ipv4_address: address.to_string(),
        }),
        Err(e @ ReserveIpv4AddressError::Other { .. })
        | Err(e @ ReserveIpv4AddressError::NoFreeIpAddress) => {
            PostResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
        Err(e @ ReserveIpv4AddressError::DeploymentNotFound(_))
        | Err(e @ ReserveIpv4AddressError::NetworkNotFound(_)) => {
            PostResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
                additional_info: Some(e.to_string()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorcerer::deploymento::MockDeploymento;
    use crate::vault::tests::create_empty_test_vault;
    use mockall::predicate;
    use std::net::Ipv4Addr;

    const DEPLOYMENT_ID: &str = "TestDeployment";
    const NETWORK_ID: &str = "TestNetwork";

    fn post_data(
        mock_result: Option<anyhow::Result<Ipv4Addr, ReserveIpv4AddressError>>,
    ) -> (Arc<Vault>, Arc<MockDeploymento>, PostPathParams) {
        let mut deploymento = MockDeploymento::new();
        if let Some(mock_result) = mock_result {
            deploymento
                .expect_reserve_ipv4_address()
                .once()
                .with(
                    predicate::always(),
                    predicate::eq(DEPLOYMENT_ID.to_string()),
                    predicate::eq(NETWORK_ID.to_string()),
                )
                .return_const(mock_result);
        }
        let vault = create_empty_test_vault();
        let path_params = PostPathParams {
            deployment_id: DEPLOYMENT_ID.to_string(),
            network_id: NETWORK_ID.to_string(),
        };
        (vault, Arc::new(deploymento), path_params)
    }

    #[tokio::test]
    async fn post_200() {
        let address = Ipv4Addr::new(10, 20, 34, 100);
        let (vault, deploymento, path_params) = post_data(Some(Ok(address)));
        assert_eq!(
            post(vault, deploymento, path_params).await,
            PostResponse::Status200_Success(PostResponse200 {
                ipv4_address: address.to_string()
            })
        );
    }

    #[tokio::test]
    async fn post_500() {
        let mock_error = ReserveIpv4AddressError::Other {
            network_id: NETWORK_ID.to_string(),
            reason: "TestError".to_string(),
        };
        let (vault, deploymento, path_params) = post_data(Some(Err(mock_error.clone())));
        assert_eq!(
            post(vault, deploymento, path_params).await,
            PostResponse::Status500_InternalServerError(models::AdditionalInfo::new(
                mock_error.to_string()
            ))
        );
    }

    #[tokio::test]
    async fn post_500_no_free_address() {
        let mock_error = ReserveIpv4AddressError::NoFreeIpAddress;
        let (vault, deploymento, path_params) = post_data(Some(Err(mock_error.clone())));
        assert_eq!(
            post(vault, deploymento, path_params).await,
            PostResponse::Status500_InternalServerError(models::AdditionalInfo::new(
                mock_error.to_string()
            ))
        );
    }

    #[tokio::test]
    async fn post_404_deployment() {
        let mock_error = ReserveIpv4AddressError::DeploymentNotFound(DEPLOYMENT_ID.to_string());
        let (vault, deploymento, path_params) = post_data(Some(Err(mock_error.clone())));
        assert_eq!(
            post(vault, deploymento, path_params).await,
            PostResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
                additional_info: Some(mock_error.to_string())
            })
        );
    }

    #[tokio::test]
    async fn post_404_network() {
        let mock_error = ReserveIpv4AddressError::NetworkNotFound(NETWORK_ID.to_string());
        let (vault, deploymento, path_params) = post_data(Some(Err(mock_error.clone())));
        assert_eq!(
            post(vault, deploymento, path_params).await,
            PostResponse::Status404_ResourceNotFound(models::OptionalAdditionalInfo {
                additional_info: Some(mock_error.to_string())
            })
        );
    }
}
