mod importius_impl;
use crate::sorcerer::Sorcerer;
use async_trait::async_trait;
pub use importius_impl::*;
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Importius: Sorcerer + 'static {}

#[cfg(test)]
impl Sorcerer for MockImportius {}
