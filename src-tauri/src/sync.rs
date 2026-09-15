use crate::models::ValidResponse;
use crate::emit_sync_status::emit_sync_status;
use std::collections::HashSet;
use tauri::{AppHandle, Manager};
use tauri_plugin_log::log;
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
    disk_name: String,
) -> Result<ValidResponse, String> {
    if path.trim().is_empty() {
        return Err("sync_assets called with an empty path".to_string());
    }
    if !std::path::Path::new(&path).exists() {
        return Err(format!("sync_assets: path does not exist: {path}"));
    }
    let album_name = match album {
        Some(name) if !name.is_empty() => name,
        _ => format!("ImmichSync"),
    };
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

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
    log::info!("sync starting");
    emit_sync_status(&app, "syncing", &disk_name, None, 0, 0);

    // immich-go doesn't report structured per-file results on stdout, but it
    // will write a text log with one line per file event (uploaded, error,
    // duplicate, ...) when given --log-file. We read that back after the run
    // to know exactly which files the server actually confirmed, instead of
    // trusting the process' overall exit status.
    let log_path = upload_log_path(&app, &disk_name)?;
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
            "--log-file",
            &log_path.to_string_lossy(),
            "--log-type",
            "text",
            &path,
        ])
        .output()
        .await
        .map_err(|e| {
            log::error!("[sync] failed to spawn immich-go for path={path}: {e}");
            e.to_string()
        })?;
    log::debug!("[sync] immich-go output: {:?}", output);

    // immich-go exits non-zero whenever *any* file failed, even if the rest
    // of the batch uploaded successfully — with --on-errors continue that's
    // expected on large syncs, so it's surfaced as a warning on the response
    // rather than failing the whole sync (which would also skip the removal
    // step below).
    let warning = if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        log::error!(
            "[sync] immich-go reported errors for some files (exit status: {:?}): {stderr}",
            output.status
        );
        Some(if stderr.is_empty() {
            "immich-go reported errors while uploading some files".to_string()
        } else {
            stderr
        })
    
    } else {
        log::info!("[sync] immich-go upload succeeded for path={path} album={album_name}");
        None
    };

    let remove_after_upload = store
        .get("rmAssets")
        .and_then(|v| v.get("value").and_then(|b| b.as_bool()))
        .unwrap_or(false);

    if remove_after_upload {
        match std::fs::read_to_string(&log_path) {
            Ok(log_contents) => {
                let confirmed = parse_confirmed_uploads(&log_contents);
                log::info!(
                    "[sync] {} file(s) confirmed uploaded by immich-go for path={path}",
                    confirmed.len()
                );
                if let Err(err) = delete_confirmed_media_files(&path, &confirmed) {
                    log::error!("[sync] failed to remove uploaded assets from {path}: {err}");
                }
            }
            Err(err) => {
                // Without the log we can't tell what actually made it to the
                // server, so we deliberately skip deletion rather than fall
                // back to deleting everything.
                log::error!(
                    "[sync] could not read immich-go log at {}, skipping asset removal: {err}",
                    log_path.display()
                );
            }
        }
    }
    if let Err(err) = std::fs::remove_file(&log_path) {
        log::debug!("[sync] failed to clean up immich-go log {}: {err}", log_path.display());
    }

    Ok(ValidResponse {
        valid: true,
        type_acc: "sync".to_string(),
        warning,
    })
}

const MEDIA_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "heic", "heif", "bmp", "tiff", "tif", "webp", "cr2", "cr3", "nef",
    "arw", "dng", "raf", "orf", "rw2", "mp4", "mov", "avi", "mkv", "m4v", "3gp", "webm",
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

/// Builds a unique path (in the app's data dir) for immich-go's `--log-file`
/// for this sync run, creating the containing directory if needed.
fn upload_log_path(app: &AppHandle, disk_name: &str) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("sync-logs");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let safe_disk_name: String = disk_name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    let file_name = format!(
        "immich-go-{safe_disk_name}-{}.log",
        chrono::Local::now().format("%Y%m%d%H%M%S%3f")
    );
    Ok(dir.join(file_name))
}

/// Event markers from immich-go's per-file log lines (see
/// https://github.com/simulot/immich-go internal/fileevent) that mean the
/// local file's content is confirmed present on the server: either just
/// uploaded, already there as a duplicate, or it replaced a lower-quality
/// version server-side. Anything else (errors, filtered/discarded files,
/// ...) is left untouched.
const CONFIRMED_UPLOAD_MARKERS: &[&str] = &[
    "uploaded successfully",
    "server has duplicate",
    "server asset upgraded",
];

/// Parses immich-go's `--log-type text` log and returns the set of local
/// file paths (relative to the synced folder, forward-slash separated, e.g.
/// `sub/photo.jpg`) it confirmed are present on the server.
///
/// Each relevant line looks like:
/// `2026-01-01 12:00:00 INF uploaded successfully file=<folder>:<relpath>`
/// where `file=...` is the last field on the line, so everything after
/// `file=` (up to the `:` separating immich-go's internal folder alias from
/// the relative path) is taken as-is, spaces included.
fn parse_confirmed_uploads(log_contents: &str) -> HashSet<String> {
    let mut confirmed = HashSet::new();
    for line in log_contents.lines() {
        let Some(marker) = CONFIRMED_UPLOAD_MARKERS
            .iter()
            .find(|marker| line.contains(*marker))
        else {
            continue;
        };
        let Some(after_marker) = line.split(marker).nth(1) else {
            continue;
        };
        let Some(file_field) = after_marker.split("file=").nth(1) else {
            continue;
        };
        let value = file_field.trim();
        if value.is_empty() {
            continue;
        }
        let relpath = value.split_once(':').map_or(value, |(_, rel)| rel);
        confirmed.insert(relpath.replace('\\', "/"));
    }
    confirmed
}

/// Deletes only the media files under `path` whose relative path is in
/// `confirmed` — the set immich-go's log reported as actually present on the
/// server (uploaded or already there). Files it skipped, filtered out, or
/// failed to upload are left in place. Subdirectories are walked but left in
/// place — only files are removed.
fn delete_confirmed_media_files(path: &str, confirmed: &HashSet<String>) -> Result<(), String> {
    let base = std::path::Path::new(path);
    let mut stack = vec![base.to_path_buf()];
    let mut errors = Vec::new();
    let mut deleted = 0usize;

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
            if !is_media {
                continue;
            }
            let relpath = match entry_path.strip_prefix(base) {
                Ok(rel) => rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/"),
                Err(_) => continue,
            };
            if !confirmed.contains(&relpath) {
                continue;
            }
            if let Err(err) = std::fs::remove_file(&entry_path) {
                errors.push(format!("{}: {err}", entry_path.display()));
            } else {
                deleted += 1;
            }
        }
    }

    log::info!("[sync] removed {deleted} confirmed-uploaded file(s) from {path}");

    if errors.is_empty() {
        Ok(())
    } else {
        log::error!("{}", errors.join("; "));
        Err(errors.join("; "))
    }
}
