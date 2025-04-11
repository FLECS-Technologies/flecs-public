pub use super::{Error, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Label {
    pub label: String,
    pub value: Option<String>,
}

impl FromStr for Label {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = Regex::new(
            r"^(?<label>[a-z](?:(?:[\-.]?[a-zA-Z0-9])*[\-.]?[a-z])?)(?:=(?<value>.*))?$",
        )?;
        let captures = regex
            .captures(s)
            .ok_or_else(|| anyhow::anyhow!("Invalid label variable string: {s}"))?;
        let label = captures
            .name("label")
            .ok_or_else(|| anyhow::anyhow!("Invalid label variable string: {s}"))?
            .as_str()
            .to_string();
        Ok(Self {
            label,
            value: captures
                .name("value")
                .map(|capture| capture.as_str().to_string()),
        })
    }
}

impl TryFrom<&flecs_app_manifest::generated::manifest_3_1_0::LabelsItem> for Label {
    type Error = Error;

    fn try_from(
        value: &flecs_app_manifest::generated::manifest_3_1_0::LabelsItem,
    ) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_from_str_ok() {
        assert_eq!(
            Label {
                label: "myLabel".to_string(),
                value: Some("value".to_string()),
            },
            Label::from_str("myLabel=value").unwrap()
        )
    }

    #[test]
    fn label_from_str_empty_value() {
        assert_eq!(
            Label {
                label: "myLabel".to_string(),
                value: Some(String::new()),
            },
            Label::from_str("myLabel=").unwrap()
        )
    }

    #[test]
    fn label_from_str_no_value() {
        assert_eq!(
            Label {
                label: "myLabel".to_string(),
                value: None,
            },
            Label::from_str("myLabel").unwrap()
        )
    }

    #[test]
    fn label_from_str_err() {
        assert!(Label::from_str("my/InvalidLabel=value").is_err())
    }

    #[test]
    fn try_label_from_labels_item() {
        let item =
            flecs_app_manifest::generated::manifest_3_1_0::LabelsItem::from_str("myLabel=value")
                .unwrap();
        assert_eq!(
            Label {
                label: "myLabel".to_string(),
                value: Some("value".to_string()),
            },
            Label::try_from(&item).unwrap()
        )
    }
}
