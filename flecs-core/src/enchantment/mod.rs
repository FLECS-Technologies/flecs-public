use std::sync::Arc;

pub mod floxy;

pub struct Enchantments {
    pub floxy: Arc<floxy::Floxy>,
}

impl Enchantments {
    #[cfg(test)]
    pub fn test_instance(test_path: std::path::PathBuf) -> Enchantments {
        Self {
            floxy: Arc::new(floxy::Floxy::test_instance(test_path.join("floxy"))),
        }
    }
}
