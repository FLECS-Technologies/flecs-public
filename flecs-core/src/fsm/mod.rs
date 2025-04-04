pub mod console_client;
mod server_impl;
pub mod world;
use crate::enchantment::floxy::Floxy;
use crate::enchantment::Enchantments;
use crate::relic::device::net::NetDeviceReaderImpl;
use crate::relic::device::usb::UsbDeviceReaderImpl;
use crate::relic::network::NetworkAdapterReaderImpl;
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
use crate::sorcerer::Sorcerers;
use crate::vault::Vault;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum::extract::DefaultBodyLimit;
use axum::{
    extract::connect_info::{self},
    extract::MatchedPath,
    http::Request,
    Router,
};
use hyper::body::Incoming;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server,
};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;
use std::{convert::Infallible, path::PathBuf, sync::Arc};
use tokio::fs;
use tokio::net::{unix::UCred, UnixListener, UnixStream};
use tower::Service;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::trace::DefaultOnResponse;
use tracing::{error, info, info_span, Span};
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
        std::env::set_var("RUST_BACKTRACE", BT_VALUE);
    }
}

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(crate::lore::tracing::default_filter())
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Tracing initialized");
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
) -> IntoMakeServiceWithConnectInfo<Router, UdsConnectInfo> {
    let server = server_impl::ServerImpl::new(
        vault,
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
                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                        path,
                        error = tracing::field::Empty
                    )
                })
                .on_failure(
                    |_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        _span.record("error", _error.to_string());
                    },
                )
                .on_response(DefaultOnResponse::default().include_headers(true)),
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

pub fn spawn_server<
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
    socket_path: PathBuf,
    enchantments: Enchantments<F>,
    vault: Arc<Vault>,
) -> ServerHandle {
    let (server_shutdown_sender, server_shutdown_receiver) = tokio::sync::oneshot::channel();
    let (server_shutdown_finished_sender, server_shutdown_finished_receiver) =
        tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        info!("Starting rust server listening on {socket_path:?}");
        crate::fsm::server(
            sorcerers,
            socket_path.clone(),
            enchantments,
            vault,
            server_shutdown_receiver,
        )
        .await
        .unwrap();
        info!("Rust server listening on {socket_path:?} stopped");
        server_shutdown_finished_sender.send(()).unwrap()
    });
    ServerHandle {
        server_shutdown_sender,
        server_shutdown_finished_receiver,
    }
}

pub async fn server<
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
    socket_path: PathBuf,
    enchantments: Enchantments<F>,
    vault: Arc<Vault>,
    shutdown_signal: tokio::sync::oneshot::Receiver<()>,
) -> Result<()> {
    serve(
        create_unix_socket(socket_path).await?,
        create_service(sorcerers, enchantments, vault).await,
        shutdown_signal,
    )
    .await;
    Ok(())
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
