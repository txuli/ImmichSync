use serde_json::json;
use std::fs::File;
pub mod models;
pub use models::CheckToken;
pub use models::Settings;
pub use models::ValidResponse;
use std::thread;
use std::time::Duration;
use sysinfo::Disks;

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
async fn save_credentials(url: &str, token: &str) -> Result<ValidResponse, String> {
    let try_save = || -> Result<(), std::io::Error> {
        let file = File::create("config.json")?;
        let data = json!({
            "url": url,
            "token": token,
        });
        serde_json::to_writer_pretty(file, &data)?;
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
/* #[tauri::command]
async fn load_config() -> Result<Settings, String> {
    let file = File::open("config.json").map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let s = serde_json::from_reader(reader).map_err(|e| e.to_string())?;
    Ok(s)
} */

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![verify_token, save_credentials])
        .setup(|app| {
            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;

            thread::spawn(|| {
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

                    println!("actual {:?}", actual_disks);
                    for disk in &actual_disks {
                        if !old_disks.iter().any(|n| n == disk) {
                            println!("adfasdf")
                        }
                    }

                    old_disks = actual_disks;
                    println!("old {:?}", old_disks);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
