import ImmichForm from "./ImmichForm"
export default function config() {
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
                <p className="text-gray-400 text-sm">Necessary to upload  the  assets of the USB</p>
                <form
                    className="grid space-y-2 my-4"
                    onSubmit={(e) => {
                        e.preventDefault();
                        // TODO: guardar credenciales
                    }}
                >
                    <label htmlFor="url"> Immich  URL</label>
                    <input id='url' type="text" placeholder=" https://immich.example.com" />
                    <label htmlFor="key"> Immich  URL</label>
                    <input id="key" type="password" placeholder="Paste your api key here"/>
                    <div className="mt-5">
                        <button type="button" className=" rounded-md px-3 py-1 mr-5"> test connection </button>
                        <button type="submit" className="bg-[#5B8DEF] text-white rounded-md px-3 py-1"> save </button>
                    </div>
                </form>
            </ImmichForm>
           </div>
        </div>
    )
}
