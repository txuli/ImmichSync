import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import Database from "@tauri-apps/plugin-sql";
import type { SyncStatus, SyncStatusEvent } from "../types";

interface SyncState {
    isConnected: boolean;
    current: SyncStatusEvent | null;
    history: SyncStatusEvent[];
    init: () => void;
}

interface ActivityRow {
    device: string;
    uploadedPhotos: number | null;
    lastSync: string;
    status: string;
    error: string | null;
}

/** Loads the most recent finished syncs from the `activity` table (survives app restarts). */
async function loadPersistedHistory(): Promise<SyncStatusEvent[]> {
    try {
        const db = await Database.load("sqlite:immichsync.db");
        const rows = await db.select<ActivityRow[]>(
            "SELECT device, uploadedPhotos, lastSync, status, error FROM activity ORDER BY lastSync DESC LIMIT 8"
        );
        return rows.map((row) => ({
            status: (row.status as SyncStatus) ?? "success",
            disk_name: row.device,
            error: row.error,
            timestamp: row.lastSync,
            uploaded_photos: row.uploadedPhotos ?? 0,
            uploaded_size: 0,
        }));
    } catch (err) {
        console.error("Failed to load sync history:", err);
        return [];
    }
}

/**
 * Persists every finished sync (success or error) as a row in `activity`,
 * and folds successful uploads into the running totals in `stats` (a
 * single row, id = 1) so they survive restarts.
 */
async function persistSyncEvent(event: SyncStatusEvent) {
    try {
        const db = await Database.load("sqlite:immichsync.db");

        await db.execute(
            "INSERT INTO activity (device, uploadedPhotos, status, error) VALUES (?, ?, ?, ?)",
            [event.disk_name, event.uploaded_photos, event.status, event.error ?? null]
        );

        if (event.status === "success" && event.uploaded_photos > 0) {
            const existing = await db.select<{ id: number }[]>(
                "SELECT id FROM stats WHERE id = 1"
            );
            if (existing.length === 0) {
                await db.execute(
                    "INSERT INTO stats (id, uploadedPhotos, uploadedSize) VALUES (1, ?, ?)",
                    [event.uploaded_photos, event.uploaded_size]
                );
            } else {
                await db.execute(
                    "UPDATE stats SET uploadedPhotos = uploadedPhotos + ?, uploadedSize = uploadedSize + ? WHERE id = 1",
                    [event.uploaded_photos, event.uploaded_size]
                );
            }
        }
    } catch (err) {
        console.error("Failed to persist sync activity:", err);
    }
}

// The store lives for the lifetime of the app, so this only needs to guard
// against subscribing more than once — every component that calls init()
// (NavBar, Dashboard, ...) shares the same subscription and state.
let listening = false;

export const useSyncStore = create<SyncState>((set) => ({
    isConnected: false,
    current: null,
    history: [],
    init: () => {
        if (listening) return;
        listening = true;

        loadPersistedHistory().then((history) => {
            // Don't clobber history if a sync already came in while this was loading.
            set((state) => ({ history: state.history.length ? state.history : history }));
        });

        listen<SyncStatusEvent>("sync-status", (event) => {
            const payload = event.payload;
            set((state) => ({
                current: payload,
                isConnected: payload.status === "syncing",
                history:
                    payload.status === "syncing"
                        ? state.history
                        : [payload, ...state.history].slice(0, 8),
            }));

            if (payload.status !== "syncing") {
                persistSyncEvent(payload);
            }
        });
    },
}));
