use crate::jeweler::app::AppStatus;
use crate::vault::pouch::AppKey;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct App {
    pub app_key: AppKey,
    pub _status: String,
    pub desired: String,
    pub _installed_size: u64,
}

pub fn app_status_from_legacy(legacy: &str) -> AppStatus {
    match legacy {
        "not installed" => AppStatus::NotInstalled,
        "manifest downloaded" => AppStatus::None,
        "token acquired" => AppStatus::None,
        "image downloaded" => AppStatus::None,
        "installed" => AppStatus::Installed,
        "removed" => AppStatus::NotInstalled,
        "purged" => AppStatus::NotInstalled,
        "orphaned" => AppStatus::None,
        _ => AppStatus::None,
    }
}
