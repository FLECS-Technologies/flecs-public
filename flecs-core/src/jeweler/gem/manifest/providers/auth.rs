#[cfg(feature = "auth")]
pub use openidconnect::IssuerUrl;
use thiserror::Error;

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
        let serde_json::Value::Object(properties) = value else {
            return Err(AuthProviderFromValueError::NotObject(value.clone()));
        };
        Ok(Self {
            issuer_url: match properties.get("issuer_url") {
                None => return Err(AuthProviderFromValueError::ValueMissing("issuer_url")),
                Some(serde_json::Value::String(issuer_url)) => try_issuer_url(issuer_url.clone())?,
                Some(val) => {
                    return Err(AuthProviderFromValueError::ValueMalformed {
                        name: "issuer_url",
                        value: val.clone(),
                    });
                }
            },
            name: match properties.get("name") {
                None => return Err(AuthProviderFromValueError::ValueMissing("name")),
                Some(serde_json::Value::String(name)) => name.clone(),
                Some(val) => {
                    return Err(AuthProviderFromValueError::ValueMalformed {
                        name: "name",
                        value: val.clone(),
                    });
                }
            },
            kind: match properties.get("kind") {
                None => return Err(AuthProviderFromValueError::ValueMissing("kind")),
                Some(serde_json::Value::String(kind)) => kind.clone(),
                Some(val) => {
                    return Err(AuthProviderFromValueError::ValueMalformed {
                        name: "kind",
                        value: val.clone(),
                    });
                }
            },
            config: value.clone(),
        })
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AuthProvider {
    pub issuer_url: IssuerUrl,
    pub name: String,
    pub kind: String,
    pub config: serde_json::Value,
}
