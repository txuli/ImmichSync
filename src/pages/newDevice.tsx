import ImmichForm from "../components/ImmichForm"
import Options from "../components/options";
import { useState, type SyntheticEvent } from "react";
import Database from '@tauri-apps/plugin-sql';
import { invoke } from "@tauri-apps/api/core";
import type { ValidResponse } from "../types";
interface NewDeviceProps {
    device: string;
    mountPoint: string;
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

export default function NewDevice({ device, mountPoint, onDone }: NewDeviceProps) {
    const [direct, setDirect] = useState(false);
    const [albumName, setAlbumName] = useState("");
    const [saving, setSaving] = useState(false);
    const [error, setError]= useState("")
    function toggle(value: boolean) {
        setDirect(value);
    }

    async function handleSave(e: SyntheticEvent<HTMLFormElement>) {
        e.preventDefault();
        setSaving(true);
        try {
            const db = await Database.load('sqlite:immichsync.db')
            await db.execute(
                'INSERT into devices (device, albumName, direct,path) VALUES (?, ?, ?,?)',
                [device, albumName, direct, mountPoint]

            );
            try {
                await invoke<ValidResponse>("sync_assets", {
                    path: mountPoint,
                    album: albumName,
                })
                onDone();
            } catch(error){
                setError(String(error))
            }
            
           
        } finally {
            setSaving(false);
        }
    }

    return (
        <div className="relative h-full bg-[#15171C]">
            <div className="p-6">
                <h2 className="text-2xl font-semibold">New device detected</h2>
                <p className="text-gray-400 mt-2">
                    Choose how ImmichSync should handle this device from now on.
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
                        </div>
                    </div>

                    <form className="grid space-y-2 my-4 min-w-0" onSubmit={handleSave}>
                        <label htmlFor="albumName">Destination album</label>
                        <input
                            id="albumName"
                            type="text"
                            placeholder="e.g. Family USB backups"
                            className="w-full min-w-0"
                            value={albumName}
                            onChange={(e) => setAlbumName(e.target.value)}
                        />

                        <div className="pt-2">
                            <Options
                                checked={direct}
                                title="Upload assets without asking"
                                description="Upload the assets directly to your server whenever this device connects"
                                onChange={toggle}
                            />
                        </div>

                        <div className="mt-5 flex justify-end">
                            <button
                                type="submit"
                                disabled={saving || !albumName.trim()}
                                className="bg-[#5B8DEF] text-white rounded-md px-4 py-1.5 text-sm disabled:opacity-50"
                            >
                                {saving ? "Saving…" : "Save and upload"}
                            </button>
                        </div>
                    </form>
                    {error? <p className="text-red-600/65 text-sm">No credentials found. Please set them up</p>:<></>}

                </ImmichForm>
            </div>
        </div>
    );
}
