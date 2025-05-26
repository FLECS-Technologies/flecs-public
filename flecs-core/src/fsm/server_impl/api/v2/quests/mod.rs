use crate::enchantment::quest_master::QuestMaster;
use crate::sorcerer::mage_quester::MageQuester;
use flecsd_axum_server::apis::quests::QuestsGetResponse as GetResponse;
use std::sync::Arc;

pub mod id;

pub async fn get<M: MageQuester>(mage_quester: Arc<M>, quest_master: QuestMaster) -> GetResponse {
    GetResponse::Status200_Success(mage_quester.get_quest_models(quest_master).await)
}
