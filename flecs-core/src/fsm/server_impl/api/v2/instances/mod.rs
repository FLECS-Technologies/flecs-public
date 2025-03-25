pub mod create;
pub mod instance_id;

use crate::quest::Quest;
use crate::sorcerer::instancius::Instancius;
use crate::vault::Vault;
use flecsd_axum_server::apis::instances::InstancesGetResponse as GetResponse;
use flecsd_axum_server::models::InstancesGetQueryParams as GetQueryParams;
use std::sync::Arc;

pub async fn get<I: Instancius>(
    vault: Arc<Vault>,
    instancius: Arc<I>,
    query_params: GetQueryParams,
) -> GetResponse {
    let instances = match query_params {
        GetQueryParams {
            version: None,
            app: None,
        } => {
            instancius
                .get_all_instances(
                    Quest::new_synced("Get info for all instances".to_string()),
                    vault,
                )
                .await
        }
        GetQueryParams { version, app } => {
            instancius
                .get_instances_filtered(
                    Quest::new_synced(format!(
                        "Get all instances matching {:?} in version {:?}",
                        app, version
                    )),
                    vault,
                    app,
                    version,
                )
                .await
        }
    };
    GetResponse::Status200_Success(instances)
}
