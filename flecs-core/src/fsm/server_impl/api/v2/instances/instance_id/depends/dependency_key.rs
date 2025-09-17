use crate::jeweler::gem::manifest::DependencyKey;
use crate::sorcerer::providius::{
    ClearDependencyError, GetDependencyError, Providius, SetDependencyError,
};
use crate::vault::Vault;
use crate::vault::pouch::instance::InstanceId;
use flecsd_axum_server::apis::experimental::{
    InstancesInstanceIdDependsDependencyKeyDeleteResponse as DeleteResponse,
    InstancesInstanceIdDependsDependencyKeyGetResponse as GetResponse,
    InstancesInstanceIdDependsDependencyKeyPutResponse as PutResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    InstancesInstanceIdDependsDependencyKeyDeletePathParams as DeletePathParams,
    InstancesInstanceIdDependsDependencyKeyGetPathParams as GetPathParams,
    InstancesInstanceIdDependsDependencyKeyPutPathParams as PutPathParams,
    ProviderReference as PutRequest,
};
use std::str::FromStr;
use std::sync::Arc;

pub mod feature;

pub async fn delete(
    vault: Arc<Vault>,
    providius: Arc<dyn Providius>,
    path_params: DeletePathParams,
) -> DeleteResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    let key = DependencyKey::new(&path_params.dependency_key);
    match providius.clear_dependency(vault, &key, instance_id).await {
        Ok(_) => DeleteResponse::Status200_ProviderRemoved,
        Err(ClearDependencyError::InstanceNotFound(_)) => {
            DeleteResponse::Status404_InstanceNotFoundOrInstanceNotDependentOnSpecifiedDependency(
                models::InstanceNotFoundOrNotDependent::InstanceNotFound,
            )
        }
        Err(ClearDependencyError::DoesNotDepend { .. }) => {
            DeleteResponse::Status404_InstanceNotFoundOrInstanceNotDependentOnSpecifiedDependency(
                models::InstanceNotFoundOrNotDependent::NotDependent,
            )
        }
        Err(e @ ClearDependencyError::InstanceRunning { .. }) => {
            DeleteResponse::Status409_StateOfTheInstancePreventsRemovalOfProvider(
                models::AdditionalInfo::new(e.to_string()),
            )
        }
        Err(e @ ClearDependencyError::FailedToCheckStatus { .. }) => {
            DeleteResponse::Status500_InternalServerError(models::AdditionalInfo::new(
                e.to_string(),
            ))
        }
    }
}

pub async fn get(
    vault: Arc<Vault>,
    providius: Arc<dyn Providius>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    let key = DependencyKey::new(&path_params.dependency_key);
    match providius.get_dependency(vault, &key, instance_id).await {
        Ok(dependency) => {
            GetResponse::Status200_HowTheDependencyIsCurrentlySolved(dependency.into())
        }
        Err(GetDependencyError::InstanceNotFound(_)) => {
            GetResponse::Status404_InstanceNotFoundOrInstanceNotDependentOnSpecifiedDependency(
                models::InstanceNotFoundOrNotDependent::InstanceNotFound,
            )
        }
        Err(GetDependencyError::DoesNotDepend { .. }) => {
            GetResponse::Status404_InstanceNotFoundOrInstanceNotDependentOnSpecifiedDependency(
                models::InstanceNotFoundOrNotDependent::NotDependent,
            )
        }
    }
}

pub async fn put(
    vault: Arc<Vault>,
    providius: Arc<dyn Providius>,
    request: PutRequest,
    path_params: PutPathParams,
) -> PutResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    let provider_reference = request.try_into().unwrap();
    let key = DependencyKey::new(&path_params.dependency_key);
    if key.features().len() != 1 {
        return PutResponse::Status400_BadRequest(models::AdditionalInfo::new(
            "This route accepts only single feature depends".to_string(),
        ));
    }
    match providius
        .set_dependency(
            vault,
            key,
            &path_params.dependency_key,
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
