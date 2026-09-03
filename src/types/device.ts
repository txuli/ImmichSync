/**
 * A row of the `devices` SQLite table (see the migration in src-tauri/src/lib.rs).
 */
export interface DbDevice {
    id: number;
    device: string;
    path: string;
    albumName: string;
    direct: boolean;
}
