use crate::emit_sync_status::emit_sync_status;
use crate::models::ValidResponse;
use chrono::{DateTime, Duration, Local, NaiveTime, Utc};
use serde_json::json;
use std::time::SystemTime;
use std::{
    collections::{ HashSet},
    vec,
};
use std::path::Path;
use tauri::{ AppHandle, Manager};
use tauri_plugin_log::log;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_store::StoreExt;
use walkdir::WalkDir;

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
    prev_scan_files(url.clone(), token.clone(), path.clone()).await;
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
    let complete_path = format!("{path}/upload");
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
            &complete_path,
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
        log::debug!(
            "[sync] failed to clean up immich-go log {}: {err}",
            log_path.display()
        );
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

pub async fn prev_scan_files(url: String, token: String, path: String) {
    log::debug!("reading dir");
    /* #[derive(Debug)] */
    pub struct Files {
        name: String,
        path: String,
        size: u64,
        created_at: SystemTime,
    }
    let mut files_list: Vec<Files> = vec![];
    for entry in WalkDir::new(&path)
        .into_iter()
        
        .filter_map(|entry| entry.ok())
    {
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() && metadata.len() > 1000 {
                let created: SystemTime = metadata.modified().unwrap();
                let file = Files {
                    name: entry.file_name().display().to_string(),
                    path: entry.path().display().to_string(),
                    size: metadata.len(),
                    created_at: created,
                };

                files_list.push(file);
              
            }
        }
    }
    
    let (Some(oldest), Some(newest)) = (
        files_list.iter().map(|f| f.created_at).min(),
        files_list.iter().map(|f| f.created_at).max(),
    ) else {
        log::debug!("[sync] prev_scan_files found no files, skipping search");
        return;
    };
    
    let start_of_day = |time: SystemTime| {
        DateTime::<Local>::from(time)
            .date_naive()
            .and_time(NaiveTime::MIN)
            .and_local_timezone(Local)
            .earliest()
    };
    let (Some(taken_after), Some(taken_before)) = (
        start_of_day(oldest).map(|day| day + Duration::days(-1)),
        start_of_day(newest).map(|day| day + Duration::days(1)),
    ) else {
        log::warn!("[sync] could not resolve local midnight for the date range");
        return;
    };
    let client = reqwest::Client::new();
    let after_utc = taken_after.with_timezone(&Utc).to_rfc3339();
    let before_utc = taken_before.with_timezone(&Utc).to_rfc3339();
    let local_fmt = |time: DateTime<Local>| time.format("%Y-%m-%d %H:%M:%S %:z").to_string();
    log::info!(
        "[sync] prev_scan_files sending {} file(s)\n  oldest file: {}\n  newest file: {}\n  takenAfter : {}  (UTC {})\n  takenBefore: {}  (UTC {})",
        files_list.len(),
        local_fmt(DateTime::<Local>::from(oldest)),
        local_fmt(DateTime::<Local>::from(newest)),
        local_fmt(taken_after),
        after_utc,
        local_fmt(taken_before),
        before_utc,
    );

    // If the search fails midway we can't tell what the server has, so every
    // file stays pending: worst case immich-go re-checks (and skips) them.
    let server_files =
        match fetch_server_files(&client, &url, &token, &after_utc, &before_utc).await {
            Ok(server_files) => server_files,
            Err(err) => {
                log::error!(
                    "[sync] prev_scan_files search failed, keeping every file as pending: {err}"
                );
                return;
            }
        };

    // A local file counts as already uploaded only if the server has one with
    // the same name (case-insensitive, like Windows) *and* the same size.
    let total = files_list.len();
    files_list.retain(|file| !server_files.contains(&(file.name.to_lowercase(), file.size)));
    log::info!(
        "[sync] prev_scan_files: {} of {total} file(s) already on the server, {} pending",
        total - files_list.len(),
        files_list.len()
    );
    for file in &files_list {
        log::debug!("[sync] pending: {} ({} bytes)", file.path, file.size);
    }

   
    let root = Path::new(&path);
    let upload_dir = root.join("upload");
    let (mut moved, mut already_there, mut failed) = (0usize, 0usize, 0usize);
    for file in &files_list {
        let source = Path::new(&file.path);
        if source.starts_with(&upload_dir) {
            already_there += 1;
            continue;
        }
        let Ok(relative) = source.strip_prefix(root) else {
            failed += 1;
            continue;
        };
        let destination = upload_dir.join(relative);
        // `rename` silently replaces an existing file on Windows.
        if destination.exists() {
            log::warn!(
                "[sync] not moving {}: {} already exists",
                source.display(),
                destination.display()
            );
            failed += 1;
            continue;
        }
        let result = match destination.parent() {
            Some(parent) => std::fs::create_dir_all(parent),
            None => Ok(()),
        }
        .and_then(|_| std::fs::rename(source, &destination));
        match result {
            Ok(()) => moved += 1,
            Err(err) => {
                log::error!("[sync] could not move {}: {err}", source.display());
                failed += 1;
            }
        }
    }
    log::info!(
        "[sync] prev_scan_files: moved {moved} file(s) to {}, {already_there} already there, {failed} failed",
        upload_dir.display()
    );
}


async fn fetch_server_files(
    client: &reqwest::Client,
    url: &str,
    token: &str,
    taken_after: &str,
    taken_before: &str,
) -> Result<HashSet<(String, u64)>, String> {
    const PAGE_SIZE: u32 = 1000;
    const MAX_PAGES: u32 = 100;

    let mut found = HashSet::new();
    let mut without_size = 0usize;
    let mut page = 1u32;
    loop {
        let body = json!({
            "takenAfter": taken_after,
            "takenBefore": taken_before,
            "size": PAGE_SIZE,
            "page": page,
        });
        let response = client
            .post(format!("{url}/api/search/metadata"))
            .header("x-api-key", token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("page {page}: {e}"))?;
        let status = response.status();
        if !status.is_success() {
            return Err(format!("page {page}: server answered {status}"));
        }
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("page {page}: could not parse the response: {e}"))?;

        for item in json["assets"]["items"].as_array().into_iter().flatten() {
            let Some(name) = item["originalFileName"].as_str() else {
                continue;
            };
            // Without a size we can't confirm it's the same file, so it is
            // left out and the local file stays pending.
            match item["exifInfo"]["fileSizeInByte"].as_u64() {
                Some(size) => {
                    found.insert((name.to_lowercase(), size));
                }
                None => without_size += 1,
            }
        }

        // `nextPage` is null on the last page; Immich sends it as a string.
        let next_page = &json["assets"]["nextPage"];
        if next_page.is_null() {
            break;
        }
        let next = next_page
            .as_str()
            .and_then(|p| p.parse::<u32>().ok())
            .or_else(|| next_page.as_u64().map(|n| n as u32));
        match next {
            Some(next) if next > page && page < MAX_PAGES => page = next,
            _ => return Err(format!("unexpected nextPage {next_page} after page {page}")),
        }
    }

    if without_size > 0 {
        log::warn!(
            "[sync] {without_size} server asset(s) came back without exifInfo.fileSizeInByte and were ignored"
        );
    }
    log::debug!("[sync] fetched {} server file(s) over {page} page(s)", found.len());
    Ok(found)
}
