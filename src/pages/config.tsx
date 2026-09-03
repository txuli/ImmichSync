
import ImmichForm from "../components/ImmichForm"
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState, type SyntheticEvent } from "react";
import Options from "../components/options";
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart';
import { useUpdateStore } from "../store/updateStore";

import { load } from '@tauri-apps/plugin-store';
import type { ValidResponse, StoredFlag } from '../types';
const store = await load('settings.json', { autoSave: true });

type CredentialState = "checking" | "valid" | "invalid" | "unset";

// e.g. https://immich.txuli.com — requires http(s)://, a host with at least
// one dot, an optional port, and no trailing slash.
const URL_PATTERN = /^https?:\/\/[a-z0-9-]+(\.[a-z0-9-]+)+(:\d+)?$/i;

function CheckCircleIcon({ className }: { className?: string }) {
    return (
        <svg viewBox="0 0 24 24" fill="none" className={className}>
            <circle cx="12" cy="12" r="9" stroke="currentColor" strokeWidth="1.8" />
            <path d="M8.5 12.5l2.5 2.5 5-5" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
    );
}

function WarningCircleIcon({ className }: { className?: string }) {
    return (
        <svg viewBox="0 0 24 24" fill="none" className={className}>
            <circle cx="12" cy="12" r="9" stroke="currentColor" strokeWidth="1.8" />
            <path d="M12 7.5v6M12 16.5v.01" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" />
        </svg>
    );
}

function SpinnerCircleIcon({ className }: { className?: string }) {
    return (
        <svg viewBox="0 0 24 24" fill="none" className={`animate-spin ${className ?? ""}`}>
            <circle cx="12" cy="12" r="9" stroke="currentColor" strokeOpacity="0.25" strokeWidth="2.5" />
            <path d="M21 12a9 9 0 0 0-9-9" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" />
        </svg>
    );
}

export default function config() {

    const [runInBackground, setRunInBackground] = useState(false);
    const [notifications, setNotifications] = useState(false);
    const [removeAssets, setRemoveAssets] = useState(false);

    // Immich credentials: whatever is already saved (shown as a "connected"
    // card with a check), and the editable form (shown when there's nothing
    // saved yet, or the user taps "Edit").
    const [credState, setCredState] = useState<CredentialState>("checking");
    const [savedUrl, setSavedUrl] = useState("");
    const [savedToken, setSavedToken] = useState("");
    const [editing, setEditing] = useState(false);
    const [urlInput, setUrlInput] = useState("");
    const [tokenInput, setTokenInput] = useState("");
    const [response, setResponse] = useState<ValidResponse | null>(null);
    const [saving, setSaving] = useState(false);
    const urlIsValid = urlInput.trim() === "" || URL_PATTERN.test(urlInput.trim());
    const urlIsCorrect = urlInput.trim() !== "" && URL_PATTERN.test(urlInput.trim());

    useEffect(() => {
        async function check() {
            setRunInBackground(await isEnabled())
        }
        async function load() {
            const notifData = await store.get<StoredFlag>('notif');
            setNotifications(notifData?.value ?? false);
            const rmAssets = await store.get<StoredFlag>('rmAssets');
            setRemoveAssets(rmAssets?.value ?? false)
        }
        async function loadCredentials() {
            const url = await store.get<string>('url');
            const token = await store.get<string>('token');
            if (!url || !token) {
                setCredState("unset");
                setEditing(true);
                return;
            }
            setSavedUrl(url);
            setSavedToken(token);
            setUrlInput(url);
            setTokenInput(token);
            try {
                const res = await invoke<ValidResponse>('verify_token', { url, token });
                setCredState(res.valid ? "valid" : "invalid");
            } catch {
                setCredState("invalid");
            }
        }

        load()
        check()
        loadCredentials()
    }, [])

    async function save(type: string, value: boolean) {
        switch (type) {
            case "notif":
                setNotifications(value)
                await store.set('notif', { value })
                break
            case "removeAssets":
                setRemoveAssets(value)
                await store.set('rmAssets', { value })
                break
        }
    }
    async function toggle(value: boolean) {
        setRunInBackground(value);
        if (value) await enable()
        else await disable();
    }

    async function handleSaveCredentials(e: SyntheticEvent<HTMLFormElement>) {
        e.preventDefault();
        if (urlInput.trim() === "" || tokenInput.trim() === "") {
            setResponse({ valid: false, type_acc: "empty" });
            return;
        }
        if (!URL_PATTERN.test(urlInput.trim())) {
            setResponse({ valid: false, type_acc: "url" });
            return;
        }
        setSaving(true);
        try {
            const verify = await invoke<ValidResponse>('verify_token', { url: urlInput, token: tokenInput });
            if (!verify.valid) {
                setResponse({ valid: false, type_acc: "credential" });
                return;
            }

            const res = await invoke<ValidResponse>('save_credentials', { url: urlInput, token: tokenInput });
            setResponse(res);
            if (res.valid) {
                setSavedUrl(urlInput);
                setSavedToken(tokenInput);
                setEditing(false);
                setResponse(null);
                setCredState("valid");
            }
        } catch {
            setResponse({ valid: false, type_acc: "credential" });
        } finally {
            setSaving(false);
        }
    }

    function handleCancelEdit() {
        setUrlInput(savedUrl);
        setTokenInput(savedToken);
        setResponse(null);
        setEditing(false);
    }

    // The startup check already ran once from App.tsx (see store/updateStore.ts);
    // this page just reads and displays whatever it found.
    const updateStatus = useUpdateStore((s) => s.status);
    const updateVersion = useUpdateStore((s) => s.version);
    const updateError = useUpdateStore((s) => s.error);
    const checkForUpdates = useUpdateStore((s) => s.checkForUpdates);
    const installUpdate = useUpdateStore((s) => s.installUpdate);


    return (
        <div className="relative h-full bg-[#15171C]">
            <div className="p-6  ">
                <h2 className="text-2xl font-semibold">Config</h2>
                <p className="text-gray-400 mt-2">
                    Ajustes de la conexión con Immich.
                </p>


            </div>
            <div className="border-t-2 w-full border-t-[#272A31]  absolute"></div>
            <div className="mt-5 px-4 sm:px-6">
                <ImmichForm>
                    <h2 className="text-xl my-4 mb-1">Immich credentials</h2>
                    <p className="text-gray-400 text-sm">Necessary to upload the assets of the USB</p>

                    {!editing ? (
                        <div className="flex items-center justify-between gap-3 my-4">
                            <div className="flex items-center gap-3 min-w-0">
                                <div className={`w-9 h-9 rounded-md flex items-center justify-center shrink-0 ${credState === "valid"
                                        ? "bg-green-500/10"
                                        : credState === "invalid"
                                            ? "bg-red-500/10"
                                            : "bg-[#5B8DEF]/10"
                                    }`}>
                                    {credState === "checking" && <SpinnerCircleIcon className="w-4.5 h-4.5 text-[#5B8DEF]" />}
                                    {credState === "valid" && <CheckCircleIcon className="w-4.5 h-4.5 text-green-500" />}
                                    {credState === "invalid" && <WarningCircleIcon className="w-4.5 h-4.5 text-red-500" />}
                                </div>
                                <div className="min-w-0">
                                    <p className="text-sm font-medium text-gray-100">
                                        {credState === "checking" && "Checking connection…"}
                                        {credState === "valid" && "Connected to Immich"}
                                        {credState === "invalid" && "Couldn't connect to Immich"}
                                    </p>
                                    <p className="text-xs text-gray-500 truncate">{savedUrl}</p>
                                </div>
                            </div>
                            <button
                                type="button"
                                onClick={() => setEditing(true)}
                                className="text-xs text-gray-400 hover:text-white transition-colors shrink-0"
                            >
                                Edit
                            </button>
                        </div>
                    ) : (
                        <form
                            className="grid space-y-2 my-4 min-w-0"
                            onSubmit={handleSaveCredentials}
                        >
                            <label htmlFor="url">Immich URL</label>
                            <div className="relative">
                                <input
                                    id="url"
                                    type="text"
                                    placeholder="https://immich.txuli.com"
                                    className={`w-full min-w-0 pr-9 ${urlIsValid ? "" : "border border-red-500 text-red-500"}`}
                                    value={urlInput}
                                    onChange={(e) => setUrlInput(e.target.value)}
                                />
                                {urlIsCorrect && (
                                    <CheckCircleIcon className="w-4 h-4 text-green-500 absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none" />
                                )}
                            </div>
                            {!urlIsValid && (
                                <p className="text-xs text-red-500 -mt-1.5">
                                    Enter a valid URL, e.g. https://immich.txuli.com
                                </p>
                            )}
                            <label htmlFor="key">Immich API key</label>
                            <input
                                id="key"
                                type="password"
                                placeholder="Paste your api key here"
                                className="w-full min-w-0"
                                value={tokenInput}
                                onChange={(e) => setTokenInput(e.target.value)}
                            />
                            <div className="mt-5 flex flex-wrap items-center gap-4">
                                <button
                                    type="submit"
                                    disabled={saving || !urlIsValid || urlInput.trim() === "" || tokenInput.trim() === ""}
                                    className="inline-flex items-center gap-1.5 bg-[#5B8DEF] text-white rounded-md px-3 py-1 disabled:cursor-not-allowed disabled:opacity-40"
                                >
                                    {saving && <SpinnerCircleIcon className="w-3.5 h-3.5" />}
                                    {saving ? "Verifying…" : "Save"}
                                </button>
                                {credState !== "unset" && (
                                    <button
                                        type="button"
                                        onClick={handleCancelEdit}
                                        className="text-xs text-gray-400 hover:text-white transition-colors"
                                    >
                                        Cancel
                                    </button>
                                )}

                                {response && !response.valid && response.type_acc == "credential" && (
                                    <p className="text-red-500">
                                        Your token is not valid.
                                    </p>
                                )}
                                {response?.valid && response.type_acc == "save" && (
                                    <p className="text-green-500">
                                        Settings saved successfully.
                                    </p>
                                )}

                                {response && !response.valid && response.type_acc == "save" && (
                                    <p className="text-red-500">
                                        Failed to save settings.
                                    </p>
                                )}
                                {response && !response.valid && response.type_acc == "empty" && (
                                    <p className="text-red-500">
                                        Please fill url and token fields.
                                    </p>
                                )}
                                {response && !response.valid && response.type_acc == "url" && (
                                    <p className="text-red-500">
                                        Enter a valid URL, e.g. https://immich.txuli.com
                                    </p>
                                )}
                            </div>
                        </form>
                    )}
                </ImmichForm>
                <ImmichForm>
                    <Options
                        checked={runInBackground}
                        title="Run on Startup"
                        description="Allow the app to start automatically when you log in."
                        onChange={toggle}
                    />
                    <Options
                        checked={notifications}
                        title="Notifications"
                        description="Receive alerts about upload progress, backup status, and potential errors."
                        onChange={(val) => save('notif', val)}
                    />
                    <Options
                        checked={removeAssets}
                        title="Remove Assets from SD Card"
                        description="Automatically delete local files from the SD card after they are successfully uploaded."
                        onChange={(val) => save('removeAssets', val)}
                    />

                </ImmichForm>
                <ImmichForm>
                    <div className="flex items-center justify-between py-4">
                        <div>
                            <h3 className="text-sm font-medium text-gray-200">App updates</h3>
                            <p className="text-xs text-gray-400 mt-0.5">
                                {updateStatus === "idle" && "Check for a new version of ImmichSync."}
                                {updateStatus === "checking" && "Checking for updates…"}
                                {updateStatus === "up-to-date" && "You're on the latest version."}
                                {updateStatus === "available" && `Version ${updateVersion} is available.`}
                                {updateStatus === "installing" && "Downloading and installing…"}
                                {updateStatus === "error" && `Could not check for updates: ${updateError}`}
                            </p>
                        </div>
                        {updateStatus !== "available" ? (
                            <button
                                onClick={checkForUpdates}
                                disabled={updateStatus === "checking" || updateStatus === "installing"}
                                className="rounded-md px-3 py-1 text-sm disabled:opacity-50"
                            >
                                Check for updates
                            </button>
                        ) : (
                            <button
                                onClick={installUpdate}
                                className="bg-[#5B8DEF] text-white rounded-md px-3 py-1 text-sm"
                            >
                                Install &amp; restart
                            </button>
                        )}
                    </div>
                </ImmichForm>
                {/* <ImmichForm>
                    <h2 className="text-xl  my-4 mb-1">Allowed extensions</h2>
                    <p className="text-gray-400 text-sm">Select the extensions that you want to sync.</p>
                    <form action="">

                    </form>
                </ImmichForm> */}
            </div>
        </div>
    )
}
