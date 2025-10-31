pub mod path;

use crate::fsm::server_impl::api::v2::models::AdditionalInfo;
use crate::fsm::server_impl::state::{LoreState, ProvidiusState, VaultState, WatchState};
use crate::sorcerer::providius::SetCoreAuthProviderError;
use axum::Json;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use utoipa::openapi::schema::SchemaType;
use utoipa::openapi::{RefOr, Schema, Type};
use utoipa::{PartialSchema, ToSchema};

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProviderReference(
    #[serde_as(as = "DisplayFromStr")] crate::jeweler::gem::instance::ProviderReference,
);

impl ToSchema for ProviderReference {}
impl PartialSchema for ProviderReference {
    fn schema() -> RefOr<Schema> {
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
}

#[utoipa::path(
    get,
    path = "/providers/auth/core",
    tag = "Experimental",
    description = "Get information on the core auth provider",
    responses(
        (status = NO_CONTENT, description = "How the core auth provider is currently set", body = crate::jeweler::gem::instance::ProviderReference),
        (status = NOT_FOUND, description = "No core auth provider set"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn get(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
) -> Response {
    match providius.get_core_providers(vault).await.auth {
        Some(provider) => (StatusCode::OK, Json(provider)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/providers/auth/core",
    tag = "Experimental",
    description = "Set a core auth provider",
    request_body(
        content = ProviderReference,
        description = "The provider that should be used",
    ),
    responses(
        (status = OK, description = "Provider was overwritten"),
        (status = CREATED, description = "Provider was set"),
        (status = BAD_REQUEST, description = "Bad request", body = AdditionalInfo),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = AdditionalInfo),
    ),
)]
pub async fn put(
    State(VaultState(vault)): State<VaultState>,
    State(ProvidiusState(providius)): State<ProvidiusState>,
    State(LoreState(lore)): State<LoreState>,
    #[cfg(feature = "auth")] State(WatchState(watch)): State<WatchState>,
    Json(provider): Json<ProviderReference>,
) -> Result<Response, SetCoreAuthProviderError> {
    match providius
        .put_core_auth_provider(
            vault,
            #[cfg(feature = "auth")]
            lore,
            #[cfg(feature = "auth")]
            watch,
            provider.0,
        )
        .await?
    {
        Some(_) => Ok(StatusCode::OK.into_response()),
        None => Ok(StatusCode::CREATED.into_response()),
    }
}
