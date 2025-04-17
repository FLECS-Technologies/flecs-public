use async_trait::async_trait;
use axum_extra::extract::Multipart;
use axum_extra::extract::multipart::MultipartError;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tracing::debug;

#[async_trait]
pub trait MultipartExt: Sized {
    async fn write_file(self, path_buf: PathBuf) -> Result<PathBuf, WriteMultipartError>;
}

#[derive(thiserror::Error, Debug)]
pub enum WriteMultipartError {
    #[error("No file name in header 'Content-Disposition'")]
    NoFileName,
    #[error("No data received")]
    NoData,
    #[error("Error receiving file as multipart: {0}")]
    Multipart(#[from] MultipartError),
    #[error("IO error writing file: {0}")]
    IO(#[from] std::io::Error),
}

#[async_trait]
impl MultipartExt for Multipart {
    async fn write_file(mut self, path_buf: PathBuf) -> Result<PathBuf, WriteMultipartError> {
        let now = std::time::Instant::now();
        let Some(mut field) = self.next_field().await? else {
            return Err(WriteMultipartError::NoData);
        };
        let file_name = field.file_name().ok_or(WriteMultipartError::NoFileName)?;
        tokio::fs::create_dir_all(&path_buf).await?;
        let file_path = path_buf.join(file_name);
        let mut file = tokio::fs::File::create(&file_path).await?;
        let mut received_bytes = 0;
        while let Some(chunk) = field.chunk().await? {
            file.write_all(chunk.as_ref()).await?;
            received_bytes += chunk.len() as u128;
        }
        let elapsed_ms = now.elapsed().as_millis();
        debug!(
            "Received {file_path:?} ({} bytes in {} ms = {} MB/s)",
            received_bytes,
            elapsed_ms,
            received_bytes / elapsed_ms / 1000
        );
        Ok(file_path)
    }
}
