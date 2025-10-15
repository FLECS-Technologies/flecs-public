use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use utoipa::openapi::{Object, ObjectBuilder};

#[cfg(feature = "auth")]
#[derive(Debug, Error)]
pub enum ReplaceHostError {
    #[error("Failed to parse issuer url with replaced host: {0}")]
    ParseError(#[from] url::ParseError),
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

#[cfg(not(feature = "auth"))]
pub type IssuerUrl = String;
#[cfg(feature = "auth")]
pub type IssuerUrl = url::Url;

fn try_issuer_url(issuer_url: &str) -> Result<IssuerUrl, AuthProviderFromValueError> {
    #[cfg(feature = "auth")]
    {
        IssuerUrl::parse(issuer_url).map_err(|_| AuthProviderFromValueError::ValueMalformed {
            name: "issuer_url",
            value: serde_json::Value::String(issuer_url.to_string()),
        })
    }
    #[cfg(not(feature = "auth"))]
    {
        Ok(issuer_url.to_string())
    }
}

impl TryFrom<&serde_json::Value> for AuthProvider {
    type Error = AuthProviderFromValueError;

    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        const PROPERTY_NAME_ISSUER_URL: &str = "issuer_url";
        const PROPERTY_NAME_NAME: &str = "name";
        const PROPERTY_NAME_KIND: &str = "kind";
        const PROPERTY_NAME_PORT: &str = "port";
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
                Some(serde_json::Value::String(issuer_url)) => try_issuer_url(issuer_url)?,
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
            port: match properties.get(PROPERTY_NAME_PORT) {
                None => return Err(AuthProviderFromValueError::ValueMissing(PROPERTY_NAME_PORT)),
                Some(serde_json::Value::Number(port)) => match port.as_u64() {
                    None => {
                        return Err(AuthProviderFromValueError::ValueMalformed {
                            name: PROPERTY_NAME_PORT,
                            value: serde_json::Value::Number(port.clone()),
                        });
                    }
                    Some(port) => {
                        if port > u16::MAX as u64 {
                            return Err(AuthProviderFromValueError::ValueMalformed {
                                name: PROPERTY_NAME_PORT,
                                value: serde_json::Value::Number(port.into()),
                            });
                        } else {
                            port as u16
                        }
                    }
                },
                Some(val) => {
                    return Err(AuthProviderFromValueError::ValueMalformed {
                        name: PROPERTY_NAME_PORT,
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
    pub port: u16,
    pub config: serde_json::Value,
}
