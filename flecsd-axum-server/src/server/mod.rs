use std::collections::HashMap;

use axum::{body::Body, extract::*, response::Response, routing::*};
use axum_extra::extract::{CookieJar, Multipart};
use bytes::Bytes;
use http::{header::CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use tracing::error;
use validator::{Validate, ValidationErrors};

use crate::{header, types::*};

#[allow(unused_imports)]
use crate::{apis, models};

/// Setup API Server.
pub fn new<I, A>(api_impl: I) -> Router
where
    I: AsRef<A> + Clone + Send + Sync + 'static,
    A: apis::apps::Apps
        + apis::console::Console
        + apis::device::Device
        + apis::flunder::Flunder
        + apis::instances::Instances
        + apis::jobs::Jobs
        + apis::system::System
        + 'static,
{
    // build our application with a route
    Router::new()
        .route("/v2/apps", get(apps_get::<I, A>))
        .route(
            "/v2/apps/:app",
            delete(apps_app_delete::<I, A>).get(apps_app_get::<I, A>),
        )
        .route("/v2/apps/install", post(apps_install_post::<I, A>))
        .route("/v2/apps/sideload", post(apps_sideload_post::<I, A>))
        .route(
            "/v2/console/authentication",
            delete(console_authentication_delete::<I, A>).put(console_authentication_put::<I, A>),
        )
        .route(
            "/v2/device/license/activation",
            post(device_license_activation_post::<I, A>),
        )
        .route(
            "/v2/device/license/activation/status",
            get(device_license_activation_status_get::<I, A>),
        )
        .route(
            "/v2/device/license/info",
            get(device_license_info_get::<I, A>),
        )
        .route(
            "/v2/device/onboarding",
            post(device_onboarding_post::<I, A>),
        )
        .route("/v2/flunder/browse", get(flunder_browse_get::<I, A>))
        .route("/v2/instances", get(instances_get::<I, A>))
        .route(
            "/v2/instances/:instance_id",
            delete(instances_instance_id_delete::<I, A>)
                .get(instances_instance_id_get::<I, A>)
                .patch(instances_instance_id_patch::<I, A>),
        )
        .route(
            "/v2/instances/:instance_id/config",
            get(instances_instance_id_config_get::<I, A>)
                .post(instances_instance_id_config_post::<I, A>),
        )
        .route(
            "/v2/instances/:instance_id/config/environment",
            delete(instances_instance_id_config_environment_delete::<I, A>)
                .get(instances_instance_id_config_environment_get::<I, A>)
                .put(instances_instance_id_config_environment_put::<I, A>),
        )
        .route(
            "/v2/instances/:instance_id/config/ports",
            delete(instances_instance_id_config_ports_delete::<I, A>)
                .get(instances_instance_id_config_ports_get::<I, A>)
                .put(instances_instance_id_config_ports_put::<I, A>),
        )
        .route(
            "/v2/instances/:instance_id/logs",
            get(instances_instance_id_logs_get::<I, A>),
        )
        .route(
            "/v2/instances/:instance_id/start",
            post(instances_instance_id_start_post::<I, A>),
        )
        .route(
            "/v2/instances/:instance_id/stop",
            post(instances_instance_id_stop_post::<I, A>),
        )
        .route("/v2/instances/create", post(instances_create_post::<I, A>))
        .route("/v2/jobs", get(jobs_get::<I, A>))
        .route(
            "/v2/jobs/:job_id",
            delete(jobs_job_id_delete::<I, A>).get(jobs_job_id_get::<I, A>),
        )
        .route("/v2/system/info", get(system_info_get::<I, A>))
        .route("/v2/system/ping", get(system_ping_get::<I, A>))
        .route("/v2/system/version", get(system_version_get::<I, A>))
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
    body: &'a models::DosManifest,
}

#[tracing::instrument(skip_all)]
fn device_onboarding_post_validation(
    body: models::DosManifest,
) -> std::result::Result<(models::DosManifest,), ValidationErrors> {
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
    Json(body): Json<models::DosManifest>,
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
fn flunder_browse_get_validation(
    query_params: models::FlunderBrowseGetQueryParams,
) -> std::result::Result<(models::FlunderBrowseGetQueryParams,), ValidationErrors> {
    query_params.validate()?;

    Ok((query_params,))
}
/// FlunderBrowseGet - GET /v2/flunder/browse
#[tracing::instrument(skip_all)]
async fn flunder_browse_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Query(query_params): Query<models::FlunderBrowseGetQueryParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::flunder::Flunder,
{
    #[allow(clippy::redundant_closure)]
    let validation =
        tokio::task::spawn_blocking(move || flunder_browse_get_validation(query_params))
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
        .flunder_browse_get(method, host, cookies, query_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::flunder::FlunderBrowseGetResponse::Status200_Success(body) => {
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
            apis::flunder::FlunderBrowseGetResponse::Status500_InternalServerError(body) => {
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
fn instances_instance_id_config_get_validation(
    path_params: models::InstancesInstanceIdConfigGetPathParams,
) -> std::result::Result<(models::InstancesInstanceIdConfigGetPathParams,), ValidationErrors> {
    path_params.validate()?;

    Ok((path_params,))
}
/// InstancesInstanceIdConfigGet - GET /v2/instances/{instance_id}/config
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_get<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigGetPathParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_get_validation(path_params)
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
        .instances_instance_id_config_get(method, host, cookies, path_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigGetResponse::Status200_Success
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
                                                apis::instances::InstancesInstanceIdConfigGetResponse::Status404_NoInstanceWithThisInstance
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

#[derive(validator::Validate)]
#[allow(dead_code)]
struct InstancesInstanceIdConfigPortsPutBodyValidator<'a> {
    #[validate(nested)]
    body: &'a models::InstancePorts,
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_ports_put_validation(
    path_params: models::InstancesInstanceIdConfigPortsPutPathParams,
    body: models::InstancePorts,
) -> std::result::Result<
    (
        models::InstancesInstanceIdConfigPortsPutPathParams,
        models::InstancePorts,
    ),
    ValidationErrors,
> {
    path_params.validate()?;
    let b = InstancesInstanceIdConfigPortsPutBodyValidator { body: &body };
    b.validate()?;

    Ok((path_params, body))
}
/// InstancesInstanceIdConfigPortsPut - PUT /v2/instances/{instance_id}/config/ports
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_ports_put<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigPortsPutPathParams>,
    State(api_impl): State<I>,
    Json(body): Json<models::InstancePorts>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_ports_put_validation(path_params, body)
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
        .instances_instance_id_config_ports_put(method, host, cookies, path_params, body)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigPortsPutResponse::Status200_ExposedPortsForInstanceWithThisInstanceIdIsSet
                                                => {
                                                  let mut response = response.status(200);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsPutResponse::Status201_ExposedPortsForInstanceWithThisInstanceIdWasCreated
                                                => {
                                                  let mut response = response.status(201);
                                                  response.body(Body::empty())
                                                },
                                                apis::instances::InstancesInstanceIdConfigPortsPutResponse::Status400_MalformedRequest
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
                                                apis::instances::InstancesInstanceIdConfigPortsPutResponse::Status404_NoInstanceWithThisInstance
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
struct InstancesInstanceIdConfigPostBodyValidator<'a> {
    #[validate(nested)]
    body: &'a models::InstanceConfig,
}

#[tracing::instrument(skip_all)]
fn instances_instance_id_config_post_validation(
    path_params: models::InstancesInstanceIdConfigPostPathParams,
    body: models::InstanceConfig,
) -> std::result::Result<
    (
        models::InstancesInstanceIdConfigPostPathParams,
        models::InstanceConfig,
    ),
    ValidationErrors,
> {
    path_params.validate()?;
    let b = InstancesInstanceIdConfigPostBodyValidator { body: &body };
    b.validate()?;

    Ok((path_params, body))
}
/// InstancesInstanceIdConfigPost - POST /v2/instances/{instance_id}/config
#[tracing::instrument(skip_all)]
async fn instances_instance_id_config_post<I, A>(
    method: Method,
    host: Host,
    cookies: CookieJar,
    Path(path_params): Path<models::InstancesInstanceIdConfigPostPathParams>,
    State(api_impl): State<I>,
    Json(body): Json<models::InstanceConfig>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instances::Instances,
{
    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || {
        instances_instance_id_config_post_validation(path_params, body)
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
        .instances_instance_id_config_post(method, host, cookies, path_params, body)
        .await;

    let mut response = Response::builder();

    let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instances::InstancesInstanceIdConfigPostResponse::Status200_Success
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
                                                apis::instances::InstancesInstanceIdConfigPostResponse::Status404_NoInstanceWithThisInstance
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
