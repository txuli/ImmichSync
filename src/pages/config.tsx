
import ImmichForm from "../components/ImmichForm"
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from "react";
import Options from "../components/options";
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart';
export default function config() {
    const [runInBackground, setRunInBackground] = useState(false);
    const [notifications, setNotifications] = useState(true);
    const [removeAssets, setRemoveAssets] = useState(true);

    useEffect(() => {
        async function check() {
            setRunInBackground(await isEnabled())
        }
        check()
    }, [])

    async function toggle(value: boolean) {
        setRunInBackground(value);
        if (value) await enable()
        else await disable();
    }
    const [response, setResponse] = useState<{ valid: boolean, type_acc: string } | null>(null);

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
                            invoke('save_credentials', { url, token }).then((res) => setResponse(res as { valid: boolean, type_acc: string }))
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
                                invoke('verify_token', { url, token }).then((res) => setResponse(res as { valid: boolean, type_acc: string }))

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
                            </div>
                        </div>


                    </form>
                </ImmichForm>
                <ImmichForm>
                    <Options
                        checked={runInBackground}
                        title="Run in Background"
                        description="Allow the app to process photos and sync media automatically in the background."
                        onChange={toggle}
                    />
                    <Options
                        checked={notifications}
                        title="Notifications"
                        description="Receive alerts about upload progress, backup status, and potential errors."
                        onChange={setNotifications}
                    />
                    <Options
                        checked={removeAssets}
                        title="Remove Assets from SD Card"
                        description="Automatically delete local files from the SD card after they are successfully uploaded."
                        onChange={setRemoveAssets}
                    />

                </ImmichForm>
            </div>
        </div>
    )
}
