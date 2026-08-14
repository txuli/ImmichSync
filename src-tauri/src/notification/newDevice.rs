use std::path::PathBuf;
use tauri::{AppHandle, Manager};

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

    eprintln!("[notification] No se encontró el icono de la app en ninguna ruta esperada");
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
#[cfg(windows)]
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
        eprintln!("[notification] No se pudo registrar el AUMID de la app: {err:?}");
    }
}

#[cfg(windows)]
pub fn notify_new_device<R: tauri::Runtime>(app: &AppHandle<R>, disk_name: &str) {
    use tauri_winrt_notification::{Duration, Toast};

    let disk_name = disk_name.to_string();
    let icon_path = app_icon_path(app);
    register_aumid(icon_path.as_deref());

    let result = Toast::new(APP_ID)
        .title("Nuevo dispositivo detectado")
        .text1(&format!("Se ha conectado: {disk_name}"))
        .add_button("Sincronizar", "sync")
        .add_button("Ignorar", "ignore")
        .duration(Duration::Short)
        .on_activated(move |action| {
            match action.as_deref() {
                Some("sync") => println!("[notification] Sincronizar pulsado para {disk_name}"),
                Some("ignore") => println!("[notification] Ignorar pulsado para {disk_name}"),
                _ => {}
            }
            Ok(())
        })
        .show();

    if let Err(err) = result {
        eprintln!("[notification] Error mostrando la notificación: {err:?}");
    }
}

#[cfg(not(windows))]
pub fn notify_new_device<R: tauri::Runtime>(app: &AppHandle<R>, disk_name: &str) {
    use tauri_plugin_notification::NotificationExt;

    let icon_path = app_icon_path(app);

    let mut builder = app
        .notification()
        .builder()
        .title("Nuevo dispositivo detectado")
        .body(format!("Se ha conectado: {disk_name}"));

    if let Some(icon_path) = &icon_path {
        if let Some(icon_path) = icon_path.to_str() {
            builder = builder.icon(icon_path);
        }
    }

    if let Err(err) = builder.show() {
        eprintln!("[notification] Error mostrando la notificación: {err:?}");
    }
}
