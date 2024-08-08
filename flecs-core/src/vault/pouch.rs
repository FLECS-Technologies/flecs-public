use std::path::Path;

pub(super) trait Pouch {
    fn close(&mut self) -> super::Result<()>;
    fn open(&mut self) -> super::Result<()>;
}

pub struct SecretPouch {}

impl Pouch for SecretPouch {
    fn close(&mut self) -> crate::vault::Result<()> {
        Ok(())
    }

    fn open(&mut self) -> crate::vault::Result<()> {
        Ok(())
    }
}

impl SecretPouch {
    pub fn new(_path: &Path) -> Self {
        Self {}
    }
}

pub struct ManifestPouch {}

impl Pouch for ManifestPouch {
    fn close(&mut self) -> crate::vault::Result<()> {
        Ok(())
    }

    fn open(&mut self) -> crate::vault::Result<()> {
        Ok(())
    }
}

impl ManifestPouch {
    pub fn new(_path: &Path) -> Self {
        Self {}
    }
}

pub struct AppPouch {}

impl Pouch for AppPouch {
    fn close(&mut self) -> crate::vault::Result<()> {
        Ok(())
    }

    fn open(&mut self) -> crate::vault::Result<()> {
        Ok(())
    }
}

impl AppPouch {
    pub fn new(_path: &Path) -> Self {
        Self {}
    }
}
