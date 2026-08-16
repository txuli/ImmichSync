use serde_json::json;
use std::fs::File;
pub mod models;
mod notification;
pub use models::CheckToken;
pub use models::Settings;
pub use models::ValidResponse;
pub mod sync;
use std::thread;
use std::time::Duration;
pub use sync::sync_assets;
use sysinfo::Disks;
use tauri::AppHandle;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

/// Passed by the autostart plugin when the OS launches the app at login,
/// so we know to keep the window hidden instead of showing it.
const HIDDEN_ARG: &str = "--hidden";

use tauri::tray::TrayIconBuilder;
#[tauri::command]
async fn verify_token(url: &str, token: &str) -> Result<ValidResponse, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/auth/validateToken", url))
        .header("x-api-key", token)
        .send()
        .await
        .map_err(|err| format!("Network error: {}", err))?;

    if response.status().is_success() {
        Ok(ValidResponse {
            valid: true,
            type_acc: "credential".to_string(),
        })
    } else {
        Ok(ValidResponse {
            valid: false,
            type_acc: "credential".to_string(),
        })
    }
}
#[tauri::command]
async fn save_credentials(app: AppHandle, url: &str, token: &str) -> Result<ValidResponse, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let try_save = || -> Result<(), std::io::Error> {
        store.set("url", json!(url));
        store.set("token", json!(token));
        /* store.save(); */
        Ok(())
    };

    match try_save() {
        Ok(_) => Ok(ValidResponse {
            valid: true,
            type_acc: "save".to_string(),
        }),
        Err(_err) => Ok(ValidResponse {
            valid: false,
            type_acc: "save".to_string(),
        }),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .arg(HIDDEN_ARG)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            verify_token,
            save_credentials,
            sync_assets
        ])
        .setup(|app| {
            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;

            // The window starts hidden (see tauri.conf.json). Only reveal it
            // unless we were launched by the autostart plugin with --hidden.
            let launched_hidden = std::env::args().any(|arg| arg == HIDDEN_ARG);
            if !launched_hidden {
                if let Some(window) = app.get_webview_window("main") {
                    window.show()?;
                }
            }

            let handle = app.handle().clone();
            thread::spawn(move || {
                let mut disks = Disks::new_with_refreshed_list();
                let mut old_disks: Vec<String> = vec![];
                loop {
                    thread::sleep(Duration::from_secs(1));

                    let actual_disks: Vec<(String, std::path::PathBuf)> = disks
                        .list()
                        .iter()
                        .filter(|disk| disk.is_removable())
                        .map(|disk| {
                            (
                                disk.name().to_string_lossy().to_string(),
                                disk.mount_point().to_path_buf(),
                            )
                        })
                        .collect();

                    disks.refresh(true);

                    for (name, mount_point) in &actual_disks {
                        if !old_disks.iter().any(|n| n == name) {
                            notification::newDevice::notify_new_device(&handle, name, mount_point);
                        }
                    }
                    old_disks = actual_disks.iter().map(|(name, _)| name.clone()).collect();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
