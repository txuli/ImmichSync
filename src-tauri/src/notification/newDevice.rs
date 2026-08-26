use crate::models::SyncStatusEvent;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

/// Broadcasts the "sync-status" event so the dashboard can show live
/// progress and a recent-activity feed.
fn emit_sync_status(app: &AppHandle, status: &str, disk_name: &str, error: Option<String>) {
    let payload = SyncStatusEvent {
        status: status.to_string(),
        disk_name: disk_name.to_string(),
        error,
        timestamp: chrono::Local::now().to_rfc3339(),
    };
    if let Err(err) = app.emit("sync-status", payload) {
        eprintln!("[notification] Failed to emit sync-status event: {err:?}");
    }
}

fn debug_log(msg: impl AsRef<str>) {
    use std::io::Write;
    let path = std::env::temp_dir().join("immichsync-debug.log");
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(
            file,
            "[{:?}] {}",
            std::time::SystemTime::now(),
            msg.as_ref()
        );
    }
}

fn app_icon_path<R: tauri::Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    if let Ok(path) = app
        .path()
        .resolve("icons/128x128.png", tauri::path::BaseDirectory::Resource)
    {
        if path.exists() {
            return Some(strip_verbatim_prefix(path));
        }
    }

    #[cfg(debug_assertions)]
    {
        let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("icons/128x128.png");
        if dev_path.exists() {
            return Some(strip_verbatim_prefix(dev_path));
        }
    }

    eprintln!("[notification] App icon not found in any expected path");
    None
}

fn strip_verbatim_prefix(path: PathBuf) -> PathBuf {
    match path.to_str() {
        Some(s) => match s.strip_prefix(r"\\?\") {
            Some(stripped) => PathBuf::from(stripped),
            None => path,
        },
        None => path,
    }
}

#[cfg(windows)]
const APP_ID: &str = "com.txuli.immichsync";
#[cfg(windows)]
const APP_NAME: &str = "ImmichSync";

#[cfg(windows)]
fn register_aumid(icon_path: Option<&std::path::Path>) {
    use windows_registry::CURRENT_USER;

    let result = (|| -> windows_registry::Result<()> {
        let key = CURRENT_USER.create(format!(r"SOFTWARE\Classes\AppUserModelId\{APP_ID}"))?;
        key.set_string("DisplayName", APP_NAME)?;
        key.set_string("IconBackgroundColor", "0")?;
        if let Some(icon_path) = icon_path {
            key.set_hstring("IconUri", &icon_path.into())?;
        }
        Ok(())
    })();

    if let Err(err) = result {
        eprintln!("[notification] Failed to register the app's AUMID: {err:?}");
    }
}

#[cfg(windows)]
pub fn notify_new_device(app: &AppHandle, disk_name: &str, mount_point: &std::path::Path) {
    use tauri_winrt_notification::{Duration, Toast};

    let disk_name = disk_name.to_string();
    let mount_point = mount_point.to_path_buf();
    let app_handle = app.clone();
    let icon_path = app_icon_path(app);
    register_aumid(icon_path.as_deref());

    let result = Toast::new(APP_ID)
        .title("New device detected")
        .text1(&format!("Connected: {disk_name}"))
        .add_button("Sync", "sync")
        .add_button("Ignore", "ignore")
        .duration(Duration::Short)
        .on_activated(move |action| {
            match action.as_deref() {
                Some("sync") => {
                    println!("[notification] Sync pressed for {disk_name}");
                    debug_log("sync button pressed, starting thread");
                    let app_handle = app_handle.clone();
                    let path = mount_point.to_string_lossy().to_string();
                    let disk_name = disk_name.clone();
                    std::thread::spawn(move || {
                        emit_sync_status(&app_handle, "syncing", &disk_name, None);
                        debug_log("thread started, calling sync_assets");
                        let sync_result = tauri::async_runtime::block_on(crate::sync::sync_assets(
                            app_handle.clone(),
                            path,
                            None,
                        ));
                        debug_log(format!("sync_assets finished, ok={}", sync_result.is_ok()));
                        match sync_result {
                            Ok(_) => {
                                debug_log("calling upload_success");
                                upload_success();
                                emit_sync_status(&app_handle, "success", &disk_name, None);
                                debug_log("upload_success finished");
                            }
                            Err(err) => {
                                eprintln!("[sync] Sync failed: {err}");
                                debug_log(format!("calling upload_failed: {err}"));
                                upload_failed(&err);
                                emit_sync_status(&app_handle, "error", &disk_name, Some(err));
                                debug_log("upload_failed finished");
                            }
                        }
                    });
                }
                Some("ignore") => println!("[notification] Ignore pressed for {disk_name}"),
                _ => {}
            }
            Ok(())
        })
        .show();

    if let Err(err) = result {
        eprintln!("[notification] Failed to show the notification: {err:?}");
    }
}
#[cfg(windows)]
pub fn upload_success() {
    use tauri_winrt_notification::{Duration, Toast};

    let result = Toast::new(APP_ID)
        .title("Sync complete")
        .text1("Your photos were uploaded to Immich successfully")
        .duration(Duration::Short)
        .show();

    debug_log(format!("upload_success: Toast::show() -> {result:?}"));
    if let Err(err) = result {
        eprintln!("[notification] Failed to show the notification: {err:?}");
    }
}

#[cfg(windows)]
pub fn upload_failed(error: &str) {
    use tauri_winrt_notification::{Duration, Toast};

    let result = Toast::new(APP_ID)
        .title("Sync failed")
        .text1(&format!("Could not upload your photos: {error}"))
        .duration(Duration::Short)
        .show();

    if let Err(err) = result {
        eprintln!("[notification] Failed to show the notification: {err:?}");
    }
}
#[cfg(not(windows))]
pub fn notify_new_device<R: tauri::Runtime>(
    app: &AppHandle<R>,
    disk_name: &str,
    _mount_point: &std::path::Path,
) {
    use tauri_plugin_notification::NotificationExt;

    let icon_path = app_icon_path(app);

    let mut builder = app
        .notification()
        .builder()
        .title("New device detected")
        .body(format!("Connected: {disk_name}"));

    if let Some(icon_path) = &icon_path {
        if let Some(icon_path) = icon_path.to_str() {
            builder = builder.icon(icon_path);
        }
    }

    if let Err(err) = builder.show() {
        eprintln!("[notification] Failed to show the notification: {err:?}");
    }
}
