pub use anyhow::Result;

pub mod tar {
    use super::Result;
    use anyhow::anyhow;
    use std::io::{Read, Write};
    use std::path::Path;
    use tar::EntryType;

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

    pub fn archive_single_file_as<W, P, S>(
        src: P,
        mut destination: W,
        new_file_name: S,
        follow_symlinks: bool,
    ) -> Result<W>
    where
        W: Write,
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let meta = std::fs::metadata(&src)?;
        anyhow::ensure!(
            !meta.is_dir(),
            "Expected single file to archive but received directory {:?}",
            src.as_ref()
        );
        let mut archive = tar::Builder::new(&mut destination);
        archive.follow_symlinks(follow_symlinks);
        archive.append_path_with_name(&src, new_file_name.as_ref())?;
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

    pub fn extract_single_file_as<R, P>(src: R, dst: P) -> Result<()>
    where
        R: Read,
        P: AsRef<Path>,
    {
        let mut archive = tar::Archive::new(src);
        let mut entries = archive.entries()?;
        match entries.next() {
            None => anyhow::bail!("No entry in archive present"),
            Some(Ok(mut entry)) => {
                anyhow::ensure!(
                    entry.header().entry_type() == EntryType::Regular,
                    "First entry in archive is not a regular file"
                );
                entry.unpack(dst)?;
                Ok(())
            }
            Some(Err(e)) => anyhow::bail!(e),
        }
    }
}
