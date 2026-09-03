
import ImmichForm from "../components/ImmichForm"
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from "react";
import Options from "../components/options";
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

import { load } from '@tauri-apps/plugin-store';
import type { ValidResponse, StoredFlag } from '../types';
const store = await load('settings.json', { autoSave: true });
export default function config() {

    const [runInBackground, setRunInBackground] = useState(false);
    const [notifications, setNotifications] = useState(false);
    // const [removeAssets, setRemoveAssets] = useState(false);

    useEffect(() => {
        async function check() {
            setRunInBackground(await isEnabled())
        }
        async function load() {
            const notifData = await store.get<StoredFlag>('notif');
            setNotifications(notifData?.value ?? false);
            /* const rmAssets = await store.get<StoredFlag>('rmAssets');
            setRemoveAssets(rmAssets?.value ?? false) */
        }

        load()
        check()
    }, [])
    async function save(type: string, value: boolean) {
        switch (type) {
            case "notif":
                setNotifications(value)
                await store.set('notif', { value })
                break
            /* case "removeAssets":
                setRemoveAssets(value)
                await store.set('rmAssets', { value })
                break */
        }
    }
    async function toggle(value: boolean) {
        setRunInBackground(value);
        if (value) await enable()
        else await disable();
    }
    const [response, setResponse] = useState<ValidResponse | null>(null);

    type UpdateStatus = "idle" | "checking" | "available" | "up-to-date" | "installing" | "error";
    const [updateStatus, setUpdateStatus] = useState<UpdateStatus>("idle");
    const [updateVersion, setUpdateVersion] = useState<string | null>(null);
    const [updateError, setUpdateError] = useState<string | null>(null);

    async function checkForUpdates() {
        setUpdateStatus("checking");
        setUpdateError(null);
        try {
            const update = await check();
            if (update) {
                setUpdateVersion(update.version);
                setUpdateStatus("available");
            } else {
                setUpdateStatus("up-to-date");
            }
        } catch (err) {
            setUpdateError(String(err));
            setUpdateStatus("error");
        }
    }

    async function installUpdate() {
        setUpdateStatus("installing");
        setUpdateError(null);
        try {
            const update = await check();
            if (update) {
                await update.downloadAndInstall();
                await relaunch();
            }
        } catch (err) {
            setUpdateError(String(err));
            setUpdateStatus("error");
        }
    }


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
                    <h2 className="text-xl  my-4 mb-1">Immich credentials</h2>
                    <p className="text-gray-400 text-sm">Necessary to upload the assets of the USB</p>
                    <form
                        className="grid space-y-2 my-4 min-w-0"
                        onSubmit={(e) => {
                            e.preventDefault();
                            const url = (document.getElementById('url') as HTMLInputElement).value;
                            const token = (document.getElementById('key') as HTMLInputElement).value;
                            if (url === "" || token === "") {
                                setResponse({ valid: false, type_acc: "empty" })
                            } else {
                                invoke('save_credentials', { url, token }).then((res) => setResponse(res as ValidResponse))
                            }


                        }}
                    >
                        <label htmlFor="url"> Immich  URL</label>
                        <input id='url' type="text" placeholder=" https://immich.example.com" className="w-full min-w-0" />
                        <label htmlFor="key"> Immich  URL</label>
                        <input id="key" type="password" placeholder="Paste your api key here" className="w-full min-w-0" />
                        <div className="mt-5 flex flex-wrap items-center gap-3">
                            <button type="button" className=" rounded-md px-3 py-1" onClick={(e) => {
                                e.preventDefault();
                                const url = (document.getElementById('url') as HTMLInputElement).value;
                                const token = (document.getElementById('key') as HTMLInputElement).value;

                                if (url === "" || token === "") {
                                    setResponse({ valid: false, type_acc: "empty" })
                                } else {
                                    invoke('verify_token', { url, token }).then((res) => setResponse(res as ValidResponse))
                                }


                            }}> test connection </button>
                            <div className="flex flex-wrap items-center gap-4">
                                <button type="submit" className="bg-[#5B8DEF] text-white rounded-md px-3 py-1">
                                    save
                                </button>

                                {response?.valid && response.type_acc == "credential" && (
                                    <p className="text-green-500">
                                        Your token is valid you can save the config.
                                    </p>
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
                            </div>
                        </div>


                    </form>
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
                    {/* <Options
                        checked={removeAssets}
                        title="Remove Assets from SD Card"
                        description="Automatically delete local files from the SD card after they are successfully uploaded."
                        onChange={(val) => save('removeAssets', val)}
                    />  */}

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
