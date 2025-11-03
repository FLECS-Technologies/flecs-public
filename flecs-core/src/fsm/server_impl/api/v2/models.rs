use crate::fsm::server_impl::api::v2::instances::instance_id::depends::dependency_key::InstanceNotFoundOrNotDependent;
use crate::quest::QuestId;
use crate::sorcerer::providius::{
    ClearDependencyError, DeleteDefaultProviderError, GetDependenciesError, GetDependencyError,
    GetFeatureProvidesError, GetProvidesError, Provider, SetCoreAuthProviderError,
    SetDefaultProviderError, SetDependencyError,
};
use crate::vault::pouch::AppKey;
use crate::vault::pouch::provider::ProviderId;
use axum::Json;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::collections::HashMap;
use utoipa::ToSchema;
use utoipa::openapi::schema::SchemaType;
use utoipa::openapi::{RefOr, Schema, Type};

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct AdditionalInfo {
    pub additional_info: String,
}

impl AdditionalInfo {
    pub fn new(info: impl Into<String>) -> Self {
        Self {
            additional_info: info.into(),
        }
    }

    pub fn into_bad_request(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }

    pub fn into_internal_server_error(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }

    pub fn into_conflict(self) -> Response {
        (StatusCode::CONFLICT, Json(self)).into_response()
    }
}

impl<T> From<T> for AdditionalInfo
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, ToSchema)]
pub enum InstanceNotFoundOrFeatureNotProvided {
    FeatureNotProvided,
    InstanceNotFound,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, ToSchema)]
pub struct GenericProvider {
    pub app_key: AppKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GenericProviders {
    pub default: Option<ProviderId>,
    pub providers: HashMap<ProviderId, GenericProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FeatureProviders {
    pub default: Option<ProviderId>,
    pub providers: HashMap<ProviderId, Provider>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct FeatureInfo {
    pub config: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PutDefaultProviderRequest {
    pub provider_id: ProviderId,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PutProviderReferenceRequest {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(schema_with = provider_reference_schema)]
    pub provider: crate::jeweler::gem::instance::ProviderReference,
}

fn provider_reference_schema() -> RefOr<Schema> {
    let id = utoipa::openapi::ObjectBuilder::new()
        .schema_type(SchemaType::Type(Type::String))
        .min_length(Some(8))
        .max_length(Some(8))
        .pattern(Some("^[0-9a-fA-F]{8}$"))
        .build();
    let default_literal = utoipa::openapi::ObjectBuilder::new()
        .schema_type(SchemaType::Type(Type::String))
        .enum_values::<Vec<_>, &str>(Some(vec!["Default"]))
        .build();
    utoipa::openapi::OneOfBuilder::new()
        .item(RefOr::T(Schema::Object(default_literal)))
        .item(RefOr::T(Schema::Object(id)))
        .into()
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Accepted {
    #[serde(rename = "jobId")]
    pub quest_id: QuestId,
}

impl Accepted {
    pub fn new(quest_id: QuestId) -> Self {
        Self { quest_id }
    }
}

impl From<QuestId> for Accepted {
    fn from(value: QuestId) -> Self {
        Self::new(value)
    }
}

impl IntoResponse for Accepted {
    fn into_response(self) -> Response {
        (StatusCode::ACCEPTED, Json(self)).into_response()
    }
}

impl IntoResponse for SetDependencyError {
    fn into_response(self) -> Response {
        match self {
            e @ SetDependencyError::NoDefaultProvider { .. }
            | e @ SetDependencyError::InstanceRunning { .. }
            | e @ SetDependencyError::KeyDoesNotContainFeature { .. }
            | e @ SetDependencyError::ProviderDoesNotExist(_)
            | e @ SetDependencyError::ProviderDoesNotProvideFeature { .. } => {
                AdditionalInfo::new(e.to_string()).into_bad_request()
            }
            SetDependencyError::InstanceNotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(InstanceNotFoundOrNotDependent::InstanceNotFound),
            )
                .into_response(),
            SetDependencyError::DoesNotDepend { .. } => (
                StatusCode::NOT_FOUND,
                Json(InstanceNotFoundOrNotDependent::NotDependent),
            )
                .into_response(),
            e @ SetDependencyError::FeatureConfigNotMatching { .. } => {
                AdditionalInfo::new(e.to_string()).into_conflict()
            }
            e @ SetDependencyError::FailedToCheckStatus(_) => {
                AdditionalInfo::new(e.to_string()).into_internal_server_error()
            }
        }
    }
}

impl IntoResponse for GetDependencyError {
    fn into_response(self) -> Response {
        match self {
            GetDependencyError::InstanceNotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(InstanceNotFoundOrNotDependent::InstanceNotFound),
            )
                .into_response(),
            GetDependencyError::DoesNotDepend { .. } => (
                StatusCode::NOT_FOUND,
                Json(InstanceNotFoundOrNotDependent::NotDependent),
            )
                .into_response(),
        }
    }
}

impl IntoResponse for ClearDependencyError {
    fn into_response(self) -> Response {
        match self {
            ClearDependencyError::InstanceNotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(InstanceNotFoundOrNotDependent::InstanceNotFound),
            )
                .into_response(),
            ClearDependencyError::DoesNotDepend { .. } => (
                StatusCode::NOT_FOUND,
                Json(InstanceNotFoundOrNotDependent::NotDependent),
            )
                .into_response(),
            e @ ClearDependencyError::InstanceRunning { .. } => {
                AdditionalInfo::new(e.to_string()).into_conflict()
            }
            e @ ClearDependencyError::FailedToCheckStatus { .. } => {
                AdditionalInfo::new(e.to_string()).into_internal_server_error()
            }
        }
    }
}

impl IntoResponse for GetDependenciesError {
    fn into_response(self) -> Response {
        match self {
            GetDependenciesError::InstanceNotFound(_) => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

impl IntoResponse for GetProvidesError {
    fn into_response(self) -> Response {
        match self {
            GetProvidesError::InstanceNotFound(_) => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

impl IntoResponse for GetFeatureProvidesError {
    fn into_response(self) -> Response {
        match self {
            GetFeatureProvidesError::InstanceNotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(InstanceNotFoundOrFeatureNotProvided::FeatureNotProvided),
            )
                .into_response(),
            GetFeatureProvidesError::DoesNotProvide { .. } => (
                StatusCode::NOT_FOUND,
                Json(InstanceNotFoundOrFeatureNotProvided::InstanceNotFound),
            )
                .into_response(),
        }
    }
}

impl IntoResponse for SetCoreAuthProviderError {
    fn into_response(self) -> Response {
        AdditionalInfo::new(self.to_string()).into_bad_request()
    }
}

impl IntoResponse for SetDefaultProviderError {
    fn into_response(self) -> Response {
        AdditionalInfo::new(self.to_string()).into_bad_request()
    }
}

impl IntoResponse for DeleteDefaultProviderError {
    fn into_response(self) -> Response {
        match self {
            e @ Self::ProviderInUse(_) => AdditionalInfo::new(e.to_string()).into_conflict(),
            e @ Self::FailedToCheckDependents { .. } => {
                AdditionalInfo::new(e.to_string()).into_internal_server_error()
            }
        }
    }
}

#[cfg(feature = "auth")]
pub mod auth {
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(Debug, Deserialize, Serialize, ToSchema)]
    pub enum ProviderOrSuperAdminNotFound {
        Provider,
        SuperAdmin,
    }

    #[derive(Debug, Deserialize, Serialize, ToSchema)]
    pub struct SuperAdmin {
        name: String,
        password: String,
    }
}
