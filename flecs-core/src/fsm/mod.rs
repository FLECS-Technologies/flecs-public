pub mod console_client;
mod server_impl;
pub mod world;
use crate::enchantment::Enchantments;
use crate::enchantment::floxy::Floxy;
use crate::fsm::server_impl::DeviceReaders;
use crate::lore::{Listener, Lore};
use crate::relic::device::net::NetDeviceReaderImpl;
use crate::relic::device::usb::UsbDeviceReaderImpl;
use crate::relic::network::NetworkAdapterReaderImpl;
use crate::sorcerer::Sorcerers;
use crate::sorcerer::appraiser::AppRaiser;
use crate::sorcerer::authmancer::Authmancer;
use crate::sorcerer::deploymento::Deploymento;
use crate::sorcerer::exportius::Exportius;
use crate::sorcerer::importius::Importius;
use crate::sorcerer::instancius::Instancius;
use crate::sorcerer::licenso::Licenso;
use crate::sorcerer::mage_quester::MageQuester;
use crate::sorcerer::manifesto::Manifesto;
use crate::sorcerer::systemus::Systemus;
use crate::vault::Vault;
#[cfg(feature = "auth")]
use crate::wall;
use axum::extract::DefaultBodyLimit;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
#[cfg(feature = "auth")]
use axum::response::IntoResponse;
#[cfg(feature = "auth")]
use axum::routing::any;
use axum::routing::{delete, get, put};
use axum::{
    Router,
    extract::MatchedPath,
    extract::connect_info::{self},
    http::Request,
};
use hyper::body::Incoming;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server,
};
pub use server_impl::ApiDoc;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::os::unix::fs::PermissionsExt;
use std::str::FromStr;
use std::time::Duration;
use std::{convert::Infallible, path::PathBuf, sync::Arc};
use tokio::fs;
use tokio::net::{TcpListener, UnixListener, UnixStream, unix::UCred};
use tower::Service;
use tower_http::classify::ServerErrorsFailureClass;
#[cfg(feature = "auth")]
use tracing::debug;
use tracing::{Span, error, info, info_span, trace_span, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct StartupError(pub String);

impl<T> From<T> for StartupError
where
    T: Error,
{
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}

impl Display for StartupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not start rust server: {}", self.0)
    }
}

impl Debug for StartupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

pub type Result<T> = std::result::Result<T, StartupError>;

pub fn init_backtracing() {
    if std::env::var("RUST_BACKTRACE").is_err() {
        #[cfg(debug_assertions)]
        const BT_VALUE: &str = "1";
        #[cfg(not(debug_assertions))]
        const BT_VALUE: &str = "0";
        unsafe { std::env::set_var("RUST_BACKTRACE", BT_VALUE) };
    }
}

pub fn init_tracing(filter: &tracing_subscriber::EnvFilter) {
    let filter = tracing_subscriber::EnvFilter::from_str(&filter.to_string())
        .expect("A valid filter should result in a valid string");
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Tracing initialized");
}

fn is_trace_level_path(path: &str) -> bool {
    path.starts_with("/v2/quests") || path.starts_with("/v2/jobs")
}

#[cfg(feature = "auth")]
async fn auth_middleware(
    axum::extract::State(watch): axum::extract::State<Arc<wall::watch::Watch>>,
    wall::watch::AuthToken(auth_token): wall::watch::AuthToken,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    if !watch.has_auth_provider().await {
        debug!("There is no core auth provider, continuing with initial setup permissions");
        request
            .extensions_mut()
            .insert(wall::watch::RolesExtension::new_with_initial_setup_roles());
    } else if let Some(token) = auth_token.as_deref() {
        match watch.verify_token(token).await {
            Err(wall::watch::Error::NoAuthProvider) => {
                debug!(
                    "Can not verify token as no auth provider is configured, continuing as anonymous"
                );
                request
                    .extensions_mut()
                    .insert(wall::watch::RolesExtension::default());
            }
            Err(e) => {
                warn!("Failed to verify token: {e}");
                return http::StatusCode::UNAUTHORIZED.into_response();
            }
            Ok(roles) => {
                debug!("Successfully verified token, roles: {:?}", roles.0);
                request.extensions_mut().insert(roles);
            }
        }
    } else {
        request
            .extensions_mut()
            .insert(wall::watch::RolesExtension::default());
    }
    next.run(request).await
}

#[cfg(feature = "auth")]
async fn roles_middleware(
    axum::extract::State(enforcer): axum::extract::State<Arc<wall::enforcer::Enforcer>>,
    axum::extract::OriginalUri(uri): axum::extract::OriginalUri,
    axum::Extension(roles): axum::Extension<wall::watch::RolesExtension>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    match enforcer
        .verify_roles(uri.path(), &roles.0, request.method())
        .await
    {
        #[cfg(not(feature = "dev-auth"))]
        Ok(false) => http::StatusCode::FORBIDDEN.into_response(),
        #[cfg(feature = "dev-auth")]
        Ok(false) => {
            warn!(
                "Authorization failed, but feature dev-auth is enabled and the request will be processed "
            );
            next.run(request).await
        }
        Ok(true) => next.run(request).await,
        Err(e) => {
            warn!("Error verifying roles: {e}");
            http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub type DefaultDeviceReaders =
    DeviceReaders<UsbDeviceReaderImpl, NetworkAdapterReaderImpl, NetDeviceReaderImpl>;

impl Default for DefaultDeviceReaders {
    fn default() -> Self {
        Self {
            usb_reader: Default::default(),
            network_adapter_reader: Default::default(),
            net_device_reader: Default::default(),
        }
    }
}

async fn create_service<
    APP: AppRaiser + 'static,
    AUTH: Authmancer + 'static,
    I: Instancius + 'static,
    L: Licenso + 'static,
    Q: MageQuester + 'static,
    M: Manifesto + 'static,
    SYS: Systemus + 'static,
    D: Deploymento + 'static,
    E: Exportius + 'static,
    IMP: Importius + 'static,
    F: Floxy + 'static,
    C,
>(
    sorcerers: Sorcerers<APP, AUTH, I, L, Q, M, SYS, D, E, IMP>,
    enchantments: Enchantments<F>,
    vault: Arc<Vault>,
    lore: Arc<Lore>,
    #[cfg(feature = "auth")] wall: wall::Wall,
) -> Result<IntoMakeServiceWithConnectInfo<Router, C>> {
    let server = server_impl::ServerImpl::new(
        vault.clone(),
        lore.clone(),
        sorcerers,
        enchantments,
        DefaultDeviceReaders::default(),
        #[cfg(feature = "auth")]
        wall.clone(),
    )
    .await;
    let server = Arc::new(server);
    let app = flecsd_axum_server::server::new(server.clone());
    let app = app
        .route(
            "/v2/instances/:instance_id/depends/:dependency_key",
            delete(server_impl::api::v2::instances::instance_id::depends::dependency_key::delete)
                .get(server_impl::api::v2::instances::instance_id::depends::dependency_key::get)
                .put(server_impl::api::v2::instances::instance_id::depends::dependency_key::put),
        )
        .route(
            "/v2/instances/:instance_id/depends/:dependency_key/:feature",
            put(
                server_impl::api::v2::instances::instance_id::depends::dependency_key::feature::put,
            ),
        )
        .route(
            "/v2/instances/:instance_id/depends",
            get(server_impl::api::v2::instances::instance_id::depends::get),
        )
        .route(
            "/v2/instances/:instance_id/provides",
            get(server_impl::api::v2::instances::instance_id::provides::get),
        )
        .route(
            "/v2/instances/:instance_id/provides/:feature",
            get(server_impl::api::v2::instances::instance_id::provides::feature::get),
        )
        .route("/v2/providers", get(server_impl::api::v2::providers::get))
        .route(
            "/v2/providers/:feature",
            get(server_impl::api::v2::providers::feature::get),
        )
        .route(
            "/v2/providers/:feature/:id",
            get(server_impl::api::v2::providers::feature::id::get),
        )
        .route(
            "/v2/providers/:feature/default",
            delete(server_impl::api::v2::providers::feature::default::delete)
                .get(server_impl::api::v2::providers::feature::default::get)
                .put(server_impl::api::v2::providers::feature::default::put),
        )
        .route(
            "/v2/system/sbom",
            get(server_impl::api::v2::system::sbom::get),
        );

    #[cfg(feature = "auth")]
    let app = app
        .route(
            "/v2/providers/auth",
            get(server_impl::api::v2::providers::auth::get),
        )
        .route(
            "/v2/providers/auth/core/*path",
            any(server_impl::api::v2::providers::auth::core::path::any),
        )
        .route(
            "/v2/providers/auth/core",
            get(server_impl::api::v2::providers::auth::core::get),
        )
        .route(
            "/v2/providers/auth/default",
            delete(server_impl::api::v2::providers::auth::default::delete)
                .get(server_impl::api::v2::providers::auth::default::get)
                .put(server_impl::api::v2::providers::auth::default::put),
        )
        .route(
            "/v2/providers/auth/default/*path",
            any(server_impl::api::v2::providers::auth::default::path::any),
        )
        .route(
            "/v2/providers/auth/:id",
            get(server_impl::api::v2::providers::auth::id::get),
        )
        .route(
            "/v2/providers/auth/:provider_id/*path",
            any(server_impl::api::v2::providers::auth::id::path::any),
        );
    // The following routes are exempt from the enforcer middleware and the roles/authentication has
    // to be checked in each handler
    #[cfg(feature = "auth")]
    let unenforced = Router::new()
        .route(
            "/v2/providers/auth/first-time-setup/flecsport",
            axum::routing::post(
                server_impl::api::v2::providers::auth::first_time_setup::flecsport::post::<IMP, F>,
            ),
        )
        .route(
            "/v2/providers/auth/core",
            put(server_impl::api::v2::providers::auth::core::put),
        );
    #[cfg(feature = "auth")]
    let unenforced = unenforced.layer(axum::middleware::from_fn_with_state(
        wall.watch.clone(),
        auth_middleware,
    ));
    #[cfg(feature = "auth")]
    let app = app
        .layer(axum::middleware::from_fn_with_state(
            wall.enforcer,
            roles_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            wall.watch,
            auth_middleware,
        ))
        .merge(unenforced);
    let app = app
        // It is not feasible to configure the body limit per route as we would have to manually
        // generated code (flecsd_axum_server::server::new). We therefore disable the limit for all
        // routes. As we should always operate behind a nginx which controls the max size per route
        // this should impose minimal security issues.
        .layer(DefaultBodyLimit::disable())
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);
                    let path = request.uri().path();
                    if is_trace_level_path(path) {
                        trace_span!(
                            "http_request",
                            method = ?request.method(),
                            matched_path,
                            path,
                            error = tracing::field::Empty
                        )
                    } else {
                        info_span!(
                            "http_request",
                            method = ?request.method(),
                            matched_path,
                            path,
                            error = tracing::field::Empty
                        )
                    }
                })
                .on_request(|req: &Request<_>, _span: &Span| {
                    let path = req.uri().path();
                    if is_trace_level_path(path) {
                        tracing::trace!("request: {} {}", req.method(), path)
                    } else {
                        tracing::debug!("request: {} {}", req.method(), path)
                    }
                })
                .on_failure(
                    |_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        _span.record("error", _error.to_string());
                    },
                )
                .on_response(
                    |res: &http::Response<axum::body::Body>, latency: Duration, span: &Span| {
                        // We have no simple way to access the path directly to check which trace
                        // level we want, so instead we check the trace level of the span which was
                        // chosen depending on the path
                        match span.metadata().map(|meta| meta.level()) {
                            Some(&tracing::Level::INFO) => {
                                tracing::debug!("response: {} in {:?}", res.status(), latency)
                            }
                            _ => tracing::trace!("response: {} in {:?}", res.status(), latency),
                        }
                    },
                ),
        );
    let app = app.with_state(server);
    Ok(app.into_make_service_with_connect_info::<C>())
}

async fn create_unix_socket(socket_path: PathBuf) -> Result<UnixListener> {
    let _ = tokio::fs::remove_file(&socket_path).await;
    if let Some(parent) = socket_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let uds = UnixListener::bind(socket_path.clone())?;
    let mut perm = fs::metadata(socket_path.clone()).await?.permissions();
    // Allow group and others to write
    perm.set_mode(perm.mode() | 0o022);
    fs::set_permissions(&socket_path, perm).await?;
    Ok(uds)
}

async fn create_tcp_listener(port: u16, bind_address: Option<IpAddr>) -> Result<TcpListener> {
    const IPV6_ANY: Ipv6Addr = Ipv6Addr::UNSPECIFIED;
    const IPV4_ANY: Ipv4Addr = Ipv4Addr::UNSPECIFIED;
    let listener = if let Some(bind_address) = bind_address {
        TcpListener::bind((bind_address, port)).await?
    } else {
        match TcpListener::bind((IPV6_ANY, port)).await {
            Ok(listener) => listener,
            Err(e) => {
                warn!(
                    "Failed to bind to ipv6 address {IPV6_ANY}, falling back to ipv4 ({IPV4_ANY}): {e}"
                );
                TcpListener::bind((IPV4_ANY, port)).await?
            }
        }
    };
    Ok(listener)
}

async fn serve<L, C>(
    mut listener: L,
    service: IntoMakeServiceWithConnectInfo<Router, C>,
    mut shutdown_signal: tokio::sync::oneshot::Receiver<()>,
) where
    L: tokio_util::net::Listener + Send,
    L::Io: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    C: for<'a> connect_info::Connected<&'a L::Io> + Clone + Send + Sync + 'static,
{
    let mut service = service;
    loop {
        tokio::select! {
            _ = &mut shutdown_signal => {
                info!("Server shutting down.");
                break
            },
            new_connection = listener.accept() => {
                let (socket, _remote_addr) = new_connection.unwrap();
                let tower_service = unwrap_infallible(service.call(&socket).await);

                tokio::spawn(async move {
                    let socket = TokioIo::new(socket);

                    let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                        tower_service.clone().call(request)
                    });

                    if let Err(err) = server::conn::auto::Builder::new(TokioExecutor::new())
                        .serve_connection_with_upgrades(socket, hyper_service)
                        .await
                    {
                        error!("failed to serve connection: {err:#}");
                    }
                });
            }
        }
    }
}

pub struct ServerHandle {
    server_shutdown_sender: tokio::sync::oneshot::Sender<()>,
    server_shutdown_finished_receiver: tokio::sync::oneshot::Receiver<()>,
}

impl ServerHandle {
    pub async fn shutdown(self) {
        self.server_shutdown_sender.send(()).unwrap();
        _ = self.server_shutdown_finished_receiver.await;
    }
}

pub async fn spawn_server<
    APP: AppRaiser + 'static,
    AUTH: Authmancer + 'static,
    I: Instancius + 'static,
    L: Licenso + 'static,
    Q: MageQuester + 'static,
    M: Manifesto + 'static,
    SYS: Systemus + 'static,
    D: Deploymento + 'static,
    E: Exportius + 'static,
    IMP: Importius + 'static,
    F: Floxy + 'static,
>(
    sorcerers: Sorcerers<APP, AUTH, I, L, Q, M, SYS, D, E, IMP>,
    enchantments: Enchantments<F>,
    vault: Arc<Vault>,
    lore: Arc<Lore>,
    #[cfg(feature = "auth")] wall: wall::Wall,
) -> Result<ServerHandle> {
    enum _Listener<T> {
        Socket(
            UnixListener,
            IntoMakeServiceWithConnectInfo<T, UdsConnectInfo>,
        ),
        Port(
            TcpListener,
            IntoMakeServiceWithConnectInfo<T, TcpConnectInfo>,
        ),
    }
    let (server_shutdown_sender, server_shutdown_receiver) = tokio::sync::oneshot::channel();
    let (server_shutdown_finished_sender, server_shutdown_finished_receiver) =
        tokio::sync::oneshot::channel();
    let (log_location, listener) = match lore.listener.clone() {
        Listener::UnixSocket(path) => (
            format!("unix socket {}", path.display()),
            _Listener::Socket(
                create_unix_socket(path).await?,
                create_service(
                    sorcerers,
                    enchantments,
                    vault,
                    lore,
                    #[cfg(feature = "auth")]
                    wall,
                )
                .await?,
            ),
        ),
        Listener::TCP { port, bind_address } => (
            if let Some(address) = bind_address {
                format!("port {address}:{port}")
            } else {
                format!("port {port}")
            },
            _Listener::Port(
                create_tcp_listener(port, bind_address).await?,
                create_service(
                    sorcerers,
                    enchantments,
                    vault,
                    lore,
                    #[cfg(feature = "auth")]
                    wall,
                )
                .await?,
            ),
        ),
    };
    tokio::spawn(async move {
        info!("Starting rust server listening on {log_location}");
        #[cfg(feature = "dev-auth")]
        warn!(
            "Feature dev-auth is enabled which will disable all authorization checks on http requests"
        );
        match listener {
            _Listener::Socket(listener, service) => {
                serve(listener, service, server_shutdown_receiver).await
            }
            _Listener::Port(listener, service) => {
                serve(listener, service, server_shutdown_receiver).await
            }
        }
        info!("Rust server listening on {log_location} stopped");
        server_shutdown_finished_sender.send(()).unwrap()
    });
    Ok(ServerHandle {
        server_shutdown_sender,
        server_shutdown_finished_receiver,
    })
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct UdsConnectInfo {
    peer_addr: Arc<tokio::net::unix::SocketAddr>,
    peer_cred: UCred,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct TcpConnectInfo {
    peer_addr: Arc<std::net::SocketAddr>,
}

impl connect_info::Connected<&UnixStream> for UdsConnectInfo {
    fn connect_info(target: &UnixStream) -> Self {
        let peer_addr = target.peer_addr().unwrap();
        let peer_cred = target.peer_cred().unwrap();
        Self {
            peer_addr: Arc::new(peer_addr),
            peer_cred,
        }
    }
}

impl connect_info::Connected<&tokio::net::TcpStream> for TcpConnectInfo {
    fn connect_info(target: &tokio::net::TcpStream) -> Self {
        let peer_addr = target.peer_addr().unwrap();
        Self {
            peer_addr: Arc::new(peer_addr),
        }
    }
}

fn unwrap_infallible<T>(result: std::result::Result<T, Infallible>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => match err {},
    }
}
