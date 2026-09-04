import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { load } from "@tauri-apps/plugin-store";
import { isEnabled as isAutostartEnabled } from "@tauri-apps/plugin-autostart";
import { useSyncStore } from "../store/syncStore";
import type { ValidResponse, StoredFlag } from "../types";

const store = await load("settings.json", { autoSave: true });

type ConnectionState = "checking" | "connected" | "disconnected" | "unset";

function formatTime(iso: string) {
    try {
        return new Date(iso).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    } catch {
        return iso;
    }
}

function StatusDot({ on }: { on: boolean }) {
    return (
        <span
            className={`inline-block w-2 h-2 rounded-full ${on ? "bg-[#4ADE80]" : "bg-gray-600"}`}
        />
    );
}

export default function dashboard() {
    const current = useSyncStore((s) => s.current);
    const history = useSyncStore((s) => s.history);
    const [connection, setConnection] = useState<ConnectionState>("checking");
    const [serverUrl, setServerUrl] = useState<string | null>(null);
    const [autostart, setAutostart] = useState(false);
    const [notifications, setNotifications] = useState(false);
    const [removeAssets, setRemoveAssets] = useState(false);

    const checkConnection = useCallback(async () => {
        const url = await store.get<string>("url");
        const token = await store.get<string>("token");
        if (!url || !token) {
            setConnection("unset");
            setServerUrl(null);
            return;
        }
        setServerUrl(url);
        setConnection("checking");
        try {
            const res = await invoke<ValidResponse>("verify_token", { url, token });
            setConnection(res.valid ? "connected" : "disconnected");
        } catch {
            setConnection("disconnected");
        }
    }, []);

    useEffect(() => {
        checkConnection();

        (async () => {
            setAutostart(await isAutostartEnabled());
            const notifData = await store.get<StoredFlag>("notif");
            setNotifications(notifData?.value ?? false);
            const rmAssets = await store.get<StoredFlag>("rmAssets");
            setRemoveAssets(rmAssets?.value ?? false);
        })();
    }, [checkConnection]);

    // current comes from the shared sync store (see App.tsx / syncStore.ts) —
    // recheck the server connection whenever a sync just finished successfully.
    useEffect(() => {
        if (current?.status === "success") {
            checkConnection();
        }
    }, [current, checkConnection]);

    const connectionBadge = {
        checking: { label: "Checking…", color: "text-gray-400", dot: false },
        connected: { label: "Connected", color: "text-green-500", dot: true },
        disconnected: { label: "Disconnected", color: "text-red-500", dot: false },
        unset: { label: "Not configured", color: "text-gray-400", dot: false },
    }[connection];

    return (
        <div className="p-6 bg-[#15171C] h-full">
            <h2 className="text-2xl font-semibold">Dashboard</h2>
            <p className="text-gray-400 mt-2">Overview of your sync activity.</p>

            <div className="grid gap-4 mt-6 sm:grid-cols-2 max-w-4xl">
                <div className="bg-[#20232B] rounded-md border border-[#272A31] p-4">
                    <div className="flex items-center justify-between">
                        <h3 className="text-sm font-medium text-gray-200">Server connection</h3>
                       {/*  <button
                            onClick={checkConnection}
                            className="text-xs text-gray-400 hover:text-white transition-colors"
                        >
                            Recheck
                        </button> */}
                    </div>
                    <div className={`flex items-center gap-2 mt-3 text-sm ${connectionBadge.color}`}>
                        <StatusDot on={connectionBadge.dot} />
                        {connectionBadge.label}
                    </div>
                    <p className="text-xs text-gray-500 mt-1 truncate">
                        {serverUrl ?? "No Immich server configured yet"}
                    </p>
                </div>

                <div className="bg-[#20232B] rounded-md border border-[#272A31] p-4">
                    <h3 className="text-sm font-medium text-gray-200">Current sync</h3>
                    {!current && (
                        <p className="text-sm text-gray-500 mt-3">
                            Idle — connect a device to start syncing.
                        </p>
                    )}
                    {current?.status === "syncing" && (
                        <div className="flex items-center gap-2 mt-3 text-sm text-[#5B8DEF]">
                            <StatusDot on />
                            Syncing {current.disk_name}…
                        </div>
                    )}
                    {current?.status === "success" && (
                        <div className={`mt-3 text-sm ${current.error ? "text-amber-500" : "text-green-500"}`}>
                            {current.disk_name} synced {current.error ? "with some errors" : "successfully"}
                            <p className="text-xs text-gray-500 mt-1">
                                at {formatTime(current.timestamp)}
                            </p>
                            {current.error && (
                                <p className="text-xs text-gray-500 mt-1 wrap-break-words">
                                    {current.error}
                                </p>
                            )}
                        </div>
                    )}
                    {current?.status === "error" && (
                        <div className="mt-3 text-sm text-red-500">
                            Failed to sync {current.disk_name}
                            <p className="text-xs text-gray-500 mt-1 wrap-break-words">
                                {current.error}
                            </p>
                        </div>
                    )}
                </div>
            </div>

            <div className="grid gap-4 mt-4 sm:grid-cols-3 max-w-4xl">
                <div className="bg-[#20232B] rounded-md border border-[#272A31] p-4 flex items-center justify-between">
                    <span className="text-sm text-gray-300">Run on startup</span>
                    <StatusDot on={autostart} />
                </div>
                <div className="bg-[#20232B] rounded-md border border-[#272A31] p-4 flex items-center justify-between">
                    <span className="text-sm text-gray-300">Notifications</span>
                    <StatusDot on={notifications} />
                </div>
                <div className="bg-[#20232B] rounded-md border border-[#272A31] p-4 flex items-center justify-between">
                    <span className="text-sm text-gray-300">Remove after upload</span>
                    <StatusDot on={removeAssets} />
                </div>
            </div>

            <div className="mt-6 max-w-4xl">
                <h3 className="text-sm font-medium text-gray-200 mb-2">Recent activity</h3>
                <div className="bg-[#20232B] rounded-md border border-[#272A31] divide-y divide-[#272A31]">
                    {history.length === 0 && (
                        <p className="text-sm text-gray-500 p-4">Nothing synced yet.</p>
                    )}
                    {history.map((entry, i) => (
                        <div key={i} className="flex items-center justify-between p-3 text-sm">
                            <div className="flex items-center gap-2 min-w-0">
                                <StatusDot on={entry.status === "success"} />
                                <span className="text-gray-200 truncate">{entry.disk_name}</span>
                            </div>
                            <div className="flex items-center gap-3 shrink-0">
                                {entry.status === "error" && (
                                    <span className="text-red-500 text-xs">failed</span>
                                )}
                                {entry.status === "success" && entry.error && (
                                    <span className="text-amber-500 text-xs">with errors</span>
                                )}
                                {entry.status === "success" && entry.uploaded_photos > 0 && (
                                    <span className="text-gray-500 text-xs">
                                        {entry.uploaded_photos} photo{entry.uploaded_photos === 1 ? "" : "s"}
                                    </span>
                                )}
                                <span className="text-gray-500 text-xs">
                                    {formatTime(entry.timestamp)}
                                </span>
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
}
