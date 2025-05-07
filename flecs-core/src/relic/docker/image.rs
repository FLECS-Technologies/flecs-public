pub use super::Result;
use crate::quest::{Progress, State, SyncQuest};
use crate::relic::docker::write_stream_to_file;
use bollard::Docker;
use bollard::auth::DockerCredentials;
use bollard::image::{CreateImageOptions, ImportImageOptions, RemoveImageOptions};
use bollard::models::{
    BuildInfo, CreateImageInfo, ErrorDetail, ImageDeleteResponseItem, ImageInspect, ProgressDetail,
};
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::default::Default;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::codec;
use tracing::{debug, trace};

impl TryFrom<&ProgressDetail> for Progress {
    type Error = ();

    fn try_from(value: &ProgressDetail) -> Result<Self, Self::Error> {
        match value {
            ProgressDetail {
                current: Some(current),
                total: Some(total),
            } => Ok(Progress {
                current: *current as u64,
                total: Some(*total as u64),
            }),
            _ => Err(()),
        }
    }
}

fn details_from_error_details(
    error: Option<&String>,
    error_detail: Option<&ErrorDetail>,
    status: Option<&String>,
) -> Option<String> {
    let error_details = match (error, error_detail) {
        (Some(error), error_details) => {
            let error_details = match error_details {
                Some(ErrorDetail {
                    code: Some(code),
                    message: Some(message),
                }) => {
                    format!(" (code {code}, {message})")
                }
                Some(ErrorDetail {
                    code: Some(code), ..
                }) => {
                    format!(" (code {code})")
                }
                Some(ErrorDetail {
                    message: Some(message),
                    ..
                }) => {
                    format!(" ({message})")
                }
                _ => String::new(),
            };
            Some(format!("{error}{error_details})"))
        }
        _ => None,
    };
    match (status, &error_details) {
        (Some(status), Some(error_details)) => Some(format!("{status}: {error_details})")),
        (Some(detail), None) | (None, Some(detail)) => Some(detail.clone()),
        _ => None,
    }
}

impl crate::quest::StatusUpdate for CreateImageInfo {
    fn progress(&self) -> Option<Progress> {
        match &self.progress_detail {
            Some(progress) => Progress::try_from(progress).ok(),
            None => None,
        }
    }

    fn details(&self) -> Option<String> {
        details_from_error_details(
            self.error.as_ref(),
            self.error_detail.as_ref(),
            self.status.as_ref(),
        )
    }

    fn state(&self) -> Option<State> {
        match (&self.error, &self.error_detail) {
            (None, None) => None,
            _ => Some(State::Failing),
        }
    }
}

/// # Examples
/// Poll and print the status until pull is complete:
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::enchantment::quest_master::QuestMaster;
/// use flecs_core::quest::Quest;
/// use flecs_core::relic::docker::image::pull;
/// use std::sync::Arc;
/// use tokio::sync::oneshot;
/// use tokio::time::{Duration, sleep};
///
/// # tokio_test::block_on(
/// async {
///     let mut quest_master = QuestMaster::default();
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let (tx, rx) = oneshot::channel();
///     let (_id, quest) = quest_master
///         .lock()
///         .await
///         .schedule_quest("Docker pull quest".to_string(), |quest| async move {
///             let pull_result = pull(
///                 quest,
///                 docker_client,
///                 Default::default(),
///                 "opensearchproject/opensearch",
///                 "latest",
///             )
///             .await;
///             tx.send(pull_result).unwrap();
///             Ok(())
///         })
///         .await
///         .unwrap();
///     loop {
///         sleep(Duration::from_millis(2000)).await;
///         println!("----------");
///         println!("{}", Quest::fmt(quest.clone()).await);
///         if quest.lock().await.state.is_finished() {
///             break;
///         }
///     }
///     let _id = rx.await.unwrap();
/// }
/// # )
/// ```
///
/// Await the complete pull without accessing any sub results:
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::quest::Quest;
/// use flecs_core::relic::docker::image::pull;
/// use std::sync::Arc;
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let quest = Quest::new_synced("Docker pull quest".to_string());
///     let id = pull(
///         quest,
///         docker_client,
///         Default::default(),
///         "opensearchproject/opensearch",
///         "latest",
///     )
///     .await
///     .unwrap();
///     println!("{id}");
/// }
/// # )
/// ```
pub async fn pull(
    quest: SyncQuest,
    docker_client: Arc<Docker>,
    credentials: Option<DockerCredentials>,
    image: &str,
    tag: &str,
) -> Result<String> {
    let options = Some(CreateImageOptions {
        from_image: image,
        tag,
        ..Default::default()
    });

    let mut last_result = None;
    let mut results = docker_client.create_image(options, None, credentials);
    while let Some(result) = results.next().await {
        let result = result?;
        quest.lock().await.update(&result);
        last_result = Some(result);
    }
    match last_result {
        None => anyhow::bail!("Received no info from docker"),
        Some(_) => {
            let id = format!("{image}:{tag}");
            let image = inspect(docker_client, &id).await?;
            image
                .ok_or_else(|| anyhow::anyhow!("Could not get image for {id}"))?
                .id
                .ok_or_else(|| anyhow::anyhow!("Could not get image id for {id}"))
        }
    }
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
pub async fn inspect(docker_client: Arc<Docker>, image: &str) -> Result<Option<ImageInspect>> {
    match docker_client.inspect_image(image).await {
        Ok(image) => Ok(Some(image)),
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) => Ok(None),
        Err(e) => Err(anyhow::anyhow!(e)),
    }
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
    let results = docker_client
        .remove_image(image, options, credentials)
        .await?;
    for result in results.iter() {
        if let Some(untagged) = result.untagged.as_ref() {
            trace!("Untagged: {untagged}");
        }
        if let Some(deleted) = result.deleted.as_ref() {
            trace!("Deleted: {deleted}");
        }
    }
    Ok(results)
}

impl crate::quest::StatusUpdate for BuildInfo {
    fn progress(&self) -> Option<Progress> {
        match &self.progress_detail {
            Some(progress) => Progress::try_from(progress).ok(),
            None => None,
        }
    }

    fn details(&self) -> Option<String> {
        details_from_error_details(
            self.error.as_ref(),
            self.error_detail.as_ref(),
            self.status.as_ref(),
        )
    }

    fn state(&self) -> Option<State> {
        match (&self.error, &self.error_detail) {
            (None, None) => None,
            _ => Some(State::Failing),
        }
    }
}
/// # Example
/// ```no_run
/// use bollard::Docker;
/// use bollard::auth::DockerCredentials;
/// use bollard::image::ImportImageOptions;
/// use flecs_core::quest::Quest;
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
///     let quest = Quest::new_synced("Docker load image".to_string());
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
///     load(
///         quest,
///         docker_client,
///         path,
///         ImportImageOptions::default(),
///         Some(credentials),
///     )
///     .await
///     .unwrap();
/// }
/// # )
/// ```
/// Await the complete load without accessing any sub results:
/// ```no_run
/// use bollard::Docker;
/// use bollard::image::ImportImageOptions;
/// use flecs_core::enchantment::quest_master::QuestMaster;
/// use flecs_core::quest::Quest;
/// use flecs_core::relic::docker::image::load;
/// use std::path::Path;
/// use std::sync::Arc;
/// use tokio::time::{Duration, sleep};
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let mut quest_master = QuestMaster::default();
///     let (_id, quest) = quest_master
///         .lock()
///         .await
///         .schedule_quest("Docker load image".to_string(), |quest| async move {
///             load(
///                 quest,
///                 docker_client,
///                 Path::new("/tmp/opensearch.latest.tar"),
///                 ImportImageOptions { quiet: false },
///                 None,
///             )
///             .await
///         })
///         .await
///         .unwrap();
///
///     loop {
///         sleep(Duration::from_millis(2000)).await;
///         println!("----------");
///         println!("{}", Quest::fmt(quest.clone()).await);
///         if quest.lock().await.state.is_finished() {
///             break;
///         }
///     }
/// }
/// # )
/// ```
pub async fn load(
    quest: SyncQuest,
    docker_client: Arc<Docker>,
    path: &Path,
    options: ImportImageOptions,
    credentials: Option<HashMap<String, DockerCredentials>>,
) -> Result<()> {
    debug!("Import image from {path:?}");
    let file = File::open(path).await?;

    let byte_stream =
        codec::FramedRead::new(file, codec::BytesCodec::new()).map(|r| r.unwrap().freeze());

    let mut stream = docker_client.import_image_stream(options, byte_stream, credentials);

    while let Some(result) = stream.next().await {
        quest.lock().await.update(&result?)
    }
    Ok(())
}

/// # Examples
/// Poll and print the progress until save is complete:
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::enchantment::quest_master::QuestMaster;
/// use flecs_core::quest::Progress;
/// use flecs_core::relic::docker::ByteStatus;
/// use flecs_core::relic::docker::image::save;
/// use std::path::Path;
/// use std::sync::Arc;
/// use tokio::time::{Duration, sleep};
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let mut quest_master = QuestMaster::default();
///     let (_id, quest) = quest_master
///         .lock()
///         .await
///         .schedule_quest("Docker save".to_string(), |quest| async move {
///             save(
///                 quest,
///                 docker_client,
///                 Path::new("/tmp/opensearch.latest.tar"),
///                 "opensearchproject/opensearch:latest",
///             )
///             .await
///         })
///         .await
///         .unwrap();
///     loop {
///         sleep(Duration::from_millis(100)).await;
///         let quest = quest.lock().await;
///         println!("----------");
///         if let Some(Progress { current, .. }) = quest.progress {
///             println!("{current} bytes written");
///         }
///         if quest.state.is_finished() {
///             println!("Docker save finished with {}", quest.state);
///             break;
///         }
///     }
/// }
/// # )
/// ```
/// Await the complete save without accessing any sub progress:
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::quest::Quest;
/// use flecs_core::relic::docker::image::save;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let quest = Quest::new_synced("Docker save".to_string());
///     let result = save(
///         quest,
///         docker_client,
///         Path::new("/tmp/opensearch.latest.tar"),
///         "opensearchproject/opensearch:latest",
///     )
///     .await
///     .unwrap();
/// }
/// # )
/// ```
pub async fn save(
    quest: SyncQuest,
    docker_client: Arc<Docker>,
    path: &Path,
    image: &str,
) -> Result<()> {
    let image = image.to_string();
    let path = path.to_path_buf();
    let result = quest
        .lock()
        .await
        .create_sub_quest(
            format!("Writing image {image} to {path:?}"),
            |quest| async move {
                let result = docker_client.export_image(&image);
                write_stream_to_file(quest, result, &path).await?;
                Ok(())
            },
        )
        .await
        .2;
    result.await
}
