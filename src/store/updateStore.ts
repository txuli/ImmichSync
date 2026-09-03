import { create } from "zustand";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type UpdateStatus = "idle" | "checking" | "available" | "up-to-date" | "installing" | "error";

interface UpdateState {
    status: UpdateStatus;
    version: string | null;
    error: string | null;
    /** Runs the startup check once; safe to call from multiple components. */
    init: () => void;
    checkForUpdates: () => Promise<void>;
    installUpdate: () => Promise<void>;
}

// The store lives for the lifetime of the app — this only needs to guard
// against running the startup check more than once.
let initialized = false;

export const useUpdateStore = create<UpdateState>((set, get) => ({
    status: "idle",
    version: null,
    error: null,
    init: () => {
        if (initialized) return;
        initialized = true;
        get().checkForUpdates();
    },
    checkForUpdates: async () => {
        set({ status: "checking", error: null });
        try {
            const update = await check();
            if (update) {
                set({ status: "available", version: update.version });
            } else {
                set({ status: "up-to-date" });
            }
        } catch (err) {
            set({ status: "error", error: String(err) });
        }
    },
    installUpdate: async () => {
        set({ status: "installing", error: null });
        try {
            const update = await check();
            if (update) {
                await update.downloadAndInstall();
                await relaunch();
            }
        } catch (err) {
            set({ status: "error", error: String(err) });
        }
    },
}));
