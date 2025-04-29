use async_trait::async_trait;
use axum::extract::rejection::{JsonRejection, MissingJsonContentType};
use axum::extract::{FromRequest, Host, Path, Query, State};
use axum::extract::{Json as AxumJson, Request};
use axum::{body::Body, response::Response, routing::*};
use axum_extra::extract::{CookieJar, Multipart};
use bytes::Bytes;
use http::{header::CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use tracing::error;
use validator::{Validate, ValidationErrors};

use crate::{header, types::*};

#[allow(unused_imports)]
use crate::{apis, models};

#[derive(Debug, Clone, Copy, Default)]
#[must_use]
pub struct Json<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = JsonRejection;

    async fn from_request(mut req: Request, state: &S) -> Result<Self, Self::Rejection> {
        req.headers_mut().insert(
            "Content-Type",
            HeaderValue::from_str("application/json").unwrap(),
        );
        let json = AxumJson::<T>::from_request(req, state).await?;
        Ok(Self(json.0))
    }
}

/// Setup API Server.
pub fn new<I, A>(api_impl: I) -> Router
where
    I: AsRef<A> + Clone + Send + Sync + 'static,
    A: apis::apps::Apps
        + apis::console::Console
        + apis::deployments::Deployments
        + apis::device::Device
        + apis::flecsport::Flecsport
        + apis::instances::Instances
        + apis::jobs::Jobs
        + apis::system::System
        + 'static,
{
    // build our application with a route
    Router::new()
        .route("/v2/apps",
            get(apps_get::<I, A>)
        )
        .route("/v2/apps/:app",
            delete(apps_app_delete::<I, A>).get(apps_app_get::<I, A>)
        )
        .route("/v2/apps/install",
            post(apps_install_post::<I, A>)
        )
        .route("/v2/apps/sideload",
            post(apps_sideload_post::<I, A>)
        )
        .route("/v2/console/authentication",
            delete(console_authentication_delete::<I, A>).put(console_authentication_put::<I, A>)
        )
        .route("/v2/deployments/:deployment_id/networks",
            get(deployments_deployment_id_networks_get::<I, A>)
        )
        .route("/v2/deployments/:deployment_id/networks/:network_id",
            get(deployments_deployment_id_networks_network_id_get::<I, A>).put(deployments_deployment_id_networks_network_id_put::<I, A>)
        )
        .route("/v2/deployments/:deployment_id/networks/:network_id/dhcp/ipv4",
            post(deployments_deployment_id_networks_network_id_dhcp_ipv4_post::<I, A>)
        )
        .route("/v2/device/license/activation",
            post(device_license_activation_post::<I, A>)
        )
        .route("/v2/device/license/activation/status",
            get(device_license_activation_status_get::<I, A>)
        )
        .route("/v2/device/license/info",
            get(device_license_info_get::<I, A>)
        )
        .route("/v2/device/onboarding",
            post(device_onboarding_post::<I, A>)
        )
        .route("/v2/exports",
            get(exports_get::<I, A>).post(exports_post::<I, A>)
        )
        .route("/v2/exports/:export_id",
            delete(exports_export_id_delete::<I, A>).get(exports_export_id_get::<I, A>)
        )
        .route("/v2/imports",
            post(imports_post::<I, A>)
        )
        .route("/v2/instances",
            get(instances_get::<I, A>)
        )
        .route("/v2/instances/:instance_id",
            delete(instances_instance_id_delete::<I, A>).get(instances_instance_id_get::<I, A>).patch(instances_instance_id_patch::<I, A>)
        )
        .route("/v2/instances/:instance_id/config/devices/usb",
            delete(instances_instance_id_config_devices_usb_delete::<I, A>).get(instances_instance_id_config_devices_usb_get::<I, A>)
        )
        .route("/v2/instances/:instance_id/config/devices/usb/:port",
            delete(instances_instance_id_config_devices_usb_port_delete::<I, A>).get(instances_instance_id_config_devices_usb_port_get::<I, A>).put(instances_instance_id_config_devices_usb_port_put::<I, A>)
        )
        .route("/v2/instances/:instance_id/config/environment",
            delete(instances_instance_id_config_environment_delete::<I, A>).get(instances_instance_id_config_environment_get::<I, A>).put(instances_instance_id_config_environment_put::<I, A>)
        )
        .route("/v2/instances/:instance_id/config/environment/:variable_name",
            delete(instances_instance_id_config_environment_variable_name_delete::<I, A>).get(instances_instance_id_config_environment_variable_name_get::<I, A>).put(instances_instance_id_config_environment_variable_name_put::<I, A>)
        )
        .route("/v2/instances/:instance_id/config/labels",
            get(instances_instance_id_config_labels_get::<I, A>)
        )
        .route("/v2/instances/:instance_id/config/labels/:label_name",
            get(instances_instance_id_config_labels_label_name_get::<I, A>)
        )
        .route("/v2/instances/:instance_id/config/mounts",
            get(instances_instance_id_config_mounts_get::<I, A>)
        )
        .route("/v2/instances/:instance_id/config/mounts/bind",
            get(instances_instance_id_config_mounts_bind_get::<I, A>)
        )
        .route("/v2/instances/:instance_id/config/mounts/bind/:container_path",
            get(instances_instance_id_config_mounts_bind_container_path_get::<I, A>)
        )
        .route("/v2/instances/:instance_id/config/mounts/volumes",
            get(instances_instance_id_config_mounts_volumes_get::<I, A>)
        )
        .route("/v2/instances/:instance_id/config/mounts/volumes/:volume_name",
            get(instances_instance_id_config_mounts_volumes_volume_name_get::<I, A>)
        )
        .route("/v2/instances/:instance_id/config/networks",
            get(instances_instance_id_config_networks_get::<I, A>).post(instances_instance_id_config_networks_post::<I, A>)
        )
        .route("/v2/instances/:instance_id/config/networks/:network_id",
            delete(instances_instance_id_config_networks_network_id_delete::<I, A>).get(instances_instance_id_config_networks_network_id_get::<I, A>)
        )
        .route("/v2/instances/:instance_id/config/ports",
            delete(instances_instance_id_config_ports_delete::<I, A>).get(instances_instance_id_config_ports_get::<I, A>)
        )
        .route("/v2/instances/:instance_id/config/ports/:transport_protocol",
            delete(instances_instance_id_config_ports_transport_protocol_delete::<I, A>).get(instances_instance_id_config_ports_transport_protocol_get::<I, A>).put(instances_instance_id_config_ports_transport_protocol_put::<I, A>)
        )
        .route("/v2/instances/:instance_id/config/ports/:transport_protocol/:host_port_range",
            delete(instances_instance_id_config_ports_transport_protocol_host_port_range_delete::<I, A>).get(instances_instance_id_config_ports_transport_protocol_host_port_range_get::<I, A>).put(instances_instance_id_config_ports_transport_protocol_host_port_range_put::<I, A>)
        )
        .route("/v2/instances/:instance_id/editor/:port",
            get(instances_instance_id_editor_port_get::<I, A>)
        )
        .route("/v2/instances/:instance_id/logs",
            get(instances_instance_id_logs_get::<I, A>)
        )
        .route("/v2/instances/:instance_id/start",
            post(instances_instance_id_start_post::<I, A>)
        )
        .route("/v2/instances/:instance_id/stop",
            post(instances_instance_id_stop_post::<I, A>)
        )
        .route("/v2/instances/create",
            post(instances_create_post::<I, A>)
        )
        .route("/v2/jobs",
            get(jobs_get::<I, A>)
        )
        .route("/v2/jobs/:job_id",
            delete(jobs_job_id_delete::<I, A>).get(jobs_job_id_get::<I, A>)
        )
        .route("/v2/system/devices",
            get(system_devices_get::<I, A>)
        )
        .route("/v2/system/devices/usb",
            get(system_devices_usb_get::<I, A>)
        )
        .route("/v2/system/devices/usb/:port",
            get(system_devices_usb_port_get::<I, A>)
        )
        .route("/v2/system/info",
            get(system_info_get::<I, A>)
        )
        .route("/v2/system/network_adapters",
            get(system_network_adapters_get::<I, A>)
        )
        .route("/v2/system/network_adapters/:network_adapter_id",
            get(system_network_adapters_network_adapter_id_get::<I, A>)
        )
        .route("/v2/system/ping",
            get(system_ping_get::<I, A>)
        )
        .route("/v2/system/version",
            get(system_version_get::<I, A>)
        )
        .with_state(api_impl)
}

#[tracing::instrument(skip_all)]
fn apps_app_delete_validation(
    path_params: models::AppsAppDeletePathParams,
    query_params: models::AppsAppDeleteQueryParams,
) -> std::result::Result<
    (
        models::AppsAppDeletePathParams,
        models::AppsAppDeleteQueryParams,
    ),
    ValidationErrors,
> {
    path_params.validate()?;
    query_params.validate()?;

    Ok((path_params, query_params))
}
/// AppsAppDelete - DELETE /v2/apps/{app}
#[tracing::instrument(skip_all)]
async fn apps_app_delete<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::AppsAppDeletePathParams>,
    Query(query_params): Query<models::AppsAppDeleteQueryParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::apps::Apps,
{
    #[allow(clippy::redundant_closure)]
    let validation =
        tokio::task::spawn_blocking(move || apps_app_delete_validation(path_params, query_params))
            .await
            .unwrap();

    let Ok((path_params, query_params)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .apps_app_delete(method, host, cookies, path_params, query_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::apps::AppsAppDeleteResponse::Status202_Accepted(body) => {
                let mut response = response.status(202);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::apps::AppsAppDeleteResponse::Status404_NoSuchAppOrApp => {
                let mut response = response.status(404);
                response.body(Body::empty())
            }
            apis::apps::AppsAppDeleteResponse::Status500_InternalServerError(body) => {
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn apps_app_get_validation(
    path_params: models::AppsAppGetPathParams,
    query_params: models::AppsAppGetQueryParams,
) -> std::result::Result<
    (models::AppsAppGetPathParams, models::AppsAppGetQueryParams),
    ValidationErrors,
> {
    path_params.validate()?;
    query_params.validate()?;

    Ok((path_params, query_params))
}
/// AppsAppGet - GET /v2/apps/{app}
#[tracing::instrument(skip_all)]
async fn apps_app_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::AppsAppGetPathParams>,
    Query(query_params): Query<models::AppsAppGetQueryParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::apps::Apps,
{
    #[allow(clippy::redundant_closure)]
    let validation =
        tokio::task::spawn_blocking(move || apps_app_get_validation(path_params, query_params))
            .await
            .unwrap();

    let Ok((path_params, query_params)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .apps_app_get(method, host, cookies, path_params, query_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::apps::AppsAppGetResponse::Status200_Success(body) => {
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::apps::AppsAppGetResponse::Status404_NoSuchAppOrApp => {
                let mut response = response.status(404);
                response.body(Body::empty())
            }
            apis::apps::AppsAppGetResponse::Status500_InternalServerError(body) => {
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn apps_get_validation() -> std::result::Result<(), ValidationErrors> {
    Ok(())
}
/// AppsGet - GET /v2/apps
#[tracing::instrument(skip_all)]
async fn apps_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::apps::Apps,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || apps_get_validation())
        .await
        .unwrap();

    let Ok(()) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl.as_ref().apps_get(method, host, cookies).await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::apps::AppsGetResponse::Status200_Success(body) => {
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::apps::AppsGetResponse::Status500_InternalServerError(body) => {
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(validator::Validate)]
#[allow(dead_code)]
struct AppsInstallPostBodyValidator<'a> {
    #[validate(nested)]
    body: &'a models::AppsInstallPostRequest,
}

#[tracing::instrument(skip_all)]
fn apps_install_post_validation(
    body: models::AppsInstallPostRequest,
) -> std::result::Result<(models::AppsInstallPostRequest,), ValidationErrors> {
    let b = AppsInstallPostBodyValidator { body: &body };
    b.validate()?;

    Ok((body,))
}
/// AppsInstallPost - POST /v2/apps/install
#[tracing::instrument(skip_all)]
async fn apps_install_post<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
    Json(body): Json<models::AppsInstallPostRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::apps::Apps,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || apps_install_post_validation(body))
        .await
        .unwrap();

    let Ok((body,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .apps_install_post(method, host, cookies, body)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::apps::AppsInstallPostResponse::Status202_Accepted(body) => {
                let mut response = response.status(202);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::apps::AppsInstallPostResponse::Status400_MalformedRequest(body) => {
                let mut response = response.status(400);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::apps::AppsInstallPostResponse::Status500_InternalServerError(body) => {
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(validator::Validate)]
#[allow(dead_code)]
struct AppsSideloadPostBodyValidator<'a> {
    #[validate(nested)]
    body: &'a models::AppsSideloadPostRequest,
}

#[tracing::instrument(skip_all)]
fn apps_sideload_post_validation(
    body: models::AppsSideloadPostRequest,
) -> std::result::Result<(models::AppsSideloadPostRequest,), ValidationErrors> {
    let b = AppsSideloadPostBodyValidator { body: &body };
    b.validate()?;

    Ok((body,))
}
/// AppsSideloadPost - POST /v2/apps/sideload
#[tracing::instrument(skip_all)]
async fn apps_sideload_post<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
    Json(body): Json<models::AppsSideloadPostRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::apps::Apps,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || apps_sideload_post_validation(body))
        .await
        .unwrap();

    let Ok((body,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .apps_sideload_post(method, host, cookies, body)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::apps::AppsSideloadPostResponse::Status202_Accepted(body) => {
                let mut response = response.status(202);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::apps::AppsSideloadPostResponse::Status400_MalformedRequest(body) => {
                let mut response = response.status(400);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn console_authentication_delete_validation() -> std::result::Result<(), ValidationErrors> {
    Ok(())
}
/// ConsoleAuthenticationDelete - DELETE /v2/console/authentication
#[tracing::instrument(skip_all)]
async fn console_authentication_delete<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::console::Console,
{
    #[allow(clippy::redundant_closure)]
    let validation =
        tokio::task::spawn_blocking(move || console_authentication_delete_validation())
            .await
            .unwrap();

    let Ok(()) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .console_authentication_delete(method, host, cookies)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::console::ConsoleAuthenticationDeleteResponse::Status204_NoContent => {
                let mut response = response.status(204);
                response.body(Body::empty())
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(validator::Validate)]
#[allow(dead_code)]
struct ConsoleAuthenticationPutBodyValidator<'a> {
    #[validate(nested)]
    body: &'a models::AuthResponseData,
}

#[tracing::instrument(skip_all)]
fn console_authentication_put_validation(
    body: models::AuthResponseData,
) -> std::result::Result<(models::AuthResponseData,), ValidationErrors> {
    let b = ConsoleAuthenticationPutBodyValidator { body: &body };
    b.validate()?;

    Ok((body,))
}
/// ConsoleAuthenticationPut - PUT /v2/console/authentication
#[tracing::instrument(skip_all)]
async fn console_authentication_put<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
    Json(body): Json<models::AuthResponseData>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::console::Console,
{
    #[allow(clippy::redundant_closure)]
    let validation =
        tokio::task::spawn_blocking(move || console_authentication_put_validation(body))
            .await
            .unwrap();

    let Ok((body,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .console_authentication_put(method, host, cookies, body)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::console::ConsoleAuthenticationPutResponse::Status204_NoContent => {
                let mut response = response.status(204);
                response.body(Body::empty())
            }
            apis::console::ConsoleAuthenticationPutResponse::Status400_MalformedRequest(body) => {
                let mut response = response.status(400);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn deployments_deployment_id_networks_get_validation(
    path_params: models::DeploymentsDeploymentIdNetworksGetPathParams,
) -> std::result::Result<(models::DeploymentsDeploymentIdNetworksGetPathParams,), ValidationErrors>
{
    path_params.validate()?;

    Ok((path_params,))
}
/// DeploymentsDeploymentIdNetworksGet - GET /v2/deployments/{deployment_id}/networks
#[tracing::instrument(skip_all)]
async fn deployments_deployment_id_networks_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::DeploymentsDeploymentIdNetworksGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::deployments::Deployments,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        deployments_deployment_id_networks_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .deployments_deployment_id_networks_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::deployments::DeploymentsDeploymentIdNetworksGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::deployments::DeploymentsDeploymentIdNetworksGetResponse::Status404_NoDeploymentWithThisDeployment
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                                apis::deployments::DeploymentsDeploymentIdNetworksGetResponse::Status500_InternalServerError
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn deployments_deployment_id_networks_network_id_dhcp_ipv4_post_validation(
    path_params: models::DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4PostPathParams,
) -> std::result::Result<
    (models::DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4PostPathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4Post - POST /v2/deployments/{deployment_id}/networks/{network_id}/dhcp/ipv4
#[tracing::instrument(skip_all)]
async fn deployments_deployment_id_networks_network_id_dhcp_ipv4_post<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4PostPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::deployments::Deployments,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        deployments_deployment_id_networks_network_id_dhcp_ipv4_post_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .deployments_deployment_id_networks_network_id_dhcp_ipv4_post(
            method,
            host,
            cookies,
            path_params,
        )
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::deployments::DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4PostResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::deployments::DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4PostResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::deployments::DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4PostResponse::Status404_ResourceNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::deployments::DeploymentsDeploymentIdNetworksNetworkIdDhcpIpv4PostResponse::Status500_InternalServerError
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn deployments_deployment_id_networks_network_id_get_validation(
    path_params: models::DeploymentsDeploymentIdNetworksNetworkIdGetPathParams,
) -> std::result::Result<
    (models::DeploymentsDeploymentIdNetworksNetworkIdGetPathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// DeploymentsDeploymentIdNetworksNetworkIdGet - GET /v2/deployments/{deployment_id}/networks/{network_id}
#[tracing::instrument(skip_all)]
async fn deployments_deployment_id_networks_network_id_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::DeploymentsDeploymentIdNetworksNetworkIdGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::deployments::Deployments,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        deployments_deployment_id_networks_network_id_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .deployments_deployment_id_networks_network_id_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::deployments::DeploymentsDeploymentIdNetworksNetworkIdGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::deployments::DeploymentsDeploymentIdNetworksNetworkIdGetResponse::Status404_ResourceNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::deployments::DeploymentsDeploymentIdNetworksNetworkIdGetResponse::Status500_InternalServerError
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(validator::Validate)]
#[allow(dead_code)]
struct DeploymentsDeploymentIdNetworksNetworkIdPutBodyValidator<'a> {
    #[validate(nested)]
    body: &'a models::PutDeploymentNetwork,
}

#[tracing::instrument(skip_all)]
fn deployments_deployment_id_networks_network_id_put_validation(
    path_params: models::DeploymentsDeploymentIdNetworksNetworkIdPutPathParams,
    body: models::PutDeploymentNetwork,
) -> std::result::Result<
    (
        models::DeploymentsDeploymentIdNetworksNetworkIdPutPathParams,
        models::PutDeploymentNetwork,
    ),
    ValidationErrors,
> {
    path_params.validate()?;
    let b = DeploymentsDeploymentIdNetworksNetworkIdPutBodyValidator { body: &body };
    b.validate()?;

    Ok((path_params, body))
}
/// DeploymentsDeploymentIdNetworksNetworkIdPut - PUT /v2/deployments/{deployment_id}/networks/{network_id}
#[tracing::instrument(skip_all)]
async fn deployments_deployment_id_networks_network_id_put<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::DeploymentsDeploymentIdNetworksNetworkIdPutPathParams>,
    State(api_impl): State<I>,
    Json(body): Json<models::PutDeploymentNetwork>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::deployments::Deployments,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        deployments_deployment_id_networks_network_id_put_validation(path_params, body)
    })
    .await
    .unwrap();

    let Ok((path_params, body)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .deployments_deployment_id_networks_network_id_put(method, host, cookies, path_params, body)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::deployments::DeploymentsDeploymentIdNetworksNetworkIdPutResponse::Status200_AlreadyCreated
                                                => {
                                                  let mut response = response.status(200);
                                                  response.body(Body::empty())
                                                },
                                                apis::deployments::DeploymentsDeploymentIdNetworksNetworkIdPutResponse::Status201_Created
                                                => {
                                                  let mut response = response.status(201);
                                                  response.body(Body::empty())
                                                },
                                                apis::deployments::DeploymentsDeploymentIdNetworksNetworkIdPutResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::deployments::DeploymentsDeploymentIdNetworksNetworkIdPutResponse::Status404_DeploymentNotFound
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                                apis::deployments::DeploymentsDeploymentIdNetworksNetworkIdPutResponse::Status500_InternalServerError
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn device_license_activation_post_validation() -> std::result::Result<(), ValidationErrors> {
    Ok(())
}
/// DeviceLicenseActivationPost - POST /v2/device/license/activation
#[tracing::instrument(skip_all)]
async fn device_license_activation_post<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::device::Device,
{
    #[allow(clippy::redundant_closure)]
    let validation =
        tokio::task::spawn_blocking(move || device_license_activation_post_validation())
            .await
            .unwrap();

    let Ok(()) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .device_license_activation_post(method, host, cookies)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::device::DeviceLicenseActivationPostResponse::Status200_Success(body) => {
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::device::DeviceLicenseActivationPostResponse::Status500_InternalServerError(
                body,
            ) => {
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn device_license_activation_status_get_validation() -> std::result::Result<(), ValidationErrors> {
    Ok(())
}
/// DeviceLicenseActivationStatusGet - GET /v2/device/license/activation/status
#[tracing::instrument(skip_all)]
async fn device_license_activation_status_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::device::Device,
{
    #[allow(clippy::redundant_closure)]
    let validation =
        tokio::task::spawn_blocking(move || device_license_activation_status_get_validation())
            .await
            .unwrap();

    let Ok(()) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .device_license_activation_status_get(method, host, cookies)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::device::DeviceLicenseActivationStatusGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::device::DeviceLicenseActivationStatusGetResponse::Status500_InternalServerError
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn device_license_info_get_validation() -> std::result::Result<(), ValidationErrors> {
    Ok(())
}
/// DeviceLicenseInfoGet - GET /v2/device/license/info
#[tracing::instrument(skip_all)]
async fn device_license_info_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::device::Device,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || device_license_info_get_validation())
        .await
        .unwrap();

    let Ok(()) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .device_license_info_get(method, host, cookies)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::device::DeviceLicenseInfoGetResponse::Status200_Success(body) => {
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(validator::Validate)]
#[allow(dead_code)]
struct DeviceOnboardingPostBodyValidator<'a> {
    #[validate(nested)]
    body: &'a models::Dosschema,
}

#[tracing::instrument(skip_all)]
fn device_onboarding_post_validation(
    body: models::Dosschema,
) -> std::result::Result<(models::Dosschema,), ValidationErrors> {
    let b = DeviceOnboardingPostBodyValidator { body: &body };
    b.validate()?;

    Ok((body,))
}
/// DeviceOnboardingPost - POST /v2/device/onboarding
#[tracing::instrument(skip_all)]
async fn device_onboarding_post<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
    Json(body): Json<models::Dosschema>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::device::Device,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || device_onboarding_post_validation(body))
        .await
        .unwrap();

    let Ok((body,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .device_onboarding_post(method, host, cookies, body)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::device::DeviceOnboardingPostResponse::Status202_Accepted(body) => {
                let mut response = response.status(202);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::device::DeviceOnboardingPostResponse::Status400_MalformedRequest(body) => {
                let mut response = response.status(400);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::device::DeviceOnboardingPostResponse::Status500_InternalServerError(body) => {
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn exports_export_id_delete_validation(
    path_params: models::ExportsExportIdDeletePathParams,
) -> std::result::Result<(models::ExportsExportIdDeletePathParams,), ValidationErrors> {
    path_params.validate()?;

    Ok((path_params,))
}
/// ExportsExportIdDelete - DELETE /v2/exports/{export_id}
#[tracing::instrument(skip_all)]
async fn exports_export_id_delete<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::ExportsExportIdDeletePathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::flecsport::Flecsport,
{
    #[allow(clippy::redundant_closure)]
    let validation =
        tokio::task::spawn_blocking(move || exports_export_id_delete_validation(path_params))
            .await
            .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .exports_export_id_delete(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::flecsport::ExportsExportIdDeleteResponse::Status200_Success => {
                let mut response = response.status(200);
                response.body(Body::empty())
            }
            apis::flecsport::ExportsExportIdDeleteResponse::Status400_ExportIdInvalid => {
                let mut response = response.status(400);
                response.body(Body::empty())
            }
            apis::flecsport::ExportsExportIdDeleteResponse::Status404_ExportNotFound => {
                let mut response = response.status(404);
                response.body(Body::empty())
            }
            apis::flecsport::ExportsExportIdDeleteResponse::Status500_InternalServerError(body) => {
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn exports_export_id_get_validation(
    path_params: models::ExportsExportIdGetPathParams,
) -> std::result::Result<(models::ExportsExportIdGetPathParams,), ValidationErrors> {
    path_params.validate()?;

    Ok((path_params,))
}
/// ExportsExportIdGet - GET /v2/exports/{export_id}
#[tracing::instrument(skip_all)]
async fn exports_export_id_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::ExportsExportIdGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::flecsport::Flecsport,
{
    #[allow(clippy::redundant_closure)]
    let validation =
        tokio::task::spawn_blocking(move || exports_export_id_get_validation(path_params))
            .await
            .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .exports_export_id_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::flecsport::ExportsExportIdGetResponse::Status200_Success(path) => {
                let file = tokio::fs::File::open(&path).await.unwrap();
                let stream = tokio_util::io::ReaderStream::new(file);
                let body = Body::from_stream(stream);
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/octet-stream").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                response.body(body)
            }
            apis::flecsport::ExportsExportIdGetResponse::Status400_ExportIdInvalid => {
                let mut response = response.status(400);
                response.body(Body::empty())
            }
            apis::flecsport::ExportsExportIdGetResponse::Status404_ExportNotFound => {
                let mut response = response.status(404);
                response.body(Body::empty())
            }
            apis::flecsport::ExportsExportIdGetResponse::Status500_InternalServerError(body) => {
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn exports_get_validation() -> std::result::Result<(), ValidationErrors> {
    Ok(())
}
/// ExportsGet - GET /v2/exports
#[tracing::instrument(skip_all)]
async fn exports_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::flecsport::Flecsport,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || exports_get_validation())
        .await
        .unwrap();

    let Ok(()) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl.as_ref().exports_get(method, host, cookies).await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::flecsport::ExportsGetResponse::Status200_Success(body) => {
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::flecsport::ExportsGetResponse::Status500_InternalServerError(body) => {
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(validator::Validate)]
#[allow(dead_code)]
struct ExportsPostBodyValidator<'a> {
    #[validate(nested)]
    body: &'a models::ExportRequest,
}

#[tracing::instrument(skip_all)]
fn exports_post_validation(
    body: models::ExportRequest,
) -> std::result::Result<(models::ExportRequest,), ValidationErrors> {
    let b = ExportsPostBodyValidator { body: &body };
    b.validate()?;

    Ok((body,))
}
/// ExportsPost - POST /v2/exports
#[tracing::instrument(skip_all)]
async fn exports_post<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
    Json(body): Json<models::ExportRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::flecsport::Flecsport,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || exports_post_validation(body))
        .await
        .unwrap();

    let Ok((body,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .exports_post(method, host, cookies, body)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::flecsport::ExportsPostResponse::Status202_Accepted(body) => {
                let mut response = response.status(202);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::flecsport::ExportsPostResponse::Status500_InternalServerError(body) => {
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn imports_post_validation(
    header_params: models::ImportsPostHeaderParams,
) -> std::result::Result<(models::ImportsPostHeaderParams,), ValidationErrors> {
    header_params.validate()?;

    Ok((header_params,))
}
/// ImportsPost - POST /v2/imports
#[tracing::instrument(skip_all)]
async fn imports_post<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    headers: HeaderMap,
    State(api_impl): State<I>,
    body: Multipart,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::flecsport::Flecsport,
{
    // Header parameters
    let header_params = {
        let header_content_disposition =
            headers.get(HeaderName::from_static("content-disposition"));

        let header_content_disposition = match header_content_disposition {
            Some(v) => match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                Ok(result) => result.0,
                Err(err) => {
                    return Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from(format!(
                            "Invalid header Content-Disposition - {}",
                            err
                        )))
                        .map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        });
                }
            },
            None => {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from("Missing required header Content-Disposition"))
                    .map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    });
            }
        };

        models::ImportsPostHeaderParams {
            content_disposition: header_content_disposition,
        }
    };

    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || imports_post_validation(header_params))
        .await
        .unwrap();

    let Ok((header_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .imports_post(method, host, cookies, header_params, body)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::flecsport::ImportsPostResponse::Status202_Accepted(body) => {
                let mut response = response.status(202);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::flecsport::ImportsPostResponse::Status400_MalformedRequest(body) => {
                let mut response = response.status(400);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::flecsport::ImportsPostResponse::Status500_InternalServerError(body) => {
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(validator::Validate)]
#[allow(dead_code)]
struct InstancesCreatePostBodyValidator<'a> {
    #[validate(nested)]
    body: &'a models::InstancesCreatePostRequest,
}

#[tracing::instrument(skip_all)]
fn instances_create_post_validation(
    body: models::InstancesCreatePostRequest,
) -> std::result::Result<(models::InstancesCreatePostRequest,), ValidationErrors> {
    let b = InstancesCreatePostBodyValidator { body: &body };
    b.validate()?;

    Ok((body,))
}
/// InstancesCreatePost - POST /v2/instances/create
#[tracing::instrument(skip_all)]
async fn instances_create_post<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
    Json(body): Json<models::InstancesCreatePostRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || instances_create_post_validation(body))
        .await
        .unwrap();

    let Ok((body,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_create_post(method, host, cookies, body)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::instances::InstancesCreatePostResponse::Status202_Accepted(body) => {
                let mut response = response.status(202);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::instances::InstancesCreatePostResponse::Status400_MalformedRequest(body) => {
                let mut response = response.status(400);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_get_validation(
    query_params: models::InstancesGetQueryParams,
) -> std::result::Result<(models::InstancesGetQueryParams,), ValidationErrors> {
    query_params.validate()?;

    Ok((query_params,))
}
/// InstancesGet - GET /v2/instances
#[tracing::instrument(skip_all)]
async fn instances_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Query(query_params): Query<models::InstancesGetQueryParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || instances_get_validation(query_params))
        .await
        .unwrap();

    let Ok((query_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_get(method, host, cookies, query_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::instances::InstancesGetResponse::Status200_Success(body) => {
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_devices_usb_delete_validation(
    path_params: models::InstancesInstanceIdConfigDevicesUsbDeletePathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigDevicesUsbDeletePathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigDevicesUsbDelete - DELETE /v2/instances/{instance_id}/config/devices/usb
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_devices_usb_delete<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigDevicesUsbDeletePathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_devices_usb_delete_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_devices_usb_delete(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbDeleteResponse::Status200_Success
                                                => {
                                                  let mut response = response.status(200);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbDeleteResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbDeleteResponse::Status404_NoInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_devices_usb_get_validation(
    path_params: models::InstancesInstanceIdConfigDevicesUsbGetPathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigDevicesUsbGetPathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigDevicesUsbGet - GET /v2/instances/{instance_id}/config/devices/usb
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_devices_usb_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigDevicesUsbGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_devices_usb_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_devices_usb_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbGetResponse::Status404_NoInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbGetResponse::Status500_InternalServerError
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_devices_usb_port_delete_validation(
    path_params: models::InstancesInstanceIdConfigDevicesUsbPortDeletePathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigDevicesUsbPortDeletePathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigDevicesUsbPortDelete - DELETE /v2/instances/{instance_id}/config/devices/usb/{port}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_devices_usb_port_delete<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigDevicesUsbPortDeletePathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_devices_usb_port_delete_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_devices_usb_port_delete(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbPortDeleteResponse::Status200_Success
                                                => {
                                                  let mut response = response.status(200);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbPortDeleteResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbPortDeleteResponse::Status404_ResourceNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_devices_usb_port_get_validation(
    path_params: models::InstancesInstanceIdConfigDevicesUsbPortGetPathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigDevicesUsbPortGetPathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigDevicesUsbPortGet - GET /v2/instances/{instance_id}/config/devices/usb/{port}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_devices_usb_port_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigDevicesUsbPortGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_devices_usb_port_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_devices_usb_port_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbPortGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbPortGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbPortGetResponse::Status404_ResourceNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbPortGetResponse::Status500_InternalServerError
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_devices_usb_port_put_validation(
    path_params: models::InstancesInstanceIdConfigDevicesUsbPortPutPathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigDevicesUsbPortPutPathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigDevicesUsbPortPut - PUT /v2/instances/{instance_id}/config/devices/usb/{port}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_devices_usb_port_put<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigDevicesUsbPortPutPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_devices_usb_port_put_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_devices_usb_port_put(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbPortPutResponse::Status200_AlreadyPassedThrough
                                                => {
                                                  let mut response = response.status(200);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbPortPutResponse::Status201_UsbDeviceWasPassedThrough
                                                => {
                                                  let mut response = response.status(201);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbPortPutResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbPortPutResponse::Status404_ResourceNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigDevicesUsbPortPutResponse::Status500_InternalServerError
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_environment_delete_validation(
    path_params: models::InstancesInstanceIdConfigEnvironmentDeletePathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigEnvironmentDeletePathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigEnvironmentDelete - DELETE /v2/instances/{instance_id}/config/environment
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_environment_delete<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigEnvironmentDeletePathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_environment_delete_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_environment_delete(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigEnvironmentDeleteResponse::Status200_EnvironmentOfInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(200);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigEnvironmentDeleteResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigEnvironmentDeleteResponse::Status404_NoInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_environment_get_validation(
    path_params: models::InstancesInstanceIdConfigEnvironmentGetPathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigEnvironmentGetPathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigEnvironmentGet - GET /v2/instances/{instance_id}/config/environment
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_environment_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigEnvironmentGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_environment_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_environment_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigEnvironmentGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigEnvironmentGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigEnvironmentGetResponse::Status404_NoInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(validator::Validate)]
#[allow(dead_code)]
struct InstancesInstanceIdConfigEnvironmentPutBodyValidator<'a> {
    #[validate(nested)]
    body: &'a models::InstanceEnvironment,
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_environment_put_validation(
    path_params: models::InstancesInstanceIdConfigEnvironmentPutPathParams,
    body: models::InstanceEnvironment,
) -> std::result::Result<
    (
        models::InstancesInstanceIdConfigEnvironmentPutPathParams,
        models::InstanceEnvironment,
    ),
    ValidationErrors,
> {
    path_params.validate()?;
    let b = InstancesInstanceIdConfigEnvironmentPutBodyValidator { body: &body };
    b.validate()?;

    Ok((path_params, body))
}
/// InstancesInstanceIdConfigEnvironmentPut - PUT /v2/instances/{instance_id}/config/environment
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_environment_put<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigEnvironmentPutPathParams>,
    State(api_impl): State<I>,
    Json(body): Json<models::InstanceEnvironment>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_environment_put_validation(path_params, body)
    })
    .await
    .unwrap();

    let Ok((path_params, body)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_environment_put(method, host, cookies, path_params, body)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigEnvironmentPutResponse::Status200_EnvironmentForInstanceWithThisInstanceIdIsSet
                                                => {
                                                  let mut response = response.status(200);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigEnvironmentPutResponse::Status201_EnvironmentForInstanceWithThisInstanceIdWasCreated
                                                => {
                                                  let mut response = response.status(201);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigEnvironmentPutResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigEnvironmentPutResponse::Status404_NoInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_environment_variable_name_delete_validation(
    path_params: models::InstancesInstanceIdConfigEnvironmentVariableNameDeletePathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigEnvironmentVariableNameDeletePathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigEnvironmentVariableNameDelete - DELETE /v2/instances/{instance_id}/config/environment/{variable_name}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_environment_variable_name_delete<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<
        models::InstancesInstanceIdConfigEnvironmentVariableNameDeletePathParams,
    >,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_environment_variable_name_delete_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_environment_variable_name_delete(
            method,
            host,
            cookies,
            path_params,
        )
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigEnvironmentVariableNameDeleteResponse::Status200_EnvironmentVariableOfInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(200);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigEnvironmentVariableNameDeleteResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigEnvironmentVariableNameDeleteResponse::Status404_ResourceNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_environment_variable_name_get_validation(
    path_params: models::InstancesInstanceIdConfigEnvironmentVariableNameGetPathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigEnvironmentVariableNameGetPathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigEnvironmentVariableNameGet - GET /v2/instances/{instance_id}/config/environment/{variable_name}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_environment_variable_name_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigEnvironmentVariableNameGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_environment_variable_name_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_environment_variable_name_get(
            method,
            host,
            cookies,
            path_params,
        )
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigEnvironmentVariableNameGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigEnvironmentVariableNameGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigEnvironmentVariableNameGetResponse::Status404_ResourceNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(validator::Validate)]
#[allow(dead_code)]
struct InstancesInstanceIdConfigEnvironmentVariableNamePutBodyValidator<'a> {
    #[validate(nested)]
    body: &'a models::InstancesInstanceIdConfigEnvironmentVariableNameGet200Response,
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_environment_variable_name_put_validation(
    path_params: models::InstancesInstanceIdConfigEnvironmentVariableNamePutPathParams,
    body: models::InstancesInstanceIdConfigEnvironmentVariableNameGet200Response,
) -> std::result::Result<
    (
        models::InstancesInstanceIdConfigEnvironmentVariableNamePutPathParams,
        models::InstancesInstanceIdConfigEnvironmentVariableNameGet200Response,
    ),
    ValidationErrors,
> {
    path_params.validate()?;
    let b = InstancesInstanceIdConfigEnvironmentVariableNamePutBodyValidator { body: &body };
    b.validate()?;

    Ok((path_params, body))
}
/// InstancesInstanceIdConfigEnvironmentVariableNamePut - PUT /v2/instances/{instance_id}/config/environment/{variable_name}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_environment_variable_name_put<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigEnvironmentVariableNamePutPathParams>,
    State(api_impl): State<I>,
    Json(body): Json<models::InstancesInstanceIdConfigEnvironmentVariableNameGet200Response>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_environment_variable_name_put_validation(path_params, body)
    })
    .await
    .unwrap();

    let Ok((path_params, body)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_environment_variable_name_put(
            method,
            host,
            cookies,
            path_params,
            body,
        )
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigEnvironmentVariableNamePutResponse::Status200_EnvironmentForInstanceWithThisInstanceIdIsSet
                                                => {
                                                  let mut response = response.status(200);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigEnvironmentVariableNamePutResponse::Status201_EnvironmentForInstanceWithThisInstanceIdWasCreated
                                                => {
                                                  let mut response = response.status(201);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigEnvironmentVariableNamePutResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigEnvironmentVariableNamePutResponse::Status404_NoInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_labels_get_validation(
    path_params: models::InstancesInstanceIdConfigLabelsGetPathParams,
) -> std::result::Result<(models::InstancesInstanceIdConfigLabelsGetPathParams,), ValidationErrors>
{
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigLabelsGet - GET /v2/instances/{instance_id}/config/labels
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_labels_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigLabelsGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_labels_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_labels_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigLabelsGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigLabelsGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigLabelsGetResponse::Status404_NoInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_labels_label_name_get_validation(
    path_params: models::InstancesInstanceIdConfigLabelsLabelNameGetPathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigLabelsLabelNameGetPathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigLabelsLabelNameGet - GET /v2/instances/{instance_id}/config/labels/{label_name}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_labels_label_name_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigLabelsLabelNameGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_labels_label_name_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_labels_label_name_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigLabelsLabelNameGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigLabelsLabelNameGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigLabelsLabelNameGetResponse::Status404_ResourceNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_mounts_bind_container_path_get_validation(
    path_params: models::InstancesInstanceIdConfigMountsBindContainerPathGetPathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigMountsBindContainerPathGetPathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigMountsBindContainerPathGet - GET /v2/instances/{instance_id}/config/mounts/bind/{container_path}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_mounts_bind_container_path_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigMountsBindContainerPathGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_mounts_bind_container_path_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_mounts_bind_container_path_get(
            method,
            host,
            cookies,
            path_params,
        )
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigMountsBindContainerPathGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigMountsBindContainerPathGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigMountsBindContainerPathGetResponse::Status404_ResourceNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_mounts_bind_get_validation(
    path_params: models::InstancesInstanceIdConfigMountsBindGetPathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigMountsBindGetPathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigMountsBindGet - GET /v2/instances/{instance_id}/config/mounts/bind
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_mounts_bind_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigMountsBindGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_mounts_bind_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_mounts_bind_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigMountsBindGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigMountsBindGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigMountsBindGetResponse::Status404_InstanceNotFound
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_mounts_get_validation(
    path_params: models::InstancesInstanceIdConfigMountsGetPathParams,
) -> std::result::Result<(models::InstancesInstanceIdConfigMountsGetPathParams,), ValidationErrors>
{
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigMountsGet - GET /v2/instances/{instance_id}/config/mounts
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_mounts_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigMountsGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_mounts_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_mounts_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigMountsGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigMountsGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigMountsGetResponse::Status404_InstanceNotFound
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_mounts_volumes_get_validation(
    path_params: models::InstancesInstanceIdConfigMountsVolumesGetPathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigMountsVolumesGetPathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigMountsVolumesGet - GET /v2/instances/{instance_id}/config/mounts/volumes
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_mounts_volumes_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigMountsVolumesGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_mounts_volumes_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_mounts_volumes_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigMountsVolumesGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigMountsVolumesGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigMountsVolumesGetResponse::Status404_InstanceNotFound
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_mounts_volumes_volume_name_get_validation(
    path_params: models::InstancesInstanceIdConfigMountsVolumesVolumeNameGetPathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigMountsVolumesVolumeNameGetPathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigMountsVolumesVolumeNameGet - GET /v2/instances/{instance_id}/config/mounts/volumes/{volume_name}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_mounts_volumes_volume_name_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigMountsVolumesVolumeNameGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_mounts_volumes_volume_name_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_mounts_volumes_volume_name_get(
            method,
            host,
            cookies,
            path_params,
        )
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigMountsVolumesVolumeNameGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigMountsVolumesVolumeNameGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigMountsVolumesVolumeNameGetResponse::Status404_ResourceNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_networks_get_validation(
    path_params: models::InstancesInstanceIdConfigNetworksGetPathParams,
) -> std::result::Result<(models::InstancesInstanceIdConfigNetworksGetPathParams,), ValidationErrors>
{
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigNetworksGet - GET /v2/instances/{instance_id}/config/networks
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_networks_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigNetworksGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_networks_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_networks_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigNetworksGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigNetworksGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigNetworksGetResponse::Status404_InstanceIdNotFound
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_networks_network_id_delete_validation(
    path_params: models::InstancesInstanceIdConfigNetworksNetworkIdDeletePathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigNetworksNetworkIdDeletePathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigNetworksNetworkIdDelete - DELETE /v2/instances/{instance_id}/config/networks/{network_id}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_networks_network_id_delete<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigNetworksNetworkIdDeletePathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_networks_network_id_delete_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_networks_network_id_delete(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigNetworksNetworkIdDeleteResponse::Status200_Success
                                                => {
                                                  let mut response = response.status(200);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigNetworksNetworkIdDeleteResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigNetworksNetworkIdDeleteResponse::Status404_InstanceIdOrNetworkNotFound
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigNetworksNetworkIdDeleteResponse::Status500_InternalServerError
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_networks_network_id_get_validation(
    path_params: models::InstancesInstanceIdConfigNetworksNetworkIdGetPathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigNetworksNetworkIdGetPathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigNetworksNetworkIdGet - GET /v2/instances/{instance_id}/config/networks/{network_id}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_networks_network_id_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigNetworksNetworkIdGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_networks_network_id_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_networks_network_id_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigNetworksNetworkIdGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigNetworksNetworkIdGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigNetworksNetworkIdGetResponse::Status404_InstanceIdOrNetworkNotFound
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(validator::Validate)]
#[allow(dead_code)]
struct InstancesInstanceIdConfigNetworksPostBodyValidator<'a> {
    #[validate(nested)]
    body: &'a models::InstancesInstanceIdConfigNetworksPostRequest,
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_networks_post_validation(
    path_params: models::InstancesInstanceIdConfigNetworksPostPathParams,
    body: models::InstancesInstanceIdConfigNetworksPostRequest,
) -> std::result::Result<
    (
        models::InstancesInstanceIdConfigNetworksPostPathParams,
        models::InstancesInstanceIdConfigNetworksPostRequest,
    ),
    ValidationErrors,
> {
    path_params.validate()?;
    let b = InstancesInstanceIdConfigNetworksPostBodyValidator { body: &body };
    b.validate()?;

    Ok((path_params, body))
}
/// InstancesInstanceIdConfigNetworksPost - POST /v2/instances/{instance_id}/config/networks
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_networks_post<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigNetworksPostPathParams>,
    State(api_impl): State<I>,
    Json(body): Json<models::InstancesInstanceIdConfigNetworksPostRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_networks_post_validation(path_params, body)
    })
    .await
    .unwrap();

    let Ok((path_params, body)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_networks_post(method, host, cookies, path_params, body)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigNetworksPostResponse::Status201_InstanceConnected
                                                    {
                                                        location
                                                    }
                                                => {
                                                    let location = match header::IntoHeaderValue(location).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling location header - {}", e))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("location"),
                                                          location
                                                      );
                                                    }
                                                  let mut response = response.status(201);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigNetworksPostResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigNetworksPostResponse::Status404_InstanceIdOrNetworkNotFound
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigNetworksPostResponse::Status409_InstanceAlreadyConnectedToNetwork
                                                => {
                                                  let mut response = response.status(409);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigNetworksPostResponse::Status500_InternalServerError
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_ports_delete_validation(
    path_params: models::InstancesInstanceIdConfigPortsDeletePathParams,
) -> std::result::Result<(models::InstancesInstanceIdConfigPortsDeletePathParams,), ValidationErrors>
{
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigPortsDelete - DELETE /v2/instances/{instance_id}/config/ports
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_ports_delete<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigPortsDeletePathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_ports_delete_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_ports_delete(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigPortsDeleteResponse::Status200_ExposedPortsOfInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(200);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsDeleteResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsDeleteResponse::Status404_NoInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_ports_get_validation(
    path_params: models::InstancesInstanceIdConfigPortsGetPathParams,
) -> std::result::Result<(models::InstancesInstanceIdConfigPortsGetPathParams,), ValidationErrors> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigPortsGet - GET /v2/instances/{instance_id}/config/ports
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_ports_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigPortsGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_ports_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_ports_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigPortsGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsGetResponse::Status404_NoInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_ports_transport_protocol_delete_validation(
    path_params: models::InstancesInstanceIdConfigPortsTransportProtocolDeletePathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigPortsTransportProtocolDeletePathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigPortsTransportProtocolDelete - DELETE /v2/instances/{instance_id}/config/ports/{transport_protocol}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_ports_transport_protocol_delete<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<
        models::InstancesInstanceIdConfigPortsTransportProtocolDeletePathParams,
    >,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_ports_transport_protocol_delete_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_ports_transport_protocol_delete(
            method,
            host,
            cookies,
            path_params,
        )
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolDeleteResponse::Status200_RemovedAllPublishedPortsOfInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(200);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolDeleteResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolDeleteResponse::Status404_ResourceNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_ports_transport_protocol_get_validation(
    path_params: models::InstancesInstanceIdConfigPortsTransportProtocolGetPathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigPortsTransportProtocolGetPathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigPortsTransportProtocolGet - GET /v2/instances/{instance_id}/config/ports/{transport_protocol}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_ports_transport_protocol_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigPortsTransportProtocolGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_ports_transport_protocol_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_ports_transport_protocol_get(
            method,
            host,
            cookies,
            path_params,
        )
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolGetResponse::Status200_PublishedPortsForInstanceWithThisInstance
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolGetResponse::Status404_ResourceNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_ports_transport_protocol_host_port_range_delete_validation(
    path_params: models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeletePathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeletePathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDelete - DELETE /v2/instances/{instance_id}/config/ports/{transport_protocol}/{host_port_range}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_ports_transport_protocol_host_port_range_delete<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<
        models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeletePathParams,
    >,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_ports_transport_protocol_host_port_range_delete_validation(
            path_params,
        )
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_ports_transport_protocol_host_port_range_delete(
            method,
            host,
            cookies,
            path_params,
        )
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeleteResponse::Status200_Success
                                                => {
                                                  let mut response = response.status(200);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeleteResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeDeleteResponse::Status404_ResourceNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_ports_transport_protocol_host_port_range_get_validation(
    path_params: models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetPathParams,
) -> std::result::Result<
    (models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetPathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGet - GET /v2/instances/{instance_id}/config/ports/{transport_protocol}/{host_port_range}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_ports_transport_protocol_host_port_range_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<
        models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetPathParams,
    >,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_ports_transport_protocol_host_port_range_get_validation(
            path_params,
        )
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_ports_transport_protocol_host_port_range_get(
            method,
            host,
            cookies,
            path_params,
        )
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangeGetResponse::Status404_ResourceNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(validator::Validate)]
#[allow(dead_code)]
struct InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutBodyValidator<'a> {
    #[validate(nested)]
    body: &'a models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutRequest,
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_ports_transport_protocol_host_port_range_put_validation(
    path_params: models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutPathParams,
    body: models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutRequest,
) -> std::result::Result<
    (
        models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutPathParams,
        models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutRequest,
    ),
    ValidationErrors,
> {
    path_params.validate()?;
    let b = InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutBodyValidator {
        body: &body,
    };
    b.validate()?;

    Ok((path_params, body))
}
/// InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePut - PUT /v2/instances/{instance_id}/config/ports/{transport_protocol}/{host_port_range}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_ports_transport_protocol_host_port_range_put<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<
        models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutPathParams,
    >,
    State(api_impl): State<I>,
    Json(body): Json<
        models::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutRequest,
    >,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_ports_transport_protocol_host_port_range_put_validation(
            path_params,
            body,
        )
    })
    .await
    .unwrap();

    let Ok((path_params, body)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_ports_transport_protocol_host_port_range_put(
            method,
            host,
            cookies,
            path_params,
            body,
        )
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutResponse::Status200_TheSpecifiedPortMappingWasSet
                                                => {
                                                  let mut response = response.status(200);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutResponse::Status201_TheSpecifiedPortMappingWasCreated
                                                => {
                                                  let mut response = response.status(201);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolHostPortRangePutResponse::Status404_ResourceNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(validator::Validate)]
#[allow(dead_code)]
struct InstancesInstanceIdConfigPortsTransportProtocolPutBodyValidator<'a> {
    #[validate(nested)]
    body: &'a Vec<models::InstancePortMapping>,
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_ports_transport_protocol_put_validation(
    path_params: models::InstancesInstanceIdConfigPortsTransportProtocolPutPathParams,
    body: Vec<models::InstancePortMapping>,
) -> std::result::Result<
    (
        models::InstancesInstanceIdConfigPortsTransportProtocolPutPathParams,
        Vec<models::InstancePortMapping>,
    ),
    ValidationErrors,
> {
    path_params.validate()?;
    let b = InstancesInstanceIdConfigPortsTransportProtocolPutBodyValidator { body: &body };
    b.validate()?;

    Ok((path_params, body))
}
/// InstancesInstanceIdConfigPortsTransportProtocolPut - PUT /v2/instances/{instance_id}/config/ports/{transport_protocol}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_ports_transport_protocol_put<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigPortsTransportProtocolPutPathParams>,
    State(api_impl): State<I>,
    Json(body): Json<Vec<models::InstancePortMapping>>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_ports_transport_protocol_put_validation(path_params, body)
    })
    .await
    .unwrap();

    let Ok((path_params, body)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_config_ports_transport_protocol_put(
            method,
            host,
            cookies,
            path_params,
            body,
        )
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolPutResponse::Status200_PublishedPortsOfInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(200);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolPutResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsTransportProtocolPutResponse::Status404_ResourceNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_delete_validation(
    path_params: models::InstancesInstanceIdDeletePathParams,
) -> std::result::Result<(models::InstancesInstanceIdDeletePathParams,), ValidationErrors> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdDelete - DELETE /v2/instances/{instance_id}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_delete<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdDeletePathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation =
        tokio::task::spawn_blocking(move || instances_instance_id_delete_validation(path_params))
            .await
            .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_delete(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdDeleteResponse::Status202_Accepted
                                                    (body)
                                                => {
                                                  let mut response = response.status(202);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdDeleteResponse::Status404_NoInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdDeleteResponse::Status500_InternalServerError
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_editor_port_get_validation(
    path_params: models::InstancesInstanceIdEditorPortGetPathParams,
) -> std::result::Result<(models::InstancesInstanceIdEditorPortGetPathParams,), ValidationErrors> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdEditorPortGet - GET /v2/instances/{instance_id}/editor/{port}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_editor_port_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdEditorPortGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_editor_port_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_editor_port_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdEditorPortGetResponse::Status302_Found
                                                    {
                                                        location
                                                    }
                                                => {
                                                    let location = match header::IntoHeaderValue(location).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling location header - {}", e))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("location"),
                                                          location
                                                      );
                                                    }
                                                  let mut response = response.status(302);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdEditorPortGetResponse::Status400_MalformedRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdEditorPortGetResponse::Status404_InstanceIdOrPortNotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdEditorPortGetResponse::Status500_InternalServerError
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_get_validation(
    path_params: models::InstancesInstanceIdGetPathParams,
) -> std::result::Result<(models::InstancesInstanceIdGetPathParams,), ValidationErrors> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdGet - GET /v2/instances/{instance_id}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation =
        tokio::task::spawn_blocking(move || instances_instance_id_get_validation(path_params))
            .await
            .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdGetResponse::Status404_NoInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdGetResponse::Status500_InternalServerError
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_logs_get_validation(
    path_params: models::InstancesInstanceIdLogsGetPathParams,
) -> std::result::Result<(models::InstancesInstanceIdLogsGetPathParams,), ValidationErrors> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdLogsGet - GET /v2/instances/{instance_id}/logs
#[tracing::instrument(skip_all)]
async fn instances_instance_id_logs_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdLogsGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation =
        tokio::task::spawn_blocking(move || instances_instance_id_logs_get_validation(path_params))
            .await
            .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_logs_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdLogsGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdLogsGetResponse::Status404_NoInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdLogsGetResponse::Status500_InternalServerError
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(validator::Validate)]
#[allow(dead_code)]
struct InstancesInstanceIdPatchBodyValidator<'a> {
    #[validate(nested)]
    body: &'a models::InstancesInstanceIdPatchRequest,
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_patch_validation(
    path_params: models::InstancesInstanceIdPatchPathParams,
    body: models::InstancesInstanceIdPatchRequest,
) -> std::result::Result<
    (
        models::InstancesInstanceIdPatchPathParams,
        models::InstancesInstanceIdPatchRequest,
    ),
    ValidationErrors,
> {
    path_params.validate()?;
    let b = InstancesInstanceIdPatchBodyValidator { body: &body };
    b.validate()?;

    Ok((path_params, body))
}
/// InstancesInstanceIdPatch - PATCH /v2/instances/{instance_id}
#[tracing::instrument(skip_all)]
async fn instances_instance_id_patch<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdPatchPathParams>,
    State(api_impl): State<I>,
    Json(body): Json<models::InstancesInstanceIdPatchRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_patch_validation(path_params, body)
    })
    .await
    .unwrap();

    let Ok((path_params, body)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_patch(method, host, cookies, path_params, body)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdPatchResponse::Status202_Accepted
                                                    (body)
                                                => {
                                                  let mut response = response.status(202);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdPatchResponse::Status404_NoInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdPatchResponse::Status500_InternalServerError
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_start_post_validation(
    path_params: models::InstancesInstanceIdStartPostPathParams,
) -> std::result::Result<(models::InstancesInstanceIdStartPostPathParams,), ValidationErrors> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdStartPost - POST /v2/instances/{instance_id}/start
#[tracing::instrument(skip_all)]
async fn instances_instance_id_start_post<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdStartPostPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_start_post_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_start_post(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdStartPostResponse::Status202_Accepted
                                                    (body)
                                                => {
                                                  let mut response = response.status(202);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdStartPostResponse::Status404_NoInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_stop_post_validation(
    path_params: models::InstancesInstanceIdStopPostPathParams,
) -> std::result::Result<(models::InstancesInstanceIdStopPostPathParams,), ValidationErrors> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdStopPost - POST /v2/instances/{instance_id}/stop
#[tracing::instrument(skip_all)]
async fn instances_instance_id_stop_post<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdStopPostPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_stop_post_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .instances_instance_id_stop_post(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdStopPostResponse::Status202_Accepted
                                                    (body)
                                                => {
                                                  let mut response = response.status(202);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::instances::InstancesInstanceIdStopPostResponse::Status404_NoInstanceWithThisInstance
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn jobs_get_validation() -> std::result::Result<(), ValidationErrors> {
    Ok(())
}
/// JobsGet - GET /v2/jobs
#[tracing::instrument(skip_all)]
async fn jobs_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::jobs::Jobs,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || jobs_get_validation())
        .await
        .unwrap();

    let Ok(()) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl.as_ref().jobs_get(method, host, cookies).await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::jobs::JobsGetResponse::Status200_Success(body) => {
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn jobs_job_id_delete_validation(
    path_params: models::JobsJobIdDeletePathParams,
) -> std::result::Result<(models::JobsJobIdDeletePathParams,), ValidationErrors> {
    path_params.validate()?;

    Ok((path_params,))
}
/// JobsJobIdDelete - DELETE /v2/jobs/{job_id}
#[tracing::instrument(skip_all)]
async fn jobs_job_id_delete<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::JobsJobIdDeletePathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::jobs::Jobs,
{
    #[allow(clippy::redundant_closure)]
    let validation =
        tokio::task::spawn_blocking(move || jobs_job_id_delete_validation(path_params))
            .await
            .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .jobs_job_id_delete(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::jobs::JobsJobIdDeleteResponse::Status200_Success => {
                let mut response = response.status(200);
                response.body(Body::empty())
            }
            apis::jobs::JobsJobIdDeleteResponse::Status404_NotFound => {
                let mut response = response.status(404);
                response.body(Body::empty())
            }
            apis::jobs::JobsJobIdDeleteResponse::Status400_JobNotFinished(body) => {
                let mut response = response.status(400);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("text/plain").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = body;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn jobs_job_id_get_validation(
    path_params: models::JobsJobIdGetPathParams,
) -> std::result::Result<(models::JobsJobIdGetPathParams,), ValidationErrors> {
    path_params.validate()?;

    Ok((path_params,))
}
/// JobsJobIdGet - GET /v2/jobs/{job_id}
#[tracing::instrument(skip_all)]
async fn jobs_job_id_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::JobsJobIdGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::jobs::Jobs,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || jobs_job_id_get_validation(path_params))
        .await
        .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .jobs_job_id_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::jobs::JobsJobIdGetResponse::Status200_Success(body) => {
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::jobs::JobsJobIdGetResponse::Status404_NotFound => {
                let mut response = response.status(404);
                response.body(Body::empty())
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn system_devices_get_validation() -> std::result::Result<(), ValidationErrors> {
    Ok(())
}
/// SystemDevicesGet - GET /v2/system/devices
#[tracing::instrument(skip_all)]
async fn system_devices_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::system::System,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || system_devices_get_validation())
        .await
        .unwrap();

    let Ok(()) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .system_devices_get(method, host, cookies)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::system::SystemDevicesGetResponse::Status200_Success(body) => {
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::system::SystemDevicesGetResponse::Status500_InternalServerError(body) => {
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn system_devices_usb_get_validation() -> std::result::Result<(), ValidationErrors> {
    Ok(())
}
/// SystemDevicesUsbGet - GET /v2/system/devices/usb
#[tracing::instrument(skip_all)]
async fn system_devices_usb_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::system::System,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || system_devices_usb_get_validation())
        .await
        .unwrap();

    let Ok(()) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .system_devices_usb_get(method, host, cookies)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::system::SystemDevicesUsbGetResponse::Status200_Success(body) => {
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::system::SystemDevicesUsbGetResponse::Status500_InternalServerError(body) => {
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn system_devices_usb_port_get_validation(
    path_params: models::SystemDevicesUsbPortGetPathParams,
) -> std::result::Result<(models::SystemDevicesUsbPortGetPathParams,), ValidationErrors> {
    path_params.validate()?;

    Ok((path_params,))
}
/// SystemDevicesUsbPortGet - GET /v2/system/devices/usb/{port}
#[tracing::instrument(skip_all)]
async fn system_devices_usb_port_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::SystemDevicesUsbPortGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::system::System,
{
    #[allow(clippy::redundant_closure)]
    let validation =
        tokio::task::spawn_blocking(move || system_devices_usb_port_get_validation(path_params))
            .await
            .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .system_devices_usb_port_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::system::SystemDevicesUsbPortGetResponse::Status200_Success(body) => {
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::system::SystemDevicesUsbPortGetResponse::Status404_DeviceNotFound => {
                let mut response = response.status(404);
                response.body(Body::empty())
            }
            apis::system::SystemDevicesUsbPortGetResponse::Status500_InternalServerError(body) => {
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn system_info_get_validation() -> std::result::Result<(), ValidationErrors> {
    Ok(())
}
/// SystemInfoGet - GET /v2/system/info
#[tracing::instrument(skip_all)]
async fn system_info_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::system::System,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || system_info_get_validation())
        .await
        .unwrap();

    let Ok(()) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .system_info_get(method, host, cookies)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::system::SystemInfoGetResponse::Status200_Sucess(body) => {
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn system_network_adapters_get_validation() -> std::result::Result<(), ValidationErrors> {
    Ok(())
}
/// SystemNetworkAdaptersGet - GET /v2/system/network_adapters
#[tracing::instrument(skip_all)]
async fn system_network_adapters_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::system::System,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || system_network_adapters_get_validation())
        .await
        .unwrap();

    let Ok(()) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .system_network_adapters_get(method, host, cookies)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::system::SystemNetworkAdaptersGetResponse::Status200_Success(body) => {
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::system::SystemNetworkAdaptersGetResponse::Status500_InternalServerError(body) => {
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn system_network_adapters_network_adapter_id_get_validation(
    path_params: models::SystemNetworkAdaptersNetworkAdapterIdGetPathParams,
) -> std::result::Result<
    (models::SystemNetworkAdaptersNetworkAdapterIdGetPathParams,),
    ValidationErrors,
> {
    path_params.validate()?;

    Ok((path_params,))
}
/// SystemNetworkAdaptersNetworkAdapterIdGet - GET /v2/system/network_adapters/{network_adapter_id}
#[tracing::instrument(skip_all)]
async fn system_network_adapters_network_adapter_id_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::SystemNetworkAdaptersNetworkAdapterIdGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::system::System,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        system_network_adapters_network_adapter_id_get_validation(path_params)
    })
    .await
    .unwrap();

    let Ok((path_params,)) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .system_network_adapters_network_adapter_id_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::system::SystemNetworkAdaptersNetworkAdapterIdGetResponse::Status200_Success
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::system::SystemNetworkAdaptersNetworkAdapterIdGetResponse::Status404_NetworkAdapterNotFound
                                                => {
                                                  let mut response = response.status(404);
                                                  response.body(Body::empty())
                                                },
                                                apis::system::SystemNetworkAdaptersNetworkAdapterIdGetResponse::Status500_InternalServerError
                                                    (body)
                                                => {
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.status(500).body(Body::empty())
                                            },
                                        };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn system_ping_get_validation() -> std::result::Result<(), ValidationErrors> {
    Ok(())
}
/// SystemPingGet - GET /v2/system/ping
#[tracing::instrument(skip_all)]
async fn system_ping_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::system::System,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || system_ping_get_validation())
        .await
        .unwrap();

    let Ok(()) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .system_ping_get(method, host, cookies)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::system::SystemPingGetResponse::Status200_Success(body) => {
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn system_version_get_validation() -> std::result::Result<(), ValidationErrors> {
    Ok(())
}
/// SystemVersionGet - GET /v2/system/version
#[tracing::instrument(skip_all)]
async fn system_version_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::system::System,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || system_version_get_validation())
        .await
        .unwrap();

    let Ok(()) = validation else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
    };

    let result = api_impl
        .as_ref()
        .system_version_get(method, host, cookies)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::system::SystemVersionGetResponse::Status200_Success(body) => {
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").map_err(|e| {
                            error!(error = ?e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?,
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(_) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            response.status(500).body(Body::empty())
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}
