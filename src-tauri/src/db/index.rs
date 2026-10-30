use sqlx::sqlite::SqlitePool;
use tauri::AppHandle;
use tauri::Manager;
use tauri_plugin_log::log;

#[derive(sqlx::FromRow, Debug)]
pub struct DeviceRow {
    pub id: i64,
    pub device: String,
    pub path: String,
    #[sqlx(rename = "albumName")]
    pub album_name: String,
    /// `"true"` / `"false"`: the frontend binds a JS boolean through
    /// tauri-plugin-sql, which stores it as JSON text rather than 0/1.
    pub direct: String
}

pub async fn get_pool(app: &AppHandle) -> Result<SqlitePool, sqlx::Error> {
    let app_data_dir = app.path().app_data_dir().map_err(|err| {
        log::error!("[db] could not resolve app data dir: {err}");
        sqlx::Error::Configuration(err.to_string().into())
    })?;
    let db_path = app_data_dir.join("immichsync.db");
    SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
        .await
        .map_err(|err| {
            log::error!("[db] failed to open {}: {err}", db_path.display());
            err
        })
}

pub async fn check(pool: &SqlitePool, device: &str) -> Result<Option<DeviceRow>, sqlx::Error> {
    sqlx::query_as::<_, DeviceRow>("SELECT * FROM devices WHERE device = ?")
        .bind(device)
        .fetch_optional(pool)
        .await
        .map_err(|err| {
            log::error!("[db] failed to query device={device}: {err}");
            err
        })
}

/// Renames the device saved as `old` to `new`. Returns whether one was found.
#[cfg(windows)]
pub async fn rename_device(pool: &SqlitePool, old: &str, new: &str) -> Result<bool, sqlx::Error> {
    sqlx::query("UPDATE devices SET device = ? WHERE id = (SELECT id FROM devices WHERE device = ? LIMIT 1)")
        .bind(new)
        .bind(old)
        .execute(pool)
        .await
        .map(|result| result.rows_affected() > 0)
        .map_err(|err| {
            log::error!("[db] failed to rename device {old} to {new}: {err}");
            err
        })
}
