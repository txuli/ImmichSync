use serde_json::json;
use tauri_plugin_log::log;
pub mod db;
pub mod models;
mod notification;
pub use models::CheckToken;
pub use models::Settings;
pub use models::ValidResponse;
pub mod sync;
pub mod scan;
use scan::scan;
pub use sync::sync_assets;
use tauri::AppHandle;
use tauri::Manager;
use tauri::WindowEvent;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_sql::{Migration, MigrationKind};
use tauri_plugin_store::StoreExt;
/// Passed by the autostart plugin when the OS launches the app at login,
/// so we know to keep the window hidden instead of showing it.
const HIDDEN_ARG: &str = "--hidden";

#[tauri::command]
async fn verify_token(url: &str, token: &str) -> Result<ValidResponse, String> {
    log::info!("[auth] verifying token against {url}");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/auth/validateToken", url))
        .header("x-api-key", token)
        .send()
        .await
        .map_err(|err| {
            log::error!("[auth] network error verifying token against {url}: {err}");
            format!("Network error: {}", err)
        })?;

    if response.status().is_success() {
        log::info!("[auth] token verified successfully for {url}");
        Ok(ValidResponse {
            valid: true,
            type_acc: "credential".to_string(),
            warning: None,
        })
    } else {
        log::warn!(
            "[auth] token verification rejected by {url}: status={}",
            response.status()
        );
        Ok(ValidResponse {
            valid: false,
            type_acc: "credential".to_string(),
            warning: None,
        })
    }
}
#[tauri::command]
async fn save_credentials(app: AppHandle, url: &str, token: &str) -> Result<ValidResponse, String> {
    let store = app.store("settings.json").map_err(|e| {
        log::error!("[settings] failed to open settings store: {e}");
        e.to_string()
    })?;
    let try_save = || -> Result<(), std::io::Error> {
        store.set("url", json!(url));
        store.set("token", json!(token));
        /* store.save(); */
        Ok(())
    };

    match try_save() {
        Ok(_) => {
            log::info!("[settings] credentials saved for {url}");
            Ok(ValidResponse {
                valid: true,
                type_acc: "save".to_string(),
                warning: None,
            })
        }
        Err(err) => {
            log::error!("[settings] failed to save credentials for {url}: {err}");
            Ok(ValidResponse {
                valid: false,
                type_acc: "save".to_string(),
                warning: None,
            })
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: "CREATE TABLE devices (id INTEGER PRIMARY KEY, device TEXT,path TEXT, albumName TEXT, direct BOOLEAN);
              CREATE TABLE stats (id INTEGER PRIMARY KEY, uploadedPhotos NUMBER, uploadedSize NUMBER);
              CREATE TABLE activity (id INTEGER PRIMARY KEY, device TEXT, uploadedPhotos NUMBER, lastSync TIMESTAMP DEFAULT CURRENT_TIMESTAMP);",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "add_activity_status_and_error",
            sql: "ALTER TABLE activity ADD COLUMN status TEXT NOT NULL DEFAULT 'success';
              ALTER TABLE activity ADD COLUMN error TEXT;",
            kind: MigrationKind::Up,
        },
    ];

    
    let database_url = "sqlite:immichsync.db";

    let result = tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::new()
                .add_migrations(database_url, migrations)
                .build(),
        )
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .arg(HIDDEN_ARG)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("immichsync".to_string()),
                    },
                ))
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            verify_token,
            save_credentials,
            sync_assets
        ])
        .setup(|app| {
            log::info!("[startup] ImmichSync starting up");
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        log::info!("[tray] quit requested");
                        app.exit(0);
                    }
                    _ => {
                        log::error!("[tray] menu item {:?} not handled", event.id);
                    }
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;
            log::info!("[startup] tray icon initialized");

            // The window starts hidden (see tauri.conf.json). Only reveal it
            // unless we were launched by the autostart plugin with --hidden.
            let launched_hidden = std::env::args().any(|arg| arg == HIDDEN_ARG);
            if let Some(window) = app.get_webview_window("main") {
                if !launched_hidden {
                    window.show()?;
                    log::info!("[startup] main window shown");
                } else {
                    log::info!("[startup] launched hidden via autostart");
                }

                let window_handle = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_handle.hide();
                    }
                });
            }

            let handle = app.handle().clone();
            scan(handle);

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                window.hide().unwrap();
            }
            _ => {}
        })
        .run(tauri::generate_context!());

    if let Err(err) = result {
        log::error!("[startup] fatal error while running tauri application: {err:?}");
        panic!("error while running tauri application: {err:?}");
    }
}
