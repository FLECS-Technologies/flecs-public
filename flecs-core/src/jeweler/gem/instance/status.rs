use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum InstanceStatus {
    // TBD
    NotCreated,
    Requested,
    ResourcesReady,
    Stopped,
    Running,
    Orphaned,
    Unknown,
}

impl From<InstanceStatus> for flecsd_axum_server::models::InstanceStatus {
    fn from(value: InstanceStatus) -> Self {
        match value {
            InstanceStatus::NotCreated => flecsd_axum_server::models::InstanceStatus::NotCreated,
            InstanceStatus::Requested => flecsd_axum_server::models::InstanceStatus::Requested,
            InstanceStatus::ResourcesReady => {
                flecsd_axum_server::models::InstanceStatus::ResourcesReady
            }
            InstanceStatus::Stopped => flecsd_axum_server::models::InstanceStatus::Stopped,
            InstanceStatus::Running => flecsd_axum_server::models::InstanceStatus::Running,
            InstanceStatus::Orphaned => flecsd_axum_server::models::InstanceStatus::Orphaned,
            InstanceStatus::Unknown => flecsd_axum_server::models::InstanceStatus::Unknown,
        }
    }
}

impl From<&str> for InstanceStatus {
    fn from(value: &str) -> Self {
        match value {
            "not created" => InstanceStatus::NotCreated,
            "orphaned" => InstanceStatus::Orphaned,
            "requested" => InstanceStatus::Requested,
            "resources ready" => InstanceStatus::ResourcesReady,
            "running" => InstanceStatus::Running,
            "stopped" => InstanceStatus::Stopped,
            _ => InstanceStatus::Unknown,
        }
    }
}
