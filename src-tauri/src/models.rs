use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug)]
pub struct CheckToken {
    pub valid: bool,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct ValidResponse {
    pub valid: bool,
    pub type_acc: String,
    /// Set when `sync_assets` completed but immich-go reported errors on
    /// some files (e.g. a transient IO error) while still uploading the
    /// rest of the batch. `None` means a clean, error-free sync.
    pub warning: Option<String>,
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
    /// Best-effort count of media files found in the synced folder.
    /// Only meaningful when `status` is "success"; 0 otherwise.
    pub uploaded_photos: i64,
    /// Combined size in bytes of those media files.
    pub uploaded_size: i64,
}
