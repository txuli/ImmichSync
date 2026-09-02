use crate::db::index::{check, get_pool};
use crate::notification;
use std::thread;
use std::time::Duration;
use sysinfo::Disks;
use tauri::AppHandle;
#[derive(Debug)]

pub enum SyncError {
    StoreError(String),
    ShellError(String),
}
#[tauri::command]
pub fn scan(app: AppHandle) {
    thread::spawn(move || {
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
                    if let Ok(pool) = tauri::async_runtime::block_on(get_pool(&app)) {
                        match tauri::async_runtime::block_on(check(&pool, name)) {
                            Ok(Some(row)) => {
                                if row.direct == "true" {
                                    let sync_result =
                                        tauri::async_runtime::block_on(crate::sync::sync_assets(
                                            app.clone(),
                                            row.path,
                                            Some(row.album_name),
                                        ));
                                } else {
                                    notification::known_device::notify_known_device(
                                        &app,
                                        &row.device,
                                        std::path::Path::new(&row.path),
                                        &row.album_name,
                                    );
                                }
                            }
                            Ok(None) => {
                                notification::new_device::notify_new_device(
                                    &app,
                                    name,
                                    mount_point,
                                );
                            }
                            Err(error) => {
                                eprintln!("Failed to check device {name}: {error:?}");
                            }
                        }
                    }
                }
            }
            old_disks = actual_disks.iter().map(|(name, _)| name.clone()).collect();
        }
    });
}
