// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod models;
pub use models::CheckToken;

#[tauri::command]
async fn verify_token(url: &str, token: &str) -> Result<CheckToken, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/auth/validateToken", url))
        .header("x-api-key",  token)
        .send()
        .await
        .map_err(|err| format!("Error de red: {}", err))?;

    if response.status().is_success() {
        Ok(CheckToken {
            valid: true,
        })
    } else {
        Ok(CheckToken{
            valid:false
        })
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![verify_token])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
