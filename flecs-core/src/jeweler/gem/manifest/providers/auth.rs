#[cfg(feature = "auth")]
pub use openidconnect::IssuerUrl;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use utoipa::openapi::{Object, ObjectBuilder};

#[cfg(not(feature = "auth"))]
pub type IssuerUrl = String;

fn try_issuer_url(issuer_url: String) -> Result<IssuerUrl, AuthProviderFromValueError> {
    #[cfg(feature = "auth")]
    {
        IssuerUrl::new(issuer_url.clone()).map_err(|_| AuthProviderFromValueError::ValueMalformed {
            name: "issuer_url",
            value: serde_json::Value::String(issuer_url),
        })
    }
    #[cfg(not(feature = "auth"))]
    {
        Ok(issuer_url)
    }
}

#[derive(Error, Debug)]
pub enum AuthProviderFromValueError {
    #[error("Value for {0} is missing")]
    ValueMissing(&'static str),
    #[error("Value for {name} is malformed: {value:?}")]
    ValueMalformed {
        name: &'static str,
        value: serde_json::Value,
    },
    #[error("Expected object with properties, received {0:?}")]
    NotObject(serde_json::Value),
}

impl TryFrom<&serde_json::Value> for AuthProvider {
    type Error = AuthProviderFromValueError;

    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        const PROPERTY_NAME_ISSUER_URL: &str = "issuer_url";
        const PROPERTY_NAME_NAME: &str = "name";
        const PROPERTY_NAME_KIND: &str = "kind";
        let serde_json::Value::Object(properties) = value else {
            return Err(AuthProviderFromValueError::NotObject(value.clone()));
        };
        Ok(Self {
            issuer_url: match properties.get(PROPERTY_NAME_ISSUER_URL) {
                None => {
                    return Err(AuthProviderFromValueError::ValueMissing(
                        PROPERTY_NAME_ISSUER_URL,
                    ));
                }
                Some(serde_json::Value::String(issuer_url)) => try_issuer_url(issuer_url.clone())?,
                Some(val) => {
                    return Err(AuthProviderFromValueError::ValueMalformed {
                        name: PROPERTY_NAME_ISSUER_URL,
                        value: val.clone(),
                    });
                }
            },
            name: match properties.get(PROPERTY_NAME_NAME) {
                None => return Err(AuthProviderFromValueError::ValueMissing(PROPERTY_NAME_NAME)),
                Some(serde_json::Value::String(name)) => name.clone(),
                Some(val) => {
                    return Err(AuthProviderFromValueError::ValueMalformed {
                        name: PROPERTY_NAME_NAME,
                        value: val.clone(),
                    });
                }
            },
            kind: match properties.get(PROPERTY_NAME_KIND) {
                None => return Err(AuthProviderFromValueError::ValueMissing(PROPERTY_NAME_KIND)),
                Some(serde_json::Value::String(kind)) => kind.clone(),
                Some(val) => {
                    return Err(AuthProviderFromValueError::ValueMalformed {
                        name: PROPERTY_NAME_KIND,
                        value: val.clone(),
                    });
                }
            },
            config: value.clone(),
        })
    }
}

fn custom_type() -> Object {
    ObjectBuilder::new()
        .schema_type(utoipa::openapi::schema::Type::String)
        .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
            utoipa::openapi::KnownFormat::Uri,
        )))
        .build()
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize, ToSchema)]
pub struct AuthProvider {
    #[schema(schema_with = custom_type)]
    pub issuer_url: IssuerUrl,
    pub name: String,
    pub kind: String,
    pub config: serde_json::Value,
}
