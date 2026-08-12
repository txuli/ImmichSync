
use std::io::Write;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::fs::File;
use serde_json::json;
pub mod models;
pub use models::CheckToken;
pub use models::ValidResponse;

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
        Ok(ValidResponse { valid: true, type_acc:"credential".to_string() })
    } else {
        Ok(ValidResponse { valid: false, type_acc:"credential".to_string() })
    }
}
#[tauri::command]
async fn save_credentials(url: &str, token: &str) -> Result<ValidResponse, String> {
    
   let try_save = || -> Result<(), std::io::Error> {
        let file = File::create("config.json")?;
        let data= json!({
            "url": url,
            "token": token,
        });
        serde_json::to_writer_pretty(file, &data)?;
        Ok(())
    };

    match try_save() {
        Ok(_) => Ok(ValidResponse { valid: true, type_acc:"save".to_string() }),
        Err(_err) => Ok(ValidResponse { valid: false, type_acc:"save".to_string() }),
    }
        
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![verify_token, save_credentials])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
