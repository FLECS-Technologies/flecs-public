pub mod console_client;
mod server_impl;
pub mod world;
use crate::enchantment::Enchantments;
use crate::enchantment::floxy::Floxy;
use crate::lore::Lore;
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
use axum::extract::DefaultBodyLimit;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
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
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::os::unix::fs::PermissionsExt;
use std::str::FromStr;
use std::time::Duration;
use std::{convert::Infallible, path::PathBuf, sync::Arc};
use tokio::fs;
use tokio::net::{UnixListener, UnixStream, unix::UCred};
use tower::Service;
use tower_http::classify::ServerErrorsFailureClass;
use tracing::{Span, error, info, info_span, trace_span};
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
>(
    sorcerers: Sorcerers<APP, AUTH, I, L, Q, M, SYS, D, E, IMP>,
    enchantments: Enchantments<F>,
    vault: Arc<Vault>,
    lore: Arc<Lore>,
) -> IntoMakeServiceWithConnectInfo<Router, UdsConnectInfo> {
    let server = server_impl::ServerImpl::new(
        vault,
        lore,
        sorcerers,
        enchantments,
        UsbDeviceReaderImpl::default(),
        NetworkAdapterReaderImpl,
        NetDeviceReaderImpl,
    )
    .await;
    let app = flecsd_axum_server::server::new(Arc::new(server))
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
                                tracing::debug!("response on: {} in {:?}", res.status(), latency)
                            }
                            _ => tracing::trace!("response on: {} in {:?}", res.status(), latency),
                        }
                    },
                ),
        );
    app.into_make_service_with_connect_info::<UdsConnectInfo>()
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

async fn serve(
    unix_listener: UnixListener,
    service: IntoMakeServiceWithConnectInfo<Router, UdsConnectInfo>,
    mut shutdown_signal: tokio::sync::oneshot::Receiver<()>,
) {
    let mut service = service;
    loop {
        tokio::select! {
            _ = &mut shutdown_signal => {
                info!("Server shutting down.");
                break
            },
            new_connection = unix_listener.accept() => {
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
) -> Result<ServerHandle> {
    let (server_shutdown_sender, server_shutdown_receiver) = tokio::sync::oneshot::channel();
    let (server_shutdown_finished_sender, server_shutdown_finished_receiver) =
        tokio::sync::oneshot::channel();
    let socket_path = lore.flecsd_socket_path.clone();
    let unix_listener = create_unix_socket(lore.flecsd_socket_path.clone()).await?;
    let service = create_service(sorcerers, enchantments, vault, lore).await;
    tokio::spawn(async move {
        info!("Starting rust server listening on {socket_path:?}");
        serve(unix_listener, service, server_shutdown_receiver).await;
        info!("Rust server listening on {socket_path:?} stopped");
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

fn unwrap_infallible<T>(result: std::result::Result<T, Infallible>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => match err {},
    }
}
