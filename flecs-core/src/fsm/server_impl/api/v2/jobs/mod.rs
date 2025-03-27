pub mod job_id;

use crate::enchantment::quest_master::QuestMaster;
use crate::sorcerer::mage_quester::MageQuester;
use flecsd_axum_server::apis::jobs::JobsGetResponse as GetResponse;
use std::sync::Arc;

pub async fn get<M: MageQuester>(magequester: Arc<M>, quest_master: QuestMaster) -> GetResponse {
    GetResponse::Status200_Success(magequester.get_jobs(quest_master).await)
}
