pub use super::{Error, Result};
use crate::quest::{Progress, SyncQuest};
use crate::relic::async_flecstract::{
    archive_single_file_to_memory, archive_to_memory, extract_from_memory,
    extract_single_file_from_memory_as,
};
use crate::relic::docker::write_stream_to_memory;
use async_compression::tokio::bufread::{GzipDecoder, GzipEncoder};
use axum::body::Bytes;
use bollard::container::{
    Config, CreateContainerOptions, DownloadFromContainerOptions, ListContainersOptions,
    RemoveContainerOptions, StopContainerOptions, UploadToContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::models::{ContainerInspectResponse, ContainerSummary};
use bollard::Docker;
use futures::Stream;
use futures_util::stream::StreamExt;
use serde::Serialize;
use std::hash::Hash;
use std::io::Cursor;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncWriteExt, BufReader, ReadBuf};
use tokio::join;
use tokio_util::codec;

/// # Example
/// ```no_run
/// use bollard::exec::{CreateExecOptions, StartExecResults};
/// use bollard::Docker;
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
        .await?;
    Ok(docker_client.start_exec(&exec.id, start_options).await?)
}

/// # Example
/// ```no_run
/// use bollard::container::StopContainerOptions;
/// use bollard::Docker;
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
    Ok(docker_client
        .stop_container(container_name, options)
        .await?)
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
    Ok(docker_client
        .start_container::<&str>(container_name, None)
        .await?)
}

/// # Example
/// ```no_run
/// use bollard::container::Config;
/// use bollard::container::CreateContainerOptions;
/// use bollard::Docker;
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
    let response = docker_client.create_container(options, config).await?;
    for warning in response.warnings {
        println!(
            "Received warning during creation of container {}: {warning}",
            response.id
        )
    }
    Ok(response.id)
}

/// # Example
/// ```no_run
/// use bollard::container::ListContainersOptions;
/// use bollard::Docker;
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
    Ok(docker_client.list_containers(options).await?)
}

/// Returns Ok(true) if the specified container was removed, Ok(false) if the container did not exist
/// and an error otherwise
/// # Example
/// ```no_run
/// use bollard::container::RemoveContainerOptions;
/// use bollard::Docker;
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
        Err(e) => Err(Error::from(e)),
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
                .await?
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
                .await?
        }
    };
    Ok(())
}

async fn copy_archive_from(
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
    quest
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
        .2
        .await
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
                    .await?;
                Ok(())
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
                        match data {
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
                                eprintln!("Could not remove {dst:?}: {e}");
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
        Err(e) => Err(anyhow::anyhow!(e)),
    }
}
