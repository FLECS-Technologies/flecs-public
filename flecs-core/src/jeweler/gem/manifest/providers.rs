use flecs_app_manifest::generated::manifest_3_2_0::ProvidesKey;
use std::collections::HashMap;
use thiserror::Error;

pub mod auth;

#[derive(Error, Debug)]
pub enum ProviderFromValueError {
    #[error(transparent)]
    Auth(#[from] auth::AuthProviderFromValueError),
}

#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct Providers {
    pub auth: Option<auth::AuthProvider>,
}

impl TryFrom<&HashMap<ProvidesKey, serde_json::Value>> for Providers {
    type Error = ProviderFromValueError;

    fn try_from(value: &HashMap<ProvidesKey, serde_json::Value>) -> Result<Self, Self::Error> {
        Ok(Self {
            auth: value
                .get(&ProvidesKey::try_from("auth").unwrap())
                .map(auth::AuthProvider::try_from)
                .transpose()?,
        })
    }
}
