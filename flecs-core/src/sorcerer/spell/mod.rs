pub(super) mod auth;
pub(super) mod license;
pub(super) mod manifest;
use flecs_console_client::apis::default_api::GetApiV2ManifestsAppVersionError;
use flecs_console_client::models::ErrorDescription;
use http::StatusCode;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum HttpError {
    ErrorDescription(ErrorDescription),
    UnknownError(serde_json::Value),
}

impl Display for HttpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::ErrorDescription(e) => write!(f, "HttpError: {:?}", e),
            HttpError::UnknownError(val) => write!(f, "HttpError::UnknownError: {:#}", val),
        }
    }
}

impl std::error::Error for HttpError {}

#[derive(Debug)]
pub enum Error {
    Http(HttpError),
    InvalidContent(String),
    NoData,
    UnexpectedData(serde_json::Value),
    UnexpectedResponse { status: StatusCode, content: String },
    Reqwest(reqwest::Error),
    ReqwestMiddleware(reqwest_middleware::Error),
    Serde(serde_json::Error),
    Io(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Http(e) => write!(f, "{}", e),
            Error::InvalidContent(s) => write!(f, "InvalidContent: {}", &s[0..200]),
            Error::UnexpectedResponse { status, content } => write!(
                f,
                "UnexpectedResponse: {}, with content {}",
                status,
                &content[0..200]
            ),
            Error::NoData => write!(f, "No data"),
            Error::UnexpectedData(val) => write!(f, "UnexpectedData: {:#}", val),
            Error::Reqwest(e) => write!(f, "error in reqwest: {}", e),
            Error::ReqwestMiddleware(e) => write!(f, "error in reqwest_middleware: {}", e),
            Error::Serde(e) => write!(f, "error in serde: {}", e),
            Error::Io(e) => write!(f, "error in IO: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Reqwest(e) => Some(e),
            Error::Serde(e) => Some(e),
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}
impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<flecs_console_client::apis::Error<GetApiV2ManifestsAppVersionError>> for Error {
    fn from(value: flecs_console_client::apis::Error<GetApiV2ManifestsAppVersionError>) -> Self {
        match value {
            flecs_console_client::apis::Error::Reqwest(e) => Error::Reqwest(e),
            flecs_console_client::apis::Error::Serde(e) => Error::Serde(e),
            flecs_console_client::apis::Error::Io(e) => Error::Io(e),
            flecs_console_client::apis::Error::ResponseError(content) => match content.entity {
                Some(GetApiV2ManifestsAppVersionError::Status403(v)) => {
                    Error::Http(HttpError::ErrorDescription(v))
                }
                Some(GetApiV2ManifestsAppVersionError::Status404(v)) => {
                    Error::Http(HttpError::ErrorDescription(ErrorDescription {
                        reason: None,
                        status_code: v.status_code,
                        status_text: v.status_text,
                    }))
                }
                Some(GetApiV2ManifestsAppVersionError::Status500(v)) => {
                    Error::Http(HttpError::ErrorDescription(v))
                }
                Some(GetApiV2ManifestsAppVersionError::UnknownValue(v)) => {
                    Error::Http(HttpError::UnknownError(v))
                }
                None => Error::Http(HttpError::ErrorDescription(ErrorDescription {
                    reason: None,
                    status_code: Some(content.status.as_u16() as i32),
                    status_text: None,
                })),
            },
            flecs_console_client::apis::Error::ReqwestMiddleware(e) => Error::ReqwestMiddleware(e),
        }
    }
}
