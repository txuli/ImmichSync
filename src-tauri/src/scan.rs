use crate::db::index::{check, get_pool, DeviceRow};
use sqlx::sqlite::SqlitePool;
use crate::notification;
use std::thread;
use std::time::Duration;
use sysinfo::{Disk, Disks};
use tauri::AppHandle;
use tauri_plugin_log::log;
use tauri_plugin_store::StoreExt;
#[derive(Debug)]

pub enum SyncError {
    StoreError(String),
    ShellError(String),
}
#[tauri::command]
pub fn scan(app: AppHandle) {
    log::info!("[scan] starting device watcher");
    thread::spawn(move || {
        let store = match app.store("settings.json") {
            Ok(store) => store,
            Err(error) => {
                log::error!("[scan] failed to open settings store: {error}");
                return;
            }
        };

        let mut disks = Disks::new_with_refreshed_list();
        let mut old_disks: Vec<String> = vec![];
        loop {
            thread::sleep(Duration::from_secs(1));

            let actual_disks: Vec<(String, std::path::PathBuf)> = disks
                .list()
                .iter()
                .filter(|disk| is_external(disk))
                .map(|disk| (device_name(disk), disk.mount_point().to_path_buf()))
                .collect();

            disks.refresh(true);

            for (name, mount_point) in &actual_disks {
                if !old_disks.iter().any(|n| n == name) {
                    log::info!(
                        "[scan] device connected: {name} at {}",
                        mount_point.display()
                    );
                    let url = store.get("url");
                    match tauri::async_runtime::block_on(get_pool(&app)) {
                        Ok(pool) => match tauri::async_runtime::block_on(find_device(&pool, name)) {
                            Ok(Some(row)) => {
                                if row.direct == "true" {
                                    log::info!(
                                        "[scan] known device {name} set to direct sync, path={} album={}",
                                        mount_point.display(), row.album_name
                                    );
                                    log::info!("url {:?}", url);
                                    if url.is_some() {
                                        let _sync_result = tauri::async_runtime::block_on(
                                            crate::sync::sync_assets(
                                                app.clone(),
                                                mount_point.to_string_lossy().to_string(),
                                                Some(row.album_name),
                                                name.to_string()
                                                
                                            ),
                                        );
                                    }
                                } else {
                                    log::info!(
                                        "[scan] known device {name} requires confirmation, notifying"
                                    );
                                    log::info!("{:?}", url);
                                    if url.is_some() {
                                        println!("url{:?}", url);
                                        notification::known_device::notify_known_device(
                                            &app,
                                            &row.device,
                                            mount_point,
                                            &row.album_name,
                                        );
                                    }
                                }
                            }
                            Ok(None) => {
                                log::info!("[scan] unknown device {name}, notifying");
                                if url.is_some() {
                                    notification::new_device::notify_new_device(
                                        &app,
                                        name,
                                        mount_point,
                                    );
                                }
                            }
                            // Already logged by `check` with query context.
                            Err(_error) => {}
                        },
                        // Already logged by `get_pool` with the db path.
                        Err(_error) => {}
                    }
                }
            }
            old_disks = actual_disks.iter().map(|(name, _)| name.clone()).collect();
        }
    });
}

/// Whether `disk` is a drive the user plugged in. Besides what sysinfo
/// reports as removable (USB on Linux), Linux counts anything the desktop
/// auto-mounted, which covers built-in SD card readers (`mmcblk`).
fn is_external(disk: &Disk) -> bool {
    #[cfg(target_os = "linux")]
    {
        let mount_point = disk.mount_point();
        if mount_point.starts_with("/media") || mount_point.starts_with("/run/media") {
            return true;
        }
    }
    disk.is_removable()
}

/// Looks up the saved device called `name`. On Windows, devices saved
/// before the volume serial was part of the name (bare label) are renamed
/// to `name` the first time a drive with that label shows up.
async fn find_device(pool: &SqlitePool, name: &str) -> Result<Option<DeviceRow>, sqlx::Error> {
    let row = check(pool, name).await?;
    #[cfg(windows)]
    if row.is_none() {
        if let Some((label, _serial)) = name.rsplit_once(" [") {
            if crate::db::index::rename_device(pool, label, name).await? {
                log::info!("[scan] saved device \"{label}\" renamed to {name}");
                return check(pool, name).await;
            }
        }
    }
    Ok(row)
}

/// Name a device is saved under (the `device` column). On Windows the
/// volume label alone isn't unique (every camera card is `CANON`), so it's
/// `"<label> [<volume serial>]"`, the serial as `dir`/`vol` print it.
#[cfg(windows)]
fn device_name(disk: &Disk) -> String {
    let mount_point = disk.mount_point();
    let mut label = disk.name().to_string_lossy().to_string();
    if label.is_empty() {
        label = mount_point.to_string_lossy().trim_end_matches('\\').to_string();
    }

    match volume_serial(mount_point) {
        Some(serial) => format!("{label} [{:04X}-{:04X}]", serial >> 16, serial & 0xFFFF),
        None => {
            log::warn!("[scan] no volume serial for {}, using its label", mount_point.display());
            label
        }
    }
}

/// Serial number of the volume mounted at `root` (e.g. `E:\`).
#[cfg(windows)]
fn volume_serial(root: &std::path::Path) -> Option<u32> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetVolumeInformationW;

    let root: Vec<u16> = root.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut serial = 0u32;
    // SAFETY: `root` is NUL-terminated and outlives the call; the buffers we
    // don't need are passed as null with a size of 0.
    let ok = unsafe {
        GetVolumeInformationW(
            root.as_ptr(),
            std::ptr::null_mut(),
            0,
            &mut serial,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        )
    };
    (ok != 0).then_some(serial)
}

/// Name a device is saved under (the `device` column): its volume label.
#[cfg(not(any(windows, target_os = "linux")))]
fn device_name(disk: &Disk) -> String {
    disk.name().to_string_lossy().to_string()
}

/// Name a device is saved under (the `device` column). On Linux
/// `disk.name()` is the device node (`/dev/sdb1`), which the next drive
/// plugged in reuses, so it's `"<label> [<filesystem UUID>]"` instead.
#[cfg(target_os = "linux")]
fn device_name(disk: &Disk) -> String {
    let node = std::path::Path::new(disk.name());
    let label = disk_link("/dev/disk/by-label", node)
        .map(|label| unescape_udev(&label))
        .or_else(|| {
            disk.mount_point()
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| disk.name().to_string_lossy().to_string());

    match disk_link("/dev/disk/by-uuid", node) {
        Some(uuid) => format!("{label} [{uuid}]"),
        None => {
            log::warn!("[scan] no filesystem UUID for {}, using its label", node.display());
            label
        }
    }
}

/// Name of the symlink in `dir` (one of udev's `/dev/disk/by-*` folders)
/// that points at the device node `node`.
#[cfg(target_os = "linux")]
fn disk_link(dir: &str, node: &std::path::Path) -> Option<String> {
    let node = std::fs::canonicalize(node).ok()?;
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .find(|entry| std::fs::canonicalize(entry.path()).ok().as_deref() == Some(node.as_path()))
        .map(|entry| entry.file_name().to_string_lossy().to_string())
}

/// Decodes the `\xHH` escapes udev uses in `/dev/disk/by-label` names
/// (`My\x20Photos` -> `My Photos`).
#[cfg(target_os = "linux")]
fn unescape_udev(name: &str) -> String {
    let mut bytes = Vec::with_capacity(name.len());
    let mut rest = name.as_bytes();
    while let Some((&byte, tail)) = rest.split_first() {
        if byte == b'\\' &&tail.first() == Some(&b'x') {
            if let Some(value) = tail
                .get(1..3)
                .and_then(|hex| std::str::from_utf8(hex).ok())
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
            {
                bytes.push(value);
                rest = &tail[3..];
                continue;
            }
        }
        bytes.push(byte);
        rest = tail;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}
