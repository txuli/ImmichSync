use tauri::AppHandle;
use tauri_plugin_log::log;

pub fn notify_sync_started(app: &AppHandle, album_name: &str) {
    log::info!("[notification] showing sync-started notification for album \"{album_name}\"");
    super::show(
        app,
        "Sync started",
        &format!("Uploading to album \"{album_name}\""),
    );
}
