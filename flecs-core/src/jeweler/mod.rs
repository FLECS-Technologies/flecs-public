pub mod app;
pub mod deployment;
pub mod gem;
pub mod instance;
pub mod network;
pub mod volume;
pub use super::Result;
use crate::jeweler::deployment::Deployment;
use crate::vault::pouch::AppKey;
use flecs_app_manifest::AppManifest;
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::result;
use std::sync::Arc;

fn serialize_deployment_id<S>(
    deployment: &Arc<dyn Deployment>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(deployment.id().as_str())
}

fn serialize_manifest_key<S>(
    manifest: &Arc<AppManifest>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let key = AppKey {
        name: manifest.app.to_string(),
        version: manifest.version.clone(),
    };
    key.serialize(serializer)
}

fn serialize_hashmap_values<K, T, S>(
    values: &HashMap<K, T>,
    serializer: S,
) -> result::Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    let vec_values: Vec<&_> = values.values().collect();
    let mut seq = serializer.serialize_seq(Some(vec_values.len()))?;
    for value in vec_values {
        seq.serialize_element(value)?;
    }
    seq.end()
}
