use crate::sorcerer::importius::Importius;
use crate::sorcerer::Sorcerer;
use async_trait::async_trait;

#[derive(Default)]
pub struct ImportiusImpl;

impl Sorcerer for ImportiusImpl {}

#[async_trait]
impl Importius for ImportiusImpl {}
