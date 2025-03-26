pub mod job_id;

use crate::sorcerer::mage_quester::MageQuester;
use flecsd_axum_server::apis::jobs::JobsGetResponse as GetResponse;
use std::sync::Arc;

pub async fn get<M: MageQuester>(magequester: Arc<M>) -> GetResponse {
    GetResponse::Status200_Success(magequester.get_jobs().await)
}
