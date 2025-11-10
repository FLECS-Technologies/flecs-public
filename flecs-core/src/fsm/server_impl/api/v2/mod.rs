use serde::Serialize;
use utoipa::openapi::path::Operation;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

pub mod apps;
pub mod console;
pub mod deployments;
pub mod device;
pub mod exports;
pub mod imports;
pub mod instances;
pub mod jobs;
pub mod manifests;
pub mod models;
pub mod providers;
pub mod quests;
pub mod system;

#[derive(Debug, Serialize)]
struct Security;

impl Modify for Security {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(schema) = openapi.components.as_mut() {
            schema.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

struct RenameOps;

impl Modify for RenameOps {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let paths = &mut openapi.paths;
        for (url, item) in paths.paths.iter_mut() {
            // Helper to rename a single Operation
            let rename = |op: &mut Option<Operation>, method: &str| {
                if let Some(operation) = op.as_mut() {
                    operation.operation_id = Some(format!(
                        "{}{}",
                        method.to_lowercase(),
                        url.replace('/', "_")
                    ));
                }
            };

            rename(&mut item.get, "GET");
            rename(&mut item.post, "POST");
            rename(&mut item.put, "PUT");
            rename(&mut item.delete, "DELETE");
            rename(&mut item.patch, "PATCH");
            rename(&mut item.head, "HEAD");
            rename(&mut item.options, "OPTIONS");
            rename(&mut item.trace, "TRACE");
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&Security, &RenameOps),
    components(schemas(
        crate::jeweler::gem::instance::InstanceId,
        crate::jeweler::gem::manifest::DependencyKey,
        crate::jeweler::gem::manifest::FeatureKey,
    )),
    // Top-level security requirement (applies to every operation by default)
    security(
        ("bearerAuth" = [])
    ),
    servers(
        (url = "http://{address}/{version}", variables(
            ("address" = (default = "localhost", description = "IP Address of FLECS Daemon")),
            ("version" = (default = "v2", description = "API Version")),
        ))
    ),
)]
#[cfg_attr(
    feature = "auth",
    openapi(paths(
        instances::instance_id::depends::get,
        instances::instance_id::depends::dependency_key::delete,
        instances::instance_id::depends::dependency_key::get,
        instances::instance_id::depends::dependency_key::put,
        instances::instance_id::depends::dependency_key::feature::put,
        instances::instance_id::provides::get,
        instances::instance_id::provides::feature::get,
        providers::get,
        providers::feature::get,
        providers::feature::default::delete,
        providers::feature::default::get,
        providers::feature::default::put,
        providers::feature::id::get,
        providers::auth::get,
        providers::auth::core::get,
        providers::auth::core::put,
        providers::auth::core::path::any,
        providers::auth::default::delete,
        providers::auth::default::get,
        providers::auth::default::put,
        providers::auth::default::path::any,
        providers::auth::first_time_setup::flecsport::post,
        providers::auth::id::get,
        providers::auth::id::path::any,
        system::sbom::get,
    ))
)]
#[cfg_attr(
    not(feature = "auth"),
    openapi(paths(
        instances::instance_id::depends::get,
        instances::instance_id::depends::dependency_key::delete,
        instances::instance_id::depends::dependency_key::get,
        instances::instance_id::depends::dependency_key::put,
        instances::instance_id::depends::dependency_key::feature::put,
        instances::instance_id::provides::get,
        instances::instance_id::provides::feature::get,
        providers::get,
        providers::feature::get,
        providers::feature::default::delete,
        providers::feature::default::get,
        providers::feature::default::put,
        providers::feature::id::get,
        system::sbom::get,
    ))
)]
pub struct ApiDoc;
