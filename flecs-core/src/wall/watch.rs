use crate::lore::AuthLoreRef;
use async_trait::async_trait;
use axum_extra::headers::HeaderMapExt;
use openidconnect::{HttpClientError, IssuerUrl};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;

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
    #[error("No issuer configured")]
    NoIssuer,
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    OpenIdConnect(#[from] openidconnect::DiscoveryError<HttpClientError<reqwest::Error>>),
}

#[derive(Clone, Default)]
pub struct RolesExtension(pub HashSet<String>);

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
        issuer_url: IssuerUrl,
    ) -> Result<MetaData, Error> {
        let provider_metadata =
            openidconnect::core::CoreProviderMetadata::discover_async(issuer_url, client).await?;
        let jwks: jsonwebtoken::jwk::JwkSet = client
            .get(provider_metadata.jwks_uri().url().clone())
            .send()
            .await?
            .json()
            .await?;
        Ok(MetaData {
            jwks,
            issuer_url: provider_metadata.issuer().clone(),
            time_stamp: std::time::Instant::now(),
        })
    }

    async fn jwk_and_issuer(
        &self,
        kid: &str,
    ) -> Result<(jsonwebtoken::jwk::Jwk, IssuerUrl), Error> {
        {
            let data = self.data.read().await;
            if data.issuer_url.is_none() {
                return Err(Error::NoIssuer);
            };
            match &data.meta {
                Some(meta) if meta.time_stamp.elapsed() < self.meta_cache_lifetime => {
                    return Ok((meta.jwk(kid)?, meta.issuer_url.clone()));
                }
                _ => {}
            }
        }

        let mut data = self.data.write().await;
        let Some(issuer_url) = data.issuer_url.clone() else {
            return Err(Error::NoIssuer);
        };
        let meta = Self::fetch_meta(&self.client, issuer_url).await?;
        let jwk = meta.jwk(kid);
        let issuer_url = meta.issuer_url.clone();
        data.meta.replace(meta);
        Ok((jwk?, issuer_url))
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
        validation.set_audience(&["account"]);
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
        let data = if let Some(issuer_url) = &lore.issuer_url {
            Data {
                meta: Self::fetch_meta(&client, issuer_url.clone()).await.ok(),
                issuer_url: Some(issuer_url.clone()),
            }
        } else {
            Data::default()
        };
        Ok(Self {
            data: RwLock::new(data),
            client,
            meta_cache_lifetime: lore.issuer_certificate_cache_lifetime,
        })
    }
}

#[derive(Default)]
struct Data {
    meta: Option<MetaData>,
    issuer_url: Option<IssuerUrl>,
}

struct MetaData {
    jwks: jsonwebtoken::jwk::JwkSet,
    issuer_url: IssuerUrl,
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
