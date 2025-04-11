mod compose_impl;
use crate::jeweler::deployment::CommonDeployment;
use async_trait::async_trait;
pub use compose_impl::*;
use erased_serde::serialize_trait_object;

#[async_trait]
pub trait ComposeDeployment: CommonDeployment {
    fn dummy(&self);
}

serialize_trait_object!(ComposeDeployment);
