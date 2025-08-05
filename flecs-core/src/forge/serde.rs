use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, DeserializeFromStr, SerializeDisplay)]
pub struct UriWrapper(pub http::Uri);

impl FromStr for UriWrapper {
    type Err = <http::Uri as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UriWrapper(http::Uri::from_str(s)?))
    }
}

impl Display for UriWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, DeserializeFromStr, SerializeDisplay)]
pub struct EnvFilterWrapper(pub tracing_subscriber::EnvFilter);

impl FromStr for EnvFilterWrapper {
    type Err = <tracing_subscriber::EnvFilter as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(EnvFilterWrapper(tracing_subscriber::EnvFilter::from_str(
            s,
        )?))
    }
}

impl Display for EnvFilterWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
