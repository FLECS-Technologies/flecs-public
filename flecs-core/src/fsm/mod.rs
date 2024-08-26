pub mod console_client;
mod server_impl;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
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

pub struct StartupError(String);

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

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug,axum::rejection=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Tracing initialized");
}

fn create_service() -> IntoMakeServiceWithConnectInfo<Router, UdsConnectInfo> {
    let server = server_impl::ServerImpl::default();
    let app = flecsd_axum_server::server::new(Arc::new(server)).layer(
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
) {
    let mut service = service;
    loop {
        let (socket, _remote_addr) = unix_listener.accept().await.unwrap();
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

pub async fn server(socket_path: PathBuf) -> Result<()> {
    serve(create_unix_socket(socket_path).await?, create_service()).await;
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
