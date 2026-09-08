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
    pub direct:String
}

pub async fn get_pool(app: &AppHandle) -> Result<SqlitePool, sqlx::Error> {
    let db_path = app.path().app_data_dir().unwrap().join("immichsync.db");
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
