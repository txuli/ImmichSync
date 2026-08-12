import ImmichForm from "./ImmichForm"
import { invoke } from '@tauri-apps/api/core';
import { useState } from "react";
export default function config() {
    const [response, setResponse] = useState<{  valid: boolean, type_acc :string  } | null>(null);
     
    return (
        <div className="relative h-full bg-[#15171C]">
            <div className="p-6  ">
                <h2 className="text-2xl font-semibold">Config</h2>
                <p className="text-gray-400 mt-2">
                    Ajustes de la conexión con Immich.
                </p>


            </div>
            <div className="border-t-2 w-full border-t-[#272A31]  absolute"></div>
            <div className=" mt-5 ml-4">
                <ImmichForm>
                    <h2 className="text-xl  my-4 mb-1">Immich credentials</h2>
                    <p className="text-gray-400 text-sm">Necessary to upload the assets of the USB</p>
                    <form
                        className="grid space-y-2 my-4"
                        onSubmit={(e) => {
                            e.preventDefault();
                            const url = (document.getElementById('url') as HTMLInputElement).value;
                            const token = (document.getElementById('key') as HTMLInputElement).value;
                            invoke('save_credentials', { url, token }).then((res) => setResponse(res as { valid: boolean, type_acc: string }))
                        }}
                    >
                        <label htmlFor="url"> Immich  URL</label>
                        <input id='url' type="text" placeholder=" https://immich.example.com" />
                        <label htmlFor="key"> Immich  URL</label>
                        <input id="key" type="password" placeholder="Paste your api key here" />
                        <div className="mt-5 flex">
                            <button type="button" className=" rounded-md px-3 py-1 mr-5" onClick={(e) => {
                                e.preventDefault();
                                const url = (document.getElementById('url') as HTMLInputElement).value;
                                const token = (document.getElementById('key') as HTMLInputElement).value;
                                invoke('verify_token', { url, token }).then((res) => setResponse(res as { valid: boolean, type_acc: string }))

                            }}> test connection </button>
                            <div className="flex items-center gap-4">
                                <button type="submit" className="bg-[#5B8DEF] text-white rounded-md px-3 py-1">
                                    save
                                </button>

                                {response?.valid  && response.type_acc=="credential"  && (
                                    <p className="text-green-500">
                                        Your token is valid you can save the config.
                                    </p>
                                )}

                                {response && !response.valid && response.type_acc=="credential" && (
                                    <p className="text-red-500">
                                        Your token is not valid.
                                    </p>
                                )}
                                {response?.valid  && response.type_acc=="save"  && (
                                    <p className="text-green-500">
                                        Settings saved successfully.
                                    </p>
                                )}

                                {response && !response.valid && response.type_acc=="save" && (
                                    <p className="text-red-500">
                                        Failed to save settings.
                                    </p>
                                )}
                            </div>
                        </div>


                    </form>
                </ImmichForm>
            </div>
        </div>
    )
}
