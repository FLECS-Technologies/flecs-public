// 'docker login' and 'docker logout' is not necessary, the calls just take a bollard::auth::DockerCredentials
pub mod container;
pub mod image;
pub mod network;
pub mod volume;

pub use super::{Error, Result};
use crate::quest::{Progress, SyncQuest};
use axum::body::Bytes;
use futures_util::{Stream, StreamExt};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

#[derive(Debug, Eq, PartialEq)]
pub enum ByteStatus {
    Complete(usize),
    Partial(usize),
    Error(usize),
}

pub struct ByteResult<T> {
    pub status: Arc<Mutex<ByteStatus>>,
    pub handle: JoinHandle<Result<T>>,
}

async fn write_stream_to_file<T>(quest: SyncQuest, stream: T, path: impl AsRef<Path>) -> Result<()>
where
    T: Stream<Item = Result<Bytes, bollard::errors::Error>> + Unpin,
{
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .await?;
    write_stream_to_writer(quest, stream, &mut file).await?;
    Ok(())
}

async fn write_stream_to_memory<T>(quest: SyncQuest, stream: T) -> Result<Vec<u8>>
where
    T: Stream<Item = Result<Bytes, bollard::errors::Error>> + Unpin,
{
    let mut buffer = Vec::new();
    write_stream_to_writer(quest, stream, &mut buffer).await?;
    Ok(buffer)
}

async fn write_stream_to_writer<T, W>(quest: SyncQuest, mut stream: T, writer: &mut W) -> Result<()>
where
    T: Stream<Item = Result<Bytes, bollard::errors::Error>> + Unpin,
    W: AsyncWriteExt + Unpin,
{
    let mut total_bytes = 0;
    while let Some(data) = stream.next().await {
        let data = data?;
        writer.write_all(data.as_ref()).await?;
        total_bytes += data.len();
        quest.lock().await.progress = Some(Progress {
            current: total_bytes as u64,
            total: None,
        });
    }
    Ok(())
}

#[derive(Debug, Default)]
pub struct Status {
    pub progress_map: BTreeMap<String, String>,
    pub finished: bool,
    pub errors: Vec<Error>,
}
