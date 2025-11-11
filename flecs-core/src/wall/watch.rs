use crate::jeweler::gem::manifest::providers::auth::Url;
use crate::lore::AuthLoreRef;
use async_trait::async_trait;
use axum_extra::headers::HeaderMapExt;
use openidconnect::{HttpClientError, IssuerUrl};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{RwLock, RwLockWriteGuard};

const INITIAL_SETUP_ROLE: &str = "tech.flecs.core.initial_setup";

pub struct Watch {
    client: reqwest::Client,
    data: RwLock<Data>,
    meta_cache_lifetime: Duration,
}

pub struct AuthToken(pub Option<String>);
#[derive(Debug, Error)]
pub enum Error {
    #[error("Key algorithm '{0}' unsupported")]
    UnsupportedAlgorithm(jsonwebtoken::jwk::KeyAlgorithm),
    #[error(transparent)]
    JsonWebToken(#[from] jsonwebtoken::errors::Error),
    #[error("No kid in header")]
    NoKid,
    #[error("Unknown kid '{0}'")]
    UnknownKid(String),
    #[error("No auth provider configured")]
    NoAuthProvider,
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    OpenIdConnect(#[from] openidconnect::DiscoveryError<HttpClientError<reqwest::Error>>),
}

#[derive(Clone, Default)]
pub struct RolesExtension(pub HashSet<String>);

impl RolesExtension {
    pub fn new_with_initial_setup_roles() -> Self {
        Self(HashSet::from([INITIAL_SETUP_ROLE.to_string()]))
    }
}

#[async_trait]
impl<S> axum::extract::FromRequestParts<S> for AuthToken
where
    S: Send + Sync,
{
    type Rejection = http::StatusCode;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;
        match headers
            .typed_try_get::<axum_extra::headers::Authorization<
                axum_extra::headers::authorization::Bearer,
            >>() {
            Ok(Some(axum_extra::headers::Authorization(bearer))) => Ok(AuthToken(Some(bearer.token().to_string()))),
            Ok(None) => Ok(AuthToken(None)),
            _ => Err(http::StatusCode::UNAUTHORIZED),
        }
    }
}

impl Watch {
    async fn fetch_meta(
        client: &reqwest::Client,
        auth_provider_meta: &AuthProviderMetaData,
    ) -> Result<MetaData, Error> {
        let (issuer_url, jwks) = match auth_provider_meta {
            AuthProviderMetaData::Oidc { issuer_url } => {
                let provider_metadata = openidconnect::core::CoreProviderMetadata::discover_async(
                    IssuerUrl::from_url(issuer_url.clone()),
                    client,
                )
                .await?;
                let jwks: jsonwebtoken::jwk::JwkSet = client
                    .get(provider_metadata.jwks_uri().url().clone())
                    .send()
                    .await?
                    .json()
                    .await?;
                (provider_metadata.issuer().url().clone(), jwks)
            }
            AuthProviderMetaData::Oauth {
                issuer_url,
                jwk_url,
            } => {
                let issuer_url: url::Url =
                    client.get(issuer_url.clone()).send().await?.json().await?;
                let jwk: jsonwebtoken::jwk::Jwk =
                    client.get(jwk_url.clone()).send().await?.json().await?;
                (issuer_url, jsonwebtoken::jwk::JwkSet { keys: vec![jwk] })
            }
        };
        Ok(MetaData {
            jwks,
            issuer_url,
            time_stamp: std::time::Instant::now(),
        })
    }

    async fn jwk_and_issuer(
        &self,
        kid: &str,
    ) -> Result<(jsonwebtoken::jwk::Jwk, IssuerUrl), Error> {
        {
            let data = self.data.read().await;
            if data.auth_provider_meta.is_none() {
                return Err(Error::NoAuthProvider);
            };
            match &data.meta {
                Some(meta) if meta.time_stamp.elapsed() < self.meta_cache_lifetime => {
                    return Ok((meta.jwk(kid)?, IssuerUrl::from_url(meta.issuer_url.clone())));
                }
                _ => {}
            }
        }

        let mut data = self.data.write().await;
        let Some(auth_meta) = &data.auth_provider_meta else {
            return Err(Error::NoAuthProvider);
        };
        let meta = Self::fetch_meta(&self.client, auth_meta).await?;
        let jwk = meta.jwk(kid);
        let issuer_url = meta.issuer_url.clone();
        data.meta.replace(meta);
        Ok((jwk?, IssuerUrl::from_url(issuer_url)))
    }

    fn algorithm_from_jwk(jwk: &jsonwebtoken::jwk::Jwk) -> Result<jsonwebtoken::Algorithm, Error> {
        Ok(
            match jwk
                .common
                .key_algorithm
                .unwrap_or(jsonwebtoken::jwk::KeyAlgorithm::HS256)
            {
                jsonwebtoken::jwk::KeyAlgorithm::HS256 => jsonwebtoken::Algorithm::HS256,
                jsonwebtoken::jwk::KeyAlgorithm::HS384 => jsonwebtoken::Algorithm::HS384,
                jsonwebtoken::jwk::KeyAlgorithm::HS512 => jsonwebtoken::Algorithm::HS512,
                jsonwebtoken::jwk::KeyAlgorithm::ES256 => jsonwebtoken::Algorithm::ES256,
                jsonwebtoken::jwk::KeyAlgorithm::ES384 => jsonwebtoken::Algorithm::ES384,
                jsonwebtoken::jwk::KeyAlgorithm::RS256 => jsonwebtoken::Algorithm::RS256,
                jsonwebtoken::jwk::KeyAlgorithm::RS384 => jsonwebtoken::Algorithm::RS384,
                jsonwebtoken::jwk::KeyAlgorithm::RS512 => jsonwebtoken::Algorithm::RS512,
                jsonwebtoken::jwk::KeyAlgorithm::PS256 => jsonwebtoken::Algorithm::PS256,
                jsonwebtoken::jwk::KeyAlgorithm::PS384 => jsonwebtoken::Algorithm::PS384,
                jsonwebtoken::jwk::KeyAlgorithm::PS512 => jsonwebtoken::Algorithm::PS512,
                jsonwebtoken::jwk::KeyAlgorithm::EdDSA => jsonwebtoken::Algorithm::EdDSA,
                alg => return Err(Error::UnsupportedAlgorithm(alg)),
            },
        )
    }

    pub async fn verify_token(&self, token: &str) -> Result<RolesExtension, Error> {
        #[derive(Debug, Serialize, Deserialize)]
        struct RealmAccess {
            roles: Vec<String>,
        }
        type Account = RealmAccess;
        #[derive(Debug, Serialize, Deserialize)]
        struct ResourceAccess {
            account: Account,
        }
        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            sub: String,
            exp: u64,
            email: String,
            preferred_username: String,
            realm_access: RealmAccess,
            resource_access: ResourceAccess,
        }
        let token_header = jsonwebtoken::decode_header(token)?;
        let kid = token_header.kid.as_deref().ok_or(Error::NoKid)?;
        let (jwk, issuer_url) = self.jwk_and_issuer(kid).await?;
        let algorithm = Self::algorithm_from_jwk(&jwk)?;
        let mut validation = jsonwebtoken::Validation::new(algorithm);
        let decoding_key = jsonwebtoken::DecodingKey::from_jwk(&jwk)?;
        validation.set_audience(&["flecs-core-api"]);
        validation.set_issuer(&[issuer_url.as_str()]);
        validation.set_required_spec_claims(&["exp", "aud", "iss", "sub"]);
        let claims = jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation)?.claims;
        Ok(RolesExtension(
            claims
                .realm_access
                .roles
                .into_iter()
                .chain(claims.resource_access.account.roles.into_iter())
                .collect(),
        ))
    }

    pub async fn new_with_lore(lore: AuthLoreRef) -> Result<Self, Error> {
        let lore = lore.as_ref().as_ref();
        let client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Self {
            data: RwLock::new(Data::default()),
            client,
            meta_cache_lifetime: lore.issuer_certificate_cache_lifetime,
        })
    }

    pub async fn data_mut(&self) -> RwLockWriteGuard<'_, Data> {
        self.data.write().await
    }

    pub async fn has_auth_provider(&self) -> bool {
        self.data.read().await.auth_provider_meta.is_some()
    }
}

#[derive(Debug)]
pub enum AuthProviderMetaData {
    Oidc { issuer_url: Url },
    Oauth { issuer_url: Url, jwk_url: Url },
}

#[derive(Debug, Default)]
pub struct Data {
    meta: Option<MetaData>,
    auth_provider_meta: Option<AuthProviderMetaData>,
}

impl Data {
    pub fn set_auth_provider_meta_data(
        &mut self,
        meta: AuthProviderMetaData,
    ) -> Option<AuthProviderMetaData> {
        self.meta = None;
        self.auth_provider_meta.replace(meta)
    }
}

#[derive(Debug)]
struct MetaData {
    jwks: jsonwebtoken::jwk::JwkSet,
    issuer_url: Url,
    time_stamp: std::time::Instant,
}

impl MetaData {
    fn jwk(&self, kid: &str) -> Result<jsonwebtoken::jwk::Jwk, Error> {
        self.jwks
            .find(kid)
            .cloned()
            .ok_or_else(|| Error::UnknownKid(kid.to_string()))
    }
}
