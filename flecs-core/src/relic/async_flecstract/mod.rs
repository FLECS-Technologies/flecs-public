pub use super::Result;
use async_compression::tokio::bufread::GzipDecoder;
use flecstract::tar::{archive, archive_single_file_as, extract, extract_single_file_as};
use std::path::{Path, PathBuf};
use tokio::io::BufReader;
use tokio_util::compat::TokioAsyncReadCompatExt;

pub async fn archive_to_file(src: &Path, dst: &Path, follow_symlinks: bool) -> Result<()> {
    let dst = dst.to_path_buf();
    let src = src.to_path_buf();
    // Potentially long synchronously blocking calls should be wrapped with tokio::task::spawn_blocking
    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::create(dst)?;
        archive(&src, file, follow_symlinks)
    })
    .await??;
    Ok(())
}

pub async fn archive_to_memory(src: &Path, follow_symlinks: bool) -> Result<Vec<u8>> {
    let data = Vec::new();
    let src = src.to_path_buf();
    // Potentially long synchronously blocking calls should be wrapped with tokio::task::spawn_blocking
    tokio::task::spawn_blocking(move || archive(&src, data, follow_symlinks)).await?
}

pub async fn archive_single_file_to_memory(
    src: &Path,
    new_file_name: String,
    follow_symlinks: bool,
) -> Result<Vec<u8>> {
    let data = Vec::new();
    let src = src.to_path_buf();
    // Potentially long synchronously blocking calls should be wrapped with tokio::task::spawn_blocking
    tokio::task::spawn_blocking(move || {
        archive_single_file_as(&src, data, new_file_name, follow_symlinks)
    })
    .await?
}

pub async fn extract_from_file(src: impl Into<PathBuf>, dst: impl Into<PathBuf>) -> Result<()> {
    let dst = dst.into();
    let src = src.into();
    // Potentially long synchronously blocking calls should be wrapped with tokio::task::spawn_blocking
    tokio::task::spawn_blocking(move || {
        let src = std::fs::File::open(&src)?;
        extract(src, dst)
    })
    .await??;
    Ok(())
}

pub async fn decompress_from_file(src: impl Into<PathBuf>, dst: impl Into<PathBuf>) -> Result<()> {
    let dst = dst.into();
    let src = src.into();
    let src = tokio::fs::File::open(&src).await?;
    let src = BufReader::new(src);
    let src = GzipDecoder::new(src);
    let archive = async_tar::Archive::new(src.compat());
    archive.unpack(dst).await?;
    Ok(())
}

pub async fn extract_from_memory(src: Vec<u8>, dst: &Path) -> Result<()> {
    let dst = dst.to_path_buf();
    // Potentially long synchronously blocking calls should be wrapped with tokio::task::spawn_blocking
    tokio::task::spawn_blocking(move || extract(src.as_slice(), dst)).await??;
    Ok(())
}

pub async fn extract_single_file_from_memory_as(src: Vec<u8>, dst: &Path) -> Result<()> {
    let dst = dst.to_path_buf();
    // Potentially long synchronously blocking calls should be wrapped with tokio::task::spawn_blocking
    tokio::task::spawn_blocking(move || extract_single_file_as(src.as_slice(), dst)).await??;
    Ok(())
}
