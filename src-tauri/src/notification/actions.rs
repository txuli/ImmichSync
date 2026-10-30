use std::path::Path;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_log::log;
use crate::emit_sync_status::emit_sync_status;

/// Uploads `mount_point` to Immich on a background thread, reporting progress
/// through `emit_sync_status` and the outcome as a notification.
pub fn start_sync(app: &AppHandle, disk_name: &str, mount_point: &Path, album_name: Option<String>) {
    log::info!("[notification] sync accepted for {disk_name}, starting thread");
    let app_handle = app.clone();
    let path = mount_point.to_string_lossy().to_string();
    let disk_name = disk_name.to_string();
    std::thread::spawn(move || {
        emit_sync_status(&app_handle, "syncing", &disk_name, None, 0, 0);
        // Snapshot the media count/size before syncing — sync_assets may
        // delete these files afterward if "remove assets after upload" is on.
        let (uploaded_photos, uploaded_size) = crate::sync::scan_media_stats(&path);
        let sync_result = tauri::async_runtime::block_on(crate::sync::sync_assets(
            app_handle.clone(),
            path.clone(),
            album_name,
            disk_name.clone()
        ));
        log::debug!(
            "[notification] sync_assets finished for {disk_name}, ok={}",
            sync_result.is_ok()
        );
        match sync_result {
            Ok(response) => {
                if let Some(warning) = response.warning {
                    super::upload_success_with_warning(&app_handle, &warning);
                    emit_sync_status(
                        &app_handle,
                        "success",
                        &disk_name,
                        Some(warning),
                        uploaded_photos,
                        uploaded_size,
                    );
                } else {
                    super::upload_success(&app_handle);
                    emit_sync_status(
                        &app_handle,
                        "success",
                        &disk_name,
                        None,
                        uploaded_photos,
                        uploaded_size,
                    );
                }
            }
            Err(err) => {
                log::error!("[notification] sync failed for {disk_name}: {err}");
                super::upload_failed(&app_handle, &err);
                emit_sync_status(&app_handle, "error", &disk_name, Some(err), 0, 0);
            }
        }
    });
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Opens the main window on the page that asks whether to sync a known
/// device to its saved album.
pub fn open_known_device(app: &AppHandle, disk_name: &str, mount_point: &Path, album_name: &str) {
    log::info!("[notification] opening known-device page for {disk_name}");
    show_main_window(app);
    if let Err(err) = app.emit(
        "navigate-known-device",
        serde_json::json!({ "diskName": disk_name, "mountPoint": mount_point, "albumName": album_name }),
    ) {
        log::error!("[notification] failed to emit navigate-known-device event: {err:?}");
    }
}

/// Starts the same sync as the notification's "Sync" button, for the
/// known-device page.
#[tauri::command]
pub fn sync_device(app: AppHandle, disk_name: String, mount_point: String, album: Option<String>) {
    start_sync(&app, &disk_name, Path::new(&mount_point), album);
}

/// Opens the main window on the page where the user picks an album for
/// `disk_name`.
pub fn open_new_device(app: &AppHandle, disk_name: &str, mount_point: &Path) {
    log::info!("[notification] choose-album pressed for {disk_name}, opening window");
    show_main_window(app);
    if let Err(err) = app.emit(
        "navigate-new-device",
        serde_json::json!({ "diskName": disk_name, "mountPoint": mount_point }),
    ) {
        log::error!("[notification] failed to emit navigate-new-device event: {err:?}");
    }
}
