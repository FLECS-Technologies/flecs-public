pub use super::Result;
use bollard::models::Volume;
use bollard::volume::{CreateVolumeOptions, RemoveVolumeOptions};
use bollard::Docker;
use std::hash::Hash;
use std::sync::Arc;

/// # Example
/// ```no_run
/// use bollard::volume::CreateVolumeOptions;
/// use bollard::Docker;
/// use flecs_core::relic::docker::volume::create;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_local_defaults().unwrap());
///     let result = create(
///         docker_client,
///         CreateVolumeOptions {
///             driver: "local",
///             name: "test_volume1",
///             ..Default::default()
///         },
///     )
///     .await
///     .unwrap();
///     println!("{:#?}", result);
/// }
/// # )
/// ```
pub async fn create<T>(
    docker_client: Arc<Docker>,
    options: CreateVolumeOptions<T>,
) -> Result<Volume>
where
    T: Into<String> + Eq + Hash + serde::ser::Serialize,
{
    Ok(docker_client.create_volume(options).await?)
}

/// # Example
/// ```no_run
/// use bollard::volume::RemoveVolumeOptions;
/// use bollard::Docker;
/// use flecs_core::relic::docker::volume::remove;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_local_defaults().unwrap());
///     remove(
///         docker_client,
///         Some(RemoveVolumeOptions { force: true }),
///         "test_volume1",
///     )
///     .await
///     .unwrap();
/// }
/// # )
/// ```
pub async fn remove(
    docker_client: Arc<Docker>,
    options: Option<RemoveVolumeOptions>,
    volume_name: &str,
) -> Result<()> {
    Ok(docker_client.remove_volume(volume_name, options).await?)
}
