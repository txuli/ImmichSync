/**
 * Shape used for every boolean flag persisted via `@tauri-apps/plugin-store`
 * in settings.json (e.g. "notif", "rmAssets").
 */
export interface StoredFlag {
    value: boolean;
}
