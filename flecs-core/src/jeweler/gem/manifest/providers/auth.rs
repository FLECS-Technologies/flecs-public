#[cfg(feature = "auth")]
use crate::wall::watch;
#[cfg(feature = "auth")]
use axum::extract::Host;
use serde::{Deserialize, Serialize};
#[cfg(feature = "auth")]
use std::net::IpAddr;
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
pub type Url = String;
#[cfg(feature = "auth")]
pub type Url = url::Url;

fn try_url_from_property(
    property: Option<&serde_json::Value>,
    property_name: &'static str,
) -> Result<Url, AuthProviderFromValueError> {
    let url = match property {
        None => {
            return Err(AuthProviderFromValueError::ValueMissing(property_name));
        }
        Some(serde_json::Value::String(issuer_url)) => issuer_url,
        Some(val) => {
            return Err(AuthProviderFromValueError::ValueMalformed {
                name: property_name,
                value: val.clone(),
            });
        }
    };
    #[cfg(feature = "auth")]
    {
        Url::parse(url).map_err(|_| AuthProviderFromValueError::ValueMalformed {
            name: property_name,
            value: serde_json::Value::String(url.to_string()),
        })
    }
    #[cfg(not(feature = "auth"))]
    {
        Ok(url.to_string())
    }
}

impl TryFrom<&serde_json::Value> for AuthProvider {
    type Error = AuthProviderFromValueError;

    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        const PROPERTY_NAME_ISSUER_URL: &str = "issuer_url";
        const PROPERTY_NAME_NAME: &str = "name";
        const PROPERTY_NAME_KIND: &str = "kind";
        const PROPERTY_NAME_PORT: &str = "port";
        const PROPERTY_NAME_JWK_URL: &str = "jwk_url";
        const PROPERTY_NAME_AUTHORIZE_URL: &str = "authorize_url";
        const PROPERTY_NAME_TOKEN_URL: &str = "token_url";
        let serde_json::Value::Object(properties) = value else {
            return Err(AuthProviderFromValueError::NotObject(value.clone()));
        };
        let config = match properties.get(PROPERTY_NAME_KIND) {
            None => return Err(AuthProviderFromValueError::ValueMissing(PROPERTY_NAME_KIND)),
            Some(serde_json::Value::String(kind)) if kind == "oidc" => AuthProviderConfig::Oidc {},
            Some(serde_json::Value::String(kind)) if kind == "oauth" => AuthProviderConfig::Oauth {
                authorize_url: try_url_from_property(
                    properties.get(PROPERTY_NAME_AUTHORIZE_URL),
                    PROPERTY_NAME_AUTHORIZE_URL,
                )?
                .into(),
                jwk_url: try_url_from_property(
                    properties.get(PROPERTY_NAME_JWK_URL),
                    PROPERTY_NAME_JWK_URL,
                )?
                .into(),
                token_url: try_url_from_property(
                    properties.get(PROPERTY_NAME_TOKEN_URL),
                    PROPERTY_NAME_TOKEN_URL,
                )?
                .into(),
            },
            Some(val) => {
                return Err(AuthProviderFromValueError::ValueMalformed {
                    name: PROPERTY_NAME_KIND,
                    value: val.clone(),
                });
            }
        };
        Ok(Self {
            issuer_url: try_url_from_property(
                properties.get(PROPERTY_NAME_ISSUER_URL),
                PROPERTY_NAME_ISSUER_URL,
            )?,
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
            properties: value.clone(),
            config,
        })
    }
}

fn uri_schema() -> Object {
    ObjectBuilder::new()
        .schema_type(utoipa::openapi::schema::Type::String)
        .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
            utoipa::openapi::KnownFormat::Uri,
        )))
        .build()
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize, ToSchema)]
#[serde(tag = "kind")]
pub enum AuthProviderConfig {
    #[serde(rename = "oidc")]
    Oidc {},
    #[serde(rename = "oauth")]
    Oauth {
        #[schema(schema_with = uri_schema)]
        jwk_url: Box<Url>,
        #[schema(schema_with = uri_schema)]
        authorize_url: Box<Url>,
        #[schema(schema_with = uri_schema)]
        token_url: Box<Url>,
    },
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize, ToSchema)]
pub struct AuthProvider {
    #[serde(flatten)]
    pub config: AuthProviderConfig,
    #[schema(schema_with = uri_schema)]
    pub issuer_url: Url,
    pub name: String,
    pub port: u16,
    pub properties: serde_json::Value,
}

impl AuthProvider {
    #[cfg(feature = "auth")]
    pub fn build_meta(&self, ip: IpAddr, port: u16) -> Option<watch::AuthProviderMetaData> {
        let mut issuer_url = self.issuer_url.clone();
        issuer_url.set_ip_host(ip).ok()?;
        issuer_url.set_port(Some(port)).ok()?;
        Some(match &self.config {
            AuthProviderConfig::Oidc { .. } => watch::AuthProviderMetaData::Oidc { issuer_url },
            AuthProviderConfig::Oauth { jwk_url, .. } => {
                let mut jwk_url = jwk_url.as_ref().clone();
                jwk_url.set_ip_host(ip).ok()?;
                jwk_url.set_port(Some(port)).ok()?;
                watch::AuthProviderMetaData::Oauth {
                    issuer_url,
                    jwk_url,
                }
            }
        })
    }
    #[cfg(feature = "auth")]
    pub fn replace_host_and_port(&mut self, host: &Host, port: u16) {
        let _ = self.issuer_url.set_host(Some(&host.0));
        let _ = self.issuer_url.set_port(Some(port));
        if let AuthProviderConfig::Oauth {
            jwk_url,
            token_url,
            authorize_url,
        } = &mut self.config
        {
            let _ = jwk_url.set_host(Some(&host.0));
            let _ = jwk_url.set_port(Some(port));
            let _ = token_url.set_host(Some(&host.0));
            let _ = token_url.set_port(Some(port));
            let _ = authorize_url.set_host(Some(&host.0));
            let _ = authorize_url.set_port(Some(port));
        }
    }
}
