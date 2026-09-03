import { useState, type SubmitEvent } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import ImmichForm from "../components/ImmichForm";

type UploadStatus = "idle" | "uploading" | "success" | "error";

function FolderIcon() {
    return (
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none">
            <path
                d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z"
                stroke="currentColor"
                strokeWidth="1.8"
                strokeLinecap="round"
                strokeLinejoin="round"
            />
        </svg>
    );
}

function InfoIcon() {
    return (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" className="shrink-0 mt-0.5">
            <circle cx="12" cy="12" r="9" stroke="#5B8DEF" strokeWidth="1.8" />
            <path d="M12 11v5.5M12 8v.01" stroke="#5B8DEF" strokeWidth="1.8" strokeLinecap="round" />
        </svg>
    );
}

function SpinnerIcon() {
    return (
        <svg width="17" height="17" viewBox="0 0 24 24" fill="none" className="animate-spin">
            <circle cx="12" cy="12" r="9" stroke="#5B8DEF" strokeOpacity="0.25" strokeWidth="2.5" />
            <path d="M21 12a9 9 0 0 0-9-9" stroke="#5B8DEF" strokeWidth="2.5" strokeLinecap="round" />
        </svg>
    );
}

function CheckIcon() {
    return (
        <svg width="17" height="17" viewBox="0 0 24 24" fill="none" className="shrink-0 mt-0.5">
            <circle cx="12" cy="12" r="9" stroke="#22C55E" strokeWidth="1.8" />
            <path d="M8.5 12.5l2.5 2.5 5-5" stroke="#22C55E" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
    );
}

function ErrorIcon() {
    return (
        <svg width="17" height="17" viewBox="0 0 24 24" fill="none" className="shrink-0 mt-0.5">
            <circle cx="12" cy="12" r="9" stroke="#EF4444" strokeWidth="1.8" />
            <path d="M12 7.5v6M12 16.5v.01" stroke="#EF4444" strokeWidth="1.8" strokeLinecap="round" />
        </svg>
    );
}

export default function manualUpload() {
    const [folderPath, setFolderPath] = useState("");
    const [albumName, setAlbumName] = useState("");
    const [status, setStatus] = useState<UploadStatus>("idle");
    const [error, setError] = useState<string | null>(null);
    const [uploadedAlbum, setUploadedAlbum] = useState("");

    async function selectFolder() {
        const selected = await open({
            directory: true,
            multiple: false,
        });

        if (typeof selected === "string") {
            setFolderPath(selected);
            setStatus("idle");
            setError(null);
        }
    }

    async function handleUpload(e: SubmitEvent<HTMLFormElement>) {
        e.preventDefault();
        if (!folderPath || status === "uploading") return;

        setStatus("uploading");
        setError(null);
        try {
            await invoke("sync_assets", {
                path: folderPath,
                album: albumName.trim() || undefined,
            });
            setUploadedAlbum(albumName.trim());
            setStatus("success");
        } catch (err) {
            setError(String(err));
            setStatus("error");
        }
    }

    return (
        <div className="relative h-full bg-[#15171C]">
            <div className="p-6">
                <h2 className="text-2xl font-semibold">Manual upload</h2>
                <p className="text-gray-400 mt-2">
                    Manually upload the photos from a folder to Immich when a device isn't detected automatically.
                </p>
            </div>
            <div className="border-t-2 w-full border-t-[#272A31] absolute"></div>
            <div className="mt-5 px-4 sm:px-6">
                <ImmichForm>
                    <form className="my-4 min-w-0" onSubmit={handleUpload}>
                        <label htmlFor="folderPath" className="text-sm font-medium text-gray-300">
                            Folder to upload
                        </label>
                        <div className="flex gap-2 mt-2">
                            <input
                                type="text"
                                name="folderPath"
                                id="folderPath"
                                value={folderPath}
                                readOnly
                                placeholder="No folder selected"
                                className="flex-1 min-w-0"
                            />
                            <button
                                type="button"
                                onClick={selectFolder}
                                className="shrink-0 flex items-center gap-1.5 rounded-md px-3 py-1"
                            >
                                <FolderIcon />
                                Select folder
                            </button>
                        </div>

                        <label htmlFor="albumName" className="block text-sm font-medium text-gray-300 mt-5">
                            Album name <span className="text-gray-500 font-normal">(optional)</span>
                        </label>
                        <input
                            type="text"
                            id="albumName"
                            name="albumName"
                            value={albumName}
                            onChange={(e) => setAlbumName(e.target.value)}
                            placeholder="ImmichSync"
                            className="w-full min-w-0 mt-2"
                        />

                        <div className="mt-3.5 flex gap-2 bg-[#5B8DEF]/6 border border-[#5B8DEF]/20 rounded-md px-3 py-2.5">
                            <InfoIcon />
                            <p className="text-xs text-gray-400 leading-relaxed">
                                If you leave this blank, a name will be generated an immichSync album automatically.
                            </p>
                        </div>

                        <div className="mt-5 flex flex-wrap items-center gap-3">
                            <button
                                type="submit"
                                disabled={!folderPath || status === "uploading"}
                                className="bg-[#5B8DEF] text-white rounded-md px-5 py-1 disabled:opacity-40 disabled:cursor-not-allowed"
                            >
                                {status === "uploading" ? "Uploading…" : "Upload"}
                            </button>
                            {!folderPath && (
                                <span className="text-xs text-gray-500">Select a folder to continue</span>
                            )}
                        </div>

                        {status === "uploading" && (
                            <div className="mt-4 flex items-center gap-2.5 rounded-md border border-[#272A31] bg-[#181B21] px-3.5 py-3">
                                <SpinnerIcon />
                                <div>
                                    <p className="text-[13px] text-gray-200 font-medium">Uploading to Immich…</p>
                                    <p className="text-xs text-gray-500 mt-0.5">Don't close the app</p>
                                </div>
                            </div>
                        )}

                        {status === "success" && (
                            <div className="mt-4 flex items-start gap-2.5 rounded-md border border-green-500/25 bg-green-500/8 px-3.5 py-3">
                                <CheckIcon />
                                <div>
                                    <p className="text-[13px] text-green-400 font-medium">
                                        Upload completed successfully
                                        {uploadedAlbum ? ` to "${uploadedAlbum}"` : ""}.
                                    </p>
                                </div>
                            </div>
                        )}

                        {status === "error" && (
                            <div className="mt-4 flex items-start gap-2.5 rounded-md border border-red-500/25 bg-red-500/8 px-3.5 py-3">
                                <ErrorIcon />
                                <div>
                                    <p className="text-[13px] text-red-400 font-medium">Failed to upload the files</p>
                                    {error && (
                                        <p className="text-xs text-gray-500 mt-0.5 wrap-break-word">{error}</p>
                                    )}
                                </div>
                            </div>
                        )}
                    </form>
                </ImmichForm>
            </div>
        </div>
    );
}
