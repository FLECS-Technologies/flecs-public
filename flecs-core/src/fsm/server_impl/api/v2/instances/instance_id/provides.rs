pub mod feature;

use crate::jeweler::gem::instance::InstanceId;
use crate::sorcerer::providius::{GetProvidesError, Providius};
use crate::vault::Vault;
use flecsd_axum_server::apis::experimental::InstancesInstanceIdProvidesGetResponse as GetResponse;
use flecsd_axum_server::models::InstancesInstanceIdProvidesGetPathParams as GetPathParams;
use flecsd_axum_server::{models, types};
use std::str::FromStr;
use std::sync::Arc;

pub async fn get(
    vault: Arc<Vault>,
    providius: Arc<dyn Providius>,
    path_params: GetPathParams,
) -> GetResponse {
    let id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match providius.get_provides(vault, id).await {
        Ok(providers) => {
            GetResponse::Status200_InformationForAllFeaturesAndTheirConfigProvidedByThisInstance(
                providers
                    .into_iter()
                    .map(|(feature, provider)| {
                        (
                            feature,
                            models::FeatureInfo {
                                config: types::Object(provider.config),
                            },
                        )
                    })
                    .collect(),
            )
        }
        Err(GetProvidesError::InstanceNotFound(_)) => GetResponse::Status404_InstanceIdNotFound,
    }
}
