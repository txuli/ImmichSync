use crate::db::index::{check, get_pool};
use crate::notification;
use std::thread;
use std::time::Duration;
use sysinfo::Disks;
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
                    log::info!(
                        "[scan] device connected: {name} at {}",
                        mount_point.display()
                    );
                    let url = store.get("url");
                    match tauri::async_runtime::block_on(get_pool(&app)) {
                        Ok(pool) => match tauri::async_runtime::block_on(check(&pool, name)) {
                            Ok(Some(row)) => {
                                if row.direct == "true" {
                                    log::info!(
                                        "[scan] known device {name} set to direct sync, path={} album={}",
                                        row.path, row.album_name
                                    );
                                    log::info!("url {:?}", url);
                                    if url.is_some() {
                                        let _sync_result = tauri::async_runtime::block_on(
                                            crate::sync::sync_assets(
                                                app.clone(),
                                                row.path,
                                                Some(row.album_name),
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
                                            std::path::Path::new(&row.path),
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
