export type SyncStatus = "syncing" | "success" | "error";

/**
 * Payload of the "sync-status" event emitted by the Rust backend
 * (src-tauri/src/models.rs) whenever a sync starts, succeeds, or fails.
 */
export interface SyncStatusEvent {
    status: SyncStatus;
    disk_name: string;
    error?: string | null;
    timestamp: string;
    /** Best-effort count of media files found in the synced folder (0 unless status is "success"). */
    uploaded_photos: number;
    /** Combined size in bytes of those media files. */
    uploaded_size: number;
}
