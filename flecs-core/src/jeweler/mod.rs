pub mod app;
pub mod deployment;
pub mod extension;
pub mod gem;
pub mod instance;
pub mod network;
pub mod volume;
pub use super::Result;
use crate::vault::pouch::AppKey;
use crate::vault::pouch::deployment::DeploymentId;
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::result;

pub trait GetDeploymentId {
    fn deployment_id(&self) -> &DeploymentId;
}

fn serialize_deployment_id<S, D, R>(
    deployment_id_provider: R,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    D: GetDeploymentId + ?Sized,
    S: Serializer,
    R: AsRef<D>,
{
    serializer.serialize_str(deployment_id_provider.as_ref().deployment_id().as_str())
}

pub trait GetAppKey {
    fn app_key(&self) -> &AppKey;
}

fn serialize_manifest_key<S, A, R>(
    manifest: R,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    A: GetAppKey,
    S: Serializer,
    R: AsRef<A>,
{
    manifest.as_ref().app_key().serialize(serializer)
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
