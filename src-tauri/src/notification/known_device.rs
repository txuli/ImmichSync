use tauri::AppHandle;
use tauri_plugin_log::log;
use super::{actions, ask, Prompt};

pub fn notify_known_device(
    app: &AppHandle,
    disk_name: &str,
    mount_point: &std::path::Path,
    album_name: &str,
) {
    log::info!("[notification] showing known-device notification for {disk_name} (album \"{album_name}\")");
    let app_handle = app.clone();
    let disk = disk_name.to_string();
    let mount_point = mount_point.to_path_buf();
    let album = album_name.to_string();

    ask(
        app,
        Prompt {
            title: "Device reconnected",
            body: &format!("{disk_name} is back — sync to \"{album_name}\"?"),
            buttons: &[("sync", "Sync"), ("ignore", "Ignore")],
            fallback_body: &format!("{disk_name} is back — click to open ImmichSync"),
            fallback_action: "default",
        },
        move |action| match action {
            "sync" => actions::start_sync(&app_handle, &disk, &mount_point, Some(album.clone())),
            "default" => actions::open_known_device(&app_handle, &disk, &mount_point, &album),
            "ignore" => log::info!("[notification] ignore pressed for {disk}"),
            _ => {}
        },
    );
}
