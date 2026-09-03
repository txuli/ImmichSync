/**
 * Mirrors the Rust `ValidResponse` struct (src-tauri/src/models.rs), returned
 * by the `verify_token`, `save_credentials` and `sync_assets` commands.
 */
export interface ValidResponse {
    valid: boolean;
    type_acc: string;
}
