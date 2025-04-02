use crate::sorcerer::exportius::Exportius;
use crate::sorcerer::Sorcerer;
use async_trait::async_trait;

#[derive(Default)]
pub struct ExportiusImpl;

impl Sorcerer for ExportiusImpl {}

#[async_trait]
impl Exportius for ExportiusImpl {}
