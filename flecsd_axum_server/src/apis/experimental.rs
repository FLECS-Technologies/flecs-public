use async_trait::async_trait;
use axum::extract::*;
use axum_extra::extract::{CookieJar, Multipart};
use bytes::Bytes;
use http::Method;
use serde::{Deserialize, Serialize};

use crate::{models, types::*};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdDependsFeatureDeleteResponse {
    /// Provider removed
    Status200_ProviderRemoved,
    /// Bad request
    Status400_BadRequest(models::AdditionalInfo),
    /// Instance not found or instance not dependent on specified feature
    Status404_InstanceNotFoundOrInstanceNotDependentOnSpecifiedFeature(
        models::InstanceNotFoundOrNotDependent,
    ),
    /// State of the instance prevents removal of provider, e.g. instance is running
    Status409_StateOfTheInstancePreventsRemovalOfProvider(models::AdditionalInfo),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdDependsFeatureGetResponse {
    /// How the dependency on a feature is currently solved
    Status200_HowTheDependencyOnAFeatureIsCurrentlySolved(models::ProviderReference),
    /// Bad request
    Status400_BadRequest(models::AdditionalInfo),
    /// Instance not found or instance not dependent on specified feature
    Status404_InstanceNotFoundOrInstanceNotDependentOnSpecifiedFeature(
        models::InstanceNotFoundOrNotDependent,
    ),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdDependsFeaturePutResponse {
    /// Provider was overwritten
    Status200_ProviderWasOverwritten,
    /// Provider was set
    Status201_ProviderWasSet,
    /// Bad request
    Status400_BadRequest(models::AdditionalInfo),
    /// Instance not found or instance not dependent on specified feature
    Status404_InstanceNotFoundOrInstanceNotDependentOnSpecifiedFeature(
        models::InstanceNotFoundOrNotDependent,
    ),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdDependsGetResponse {
    /// All dependencies of the specified instance and how they are currently solved
    Status200_AllDependenciesOfTheSpecifiedInstanceAndHowTheyAreCurrentlySolved(
        std::collections::HashMap<String, models::ProviderReference>,
    ),
    /// Instance not found
    Status404_InstanceNotFound,
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdProvidesFeatureGetResponse {
    /// Information of the specified feature provided by the specified instance
    Status200_InformationOfTheSpecifiedFeatureProvidedByTheSpecifiedInstance(models::FeatureInfo),
    /// Instance not found or feature not provided by instance
    Status404_InstanceNotFoundOrFeatureNotProvidedByInstance(
        models::InstanceNotFoundOrFeatureNotProvided,
    ),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum InstancesInstanceIdProvidesGetResponse {
    /// Information for all features and their config provided by this instance
    Status200_InformationForAllFeaturesAndTheirConfigProvidedByThisInstance(
        std::collections::HashMap<String, models::FeatureInfo>,
    ),
    /// Instance id not found
    Status404_InstanceIdNotFound,
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersAuthCoreFirstTimeSetupSuperAdminGetResponse {
    /// Super admin of core auth provider set
    Status204_SuperAdminOfCoreAuthProviderSet,
    /// Super admin of core auth provider not set or no core provider set
    Status404_SuperAdminOfCoreAuthProviderNotSetOrNoCoreProviderSet(models::NotFound),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersAuthCoreFirstTimeSetupSuperAdminPostResponse {
    /// Super admin of core auth provider set
    Status200_SuperAdminOfCoreAuthProviderSet,
    /// Invalid super admin
    Status400_InvalidSuperAdmin(models::AdditionalInfo),
    /// Forbidden
    Status403_Forbidden,
    /// No core auth provider present
    Status404_NoCoreAuthProviderPresent,
    /// Failed to set super admin of core auth provider
    Status500_FailedToSetSuperAdminOfCoreAuthProvider(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersAuthCoreGetResponse {
    /// How the core auth provider is currently set
    Status200_HowTheCoreAuthProviderIsCurrentlySet(models::ProviderReference),
    /// No core auth provider set
    Status404_NoCoreAuthProviderSet,
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersAuthCorePutResponse {
    /// Provider was overwritten
    Status200_ProviderWasOverwritten,
    /// Provider was set
    Status201_ProviderWasSet,
    /// Bad request
    Status400_BadRequest(models::AdditionalInfo),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersAuthDefaultDeleteResponse {
    /// Remove the default auth provider
    Status200_RemoveTheDefaultAuthProvider,
    /// No default auth provider was found
    Status404_NoDefaultAuthProviderWasFound,
    /// The current state does not allow the removal of the default auth provider, e.g. a running instance is using it
    Status409_TheCurrentStateDoesNotAllowTheRemovalOfTheDefaultAuthProvider(models::AdditionalInfo),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersAuthDefaultFirstTimeSetupSuperAdminGetResponse {
    /// Super admin of default auth provider set
    Status204_SuperAdminOfDefaultAuthProviderSet,
    /// Super admin of default auth provider not set or no default provider set
    Status404_SuperAdminOfDefaultAuthProviderNotSetOrNoDefaultProviderSet(models::NotFound),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersAuthDefaultFirstTimeSetupSuperAdminPostResponse {
    /// Super admin of default auth provider set
    Status200_SuperAdminOfDefaultAuthProviderSet,
    /// Invalid super admin
    Status400_InvalidSuperAdmin(models::AdditionalInfo),
    /// Forbidden
    Status403_Forbidden,
    /// No default auth provider present
    Status404_NoDefaultAuthProviderPresent,
    /// Failed to set super admin of default auth provider
    Status500_FailedToSetSuperAdminOfDefaultAuthProvider(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersAuthDefaultGetResponse {
    /// Default auth provider was found
    Status200_DefaultAuthProviderWasFound(models::AuthProvider),
    /// No default auth provider was found
    Status404_NoDefaultAuthProviderWasFound,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersAuthDefaultPutResponse {
    /// Default auth provider was replaced
    Status200_DefaultAuthProviderWasReplaced,
    /// Default auth provider was set
    Status201_DefaultAuthProviderWasSet,
    /// Bad request
    Status400_BadRequest(models::AdditionalInfo),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersAuthFirstTimeSetupFlecsportPostResponse {
    /// First time setup of auth providers via flecsport triggered
    Status202_FirstTimeSetupOfAuthProvidersViaFlecsportTriggered,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersAuthGetResponse {
    /// Information for all auth providers
    Status200_InformationForAllAuthProviders(models::AuthProviders),
    /// Internal Server Error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersAuthIdFirstTimeSetupSuperAdminGetResponse {
    /// Super admin of specified auth provider set
    Status204_SuperAdminOfSpecifiedAuthProviderSet,
    /// Super admin of specified auth provider not set or specified provider not found
    Status404_SuperAdminOfSpecifiedAuthProviderNotSetOrSpecifiedProviderNotFound(models::NotFound),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersAuthIdFirstTimeSetupSuperAdminPostResponse {
    /// Super admin of specified auth provider set
    Status200_SuperAdminOfSpecifiedAuthProviderSet,
    /// Invalid super admin
    Status400_InvalidSuperAdmin(models::AdditionalInfo),
    /// Forbidden
    Status403_Forbidden,
    /// Specified auth provider not found
    Status404_SpecifiedAuthProviderNotFound,
    /// Failed to set super admin of specified auth provider
    Status500_FailedToSetSuperAdminOfSpecifiedAuthProvider(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersAuthIdGetResponse {
    /// Auth provider was found
    Status200_AuthProviderWasFound(models::AuthProvider),
    /// Auth provider was not found
    Status404_AuthProviderWasNotFound,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersFeatureDefaultDeleteResponse {
    /// Default provider for specified feature unset
    Status200_DefaultProviderForSpecifiedFeatureUnset,
    /// Bad request
    Status400_BadRequest(models::AdditionalInfo),
    /// Default provider for specified feature was not found
    Status404_DefaultProviderForSpecifiedFeatureWasNotFound,
    /// The current state does not allow the removal of the default provider, e.g. a running instance is using it
    Status409_TheCurrentStateDoesNotAllowTheRemovalOfTheDefaultProvider(models::AdditionalInfo),
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersFeatureDefaultGetResponse {
    /// Default provider for specified feature was found
    Status200_DefaultProviderForSpecifiedFeatureWasFound(models::FeatureProvider),
    /// Default provider for specified feature was not found
    Status404_DefaultProviderForSpecifiedFeatureWasNotFound,
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersFeatureDefaultPutResponse {
    /// Default provider for specified feature was replaced
    Status200_DefaultProviderForSpecifiedFeatureWasReplaced,
    /// Default provider for specified feature was set
    Status201_DefaultProviderForSpecifiedFeatureWasSet,
    /// Bad request
    Status400_BadRequest(models::AdditionalInfo),
    /// Provider not found
    Status404_ProviderNotFound,
    /// Internal server error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersFeatureGetResponse {
    /// Information for all providers of the specified feature
    Status200_InformationForAllProvidersOfTheSpecifiedFeature(models::FeatureProviders),
    /// Bad Request
    Status400_BadRequest(models::AdditionalInfo),
    /// Internal Server Error
    Status500_InternalServerError(models::AdditionalInfo),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersFeatureIdGetResponse {
    /// Provider was found
    Status200_ProviderWasFound(models::FeatureProvider),
    /// Bad request
    Status400_BadRequest(models::AdditionalInfo),
    /// Provider was not found
    Status404_ProviderWasNotFound,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ProvidersGetResponse {
    /// Information for all providers
    Status200_InformationForAllProviders(models::Providers),
    /// Internal Server Error
    Status500_InternalServerError(models::AdditionalInfo),
}

/// Experimental
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Experimental {
    /// InstancesInstanceIdDependsFeatureDelete - DELETE /v2/instances/{instance_id}/depends/{feature}
    async fn instances_instance_id_depends_feature_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdDependsFeatureDeletePathParams,
    ) -> Result<InstancesInstanceIdDependsFeatureDeleteResponse, ()>;

    /// InstancesInstanceIdDependsFeatureGet - GET /v2/instances/{instance_id}/depends/{feature}
    async fn instances_instance_id_depends_feature_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdDependsFeatureGetPathParams,
    ) -> Result<InstancesInstanceIdDependsFeatureGetResponse, ()>;

    /// InstancesInstanceIdDependsFeaturePut - PUT /v2/instances/{instance_id}/depends/{feature}
    async fn instances_instance_id_depends_feature_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdDependsFeaturePutPathParams,
        body: models::ProviderReference,
    ) -> Result<InstancesInstanceIdDependsFeaturePutResponse, ()>;

    /// InstancesInstanceIdDependsGet - GET /v2/instances/{instance_id}/depends
    async fn instances_instance_id_depends_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdDependsGetPathParams,
    ) -> Result<InstancesInstanceIdDependsGetResponse, ()>;

    /// InstancesInstanceIdProvidesFeatureGet - GET /v2/instances/{instance_id}/provides/{feature}
    async fn instances_instance_id_provides_feature_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdProvidesFeatureGetPathParams,
    ) -> Result<InstancesInstanceIdProvidesFeatureGetResponse, ()>;

    /// InstancesInstanceIdProvidesGet - GET /v2/instances/{instance_id}/provides
    async fn instances_instance_id_provides_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::InstancesInstanceIdProvidesGetPathParams,
    ) -> Result<InstancesInstanceIdProvidesGetResponse, ()>;

    /// ProvidersAuthCoreFirstTimeSetupSuperAdminGet - GET /v2/providers/auth/core/first-time-setup/super-admin
    async fn providers_auth_core_first_time_setup_super_admin_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<ProvidersAuthCoreFirstTimeSetupSuperAdminGetResponse, ()>;

    /// ProvidersAuthCoreFirstTimeSetupSuperAdminPost - POST /v2/providers/auth/core/first-time-setup/super-admin
    async fn providers_auth_core_first_time_setup_super_admin_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        body: models::SuperAdmin,
    ) -> Result<ProvidersAuthCoreFirstTimeSetupSuperAdminPostResponse, ()>;

    /// ProvidersAuthCoreGet - GET /v2/providers/auth/core
    async fn providers_auth_core_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<ProvidersAuthCoreGetResponse, ()>;

    /// ProvidersAuthCorePut - PUT /v2/providers/auth/core
    async fn providers_auth_core_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        body: models::ProviderReference,
    ) -> Result<ProvidersAuthCorePutResponse, ()>;

    /// ProvidersAuthDefaultDelete - DELETE /v2/providers/auth/default
    async fn providers_auth_default_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<ProvidersAuthDefaultDeleteResponse, ()>;

    /// ProvidersAuthDefaultFirstTimeSetupSuperAdminGet - GET /v2/providers/auth/default/first-time-setup/super-admin
    async fn providers_auth_default_first_time_setup_super_admin_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<ProvidersAuthDefaultFirstTimeSetupSuperAdminGetResponse, ()>;

    /// ProvidersAuthDefaultFirstTimeSetupSuperAdminPost - POST /v2/providers/auth/default/first-time-setup/super-admin
    async fn providers_auth_default_first_time_setup_super_admin_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        body: models::SuperAdmin,
    ) -> Result<ProvidersAuthDefaultFirstTimeSetupSuperAdminPostResponse, ()>;

    /// ProvidersAuthDefaultGet - GET /v2/providers/auth/default
    async fn providers_auth_default_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<ProvidersAuthDefaultGetResponse, ()>;

    /// ProvidersAuthDefaultPut - PUT /v2/providers/auth/default
    async fn providers_auth_default_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        body: models::PutDefaultProviderRequest,
    ) -> Result<ProvidersAuthDefaultPutResponse, ()>;

    /// ProvidersAuthFirstTimeSetupFlecsportPost - POST /v2/providers/auth/first-time-setup/flecsport
    async fn providers_auth_first_time_setup_flecsport_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<ProvidersAuthFirstTimeSetupFlecsportPostResponse, ()>;

    /// ProvidersAuthGet - GET /v2/providers/auth
    async fn providers_auth_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<ProvidersAuthGetResponse, ()>;

    /// ProvidersAuthIdFirstTimeSetupSuperAdminGet - GET /v2/providers/auth/{id}/first-time-setup/super-admin
    async fn providers_auth_id_first_time_setup_super_admin_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::ProvidersAuthIdFirstTimeSetupSuperAdminGetPathParams,
    ) -> Result<ProvidersAuthIdFirstTimeSetupSuperAdminGetResponse, ()>;

    /// ProvidersAuthIdFirstTimeSetupSuperAdminPost - POST /v2/providers/auth/{id}/first-time-setup/super-admin
    async fn providers_auth_id_first_time_setup_super_admin_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::ProvidersAuthIdFirstTimeSetupSuperAdminPostPathParams,
        body: models::SuperAdmin,
    ) -> Result<ProvidersAuthIdFirstTimeSetupSuperAdminPostResponse, ()>;

    /// ProvidersAuthIdGet - GET /v2/providers/auth/{id}
    async fn providers_auth_id_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::ProvidersAuthIdGetPathParams,
    ) -> Result<ProvidersAuthIdGetResponse, ()>;

    /// ProvidersFeatureDefaultDelete - DELETE /v2/providers/{feature}/default
    async fn providers_feature_default_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::ProvidersFeatureDefaultDeletePathParams,
    ) -> Result<ProvidersFeatureDefaultDeleteResponse, ()>;

    /// ProvidersFeatureDefaultGet - GET /v2/providers/{feature}/default
    async fn providers_feature_default_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::ProvidersFeatureDefaultGetPathParams,
    ) -> Result<ProvidersFeatureDefaultGetResponse, ()>;

    /// ProvidersFeatureDefaultPut - PUT /v2/providers/{feature}/default
    async fn providers_feature_default_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::ProvidersFeatureDefaultPutPathParams,
        body: models::PutDefaultProviderRequest,
    ) -> Result<ProvidersFeatureDefaultPutResponse, ()>;

    /// ProvidersFeatureGet - GET /v2/providers/{feature}
    async fn providers_feature_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::ProvidersFeatureGetPathParams,
    ) -> Result<ProvidersFeatureGetResponse, ()>;

    /// ProvidersFeatureIdGet - GET /v2/providers/{feature}/{id}
    async fn providers_feature_id_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: models::ProvidersFeatureIdGetPathParams,
    ) -> Result<ProvidersFeatureIdGetResponse, ()>;

    /// ProvidersGet - GET /v2/providers
    async fn providers_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<ProvidersGetResponse, ()>;
}
