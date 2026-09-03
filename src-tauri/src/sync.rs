use crate::models::ValidResponse;
use chrono::Local;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_store::StoreExt;

#[derive(Debug)]
pub enum SyncError {
    StoreError(String),
    ShellError(String),
}

#[tauri::command]
pub async fn sync_assets(
    app: AppHandle,
    path: String,
    album: Option<String>,
) -> Result<ValidResponse, String> {
    if path.trim().is_empty() {
        return Err("sync_assets called with an empty path".to_string());
    }
    if !std::path::Path::new(&path).exists() {
        return Err(format!("sync_assets: path does not exist: {path}"));
    }

    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let album_name = match album {
        Some(name) if !name.is_empty() => name,
        _ => format!("ImmichSync"),
    };

    let url = store
        .get("url")
        .and_then(|v| v.as_str().map(str::to_string))
        .ok_or_else(|| "missing url in settings".to_string())?;
    let token = store
        .get("token")
        .and_then(|v| v.as_str().map(str::to_string))
        .ok_or_else(|| "missing token in settings".to_string())?;
    crate::notification::new_sync::notify_sync_started(&app, &album_name);
    let sidecar = app
        .shell()
        .sidecar("immich-go")
        .map_err(|e| e.to_string())?;

    let output = sidecar
        .args([
            "upload",
            "from-folder",
            "--server",
            &url,
            "--api-key",
            &token,
            "--into-album",
            &album_name,
            "--no-ui",
            &path,
        ])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(ValidResponse {
        valid: true,
        type_acc: "sync".to_string(),
    })
}
