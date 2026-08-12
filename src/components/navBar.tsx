import { useState } from "react"

export type View = "dashboard" | "config";

interface NavBarProps {
    active: View;
    onSelect: (view: View) => void;
}

export default function navBar({ active, onSelect }: NavBarProps) {
    const [isConnected] = useState(false)
    return (
        <>
            <div className="bg-[#1A1D24] h-full  w-1/5 relative ">
                <div className="h-5/6">
                    <div className="pt-4">
                        <h1 className="text-2xl">Immich Sync</h1>
                        <p className="text-gray-500 text-center">USB to immich</p>
                    </div>
                    <div className="grid space-y-4">
                        <button
                            onClick={() => onSelect("dashboard")}
                            className={`w-3/4 mx-auto rounded-lg py-2 transition-colors ${active === "dashboard" ? "bg-[#2A2E3A] text-white" : "text-gray-400 hover:bg-[#20232B]"}`}
                        >
                            Dashboard
                        </button>
                        <button
                            onClick={() => onSelect("config")}
                            className={`w-3/4 mx-auto rounded-lg py-2 transition-colors ${active === "config" ? "bg-[#2A2E3A] text-white" : "text-gray-400 hover:bg-[#20232B]"}`}
                        >
                            Config
                        </button>
                    </div>
                </div>

                <div>

                </div>
                <div className=" bottom-0 bg-[#20232B] h-1/9 w-3/4 rounded-2xl text-md  mx-auto border border-gray-500">
                    {
                        isConnected && (
                            <div className="text-center mt-3">
                                Device connected and uploading
                            </div>
                        )
                    }
                    {
                        !isConnected && (
                            <div className="text-center mt-3">
                                no device to upload
                                <div className=" text-sm text-gray-400">
                                    please connect a device  to  start uploading
                                </div>
                            </div>
                        )
                    }
                </div>
            </div>
        </>)
}