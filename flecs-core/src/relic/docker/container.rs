pub use super::Result;
use crate::relic::async_flecstract::{archive_to_memory, extract_from_memory};
use crate::relic::docker::{write_stream_to_memory, ByteResult, ByteStatus};
use bollard::container::{
    Config, CreateContainerOptions, DownloadFromContainerOptions, ListContainersOptions,
    RemoveContainerOptions, StartContainerOptions, StopContainerOptions, UploadToContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::models::{ContainerCreateResponse, ContainerInspectResponse, ContainerSummary};
use bollard::Docker;
use futures_util::stream::StreamExt;
use serde::Serialize;
use std::hash::Hash;
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio::sync::Mutex;
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
///     start::<&str>(docker_client, "8c9a8332827c", None)
///         .await
///         .unwrap();
/// }
/// # )
/// ```
pub async fn start<T>(
    docker_client: Arc<Docker>,
    container_name: &str,
    options: Option<StartContainerOptions<T>>,
) -> Result<()>
where
    T: Into<String> + Serialize,
{
    Ok(docker_client
        .start_container(container_name, options)
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
) -> Result<ContainerCreateResponse>
where
    T: Into<String> + Eq + Hash + Serialize,
    Z: Into<String> + Hash + Eq + Serialize,
{
    Ok(docker_client.create_container(options, config).await?)
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
///     remove(docker_client, options, "8c9a8332827c")
///         .await
///         .unwrap();
/// }
/// # )
/// ```
pub async fn remove(
    docker_client: Arc<Docker>,
    options: Option<RemoveContainerOptions>,
    container_name: &str,
) -> Result<()> {
    Ok(docker_client
        .remove_container(container_name, options)
        .await?)
}

pub enum Data {
    File(File),
    InMemory(Vec<u8>),
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::relic::docker::container::copy_to;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let src = Path::new("/path/on/host");
///     let dst = Path::new("/path/on/container");
///     copy_to(docker_client, src, dst, "my-container", false)
///         .await
///         .unwrap();
/// }
/// # )
/// ```
pub async fn copy_to(
    docker_client: Arc<Docker>,
    src_path: &Path,
    dst_path: &Path,
    container_name: &str,
    follow_symlinks: bool,
) -> Result<()> {
    let archive = archive_to_memory(src_path, follow_symlinks).await?;
    let archive = Data::InMemory(archive);
    copy_archive_to(
        docker_client,
        Some(UploadToContainerOptions {
            path: dst_path.to_str().unwrap(),
            no_overwrite_dir_non_dir: "false",
        }),
        archive,
        container_name,
    )
    .await
}

async fn copy_archive_to<T>(
    docker_client: Arc<Docker>,
    options: Option<UploadToContainerOptions<T>>,
    archive: Data,
    container_name: &str,
) -> Result<()>
where
    T: Into<String> + Eq + Hash + Serialize,
{
    match archive {
        Data::File(file) => {
            let byte_stream =
                codec::FramedRead::new(file, codec::BytesCodec::new()).map(|r| r.unwrap().freeze());
            docker_client
                .upload_to_container_streaming(container_name, options, byte_stream)
                .await?
        }
        Data::InMemory(data) => {
            let reader = Cursor::new(data);
            let byte_stream = codec::FramedRead::new(reader, codec::BytesCodec::new())
                .map(|r| r.unwrap().freeze());
            docker_client
                .upload_to_container_streaming(container_name, options, byte_stream)
                .await?
        }
    };
    Ok(())
}

async fn copy_archive_from(
    docker_client: Arc<Docker>,
    src: &Path,
    container_name: &str,
) -> ByteResult<Vec<u8>> {
    let status = Arc::new(Mutex::new(ByteStatus::Partial(0)));
    let options = Some(DownloadFromContainerOptions {
        path: src.to_string_lossy().to_string(),
    });
    let container_name = container_name.to_string();
    let closure_status = status.clone();
    let handle = tokio::spawn(async move {
        let result = docker_client.download_from_container(&container_name, options);
        let result = write_stream_to_memory(result, closure_status.clone()).await;
        if result.is_err() {
            closure_status.lock().await.fail();
        }
        result
    });
    ByteResult { status, handle }
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::relic::docker::container::copy_from;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let dst = Path::new("/path/on/host");
///     let src = Path::new("/path/on/container");
///     copy_from(docker_client, src, dst, "my-container")
///         .await
///         .unwrap();
/// }
/// # )
/// ```
pub async fn copy_from(
    docker_client: Arc<Docker>,
    src: &Path,
    dst: &Path,
    container_name: &str,
) -> Result<()> {
    let archive = copy_archive_from(docker_client, src, container_name)
        .await
        .handle
        .await??;
    extract_from_memory(archive, dst).await
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
