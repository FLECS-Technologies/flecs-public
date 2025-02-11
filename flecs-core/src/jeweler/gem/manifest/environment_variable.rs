pub use super::{Error, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct EnvironmentVariable {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl Display for EnvironmentVariable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.value.as_ref() {
            None => write!(f, "{}", self.name),
            Some(value) => write!(f, "{}={value}", self.name),
        }
    }
}

impl TryFrom<&flecs_app_manifest::generated::manifest_3_1_0::EnvItem> for EnvironmentVariable {
    type Error = Error;

    fn try_from(
        value: &flecs_app_manifest::generated::manifest_3_1_0::EnvItem,
    ) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl FromStr for EnvironmentVariable {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = Regex::new(r"^(?<name>[a-zA-Z]+[a-zA-Z0-9_\-\\.]*)(:?=(?<value>.*))?$")?;
        let captures = regex
            .captures(s)
            .ok_or_else(|| anyhow::anyhow!("Invalid environment variable string: {s}"))?;
        let name = captures
            .name("name")
            .ok_or_else(|| anyhow::anyhow!("Invalid environment variable string: {s}"))?
            .as_str()
            .to_string();
        Ok(Self {
            name,
            value: captures
                .name("value")
                .map(|capture| capture.as_str().to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_variable_from_str_ok() {
        assert_eq!(
            EnvironmentVariable {
                name: "MY_ENV".to_string(),
                value: Some("VALUE".to_string()),
            },
            EnvironmentVariable::from_str("MY_ENV=VALUE").unwrap()
        )
    }

    #[test]
    fn environment_variable_from_str_empty_value() {
        assert_eq!(
            EnvironmentVariable {
                name: "MY_ENV".to_string(),
                value: Some(String::new()),
            },
            EnvironmentVariable::from_str("MY_ENV=").unwrap()
        )
    }

    #[test]
    fn environment_variable_from_str_no_value() {
        assert_eq!(
            EnvironmentVariable {
                name: "MY_ENV".to_string(),
                value: None,
            },
            EnvironmentVariable::from_str("MY_ENV").unwrap()
        )
    }

    #[test]
    fn environment_variable_from_str_err() {
        assert!(EnvironmentVariable::from_str("MY_INVALID/ENV=VALUE").is_err())
    }

    #[test]
    fn display_environment_variable() {
        assert_eq!(
            format!(
                "{}",
                EnvironmentVariable {
                    name: "MY_ENV".to_string(),
                    value: Some("VALUE".to_string()),
                }
            ),
            "MY_ENV=VALUE"
        );
        assert_eq!(
            format!(
                "{}",
                EnvironmentVariable {
                    name: "MY_ENV".to_string(),
                    value: None,
                }
            ),
            "MY_ENV"
        );
    }

    #[test]
    fn try_environment_variable_from_env_item() {
        let item = flecs_app_manifest::generated::manifest_3_1_0::EnvItem::from_str("MY_ENV=VALUE")
            .unwrap();
        assert_eq!(
            EnvironmentVariable::try_from(&item).unwrap(),
            EnvironmentVariable {
                name: "MY_ENV".to_string(),
                value: Some("VALUE".to_string()),
            }
        )
    }
}
