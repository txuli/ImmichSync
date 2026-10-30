import ImmichForm from "../components/ImmichForm"
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import PageContainer from "../components/PageContainer";

interface KnownDeviceProps {
    device: string;
    mountPoint: string;
    albumName: string;
    onDone: () => void;
}

function DriveIcon({ className }: { className?: string }) {

    return (
        <svg
            className={className}
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth={1.5}
            strokeLinecap="round"
            strokeLinejoin="round"
        >
            <rect x="3" y="8" width="18" height="11" rx="2" />
            <path d="M7 8V6a2 2 0 0 1 2-2h6a2 2 0 0 1 2 2v2" />
            <circle cx="9" cy="13.5" r="1" fill="currentColor" stroke="none" />
            <path d="M13 13.5h4" />
        </svg>
    );
}

export default function KnownDevice({ device, mountPoint, albumName, onDone }: KnownDeviceProps) {
    const [starting, setStarting] = useState(false);
    const [error, setError] = useState("");

    async function handleSync() {
        setStarting(true);
        try {
            // The sync runs in the background; progress shows up on the dashboard.
            await invoke("sync_device", {
                diskName: device,
                mountPoint,
                album: albumName,
            });
            onDone();
        } catch (error) {
            setError(String(error));
        } finally {
            setStarting(false);
        }
    }

    return (
        <PageContainer>
            <div className="px-6 pb-6">
                <h2 className="text-2xl font-semibold">Device reconnected</h2>
                <p className="text-gray-400 mt-2">
                    Upload the new photos on this device to its saved album?
                </p>
            </div>
            <div className="border-t-2 w-full border-t-[#272A31] absolute"></div>
            <div className="mt-5 px-4 sm:px-6">
                <ImmichForm>
                    <div className="flex items-center gap-3 my-4">
                        <div className="w-10 h-10 rounded-md bg-[#5B8DEF]/10 flex items-center justify-center shrink-0">
                            <DriveIcon className="w-5 h-5 text-[#5B8DEF]" />
                        </div>
                        <div className="min-w-0">
                            <p className="text-xs text-gray-400">Device</p>
                            <p className="text-sm font-medium text-gray-100 truncate">
                                {device || "Unknown device"}
                            </p>
                            <p className="text-xs text-gray-500 truncate">{mountPoint}</p>
                        </div>
                    </div>

                    <div className="my-4 min-w-0">
                        <p className="text-xs text-gray-400">Destination album</p>
                        <p className="text-sm font-medium text-gray-100 truncate">{albumName}</p>
                    </div>

                    <div className="mt-5 mb-4 flex justify-end gap-2">
                        <button
                            type="button"
                            onClick={onDone}
                            disabled={starting}
                            className="border border-[#272A31] text-gray-300 rounded-md px-4 py-1.5 text-sm disabled:opacity-50"
                        >
                            Ignore
                        </button>
                        <button
                            type="button"
                            onClick={handleSync}
                            disabled={starting}
                            className="bg-[#5B8DEF] text-white rounded-md px-4 py-1.5 text-sm disabled:opacity-50"
                        >
                            {starting ? "Starting…" : "Sync now"}
                        </button>
                    </div>
                    {error ? <p className="text-red-600/65 text-sm mb-4">{error}</p> : <></>}
                </ImmichForm>
            </div>
        </PageContainer>
    );
}
