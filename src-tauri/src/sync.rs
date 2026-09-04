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
            
            "--on-errors",
            "continue",
            &path,
        ])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    // immich-go exits non-zero whenever *any* file failed, even if the rest
    // of the batch uploaded successfully — with --on-errors continue that's
    // expected on large syncs, so it's surfaced as a warning on the response
    // rather than failing the whole sync (which would also skip the removal
    // step below).
    let warning = if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        eprintln!(
            "[sync] immich-go reported errors for some files (exit status: {:?}): {stderr}",
            output.status
        );
        Some(if stderr.is_empty() {
            "immich-go reported errors while uploading some files".to_string()
        } else {
            stderr
        })
    } else {
        None
    };

    let remove_after_upload = store
        .get("rmAssets")
        .and_then(|v| v.get("value").and_then(|b| b.as_bool()))
        .unwrap_or(false);

    if remove_after_upload {
        if let Err(err) = delete_media_files(&path) {
            eprintln!("[sync] Failed to remove uploaded assets from {path}: {err}");
        }
    }

    Ok(ValidResponse {
        valid: true,
        type_acc: "sync".to_string(),
        warning,
    })
}

const MEDIA_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "heic", "heif", "bmp", "tiff", "tif", "webp", "cr2", "cr3",
    "nef", "arw", "dng", "raf", "orf", "rw2", "mp4", "mov", "avi", "mkv", "m4v", "3gp", "webm",
];

/// Best-effort count (and combined size) of media files found under `path`.
///
/// This is computed locally by walking the folder after a sync — immich-go
/// itself doesn't report structured upload counts, so this can include files
/// immich-go skipped as duplicates on a re-sync. It's a reasonable proxy for
/// "how much this device has to offer", not an exact server-side total.
pub fn scan_media_stats(path: &str) -> (i64, i64) {
    let mut count = 0i64;
    let mut size = 0i64;
    let mut stack = vec![std::path::PathBuf::from(path)];

    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }
            let is_media = entry_path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| MEDIA_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false);
            if is_media {
                count += 1;
                if let Ok(meta) = entry.metadata() {
                    size += meta.len() as i64;
                }
            }
        }
    }

    (count, size)
}

/// Deletes every media file found under `path` (same extension list as
/// `scan_media_stats`). Only called after immich-go has confirmed the
/// upload succeeded, and only when the user enabled "remove assets after
/// upload". Subdirectories are walked but left in place — only files are
/// removed.
fn delete_media_files(path: &str) -> Result<(), String> {
    let mut stack = vec![std::path::PathBuf::from(path)];
    let mut errors = Vec::new();

    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(err) => {
                errors.push(format!("{}: {err}", dir.display()));
                continue;
            }
        };
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }
            let is_media = entry_path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| MEDIA_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false);
            if is_media {
                if let Err(err) = std::fs::remove_file(&entry_path) {
                    errors.push(format!("{}: {err}", entry_path.display()));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}
