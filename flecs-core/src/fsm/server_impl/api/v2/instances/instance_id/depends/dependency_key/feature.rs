use crate::jeweler::gem::manifest::DependencyKey;
use crate::sorcerer::providius::{Providius, SetDependencyError};
use crate::vault::Vault;
use crate::vault::pouch::instance::InstanceId;
use flecsd_axum_server::apis::experimental::InstancesInstanceIdDependsDependencyKeyFeaturePutResponse as PutResponse;
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    InstancesInstanceIdDependsDependencyKeyFeaturePutPathParams as PutPathParams,
    ProviderReference as PutRequest,
};
use std::str::FromStr;
use std::sync::Arc;

pub async fn put(
    vault: Arc<Vault>,
    providius: Arc<dyn Providius>,
    request: PutRequest,
    path_params: PutPathParams,
) -> PutResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    let provider_reference = request.try_into().unwrap();
    let key = DependencyKey::new(&path_params.dependency_key);
    match providius
        .set_dependency(
            vault,
            key,
            &path_params.feature,
            instance_id,
            provider_reference,
        )
        .await
    {
        Ok(Some(_)) => PutResponse::Status200_ProviderWasOverwritten,
        Ok(None) => PutResponse::Status201_ProviderWasSet,
        Err(e @ SetDependencyError::NoDefaultProvider { .. })
        | Err(e @ SetDependencyError::InstanceRunning { .. })
        | Err(e @ SetDependencyError::KeyDoesNotContainFeature { .. })
        | Err(e @ SetDependencyError::ProviderDoesNotExist(_))
        | Err(e @ SetDependencyError::ProviderDoesNotProvideFeature { .. }) => {
            PutResponse::Status400_BadRequest(models::AdditionalInfo::new(e.to_string()))
        }
        Err(SetDependencyError::InstanceNotFound(_)) => {
            PutResponse::Status404_InstanceNotFoundOrInstanceNotDependentOnSpecifiedFeature(
                models::InstanceNotFoundOrNotDependent::InstanceNotFound,
            )
        }
        Err(SetDependencyError::DoesNotDepend { .. }) => {
            PutResponse::Status404_InstanceNotFoundOrInstanceNotDependentOnSpecifiedFeature(
                models::InstanceNotFoundOrNotDependent::NotDependent,
            )
        }
        Err(e @ SetDependencyError::FeatureConfigNotMatching { .. }) => {
            PutResponse::Status409_ProviderConflictsWithRequirementsOfDependency(
                models::AdditionalInfo::new(e.to_string()),
            )
        }
        Err(e @ SetDependencyError::FailedToCheckStatus(_)) => {
            PutResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}
