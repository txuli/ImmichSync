use crate::models::SyncStatusEvent;
use tauri::{AppHandle, Emitter};
use tauri_plugin_log::log;

/// Broadcasts the "sync-status" event so the dashboard can show live
/// progress and a recent-activity feed.
pub fn emit_sync_status(
    app: &AppHandle,
    status: &str,
    disk_name: &str,
    error: Option<String>,
    uploaded_photos: i64,
    uploaded_size: i64,
) {
    let payload = SyncStatusEvent {
        status: status.to_string(),
        disk_name: disk_name.to_string(),
        error,
        timestamp: chrono::Local::now().to_rfc3339(),
        uploaded_photos,
        uploaded_size,
    };
    if let Err(err) = app.emit("sync-status", payload) {
        log::error!("[notification] failed to emit sync-status event: {err:?}");
    }
}