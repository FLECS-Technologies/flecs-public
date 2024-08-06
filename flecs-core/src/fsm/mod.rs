use axum::async_trait;
use axum::extract::*;
use axum_extra::extract::CookieJar;
use flecsd_axum_server::apis::apps::{
    Apps, AppsAppDeleteResponse, AppsAppGetResponse, AppsGetResponse, AppsInstallPostResponse,
    AppsSideloadPostResponse,
};
use flecsd_axum_server::apis::console::{
    Console, ConsoleAuthenticationDeleteResponse, ConsoleAuthenticationPutResponse,
};
use flecsd_axum_server::apis::device::{
    Device, DeviceLicenseActivationPostResponse, DeviceLicenseActivationStatusGetResponse,
    DeviceLicenseInfoGetResponse, DeviceOnboardingPostResponse,
};
use flecsd_axum_server::apis::flunder::{Flunder, FlunderBrowseGetResponse};
use flecsd_axum_server::apis::instances::{
    Instances, InstancesCreatePostResponse, InstancesGetResponse,
    InstancesInstanceIdConfigEnvironmentDeleteResponse,
    InstancesInstanceIdConfigEnvironmentGetResponse,
    InstancesInstanceIdConfigEnvironmentPutResponse, InstancesInstanceIdConfigGetResponse,
    InstancesInstanceIdConfigPortsDeleteResponse, InstancesInstanceIdConfigPortsGetResponse,
    InstancesInstanceIdConfigPortsPutResponse, InstancesInstanceIdConfigPostResponse,
    InstancesInstanceIdDeleteResponse, InstancesInstanceIdGetResponse,
    InstancesInstanceIdLogsGetResponse, InstancesInstanceIdPatchResponse,
    InstancesInstanceIdStartPostResponse, InstancesInstanceIdStopPostResponse,
};
use flecsd_axum_server::apis::jobs::{
    Jobs, JobsGetResponse, JobsJobIdDeleteResponse, JobsJobIdGetResponse,
};
use flecsd_axum_server::apis::system::{
    System, SystemInfoGetResponse, SystemPingGetResponse, SystemVersionGetResponse,
};
use flecsd_axum_server::models::{
    AppKey, AppStatus, AppsAppDeletePathParams, AppsAppDeleteQueryParams, AppsAppGetPathParams,
    AppsAppGetQueryParams, AppsInstallPostRequest, AppsSideloadPostRequest, AuthResponseData,
    DosManifest, FlunderBrowseGetQueryParams, InstalledApp, InstanceConfig, InstanceEnvironment,
    InstancePorts, InstancesCreatePostRequest, InstancesGetQueryParams,
    InstancesInstanceIdConfigEnvironmentDeletePathParams,
    InstancesInstanceIdConfigEnvironmentGetPathParams,
    InstancesInstanceIdConfigEnvironmentPutPathParams, InstancesInstanceIdConfigGetPathParams,
    InstancesInstanceIdConfigPortsDeletePathParams, InstancesInstanceIdConfigPortsGetPathParams,
    InstancesInstanceIdConfigPortsPutPathParams, InstancesInstanceIdConfigPostPathParams,
    InstancesInstanceIdDeletePathParams, InstancesInstanceIdGetPathParams,
    InstancesInstanceIdLogsGetPathParams, InstancesInstanceIdPatchPathParams,
    InstancesInstanceIdPatchRequest, InstancesInstanceIdStartPostPathParams,
    InstancesInstanceIdStopPostPathParams, JobsJobIdDeletePathParams, JobsJobIdGetPathParams,
};
use http::Method;

struct ServerImpl {}

#[async_trait]
impl Apps for ServerImpl {
    async fn apps_app_delete(
        &self,
        _method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: AppsAppDeletePathParams,
        query_params: AppsAppDeleteQueryParams,
    ) -> Result<AppsAppDeleteResponse, String> {
        todo!()
    }

    async fn apps_app_get(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: AppsAppGetPathParams,
        query_params: AppsAppGetQueryParams,
    ) -> Result<AppsAppGetResponse, String> {
        println!(
            "Received app request from {} for app {} in version {}",
            host.0,
            path_params.app,
            query_params.version.unwrap_or("unknown".to_string())
        );
        Ok(AppsAppGetResponse::Status200_Success(vec![InstalledApp {
            app_key: AppKey {
                name: "some app".into(),
                version: "1.0.2".into(),
            },
            status: AppStatus::Installed,
            desired: AppStatus::Installed,
            editor: ":123".into(),
            installed_size: 1234,
            multi_instance: false,
        }]))
    }

    async fn apps_get(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
    ) -> Result<AppsGetResponse, String> {
        todo!()
    }

    async fn apps_install_post(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        body: AppsInstallPostRequest,
    ) -> Result<AppsInstallPostResponse, String> {
        todo!()
    }

    async fn apps_sideload_post(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        body: AppsSideloadPostRequest,
    ) -> Result<AppsSideloadPostResponse, String> {
        todo!()
    }
}

#[async_trait]
impl Console for ServerImpl {
    async fn console_authentication_delete(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
    ) -> Result<ConsoleAuthenticationDeleteResponse, String> {
        todo!()
    }

    async fn console_authentication_put(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        body: AuthResponseData,
    ) -> Result<ConsoleAuthenticationPutResponse, String> {
        todo!()
    }
}

#[async_trait]
impl Device for ServerImpl {
    async fn device_license_activation_post(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
    ) -> Result<DeviceLicenseActivationPostResponse, String> {
        todo!()
    }

    async fn device_license_activation_status_get(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
    ) -> Result<DeviceLicenseActivationStatusGetResponse, String> {
        todo!()
    }

    async fn device_license_info_get(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
    ) -> Result<DeviceLicenseInfoGetResponse, String> {
        todo!()
    }

    async fn device_onboarding_post(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        body: DosManifest,
    ) -> Result<DeviceOnboardingPostResponse, String> {
        todo!()
    }
}

#[async_trait]
impl Flunder for ServerImpl {
    async fn flunder_browse_get(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        query_params: FlunderBrowseGetQueryParams,
    ) -> Result<FlunderBrowseGetResponse, String> {
        todo!()
    }
}

#[async_trait]
impl Instances for ServerImpl {
    async fn instances_create_post(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        body: InstancesCreatePostRequest,
    ) -> Result<InstancesCreatePostResponse, String> {
        todo!()
    }

    async fn instances_get(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        query_params: InstancesGetQueryParams,
    ) -> Result<InstancesGetResponse, String> {
        todo!()
    }

    async fn instances_instance_id_config_environment_delete(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: InstancesInstanceIdConfigEnvironmentDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigEnvironmentDeleteResponse, String> {
        todo!()
    }

    async fn instances_instance_id_config_environment_get(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: InstancesInstanceIdConfigEnvironmentGetPathParams,
    ) -> Result<InstancesInstanceIdConfigEnvironmentGetResponse, String> {
        todo!()
    }

    async fn instances_instance_id_config_environment_put(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: InstancesInstanceIdConfigEnvironmentPutPathParams,
        body: InstanceEnvironment,
    ) -> Result<InstancesInstanceIdConfigEnvironmentPutResponse, String> {
        todo!()
    }

    async fn instances_instance_id_config_get(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: InstancesInstanceIdConfigGetPathParams,
    ) -> Result<InstancesInstanceIdConfigGetResponse, String> {
        todo!()
    }

    async fn instances_instance_id_config_ports_delete(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: InstancesInstanceIdConfigPortsDeletePathParams,
    ) -> Result<InstancesInstanceIdConfigPortsDeleteResponse, String> {
        todo!()
    }

    async fn instances_instance_id_config_ports_get(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: InstancesInstanceIdConfigPortsGetPathParams,
    ) -> Result<InstancesInstanceIdConfigPortsGetResponse, String> {
        todo!()
    }

    async fn instances_instance_id_config_ports_put(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: InstancesInstanceIdConfigPortsPutPathParams,
        body: InstancePorts,
    ) -> Result<InstancesInstanceIdConfigPortsPutResponse, String> {
        todo!()
    }

    async fn instances_instance_id_config_post(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: InstancesInstanceIdConfigPostPathParams,
        body: InstanceConfig,
    ) -> Result<InstancesInstanceIdConfigPostResponse, String> {
        todo!()
    }

    async fn instances_instance_id_delete(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: InstancesInstanceIdDeletePathParams,
    ) -> Result<InstancesInstanceIdDeleteResponse, String> {
        todo!()
    }

    async fn instances_instance_id_get(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: InstancesInstanceIdGetPathParams,
    ) -> Result<InstancesInstanceIdGetResponse, String> {
        todo!()
    }

    async fn instances_instance_id_logs_get(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: InstancesInstanceIdLogsGetPathParams,
    ) -> Result<InstancesInstanceIdLogsGetResponse, String> {
        todo!()
    }

    async fn instances_instance_id_patch(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: InstancesInstanceIdPatchPathParams,
        body: InstancesInstanceIdPatchRequest,
    ) -> Result<InstancesInstanceIdPatchResponse, String> {
        todo!()
    }

    async fn instances_instance_id_start_post(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: InstancesInstanceIdStartPostPathParams,
    ) -> Result<InstancesInstanceIdStartPostResponse, String> {
        todo!()
    }

    async fn instances_instance_id_stop_post(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: InstancesInstanceIdStopPostPathParams,
    ) -> Result<InstancesInstanceIdStopPostResponse, String> {
        todo!()
    }
}

#[async_trait]
impl Jobs for ServerImpl {
    async fn jobs_get(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
    ) -> Result<JobsGetResponse, String> {
        todo!()
    }

    async fn jobs_job_id_delete(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: JobsJobIdDeletePathParams,
    ) -> Result<JobsJobIdDeleteResponse, String> {
        todo!()
    }

    async fn jobs_job_id_get(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
        path_params: JobsJobIdGetPathParams,
    ) -> Result<JobsJobIdGetResponse, String> {
        todo!()
    }
}

#[async_trait]
impl System for ServerImpl {
    async fn system_info_get(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
    ) -> Result<SystemInfoGetResponse, String> {
        todo!()
    }

    async fn system_ping_get(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
    ) -> Result<SystemPingGetResponse, String> {
        todo!()
    }

    async fn system_version_get(
        &self,
        method: http::method::Method,
        host: Host,
        cookies: axum_extra::extract::cookie::CookieJar,
    ) -> Result<SystemVersionGetResponse, String> {
        todo!()
    }
}

#[cfg(unix)]
mod unix {
    use crate::fsm::ServerImpl;
    use axum::{
        body::Body,
        extract::connect_info::{self, ConnectInfo},
        extract::MatchedPath,
        http::{Method, Request, StatusCode},
        routing::get,
        Router,
    };
    use axum::{
        body::Bytes,
        http::HeaderMap,
        response::{Html, Response},
    };
    use http_body_util::BodyExt;
    use hyper::body::Incoming;
    use hyper_util::{
        rt::{TokioExecutor, TokioIo},
        server,
    };
    use std::time::Duration;
    use std::{convert::Infallible, path::PathBuf, sync::Arc};
    use tokio::net::TcpListener;
    use tokio::net::{unix::UCred, UnixListener, UnixStream};
    use tower::Service;
    use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
    use tracing::{info_span, Span};
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    pub async fn server() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                    // axum logs rejections from built-in extractors with the `axum::rejection`
                    // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                    "example_tracing_aka_logging=debug,tower_http=debug,axum::rejection=debug"
                        .into()
                }),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();

        let path = PathBuf::from("/tmp/axum/helloworld");

        let _ = tokio::fs::remove_file(&path).await;
        tokio::fs::create_dir_all(path.parent().unwrap())
            .await
            .unwrap();

        let uds = UnixListener::bind(path.clone()).unwrap();
        let server = ServerImpl {};
        let app = flecsd_axum_server::server::new(Arc::new(server)).layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    // Log the matched route's path (with placeholders not filled in).
                    // Use request.uri() or OriginalUri if you want the real path.
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
                ),
        );

        let mut make_service = app.into_make_service_with_connect_info::<UdsConnectInfo>();

        // See https://github.com/tokio-rs/axum/blob/main/examples/serve-with-hyper/src/main.rs for
        // more details about this setup
        loop {
            let (socket, _remote_addr) = uds.accept().await.unwrap();
            let tower_service = unwrap_infallible(make_service.call(&socket).await);

            tokio::spawn(async move {
                let socket = TokioIo::new(socket);

                let hyper_service =
                    hyper::service::service_fn(move |request: Request<Incoming>| {
                        let result = tower_service.clone().call(request);
                        result
                    });

                if let Err(err) = server::conn::auto::Builder::new(TokioExecutor::new())
                    .serve_connection_with_upgrades(socket, hyper_service)
                    .await
                {
                    eprintln!("failed to serve connection: {err:#}");
                }
            });
        }
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

    fn unwrap_infallible<T>(result: Result<T, Infallible>) -> T {
        match result {
            Ok(value) => value,
            Err(err) => match err {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn start_server() {
        unix::server().await;
    }
}
