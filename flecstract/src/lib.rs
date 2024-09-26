pub use anyhow::Result;

pub mod tar {
    use super::Result;
    use anyhow::anyhow;
    use std::io::{Read, Write};
    use std::path::Path;

    pub fn archive<W, P>(src: P, mut destination: W, follow_symlinks: bool) -> Result<W>
    where
        W: Write,
        P: AsRef<Path>,
    {
        let meta = std::fs::metadata(&src)?;
        let mut archive = tar::Builder::new(&mut destination);
        archive.follow_symlinks(follow_symlinks);
        if meta.is_dir() {
            for entry in std::fs::read_dir(&src)? {
                let entry = entry?;
                let relative_path = entry.file_name();
                if entry.metadata()?.is_dir() {
                    archive.append_dir_all(relative_path, entry.path())?;
                } else {
                    archive.append_path_with_name(entry.path(), relative_path)?;
                }
            }
        } else {
            let file_name = src
                .as_ref()
                .file_name()
                .ok_or(anyhow!("File has no file name"))?;
            archive.append_path_with_name(&src, file_name)?;
        }
        archive.into_inner()?;
        Ok(destination)
    }

    pub fn extract<R, P>(src: R, dst: P) -> Result<()>
    where
        R: Read,
        P: AsRef<Path>,
    {
        tar::Archive::new(src).unpack(dst)?;
        Ok(())
    }
}
