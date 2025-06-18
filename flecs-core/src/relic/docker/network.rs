pub use super::Result;
use crate::relic::docker::map_bollard_error;
use bollard::Docker;
use bollard::models::Network;
use bollard::network::{
    ConnectNetworkOptions, CreateNetworkOptions, DisconnectNetworkOptions, InspectNetworkOptions,
    ListNetworksOptions,
};
use std::hash::Hash;
use std::sync::Arc;

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use bollard::network::InspectNetworkOptions;
/// use flecs_core::relic::docker::network::inspect;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let result = inspect(
///         docker_client,
///         "some-network",
///         Some(InspectNetworkOptions {
///             verbose: false,
///             scope: "local",
///         }),
///     )
///     .await
///     .unwrap();
///     println!("{:#?}", result);
/// }
/// # )
/// ```
pub async fn inspect<T>(
    docker_client: Arc<Docker>,
    network: &str,
    options: Option<InspectNetworkOptions<T>>,
) -> Result<Option<Network>>
where
    T: Into<String> + serde::ser::Serialize,
{
    match docker_client.inspect_network(network, options).await {
        Ok(network) => Ok(Some(network)),
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) => Ok(None),
        Err(e) => Err(map_bollard_error(e)),
    }
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use bollard::network::ListNetworksOptions;
/// use flecs_core::relic::docker::network::list;
/// use std::collections::HashMap;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let result = list(
///         docker_client,
///         Some(ListNetworksOptions {
///             filters: HashMap::from([
///                 ("name", vec!["network1", "network2"]),
///                 ("scope", vec!["local"]),
///             ]),
///         }),
///     )
///     .await
///     .unwrap();
///     println!("{:#?}", result);
/// }
/// # )
/// ```
pub async fn list<T>(
    docker_client: Arc<Docker>,
    options: Option<ListNetworksOptions<T>>,
) -> Result<Vec<Network>>
where
    T: Into<String> + Eq + Hash + serde::ser::Serialize,
{
    docker_client
        .list_networks(options)
        .await
        .map_err(map_bollard_error)
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use bollard::network::CreateNetworkOptions;
/// use flecs_core::relic::docker::network::create;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     let result = create(
///         docker_client,
///         CreateNetworkOptions {
///             name: "my-network",
///             driver: "bridge",
///             ..CreateNetworkOptions::default()
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
    options: CreateNetworkOptions<T>,
) -> Result<Network>
where
    T: Into<String> + Eq + Hash + serde::ser::Serialize,
{
    let response = docker_client
        .create_network(options)
        .await
        .map_err(map_bollard_error)?;
    inspect::<&str>(docker_client, &response.id, None)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Could not get network after creation"))
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use flecs_core::relic::docker::network::remove;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     remove(docker_client, "some-network").await.unwrap();
/// }
/// # )
/// ```
pub async fn remove(docker_client: Arc<Docker>, network_name: &str) -> Result<()> {
    docker_client
        .remove_network(network_name)
        .await
        .map_err(map_bollard_error)
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use bollard::models::EndpointSettings;
/// use bollard::network::ConnectNetworkOptions;
/// use flecs_core::relic::docker::network::connect;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     connect(
///         docker_client,
///         "some-network",
///         ConnectNetworkOptions {
///             container: "my_container",
///             endpoint_config: EndpointSettings::default(),
///         },
///     )
///     .await
///     .unwrap();
/// }
/// # )
/// ```
pub async fn connect<T>(
    docker_client: Arc<Docker>,
    network_name: &str,
    options: ConnectNetworkOptions<T>,
) -> Result<()>
where
    T: Into<String> + Eq + Hash + serde::ser::Serialize,
{
    docker_client
        .connect_network(network_name, options)
        .await
        .map_err(map_bollard_error)
}

/// # Example
/// ```no_run
/// use bollard::Docker;
/// use bollard::network::DisconnectNetworkOptions;
/// use flecs_core::relic::docker::network::disconnect;
/// use std::sync::Arc;
///
/// # tokio_test::block_on(
/// async {
///     let docker_client = Arc::new(Docker::connect_with_defaults().unwrap());
///     disconnect(
///         docker_client,
///         "some-network",
///         DisconnectNetworkOptions {
///             container: "my_container",
///             force: false,
///         },
///     )
///     .await
///     .unwrap();
/// }
/// # )
/// ```
pub async fn disconnect<T>(
    docker_client: Arc<Docker>,
    network_name: &str,
    options: DisconnectNetworkOptions<T>,
) -> Result<()>
where
    T: Into<String> + Eq + Hash + serde::ser::Serialize,
{
    docker_client
        .disconnect_network(network_name, options)
        .await
        .map_err(map_bollard_error)
}
