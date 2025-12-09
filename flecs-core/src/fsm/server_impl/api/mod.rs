use crate::fsm::server_impl::api::v2::models::AdditionalInfo;
use crate::sorcerer::providius::ForwardedHeaders;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::response::{IntoResponse, Response};
use http::header::ToStrError;
use http::request::Parts;
use http::{HeaderMap, HeaderName};
use std::num::ParseIntError;
use thiserror::Error;

pub mod v2;

#[derive(Debug, Error)]
pub enum ForwardedHeaderExtractError {
    #[error("Invalid header {header_name}: {error}")]
    NoStr {
        error: ToStrError,
        header_name: HeaderName,
    },
    #[error("Invalid header {header_name}: {error}")]
    NotU16 {
        error: ParseIntError,
        header_name: HeaderName,
    },
}

impl IntoResponse for ForwardedHeaderExtractError {
    fn into_response(self) -> Response {
        AdditionalInfo::new(self.to_string()).into_bad_request()
    }
}

impl ForwardedHeaders {
    const PROTOCOL_HEADER_NAME: &'static str = "x-forwarded-proto";
    const PORT_HEADER_NAME: &'static str = "x-forwarded-port";
    const HOST_HEADER_NAME: &'static str = "x-forwarded-host";
    fn extract_str_header(
        headers: &HeaderMap,
        header_name: HeaderName,
    ) -> Result<Option<&str>, ForwardedHeaderExtractError> {
        headers
            .get(&header_name)
            .map(|value| value.to_str())
            .transpose()
            .map_err(|error| ForwardedHeaderExtractError::NoStr {
                error,
                header_name: header_name.clone(),
            })
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for ForwardedHeaders {
    type Rejection = ForwardedHeaderExtractError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let protocol = Self::extract_str_header(
            &parts.headers,
            HeaderName::from_static(Self::PROTOCOL_HEADER_NAME),
        )?
        .map(|s| s.to_string());
        let port = Self::extract_str_header(
            &parts.headers,
            HeaderName::from_static(Self::PORT_HEADER_NAME),
        )?
        .map(|s| {
            s.parse::<u16>()
                .map_err(|error| ForwardedHeaderExtractError::NotU16 {
                    error,
                    header_name: HeaderName::from_static(Self::PORT_HEADER_NAME),
                })
        })
        .transpose()?;
        let host = Self::extract_str_header(
            &parts.headers,
            HeaderName::from_static(Self::HOST_HEADER_NAME),
        )?
        .map(|s| s.to_string());
        Ok(Self {
            protocol,
            port,
            host,
        })
    }
}
