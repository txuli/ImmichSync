use serde_json::json;
use std::fs::File;
pub mod models;
mod notification;
pub use models::CheckToken;
pub use models::Settings;
pub use models::ValidResponse;
use std::thread;
use std::time::Duration;
use sysinfo::Disks;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use tauri::tray::TrayIconBuilder;
#[tauri::command]
async fn verify_token(url: &str, token: &str) -> Result<ValidResponse, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/auth/validateToken", url))
        .header("x-api-key", token)
        .send()
        .await
        .map_err(|err| format!("Error de red: {}", err))?;

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
async fn save_credentials(app: AppHandle,url: &str, token: &str) -> Result<ValidResponse, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let try_save = || -> Result<(), std::io::Error> {
        store.set("url",json!(url));
        store.set("token",json!(token));
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
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![verify_token, save_credentials])
        .setup(|app| {
            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;

            let handle = app.handle().clone();
            thread::spawn(move || {
                let mut disks = Disks::new_with_refreshed_list();
                let mut old_disks: Vec<String> = vec![];
                let mut actual_disks: Vec<String> = vec![];
                loop {
                    thread::sleep(Duration::from_secs(1));

                    actual_disks = disks
                        .list()
                        .iter()
                        .filter(|disk| disk.is_removable())
                        .map(|disk| disk.name().to_string_lossy().to_string())
                        .collect();

                    disks.refresh(true);
   
                    for disk in &actual_disks {
                        if !old_disks.iter().any(|n| n == disk) {
                            notification::newDevice::notify_new_device(&handle, disk);
                        }
                    }
                    old_disks = actual_disks;     
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
