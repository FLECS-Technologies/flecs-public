use crate::jeweler::gem::instance::InstanceId;
use crate::sorcerer::providius::{GetFeatureProvidesError, Providius};
use crate::vault::Vault;
use flecsd_axum_server::apis::experimental::InstancesInstanceIdProvidesFeatureGetResponse as GetResponse;
use flecsd_axum_server::models::InstancesInstanceIdProvidesFeatureGetPathParams as GetPathParams;
use flecsd_axum_server::{models, types};
use std::str::FromStr;
use std::sync::Arc;

pub async fn get(
    vault: Arc<Vault>,
    providius: Arc<dyn Providius>,
    path_params: GetPathParams,
) -> GetResponse {
    let id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match providius
        .get_feature_provides(vault, &path_params.feature, id)
        .await
    {
        Ok(provider) => {
            GetResponse::Status200_InformationOfTheSpecifiedFeatureProvidedByTheSpecifiedInstance(
                models::FeatureInfo {
                    config: types::Object(provider.config),
                },
            )
        }
        Err(GetFeatureProvidesError::DoesNotProvide { .. }) => {
            GetResponse::Status404_InstanceNotFoundOrFeatureNotProvidedByInstance(
                models::InstanceNotFoundOrFeatureNotProvided::FeatureNotProvided,
            )
        }
        Err(GetFeatureProvidesError::InstanceNotFound(_)) => {
            GetResponse::Status404_InstanceNotFoundOrFeatureNotProvidedByInstance(
                models::InstanceNotFoundOrFeatureNotProvided::InstanceNotFound,
            )
        }
    }
}
