use crate::jeweler::gem::instance::StoredProviderReference;
use crate::jeweler::gem::manifest::DependencyKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct InstanceConfig {
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub dependencies: HashMap<DependencyKey, StoredProviderReference>,
}
