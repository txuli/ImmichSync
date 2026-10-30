use tauri::AppHandle;
use tauri_plugin_log::log;
use super::{actions, ask, Prompt};

pub fn notify_new_device(app: &AppHandle, disk_name: &str, mount_point: &std::path::Path) {
    log::info!("[notification] showing new-device notification for {disk_name}");
    let app_handle = app.clone();
    let disk = disk_name.to_string();
    let mount_point = mount_point.to_path_buf();

    ask(
        app,
        Prompt {
            title: "New device detected",
            body: &format!("Connected: {disk_name}"),
            buttons: &[
                ("new", "Choose album"),
                ("device", "Quick upload"),
                ("ignore", "Ignore"),
            ],
            fallback_body: &format!("Connected: {disk_name} — click to choose an album"),
            fallback_action: "new",
        },
        move |action| match action {
            "device" => actions::start_sync(&app_handle, &disk, &mount_point, None),
            "new" | "default" => actions::open_new_device(&app_handle, &disk, &mount_point),
            "ignore" => log::info!("[notification] ignore pressed for {disk}"),
            _ => {}
        },
    );
}
