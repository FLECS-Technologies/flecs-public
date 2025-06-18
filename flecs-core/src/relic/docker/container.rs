pub use super::{Error, Result};
use crate::quest::{Progress, SyncQuest};
use crate::relic::async_flecstract::{
    archive_single_file_to_memory, archive_to_memory, extract_from_memory,
    extract_single_file_from_memory_as,
};
use crate::relic::docker::{map_bollard_error, write_stream_to_file, write_stream_to_memory};
use async_compression::tokio::bufread::{GzipDecoder, GzipEncoder};
use axum::body::Bytes;
use bollard::Docker;
use bollard::container::{
    Config, CreateContainerOptions, DownloadFromContainerOptions, ListContainersOptions, LogOutput,
    LogsOptions, RemoveContainerOptions, StopContainerOptions, UploadToContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::models::{ContainerInspectResponse, ContainerSummary};
use futures::Stream;
use futures_util::stream::StreamExt;
use serde::Serialize;
use std::hash::Hash;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncWriteExt, BufReader, ReadBuf};
use tokio::join;
use tokio_util::codec;
use tracing::{error, warn};

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use bollard::exec::{CreateExecOptions, StartExecResults};
/// use flecs_core::relic::docker::container::exec;
/// use futures_util::StreamExt;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let create_options = CreateExecOptions {
///         attach_stderr: Some(true),
///         attach_stdout: Some(true),
///         cmd: Some(vec!["ls", "-la"]),
///         ..CreateExecOptions::default()
///     };
///     let result = exec(docker_client, "flecs-webapp", create_options, None)
///         .await
///         .unwrap();
///     match result {
///         StartExecResults::Attached { mut output, .. } => {
///             while let Some(output) = output.next().await {
///                 match output {
///                     Ok(msg) => {
///                         println!("{msg}")
///                     }
///                     Err(e) => {
///                         eprintln!("Error: {e}")
///                     }
///                 }
///             }
///         }
///         _ => {
///             panic!("Expected attached result")
///         }
///     }
/// }
/// # )
/// ```
pub async fn exec<T>(
    docker_client: Arc<Docker>,
    container_name: &str,
    create_options: CreateExecOptions<T>,
    start_options: Option<StartExecOptions>,
) -> Result<StartExecResults>
where
    T: Into<String> + serde::ser::Serialize,
{
    let exec = docker_client
        .create_exec(container_name, create_options)
        .await
        .map_err(map_bollard_error)?;
    docker_client
        .start_exec(&exec.id, start_options)
        .await
        .map_err(map_bollard_error)
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use bollard::container::StopContainerOptions;
/// use flecs_core::relic::docker::container::stop;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     stop(
///         docker_client,
///         "8c9a8332827c",
///         Some(StopContainerOptions { t: 20 }),
///     )
///     .await
///     .unwrap();
/// }
/// # )
/// ```
pub async fn stop(
    docker_client: Arc<Docker>,
    container_name: &str,
    options: Option<StopContainerOptions>,
) -> Result<()> {
    docker_client
        .stop_container(container_name, options)
        .await
        .map_err(map_bollard_error)
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::relic::docker::container::start;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     start(docker_client, "8c9a8332827c").await.unwrap();
/// }
/// # )
/// ```
pub async fn start(docker_client: Arc<Docker>, container_name: &str) -> Result<()> {
    docker_client
        .start_container::<&str>(container_name, None)
        .await
        .map_err(map_bollard_error)
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use bollard::container::Config;
/// use bollard::container::CreateContainerOptions;
/// use flecs_core::relic::docker::container::create;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     create(
///         docker_client,
///         Some(CreateContainerOptions {
///             name: "my-container",
///             platform: None,
///         }),
///         Config {
///             hostname: Some("my.container"),
///             ..Config::default()
///         },
///     )
///     .await
///     .unwrap();
/// }
/// # )
/// ```
pub async fn create<T, Z>(
    docker_client: Arc<Docker>,
    options: Option<CreateContainerOptions<T>>,
    config: Config<Z>,
) -> Result<String>
where
    T: Into<String> + Eq + Hash + Serialize,
    Z: Into<String> + Hash + Eq + Serialize,
{
    let response = docker_client
        .create_container(options, config)
        .await
        .map_err(map_bollard_error)?;
    for warning in response.warnings {
        warn!(
            "Received warning during creation of container {}: {warning}",
            response.id
        )
    }
    Ok(response.id)
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use bollard::container::ListContainersOptions;
/// use flecs_core::relic::docker::container::list;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let options: Option<ListContainersOptions<&str>> = None;
///     let result = list(docker_client, options).await.unwrap();
///     println!("{:#?}", result);
/// }
/// # )
/// ```
pub async fn list<T>(
    docker_client: Arc<Docker>,
    options: Option<ListContainersOptions<T>>,
) -> Result<Vec<ContainerSummary>>
where
    T: Into<String> + Eq + Hash + Serialize,
{
    docker_client
        .list_containers(options)
        .await
        .map_err(map_bollard_error)
}

/// Returns Ok(true) if the specified container was removed, Ok(false) if the container did not exist
/// and an error otherwise
/// # Example
/// ```no_run
/// use bollard::Docker;
/// use bollard::container::RemoveContainerOptions;
/// use flecs_core::relic::docker::container::remove;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let options: Option<RemoveContainerOptions> = None;
///     let was_container_removed = remove(docker_client, options, "8c9a8332827c")
///         .await
///         .unwrap();
/// }
/// # )
/// ```
pub async fn remove(
    docker_client: Arc<Docker>,
    options: Option<RemoveContainerOptions>,
    container_name: &str,
) -> Result<bool> {
    match docker_client
        .remove_container(container_name, options)
        .await
    {
        Ok(_) => Ok(true),
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) => Ok(false),
        Err(e) => Err(map_bollard_error(e)),
    }
}

pub enum Data {
    File(File),
    InMemory(Vec<u8>),
}
//TODO: Differentiate between copying single files and directories
/// # Examples
/// ## Copy a directory to a container
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::quest::Quest;
/// use flecs_core::relic::docker::container::copy_to;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let quest = Quest::new_synced("Copy to container".to_string());
///     let src = Path::new("/directory/on/host");
///     let dst = Path::new("/directory/on/container");
///     copy_to(docker_client, quest, src, dst, "my-container", false, false)
///         .await
///         .unwrap();
/// }
/// # )
/// ```
/// ## Copy a single file to a container
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::quest::Quest;
/// use flecs_core::relic::docker::container::copy_to;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let quest = Quest::new_synced("Copy to container".to_string());
///     let src = Path::new("/file/on/host.txt");
///     let dst = Path::new("/file/on/container.txt");
///     copy_to(docker_client, quest, src, dst, "my-container", false, true)
///         .await
///         .unwrap();
/// }
/// # )
/// ```
pub async fn copy_to(
    docker_client: Arc<Docker>,
    quest: SyncQuest,
    src_path: &Path,
    dst_path: &Path,
    container_name: &str,
    follow_symlinks: bool,
    is_dst_file_path: bool,
) -> Result<()> {
    // TODO: Create subquests, use streamed/async archiving
    let archive = if is_dst_file_path {
        archive_single_file_to_memory(
            src_path,
            dst_path
                .file_name()
                .ok_or_else(|| {
                    anyhow::anyhow!("Expected destination path '{dst_path:?}' to be a file")
                })?
                .to_string_lossy()
                .to_string(),
            true,
        )
        .await?
    } else {
        archive_to_memory(src_path, follow_symlinks).await?
    };
    let dst_path = if is_dst_file_path {
        dst_path
            .parent()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Expected destination path '{dst_path:?}' to have a parent directory"
                )
            })?
            .to_string_lossy()
            .to_string()
    } else {
        dst_path.to_string_lossy().to_string()
    };
    let archive = Data::InMemory(archive);
    copy_archive_to(
        docker_client,
        quest,
        Some(UploadToContainerOptions {
            path: dst_path.as_str(),
            no_overwrite_dir_non_dir: "false",
        }),
        archive,
        container_name,
    )
    .await
}

pub async fn copy_archive_file_to(
    docker_client: Arc<Docker>,
    quest: SyncQuest,
    archive_path: PathBuf,
    extract_path: PathBuf,
    container_name: &str,
) -> Result<()> {
    let options = Some(UploadToContainerOptions {
        path: extract_path.to_string_lossy(),
        ..Default::default()
    });
    let file = File::open(archive_path).await?;
    copy_archive_to(
        docker_client,
        quest,
        options,
        Data::File(file),
        container_name,
    )
    .await
}

async fn copy_archive_to<T>(
    docker_client: Arc<Docker>,
    quest: SyncQuest,
    options: Option<UploadToContainerOptions<T>>,
    archive: Data,
    container_name: &str,
) -> Result<()>
where
    T: Into<String> + Eq + Hash + Serialize,
{
    match archive {
        Data::File(file) => {
            let total = file.metadata().await.map(|meta| meta.len()).ok();
            quest.lock().await.progress = Some(Progress { total, current: 0 });
            let byte_stream = codec::FramedRead::new(file, codec::BytesCodec::new())
                .map(|r| r.unwrap().freeze())
                .then(move |data| {
                    let quest = quest.clone();
                    async move {
                        quest.lock().await.add_progress(data.len() as u64);
                        data
                    }
                });
            docker_client
                .upload_to_container_streaming(container_name, options, byte_stream)
                .await
                .map_err(map_bollard_error)?
        }
        Data::InMemory(data) => {
            quest.lock().await.progress = Some(Progress {
                total: Some(data.len() as u64),
                current: 0,
            });
            let reader = Cursor::new(data);
            let byte_stream = codec::FramedRead::new(reader, codec::BytesCodec::new())
                .map(|r| r.unwrap().freeze())
                .then(move |data| {
                    let quest = quest.clone();
                    async move {
                        quest.lock().await.add_progress(data.len() as u64);
                        data
                    }
                });
            docker_client
                .upload_to_container_streaming(container_name, options, byte_stream)
                .await
                .map_err(map_bollard_error)?
        }
    };
    Ok(())
}

pub async fn copy_archive_from(
    quest: SyncQuest,
    docker_client: Arc<Docker>,
    src: &Path,
    container_name: &str,
) -> Result<Vec<u8>> {
    let options = Some(DownloadFromContainerOptions {
        path: src.to_string_lossy().to_string(),
    });
    let container_name = container_name.to_string();
    let (.., result) = quest
        .lock()
        .await
        .create_sub_quest(
            format!("Download archive {src:?} from container {container_name}"),
            |quest| async move {
                let result = docker_client.download_from_container(&container_name, options);
                write_stream_to_memory(quest, result).await
            },
        )
        .await;
    result.await
}

/// Copies the content in the container at the given path into an archive at the specified location.
pub async fn copy_archive_to_file(
    quest: SyncQuest,
    docker_client: Arc<Docker>,
    src: &Path,
    dst: PathBuf,
    container_name: &str,
) -> Result<()> {
    let options = Some(DownloadFromContainerOptions {
        path: src.to_string_lossy().to_string(),
    });
    let container_name = container_name.to_string();
    let (.., result) = quest
        .lock()
        .await
        .create_sub_quest(
            format!("Download archive {src:?} from container {container_name} to {dst:?}"),
            |quest| async move {
                let result = docker_client.download_from_container(&container_name, options);
                match write_stream_to_file(quest, result, &dst).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        if let Err(e) = tokio::fs::remove_file(&dst).await {
                            error!(
                                "Failed to remove {dst:?} after failing to download archive: {e}"
                            );
                        }
                        Err(e)
                    }
                }
            },
        )
        .await;
    result.await
}

/// The argument 'is_dst_file_path' should be set to true if 'dst' denotes a file,
/// otherwise 'dst' will be interpreted as a directory and 'src' copied into it.
/// # Examples
/// ## Copy directory
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::quest::Quest;
/// use flecs_core::relic::docker::container::copy_from;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let quest = Quest::new_synced("Docker copy from".to_string());
///     let dst = Path::new("/path/on/host");
///     let src = Path::new("/path/on/container");
///     copy_from(quest, docker_client, src, dst, "my-container", false)
///         .await
///         .unwrap();
/// }
/// # )
/// ```
/// ## Copy file into directory (is_dst_file_path = false)
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::quest::Quest;
/// use flecs_core::relic::docker::container::copy_from;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let quest = Quest::new_synced("Docker copy from".to_string());
///     let dst = Path::new("/path/on/host");
///     let src = Path::new("/path/on/container.file");
///     copy_from(quest, docker_client, src, dst, "my-container", false)
///         .await
///         .unwrap();
/// }
/// # )
/// ```
/// ## Copy file with different name (is_dst_file_path = true)
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::quest::Quest;
/// use flecs_core::relic::docker::container::copy_from;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let quest = Quest::new_synced("Docker copy from".to_string());
///     let dst = Path::new("/path/on/host.file");
///     let src = Path::new("/path/on/container.file");
///     copy_from(quest, docker_client, src, dst, "my-container", true)
///         .await
///         .unwrap();
/// }
/// # )
/// ```
pub async fn copy_from(
    quest: SyncQuest,
    docker_client: Arc<Docker>,
    src: &Path,
    dst: &Path,
    container_name: &str,
    is_dst_file_path: bool,
) -> Result<()> {
    let container_name = container_name.to_string();
    let src = src.to_path_buf();
    let dst = dst.to_path_buf();
    let archive =
        quest
            .lock()
            .await
            .create_sub_quest(
                format!("Download archive {src:?} from {container_name}"),
                |quest| async move {
                    copy_archive_from(quest, docker_client, &src, &container_name).await
                },
            )
            .await
            .2;
    let result = quest
        .lock()
        .await
        .create_sub_quest(format!("Extract archive to {dst:?}"), |_quest| async move {
            // TODO: Use streamed/async extracting
            if is_dst_file_path {
                extract_single_file_from_memory_as(archive.await?, &dst).await
            } else {
                extract_from_memory(archive.await?, &dst).await
            }
        })
        .await
        .2;
    result.await
}

struct AsyncReadStream<R>(R);

impl<R: AsyncRead + Unpin> Stream for AsyncReadStream<R> {
    type Item = Bytes;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut data = vec![0; 256];
        let mut buf = ReadBuf::new(&mut data);
        let mut pinned = Pin::new(&mut self.0);
        match pinned.as_mut().poll_read(cx, &mut buf) {
            Poll::Ready(Ok(_)) => {
                let data = Bytes::from(buf.filled().to_vec());
                match data.len() {
                    0 => Poll::Ready(None),
                    _ => Poll::Ready(Some(data)),
                }
            }
            Poll::Ready(Err(_)) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::quest::Quest;
/// use flecs_core::relic::docker::container::upload_gzip_file_streamed;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let dst = Path::new("/tmp/test");
///     let src = Path::new("/tmp/test-archive.tar.gz");
///     let container_name = "beautiful_haibt";
///     let quest = Quest::new_synced(format!(
///         "Upload {src:?} to {dst:?} on container {container_name}"
///     ));
///
///     upload_gzip_file_streamed(docker_client, quest, src, dst, container_name)
///         .await
///         .unwrap();
/// }
/// # )
/// ```
pub async fn upload_gzip_file_streamed(
    docker_client: Arc<Docker>,
    quest: SyncQuest,
    src: &Path,
    dst: &Path,
    container_name: &str,
) -> Result<()> {
    let file = File::open(src).await?;
    let total = Some(file.metadata().await?.len());
    let (read_file, mut write_file) = tokio::io::simplex(1024);
    let (read, mut write) = tokio::io::simplex(1024);
    let read_result = quest
        .lock()
        .await
        .create_sub_quest(format!("Reading file {src:?}"), |quest| async move {
            quest.lock().await.progress = Some(Progress { total, current: 0 });
            let mut reader = codec::FramedRead::new(file, codec::BytesCodec::new());
            while let Some(data) = reader.next().await {
                match data {
                    Err(e) => {
                        write_file.shutdown().await?;
                        anyhow::bail!(e);
                    }
                    Ok(data) => {
                        let len = data.len();
                        if let Err(e) = write_file.write_all(data.as_ref()).await {
                            write_file.shutdown().await?;
                            anyhow::bail!(e);
                        };
                        quest.lock().await.add_progress(len as u64);
                    }
                }
            }
            write_file.shutdown().await?;
            Ok(())
        })
        .await
        .2;
    let decompress_result = quest
        .lock()
        .await
        .create_sub_quest("Decompress file".to_string(), |quest| async move {
            quest.lock().await.progress = Some(Progress {
                current: 0,
                total: None,
            });
            let mut reader = codec::FramedRead::new(
                GzipDecoder::new(BufReader::new(read_file)),
                codec::BytesCodec::new(),
            );
            while let Some(data) = reader.next().await {
                match data {
                    Ok(data) => {
                        let written = data.len();
                        if let Err(e) = write.write_all(data.as_ref()).await {
                            write.shutdown().await?;
                            anyhow::bail!(e)
                        }
                        quest.lock().await.add_progress(written as u64);
                    }
                    Err(e) => {
                        write.shutdown().await?;
                        anyhow::bail!(e)
                    }
                }
            }
            write.shutdown().await?;
            Ok(())
        })
        .await
        .2;
    let upload_result = quest
        .lock()
        .await
        .create_sub_quest(format!("Upload to container {container_name}"), |_quest| {
            let path = dst.to_string_lossy().to_string();
            let container_name = container_name.to_string();
            async move {
                let stream = AsyncReadStream(read);
                docker_client
                    .upload_to_container_streaming(
                        &container_name,
                        Some(UploadToContainerOptions {
                            path,
                            no_overwrite_dir_non_dir: "false".to_string(),
                        }),
                        stream,
                    )
                    .await
                    .map_err(map_bollard_error)?;
                Ok::<(), anyhow::Error>(())
            }
        })
        .await
        .2;
    let (read_result, upload_result, decompress_result) =
        join!(read_result, upload_result, decompress_result);
    let _ = (read_result?, upload_result?, decompress_result?);
    Ok(())
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::quest::Quest;
/// use flecs_core::relic::docker::container::download_gzip_streamed;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let src = Path::new("/tmp/test");
///     let dst = Path::new("/tmp/test-archive.tar.gz");
///     let container_name = "beautiful_haibt";
///     let quest = Quest::new_synced(format!("Download {src:?} from {container_name} to {dst:?}"));
///
///     download_gzip_streamed(docker_client, quest, src, dst, container_name)
///         .await
///         .unwrap();
/// }
/// # )
/// ```
pub async fn download_gzip_streamed(
    docker_client: Arc<Docker>,
    quest: SyncQuest,
    src: &Path,
    dst: &Path,
    container_name: &str,
) -> Result<()> {
    let (read_download, mut write_download) = tokio::io::simplex(1024);
    let (read, mut write) = tokio::io::simplex(1024);
    let download_result = quest
        .lock()
        .await
        .create_sub_quest(
            format!("Downloading {src:?} from {container_name}"),
            |quest| {
                let options = Some(DownloadFromContainerOptions {
                    path: src.to_string_lossy().to_string(),
                });
                let container_name = container_name.to_string();
                async move {
                    quest.lock().await.progress = Some(Progress {
                        total: None,
                        current: 0,
                    });
                    let mut download_stream =
                        docker_client.download_from_container(&container_name, options);
                    while let Some(data) = download_stream.next().await {
                        match data.map_err(map_bollard_error) {
                            Err(e) => {
                                write_download.shutdown().await?;
                                anyhow::bail!(e);
                            }
                            Ok(data) => {
                                let len = data.len();
                                if let Err(e) = write_download.write_all(data.as_ref()).await {
                                    write_download.shutdown().await?;
                                    anyhow::bail!(e);
                                };
                                quest.lock().await.add_progress(len as u64);
                            }
                        }
                    }
                    write_download.shutdown().await?;
                    Ok(())
                }
            },
        )
        .await
        .2;
    let compress_result = quest
        .lock()
        .await
        .create_sub_quest("Compress download".to_string(), |quest| async move {
            quest.lock().await.progress = Some(Progress {
                current: 0,
                total: None,
            });
            let mut reader = codec::FramedRead::new(
                GzipEncoder::new(BufReader::new(read_download)),
                codec::BytesCodec::new(),
            );
            while let Some(data) = reader.next().await {
                match data {
                    Ok(data) => {
                        let written = data.len();
                        if let Err(e) = write.write_all(data.as_ref()).await {
                            write.shutdown().await?;
                            anyhow::bail!(e)
                        }
                        quest.lock().await.add_progress(written as u64);
                    }
                    Err(e) => {
                        write.shutdown().await?;
                        anyhow::bail!(e)
                    }
                }
            }
            write.shutdown().await?;
            Ok(())
        })
        .await
        .2;
    let save_result = quest
        .lock()
        .await
        .create_sub_quest(format!("Write to disk at {dst:?}"), |quest| {
            let dst = dst.to_path_buf();
            async move {
                quest.lock().await.progress = Some(Progress {
                    total: None,
                    current: 0,
                });
                let mut file = File::create(&dst).await?;
                let mut stream = AsyncReadStream(read);
                while let Some(data) = stream.next().await {
                    match file.write_all(data.as_ref()).await {
                        Ok(()) => {
                            quest.lock().await.add_progress(data.len() as u64);
                        }
                        Err(e) => {
                            if let Err(e) = tokio::fs::remove_file(&dst).await {
                                error!("Could not remove {dst:?}: {e}");
                            }
                            anyhow::bail!(e);
                        }
                    }
                }
                Ok(())
            }
        })
        .await
        .2;
    let (save_result, download_result, compress_result) =
        join!(save_result, download_result, compress_result);
    let _ = (save_result?, download_result?, compress_result?);
    Ok(())
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::relic::docker::container::inspect;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let container_name = "beautiful_haibt";
///
///     println!(
///         "{:?}",
///         inspect(docker_client, container_name)
///             .await
///             .unwrap()
///             .unwrap()
///     );
/// }
/// # )
/// ```
pub async fn inspect(
    docker_client: Arc<Docker>,
    container: &str,
) -> Result<Option<ContainerInspectResponse>> {
    match docker_client.inspect_container(container, None).await {
        Ok(container) => Ok(Some(container)),
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) => Ok(None),
        Err(e) => Err(map_bollard_error(e)),
    }
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::quest::Quest;
/// use flecs_core::relic::docker::container::logs;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let container_name = "beautiful_haibt";
///
///     println!(
///         "{:?}",
///         logs(
///             docker_client,
///             Quest::new_synced("Get logs".to_string()),
///             container_name
///         )
///         .await
///         .unwrap()
///     );
/// }
/// # )
/// ```
pub async fn logs(
    docker_client: Arc<Docker>,
    quest: SyncQuest,
    container: &str,
) -> Result<(String, String)> {
    let mut stream = docker_client.logs(
        container,
        Some(LogsOptions {
            stderr: true,
            stdout: true,
            tail: "all",
            ..LogsOptions::default()
        }),
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    while let Some(data) = stream.next().await {
        match data.map_err(map_bollard_error)? {
            LogOutput::StdErr { message } => {
                stderr.push(String::from_utf8_lossy(&message).to_string());
                quest.lock().await.add_progress(message.len() as u64);
            }
            LogOutput::StdOut { message } => {
                stdout.push(String::from_utf8_lossy(&message).to_string());
                quest.lock().await.add_progress(message.len() as u64);
            }
            _ => {}
        }
    }
    Ok((stdout.join("\n"), stderr.join("\n")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::ClientVersion;

    fn example_container_inspect() -> serde_json::Value {
        // Example taken from https://docs.docker.com/reference/api/engine/version/v1.46/#tag/Container/operation/ContainerInspect
        serde_json::json!({
          "AppArmorProfile": "",
          "Args": [
            "-c",
            "exit 9"
          ],
          "Config": {
            "AttachStderr": true,
            "AttachStdin": false,
            "AttachStdout": true,
            "Cmd": [
              "/bin/sh",
              "-c",
              "exit 9"
            ],
            "Domainname": "",
            "Env": [
              "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
            ],
            "Healthcheck": {
              "Test": [
                "CMD-SHELL",
                "exit 0"
              ]
            },
            "Hostname": "ba033ac44011",
            "Image": "ubuntu",
            "Labels": {
              "com.example.vendor": "Acme",
              "com.example.license": "GPL",
              "com.example.version": "1.0"
            },
            "MacAddress": "",
            "NetworkDisabled": false,
            "OpenStdin": false,
            "StdinOnce": false,
            "Tty": false,
            "User": "",
            "Volumes": {
              "/volumes/data": {}
            },
            "WorkingDir": "",
            "StopSignal": "SIGTERM",
            "StopTimeout": 10
          },
          "Created": "2015-01-06T15:47:31.485331387Z",
          "Driver": "overlay2",
          "ExecIDs": [
            "b35395de42bc8abd327f9dd65d913b9ba28c74d2f0734eeeae84fa1c616a0fca",
            "3fc1232e5cd20c8de182ed81178503dc6437f4e7ef12b52cc5e8de020652f1c4"
          ],
          "HostConfig": {
            "MaximumIOps": 0,
            "MaximumIOBps": 0,
            "BlkioWeight": 0,
            "ContainerIDFile": "",
            "CpusetCpus": "",
            "CpusetMems": "",
            "CpuPercent": 80,
            "CpuShares": 0,
            "CpuPeriod": 100000,
            "CpuRealtimePeriod": 1000000,
            "CpuRealtimeRuntime": 10000,
            "Devices": [],
            "DeviceRequests": [
              {
                "Driver": "nvidia",
                "Count": -1,
                "DeviceIDs\"": [
                  "0",
                  "1",
                  "GPU-fef8089b-4820-abfc-e83e-94318197576e"
                ],
                "Capabilities": [
                  [
                    "gpu",
                    "nvidia",
                    "compute"
                  ]
                ],
                "Options": {
                  "property1": "string",
                  "property2": "string"
                }
              }
            ],
            "IpcMode": "",
            "Memory": 0,
            "MemorySwap": 0,
            "MemoryReservation": 0,
            "OomKillDisable": false,
            "OomScoreAdj": 500,
            "NetworkMode": "bridge",
            "PidMode": "",
            "PortBindings": {},
            "Privileged": false,
            "ReadonlyRootfs": false,
            "PublishAllPorts": false,
            "RestartPolicy": {
              "MaximumRetryCount": 2,
              "Name": "on-failure"
            },
            "LogConfig": {
              "Type": "json-file"
            },
            "Sysctls": {
              "net.ipv4.ip_forward": "1"
            },
            "Ulimits": [],
            "VolumeDriver": "",
            "ShmSize": 67108864
          },
          "HostnamePath": "/var/lib/docker/containers/ba033ac4401106a3b513bc9d639eee123ad78ca3616b921167cd74b20e25ed39/hostname",
          "HostsPath": "/var/lib/docker/containers/ba033ac4401106a3b513bc9d639eee123ad78ca3616b921167cd74b20e25ed39/hosts",
          "LogPath": "/var/lib/docker/containers/1eb5fabf5a03807136561b3c00adcd2992b535d624d5e18b6cdc6a6844d9767b/1eb5fabf5a03807136561b3c00adcd2992b535d624d5e18b6cdc6a6844d9767b-json.log",
          "Id": "ba033ac4401106a3b513bc9d639eee123ad78ca3616b921167cd74b20e25ed39",
          "Image": "04c5d3b7b0656168630d3ba35d8889bd0e9caafcaeb3004d2bfbc47e7c5d35d2",
          "MountLabel": "",
          "Name": "/boring_euclid",
          "NetworkSettings": {
            "Bridge": "",
            "SandboxID": "",
            "HairpinMode": false,
            "LinkLocalIPv6Address": "",
            "LinkLocalIPv6PrefixLen": 0,
            "SandboxKey": "",
            "EndpointID": "",
            "Gateway": "",
            "GlobalIPv6Address": "",
            "GlobalIPv6PrefixLen": 0,
            "IPAddress": "",
            "IPPrefixLen": 0,
            "IPv6Gateway": "",
            "MacAddress": "",
            "Networks": {
              "bridge": {
                "NetworkID": "7ea29fc1412292a2d7bba362f9253545fecdfa8ce9a6e37dd10ba8bee7129812",
                "EndpointID": "7587b82f0dada3656fda26588aee72630c6fab1536d36e394b2bfbcf898c971d",
                "Gateway": "172.17.0.1",
                "IPAddress": "172.17.0.2",
                "IPPrefixLen": 16,
                "IPv6Gateway": "",
                "GlobalIPv6Address": "",
                "GlobalIPv6PrefixLen": 0,
                "MacAddress": "02:42:ac:12:00:02"
              }
            }
          },
          "Path": "/bin/sh",
          "ProcessLabel": "",
          "ResolvConfPath": "/var/lib/docker/containers/ba033ac4401106a3b513bc9d639eee123ad78ca3616b921167cd74b20e25ed39/resolv.conf",
          "RestartCount": 1,
          "State": {
            "Error": "",
            "ExitCode": 9,
            "FinishedAt": "2015-01-06T15:47:32.080254511Z",
            "Health": {
              "Status": "healthy",
              "FailingStreak": 0,
              "Log": [
                {
                  "Start": "2019-12-22T10:59:05.6385933Z",
                  "End": "2019-12-22T10:59:05.8078452Z",
                  "ExitCode": 0,
                  "Output": ""
                }
              ]
            },
            "OOMKilled": false,
            "Dead": false,
            "Paused": false,
            "Pid": 0,
            "Restarting": false,
            "Running": true,
            "StartedAt": "2015-01-06T15:47:32.072697474Z",
            "Status": "running"
          },
          "Mounts": [
            {
              "Name": "fac362...80535",
              "Source": "/data",
              "Destination": "/data",
              "Driver": "local",
              "Mode": "ro,Z",
              "RW": false,
              "Propagation": ""
            }
          ]
        })
    }

    fn example_container_list() -> serde_json::Value {
        // Example taken from https://docs.docker.com/reference/api/engine/version/v1.46/#tag/Container/operation/ContainerList
        serde_json::json!([
          {
            "Id": "8dfafdbc3a40",
            "Names": [
              "/boring_feynman"
            ],
            "Image": "ubuntu:latest",
            "ImageID": "d74508fb6632491cea586a1fd7d748dfc5274cd6fdfedee309ecdcbc2bf5cb82",
            "Command": "echo 1",
            "Created": 1367854155,
            "State": "Exited",
            "Status": "Exit 0",
            "Ports": [
              {
                "PrivatePort": 2222,
                "PublicPort": 3333,
                "Type": "tcp"
              }
            ],
            "Labels": {
              "com.example.vendor": "Acme",
              "com.example.license": "GPL",
              "com.example.version": "1.0"
            },
            "SizeRw": 12288,
            "SizeRootFs": 0,
            "HostConfig": {
              "NetworkMode": "default",
              "Annotations": {
                "io.kubernetes.docker.type": "container"
              }
            },
            "NetworkSettings": {
              "Networks": {
                "bridge": {
                  "NetworkID": "7ea29fc1412292a2d7bba362f9253545fecdfa8ce9a6e37dd10ba8bee7129812",
                  "EndpointID": "2cdc4edb1ded3631c81f57966563e5c8525b81121bb3706a9a9a3ae102711f3f",
                  "Gateway": "172.17.0.1",
                  "IPAddress": "172.17.0.2",
                  "IPPrefixLen": 16,
                  "IPv6Gateway": "",
                  "GlobalIPv6Address": "",
                  "GlobalIPv6PrefixLen": 0,
                  "MacAddress": "02:42:ac:11:00:02"
                }
              }
            },
            "Mounts": [
              {
                "Name": "fac362...80535",
                "Source": "/data",
                "Destination": "/data",
                "Driver": "local",
                "Mode": "ro,Z",
                "RW": false,
                "Propagation": ""
              }
            ]
          },
          {
            "Id": "9cd87474be90",
            "Names": [
              "/coolName"
            ],
            "Image": "ubuntu:latest",
            "ImageID": "d74508fb6632491cea586a1fd7d748dfc5274cd6fdfedee309ecdcbc2bf5cb82",
            "Command": "echo 222222",
            "Created": 1367854155,
            "State": "Exited",
            "Status": "Exit 0",
            "Ports": [],
            "Labels": {},
            "SizeRw": 12288,
            "SizeRootFs": 0,
            "HostConfig": {
              "NetworkMode": "default",
              "Annotations": {
                "io.kubernetes.docker.type": "container",
                "io.kubernetes.sandbox.id": "3befe639bed0fd6afdd65fd1fa84506756f59360ec4adc270b0fdac9be22b4d3"
              }
            },
            "NetworkSettings": {
              "Networks": {
                "bridge": {
                  "NetworkID": "7ea29fc1412292a2d7bba362f9253545fecdfa8ce9a6e37dd10ba8bee7129812",
                  "EndpointID": "88eaed7b37b38c2a3f0c4bc796494fdf51b270c2d22656412a2ca5d559a64d7a",
                  "Gateway": "172.17.0.1",
                  "IPAddress": "172.17.0.8",
                  "IPPrefixLen": 16,
                  "IPv6Gateway": "",
                  "GlobalIPv6Address": "",
                  "GlobalIPv6PrefixLen": 0,
                  "MacAddress": "02:42:ac:11:00:08"
                }
              }
            },
            "Mounts": []
          },
          {
            "Id": "3176a2479c92",
            "Names": [
              "/sleepy_dog"
            ],
            "Image": "ubuntu:latest",
            "ImageID": "d74508fb6632491cea586a1fd7d748dfc5274cd6fdfedee309ecdcbc2bf5cb82",
            "Command": "echo 3333333333333333",
            "Created": 1367854154,
            "State": "Exited",
            "Status": "Exit 0",
            "Ports": [],
            "Labels": {},
            "SizeRw": 12288,
            "SizeRootFs": 0,
            "HostConfig": {
              "NetworkMode": "default",
              "Annotations": {
                "io.kubernetes.image.id": "d74508fb6632491cea586a1fd7d748dfc5274cd6fdfedee309ecdcbc2bf5cb82",
                "io.kubernetes.image.name": "ubuntu:latest"
              }
            },
            "NetworkSettings": {
              "Networks": {
                "bridge": {
                  "NetworkID": "7ea29fc1412292a2d7bba362f9253545fecdfa8ce9a6e37dd10ba8bee7129812",
                  "EndpointID": "8b27c041c30326d59cd6e6f510d4f8d1d570a228466f956edf7815508f78e30d",
                  "Gateway": "172.17.0.1",
                  "IPAddress": "172.17.0.6",
                  "IPPrefixLen": 16,
                  "IPv6Gateway": "",
                  "GlobalIPv6Address": "",
                  "GlobalIPv6PrefixLen": 0,
                  "MacAddress": "02:42:ac:11:00:06"
                }
              }
            },
            "Mounts": []
          },
          {
            "Id": "4cb07b47f9fb",
            "Names": [
              "/running_cat"
            ],
            "Image": "ubuntu:latest",
            "ImageID": "d74508fb6632491cea586a1fd7d748dfc5274cd6fdfedee309ecdcbc2bf5cb82",
            "Command": "echo 444444444444444444444444444444444",
            "Created": 1367854152,
            "State": "Exited",
            "Status": "Exit 0",
            "Ports": [],
            "Labels": {},
            "SizeRw": 12288,
            "SizeRootFs": 0,
            "HostConfig": {
              "NetworkMode": "default",
              "Annotations": {
                "io.kubernetes.config.source": "api"
              }
            },
            "NetworkSettings": {
              "Networks": {
                "bridge": {
                  "NetworkID": "7ea29fc1412292a2d7bba362f9253545fecdfa8ce9a6e37dd10ba8bee7129812",
                  "EndpointID": "d91c7b2f0644403d7ef3095985ea0e2370325cd2332ff3a3225c4247328e66e9",
                  "Gateway": "172.17.0.1",
                  "IPAddress": "172.17.0.5",
                  "IPPrefixLen": 16,
                  "IPv6Gateway": "",
                  "GlobalIPv6Address": "",
                  "GlobalIPv6PrefixLen": 0,
                  "MacAddress": "02:42:ac:11:00:05"
                }
              }
            },
            "Mounts": []
          }
        ])
    }

    pub async fn create_test_server_and_config() -> (mockito::ServerGuard, Arc<Docker>) {
        let server = mockito::Server::new_async().await;
        let client = Docker::connect_with_http(
            &server.url(),
            2,
            &ClientVersion {
                major_version: 1,
                minor_version: 46,
            },
        )
        .unwrap();
        (server, Arc::new(client))
    }

    #[tokio::test]
    async fn stop_container_ok() {
        let (mut mock_server, client) = create_test_server_and_config().await;
        let container_name = "test_container";
        let mock = mock_server
            .mock(
                "POST",
                format!("/containers/{container_name}/stop").as_str(),
            )
            .with_status(200)
            .create_async()
            .await;
        stop(client, container_name, None).await.unwrap();
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn stop_container_err() {
        let (mut mock_server, client) = create_test_server_and_config().await;
        let container_name = "test_container";
        let body = serde_json::to_vec(&serde_json::json!({
          "message": "Something went wrong."
        }))
        .unwrap();
        let mock = mock_server
            .mock(
                "POST",
                format!("/containers/{container_name}/stop").as_str(),
            )
            .with_status(500)
            .with_body(&body)
            .create_async()
            .await;
        assert!(stop(client, container_name, None).await.is_err());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn start_container_ok() {
        let (mut mock_server, client) = create_test_server_and_config().await;
        let container_name = "test_container";
        let mock = mock_server
            .mock(
                "POST",
                format!("/containers/{container_name}/start").as_str(),
            )
            .with_status(200)
            .create_async()
            .await;
        start(client, container_name).await.unwrap();
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn start_container_err() {
        let (mut mock_server, client) = create_test_server_and_config().await;
        let container_name = "test_container";
        let body = serde_json::to_vec(&serde_json::json!({
            "message": "Something went wrong.",

        }))
        .unwrap();
        let mock = mock_server
            .mock(
                "POST",
                format!("/containers/{container_name}/start").as_str(),
            )
            .with_status(500)
            .with_body(&body)
            .create_async()
            .await;
        assert!(start(client, container_name).await.is_err());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn create_container_ok() {
        let (mut mock_server, client) = create_test_server_and_config().await;
        let body = serde_json::to_vec(&serde_json::json!({
            "Id": "12345678",
            "Warnings": ["Warning1", "Warning2"]
        }))
        .unwrap();
        let mock = mock_server
            .mock("POST", "/containers/create")
            .with_status(201)
            .with_body(&body)
            .create_async()
            .await;
        assert_eq!(
            "12345678",
            create(
                client,
                Option::<CreateContainerOptions<&str>>::None,
                Config::<&str>::default(),
            )
            .await
            .unwrap()
        );
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn create_container_err() {
        let (mut mock_server, client) = create_test_server_and_config().await;
        let body = serde_json::to_vec(&serde_json::json!({
            "message": "Something went wrong."
        }))
        .unwrap();
        let mock = mock_server
            .mock("POST", "/containers/create")
            .with_status(500)
            .with_body(&body)
            .create_async()
            .await;
        assert!(
            create(
                client,
                Option::<CreateContainerOptions<&str>>::None,
                Config::<&str>::default(),
            )
            .await
            .is_err()
        );
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn list_container_ok() {
        let (mut mock_server, client) = create_test_server_and_config().await;
        let body = serde_json::to_vec(&example_container_list()).unwrap();
        let mock = mock_server
            .mock("GET", "/containers/json")
            .with_status(201)
            .with_body(&body)
            .create_async()
            .await;
        let containers = list(client, Option::<ListContainersOptions<&str>>::None)
            .await
            .unwrap();
        assert_eq!(containers.len(), 4);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn list_container_err() {
        let (mut mock_server, client) = create_test_server_and_config().await;
        let body = serde_json::to_vec(&serde_json::json!({
            "message": "Something went wrong."
        }))
        .unwrap();
        let mock = mock_server
            .mock("GET", "/containers/json")
            .with_status(500)
            .with_body(&body)
            .create_async()
            .await;
        assert!(
            list(client, Option::<ListContainersOptions<&str>>::None,)
                .await
                .is_err()
        );
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn remove_container_ok() {
        let (mut mock_server, client) = create_test_server_and_config().await;
        let body = serde_json::to_vec(&example_container_list()).unwrap();
        let mock = mock_server
            .mock("DELETE", "/containers/12345678")
            .with_status(204)
            .with_body(&body)
            .create_async()
            .await;
        assert!(remove(client, None, "12345678").await.unwrap());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn remove_container_err() {
        let (mut mock_server, client) = create_test_server_and_config().await;
        let body = serde_json::to_vec(&serde_json::json!({
            "message": "Something went wrong."
        }))
        .unwrap();
        let mock = mock_server
            .mock("DELETE", "/containers/12345678")
            .with_status(500)
            .with_body(&body)
            .create_async()
            .await;
        assert!(remove(client, None, "12345678").await.is_err());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn remove_container_404() {
        let (mut mock_server, client) = create_test_server_and_config().await;
        let body = serde_json::to_vec(&serde_json::json!({
            "message": "No such container: 12345678."
        }))
        .unwrap();
        let mock = mock_server
            .mock("DELETE", "/containers/12345678")
            .with_status(404)
            .with_body(&body)
            .create_async()
            .await;
        assert!(!remove(client, None, "12345678").await.unwrap());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn inspect_container_ok() {
        let (mut mock_server, client) = create_test_server_and_config().await;
        let body = serde_json::to_vec(&example_container_inspect()).unwrap();
        let mock = mock_server
            .mock("GET", "/containers/12345678/json")
            .with_status(200)
            .with_body(&body)
            .create_async()
            .await;
        assert!(inspect(client, "12345678").await.unwrap().is_some());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn inspect_container_err() {
        let (mut mock_server, client) = create_test_server_and_config().await;
        let body = serde_json::to_vec(&serde_json::json!({
            "message": "Something went wrong."
        }))
        .unwrap();
        let mock = mock_server
            .mock("GET", "/containers/12345678/json")
            .with_status(500)
            .with_body(&body)
            .create_async()
            .await;
        assert!(inspect(client, "12345678").await.is_err());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn inspect_container_404() {
        let (mut mock_server, client) = create_test_server_and_config().await;
        let body = serde_json::to_vec(&serde_json::json!({
            "message": "No such container: 12345678."
        }))
        .unwrap();
        let mock = mock_server
            .mock("GET", "/containers/12345678/json")
            .with_status(404)
            .with_body(&body)
            .create_async()
            .await;
        assert!(inspect(client, "12345678").await.unwrap().is_none());
        mock.assert_async().await;
    }
}
