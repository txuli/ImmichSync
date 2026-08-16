use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug)]
pub struct CheckToken {
    pub valid: bool,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct ValidResponse {
    pub valid: bool,
    pub type_acc: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Settings {
    pub url: String,
    pub token: String,
}
/// Broadcast to the frontend on the "sync-status" event so the dashboard
/// can show live sync progress and a recent-activity feed.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SyncStatusEvent {
    /// "syncing" | "success" | "error"
    pub status: String,
    pub disk_name: String,
    pub error: Option<String>,
    pub timestamp: String,
}
