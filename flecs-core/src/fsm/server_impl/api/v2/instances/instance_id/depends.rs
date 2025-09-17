pub mod dependency_key;

use crate::jeweler::gem::instance::ProviderReference;
use crate::sorcerer::providius::{Dependency, GetDependenciesError, Providius};
use crate::vault::Vault;
use crate::vault::pouch::instance::InstanceId;
use flecsd_axum_server::apis::experimental::InstancesInstanceIdDependsGetResponse as GetResponse;
use flecsd_axum_server::models::InstancesInstanceIdDependsGetPathParams as GetPathParams;
use flecsd_axum_server::{models, types};
use std::str::FromStr;
use std::sync::Arc;

impl From<Dependency> for models::Dependency {
    fn from(val: Dependency) -> Self {
        models::Dependency {
            provider: val.provider.map(|provides| match provides.provider {
                ProviderReference::Default => types::Nullable::Present(models::Provider {
                    provider_reference: models::ProviderReference::String(Box::new(
                        "Default".to_string(),
                    )),
                    provided_feature: provides.provided_feature,
                }),
                ProviderReference::Provider(provider_id) => {
                    types::Nullable::Present(models::Provider {
                        provider_reference: models::ProviderReference::ProviderReferenceOneOf(
                            Box::new(models::ProviderReferenceOneOf::new(provider_id.to_string())),
                        ),
                        provided_feature: provides.provided_feature,
                    })
                }
            }),
            requirements: types::Object(val.config),
        }
    }
}

pub async fn get(
    vault: Arc<Vault>,
    providius: Arc<dyn Providius>,
    path_params: GetPathParams,
) -> GetResponse {
    let instance_id = InstanceId::from_str(&path_params.instance_id).unwrap();
    match providius.get_dependencies(vault, instance_id).await {
        Err(GetDependenciesError::InstanceNotFound(_)) => GetResponse::Status404_InstanceNotFound,
        Ok(dependencies) => {
            GetResponse::Status200_AllDependenciesOfTheSpecifiedInstanceAndHowTheyAreCurrentlySolved(
                dependencies
                    .into_iter()
                    .map(|(key, dependency)| (key.to_string(), dependency.into()))
                    .collect(),
            )
        }
    }
}
