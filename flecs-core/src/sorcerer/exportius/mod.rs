mod exportius_impl;
use crate::sorcerer::Sorcerer;
use async_trait::async_trait;
pub use exportius_impl::*;
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Exportius: Sorcerer + 'static {}

#[cfg(test)]
impl Sorcerer for MockExportius {}
