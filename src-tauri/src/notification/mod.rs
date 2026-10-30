pub mod known_device;
pub mod new_device;
pub mod new_sync;

pub mod actions;

use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri_plugin_log::log;

fn app_icon_path<R: tauri::Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    if let Ok(path) = app
        .path()
        .resolve("icons/128x128.png", tauri::path::BaseDirectory::Resource)
    {
        if path.exists() {
            return Some(strip_verbatim_prefix(path));
        }
    }

    #[cfg(debug_assertions)]
    {
        let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("icons/128x128.png");
        if dev_path.exists() {
            return Some(strip_verbatim_prefix(dev_path));
        }
    }

    log::warn!("[notification] app icon not found in any expected path");
    None
}

fn strip_verbatim_prefix(path: PathBuf) -> PathBuf {
    match path.to_str() {
        Some(s) => match s.strip_prefix(r"\\?\") {
            Some(stripped) => PathBuf::from(stripped),
            None => path,
        },
        None => path,
    }
}

#[cfg(windows)]
const APP_ID: &str = "com.txuli.immichsync";
const APP_NAME: &str = "ImmichSync";

#[cfg(windows)]
fn register_aumid(icon_path: Option<&std::path::Path>) {
    use windows_registry::CURRENT_USER;

    let result = (|| -> windows_registry::Result<()> {
        let key = CURRENT_USER.create(format!(r"SOFTWARE\Classes\AppUserModelId\{APP_ID}"))?;
        key.set_string("DisplayName", APP_NAME)?;
        key.set_string("IconBackgroundColor", "0")?;
        if let Some(icon_path) = icon_path {
            key.set_hstring("IconUri", &icon_path.into())?;
        }
        Ok(())
    })();

    if let Err(err) = result {
        log::error!("[notification] failed to register the app's AUMID: {err:?}");
    }
}

/// A notification that asks the user what to do with a device.
// The fallback fields are only read on Linux.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) struct Prompt<'a> {
    pub title: &'a str,
    pub body: &'a str,
    /// Buttons as (action id, label).
    pub buttons: &'a [(&'static str, &'static str)],
    /// Body used where the notification server can't show buttons.
    pub fallback_body: &'a str,
    /// Action reported when that buttonless notification is clicked.
    pub fallback_action: &'static str,
}

/// Plain informational notification, without buttons.
#[cfg(windows)]
fn show(app: &AppHandle, title: &str, body: &str) {
    use tauri_winrt_notification::{Duration, Toast};

    register_aumid(app_icon_path(app).as_deref());
    let result = Toast::new(APP_ID)
        .title(title)
        .text1(body)
        .duration(Duration::Short)
        .show();

    if let Err(err) = result {
        log::error!("[notification] failed to show \"{title}\": {err:?}");
    }
}

#[cfg(target_os = "linux")]
fn show(app: &AppHandle, title: &str, body: &str) {
    let mut notification = notify_rust::Notification::new();
    notification.appname(APP_NAME).summary(title).body(body);
    if let Some(icon_path) = app_icon_path(app) {
        notification.icon(&icon_path.to_string_lossy());
    }

    if let Err(err) = notification.show() {
        log::error!("[notification] failed to show \"{title}\": {err:?}");
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
fn show(app: &AppHandle, title: &str, body: &str) {
    use tauri_plugin_notification::NotificationExt;

    let mut builder = app.notification().builder().title(title).body(body);
    if let Some(icon_path) = app_icon_path(app) {
        builder = builder.icon(icon_path.to_string_lossy());
    }

    if let Err(err) = builder.show() {
        log::error!("[notification] failed to show \"{title}\": {err:?}");
    }
}

/// Shows `prompt` and calls `on_action` with the id of the button the user
/// picks. Ids that aren't one of the prompt's buttons are dropped.
#[cfg(windows)]
fn ask<F>(app: &AppHandle, prompt: Prompt, on_action: F)
where
    F: Fn(&str) + Send + 'static,
{
    use tauri_winrt_notification::{Duration, Toast};

    register_aumid(app_icon_path(app).as_deref());
    let allowed: Vec<&'static str> = prompt.buttons.iter().map(|(id, _)| *id).collect();
    let mut toast = Toast::new(APP_ID)
        .title(prompt.title)
        .text1(prompt.body)
        .duration(Duration::Short);
    for (id, label) in prompt.buttons {
        toast = toast.add_button(label, id);
    }

    let result = toast
        .on_activated(move |action| {
            match action.as_deref() {
                Some(action) if allowed.iter().any(|id| *id == action) => on_action(action),
                _ => {}
            }
            Ok(())
        })
        .show();

    if let Err(err) = result {
        log::error!("[notification] failed to show \"{}\": {err:?}", prompt.title);
    }
}

/// Shows `prompt` and calls `on_action` with the id of the button the user
/// picks. Clicking the notification body reports `"default"`. Servers that
/// can't show buttons (some tiling-WM daemons) get `fallback_body` instead,
/// and a click on it reports `fallback_action`.
#[cfg(target_os = "linux")]
fn ask<F>(app: &AppHandle, prompt: Prompt, on_action: F)
where
    F: Fn(&str) + Send + 'static,
{
    use notify_rust::{Hint, Notification, Timeout};

    let has_buttons = notify_rust::get_capabilities()
        .map(|caps| caps.iter().any(|cap| cap == "actions"))
        .unwrap_or(false);

    let mut notification = Notification::new();
    notification
        .appname(APP_NAME)
        .summary(prompt.title)
        .hint(Hint::Resident(true))
        .timeout(Timeout::Never);
    if let Some(icon_path) = app_icon_path(app) {
        notification.icon(&icon_path.to_string_lossy());
    }

    let mut allowed: Vec<&'static str> = vec!["default"];
    if has_buttons {
        notification.body(prompt.body);
        for (id, label) in prompt.buttons {
            notification.action(id, label);
            allowed.push(id);
        }
    } else {
        log::info!("[notification] server can't show buttons, using the click fallback");
        notification.body(prompt.fallback_body);
    }
    // "default" is the click on the notification body itself.
    notification.action("default", "Open");

    let title = prompt.title.to_string();
    let fallback_action = prompt.fallback_action;
    // `wait_for_action` blocks until the user acts or dismisses it.
    std::thread::spawn(move || match notification.show() {
        Ok(handle) => handle.wait_for_action(|action| match action {
            "default" if !has_buttons => on_action(fallback_action),
            action if allowed.iter().any(|id| *id == action) => on_action(action),
            _ => {}
        }),
        Err(err) => log::error!("[notification] failed to show \"{title}\": {err:?}"),
    });
}

/// Desktops without notification buttons: show the prompt as plain text.
#[cfg(not(any(windows, target_os = "linux")))]
fn ask<F>(app: &AppHandle, prompt: Prompt, _on_action: F)
where
    F: Fn(&str) + Send + 'static,
{
    show(app, prompt.title, prompt.body);
}

pub fn upload_success(app: &AppHandle) {
    show(
        app,
        "Sync complete",
        "Your photos were uploaded to Immich successfully",
    );
}

/// Same as `upload_success`, but for a sync that completed with some files
/// skipped or errored (e.g. a transient IO error on a large batch) — the
/// rest of the batch still uploaded, so this isn't a failure, just a
/// heads-up.
pub fn upload_success_with_warning(app: &AppHandle, warning: &str) {
    show(
        app,
        "Sync complete with some errors",
        &format!("Most photos uploaded, but some failed: {warning}"),
    );
}

pub fn upload_failed(app: &AppHandle, error: &str) {
    show(
        app,
        "Sync failed",
        &format!("Could not upload your photos: {error}"),
    );
}
