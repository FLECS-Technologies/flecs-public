pub use super::Result;
use crate::relic::docker::{
    error_detail_to_update, progress_detail_to_update, write_stream_to_file, ByteResult,
    ByteStatus, CallResult, Status, StatusUpdate,
};
use bollard::auth::DockerCredentials;
use bollard::image::{CreateImageOptions, ImportImageOptions, RemoveImageOptions};
use bollard::models::{BuildInfo, CreateImageInfo, ImageDeleteResponseItem, ImageInspect};
use bollard::Docker;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::default::Default;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio::sync::Mutex;
use tokio_util::codec;

impl StatusUpdate for CreateImageInfo {
    fn get_status_update(&self) -> Option<(String, String)> {
        match self {
            CreateImageInfo {
                id: Some(id),
                status: Some(status),
                progress_detail,
                ..
            } => {
                let details = progress_detail_to_update(progress_detail);
                Some((id.clone(), format!("{status}{details}")))
            }
            CreateImageInfo {
                id: Some(id),
                error: Some(error),
                error_detail,
                ..
            } => {
                let details = error_detail_to_update(error_detail);
                Some((id.clone(), format!("Error: {error}{details}")))
            }
            _ => None,
        }
    }
}

impl StatusUpdate for BuildInfo {
    fn get_status_update(&self) -> Option<(String, String)> {
        match self {
            Self {
                id: Some(id),
                status: Some(status),
                progress_detail,
                ..
            } => {
                let details = progress_detail_to_update(progress_detail);
                Some((id.clone(), format!("{status}{details}")))
            }
            Self {
                id: Some(id),
                error: Some(error),
                error_detail,
                ..
            } => {
                let details = error_detail_to_update(error_detail);
                Some((id.clone(), format!("Error: {error}{details}")))
            }
            _ => None,
        }
    }
}

/// # Examples
/// Poll and print the status until pull is complete:
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::relic::docker::image::pull;
/// use std::sync::Arc;
/// use tokio::time::{sleep, Duration};
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let result = pull(
///         docker_client,
///         Default::default(),
///         "opensearchproject/opensearch",
///         "latest",
///         "linux/amd64",
///     );
///     loop {
///         sleep(Duration::from_millis(2000)).await;
///         let result = result.status.lock().await;
///         println!("----------");
///         println!("{result}");
///         if result.finished {
///             break;
///         }
///     }
///     result.handle.await.unwrap();
/// }
/// # )
/// ```
///
/// Await the complete pull without accessing any sub results:
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::relic::docker::image::pull;
/// use std::sync::Arc;
/// use tokio::time::{sleep, Duration};
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let result = pull(
///         docker_client,
///         Default::default(),
///         "opensearchproject/opensearch",
///         "latest",
///         "linux/arm64",
///     );
///     result.handle.await.unwrap();
///     let result = result.status.lock().await;
///     println!("{result}");
/// }
/// # )
/// ```
pub fn pull(
    docker_client: Arc<Docker>,
    credentials: Option<DockerCredentials>,
    image: &str,
    tag: &str,
    platform: &str,
) -> CallResult<()> {
    let status: Arc<Mutex<Status>> = Default::default();
    let image = image.to_string();
    let tag = tag.to_string();
    let platform = platform.to_string();
    let closure_status = status.clone();
    let handle = tokio::spawn(async move {
        let status = closure_status;
        let options = Some(CreateImageOptions {
            from_image: image.as_str(),
            tag: tag.as_str(),
            platform: platform.as_str(),
            ..Default::default()
        });

        let mut results = docker_client.create_image(options, None, credentials);
        while let Some(result) = results.next().await {
            status
                .lock()
                .await
                .add_result(result.map_err(anyhow::Error::from));
        }
        status.lock().await.finish();
    });
    CallResult { status, handle }
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::relic::docker::image::inspect;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let result = inspect(docker_client, "opensearchproject/opensearch:latest")
///         .await
///         .unwrap();
///     println!("{:#?}", result);
/// }
/// # )
/// ```
pub async fn inspect(docker_client: Arc<Docker>, image: &str) -> Result<ImageInspect> {
    Ok(docker_client.inspect_image(image).await?)
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::relic::docker::image::remove;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let result = remove(
///         docker_client,
///         "opensearchproject/opensearch:latest",
///         None,
///         None,
///     )
///     .await
///     .unwrap();
///     for image in result {
///         if let Some(untagged_image) = image.untagged {
///             println!("Untagged image: {untagged_image}");
///         }
///         if let Some(deleted_image) = image.deleted {
///             println!("Deleted image: {deleted_image}");
///         }
///     }
/// }
/// # )
/// ```
pub async fn remove(
    docker_client: Arc<Docker>,
    image: &str,
    options: Option<RemoveImageOptions>,
    credentials: Option<DockerCredentials>,
) -> Result<Vec<ImageDeleteResponseItem>> {
    Ok(docker_client
        .remove_image(image, options, credentials)
        .await?)
}

/// # Example
/// ```no_run
/// use bollard::auth::DockerCredentials;
/// use bollard::image::ImportImageOptions;
/// use bollard::Docker;
/// use flecs_core::relic::docker::image::load;
/// use std::collections::HashMap;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let path = Path::new("/path/to/image.tar");
///     let password = Some("e8bb92d7-cf1c-4f50-802f-3482b3ac38c4".to_string());
///     let username = Some("peter".to_string());
///     let credentials: HashMap<String, DockerCredentials> = HashMap::from([
///         (
///             // For legacy reasons the docker hub registry has to specified exactly like this
///             "https://index.docker.io/v1/".to_string(),
///             DockerCredentials {
///                 username,
///                 password,
///                 ..DockerCredentials::default()
///             },
///         ),
///         (
///             "my-private-registry.com".to_string(),
///             DockerCredentials::default(),
///         ),
///     ]);
///
///     let result = load(
///         docker_client,
///         path,
///         ImportImageOptions::default(),
///         Some(credentials),
///     )
///     .await;
///
///     result.handle.await.unwrap();
///     let _result = result.status.lock().await;
/// }
/// # )
/// ```
/// Await the complete load without accessing any sub results:
/// ```no_run
/// use bollard::image::ImportImageOptions;
/// use bollard::Docker;
/// use flecs_core::relic::docker::image::load;
/// use std::path::Path;
/// use std::sync::Arc;
/// use tokio::time::{sleep, Duration};
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let result = load(
///         docker_client,
///         Path::new("/tmp/opensearch.latest.tar"),
///         ImportImageOptions { quiet: false },
///         None,
///     )
///     .await;
///
///     loop {
///         sleep(Duration::from_millis(100)).await;
///         let result = result.status.lock().await;
///         println!("----------");
///         println!("{result}");
///         if result.finished {
///             break;
///         }
///     }
///     result.handle.await.unwrap();
/// }
/// # )
/// ```
pub async fn load(
    docker_client: Arc<Docker>,
    path: &Path,
    options: ImportImageOptions,
    credentials: Option<HashMap<String, DockerCredentials>>,
) -> CallResult<()> {
    let status: Arc<Mutex<Status>> = Default::default();
    let path = path.to_path_buf();
    let closure_status = status.clone();

    let handle = tokio::spawn(async move {
        let status = closure_status;
        let file = match File::open(path).await {
            Ok(file) => file,
            Err(e) => {
                let mut status = status.lock().await;
                status.add_error(anyhow::anyhow!(e));
                status.finish();
                return;
            }
        };

        let byte_stream =
            codec::FramedRead::new(file, codec::BytesCodec::new()).map(|r| r.unwrap().freeze());

        let mut stream = docker_client.import_image_stream(options, byte_stream, credentials);

        while let Some(result) = stream.next().await {
            status
                .lock()
                .await
                .add_result(result.map_err(anyhow::Error::from));
        }
        status.lock().await.finish();
    });
    CallResult { status, handle }
}

/// # Examples
/// Poll and print the progress until save is complete:
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::relic::docker::image::save;
/// use flecs_core::relic::docker::ByteStatus;
/// use std::path::Path;
/// use std::sync::Arc;
/// use tokio::time::{sleep, Duration};
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let result = save(
///         docker_client,
///         Path::new("/tmp/opensearch.latest.tar"),
///         "opensearchproject/opensearch:latest",
///     );
///     loop {
///         sleep(Duration::from_millis(100)).await;
///         let result = result.status.lock().await;
///         println!("----------");
///         match *result {
///             ByteStatus::Complete(bytes) => {
///                 println!("Export complete, {bytes} bytes written");
///                 break;
///             }
///             ByteStatus::Partial(bytes) => {
///                 println!("Export ongoing, {bytes} bytes written");
///             }
///             ByteStatus::Error(bytes) => {
///                 println!("Export failed, {bytes} bytes written");
///                 break;
///             }
///         }
///     }
///     result.handle.await.unwrap().unwrap();
/// }
/// # )
/// ```
/// Await the complete save without accessing any sub progress:
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::relic::docker::image::save;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let result = save(
///         docker_client,
///         Path::new("/tmp/opensearch.latest.tar"),
///         "opensearchproject/opensearch:latest",
///     );
///     result.handle.await.unwrap().unwrap();
///     let result = result.status.lock().await;
///     println!("{:?}", *result);
/// }
/// # )
/// ```
pub fn save(docker_client: Arc<Docker>, path: &Path, image: &str) -> ByteResult {
    let status = Arc::new(Mutex::new(ByteStatus::Partial(0)));
    let image = image.to_string();
    let path = path.to_path_buf();
    let closure_status = status.clone();
    let handle = tokio::spawn(async move {
        let result = docker_client.export_image(&image);

        let result = write_stream_to_file(result, &path, closure_status.clone()).await;
        if result.is_err() {
            closure_status.lock().await.fail();
        }
        result
    });
    ByteResult { status, handle }
}
