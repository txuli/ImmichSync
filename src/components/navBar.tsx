import { useState } from "react"
import logo from "../assets/logo.svg"

export type View = "dashboard" | "config";

interface NavBarProps {
    active: View;
    onSelect: (view: View) => void;
}

export default function navBar({ active, onSelect }: NavBarProps) {
    const [isConnected] = useState(false)
    return (
        <div className="bg-[#1A1D24] h-full w-56 shrink-0 flex flex-col">
            <div className="flex-1 overflow-auto">
                <div className="pt-4 px-2 flex flex-col items-center">
                    <img src={logo} alt="Immich Sync" className="w-14 h-14" />
                    <h1 className="text-2xl text-center mt-2">Immich Sync</h1>
                    <p className="text-gray-500 text-center">USB to immich</p>
                </div>
                <div className="grid space-y-4 mt-4">
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

            <div className="bg-[#20232B] w-3/4 mx-auto mb-4 rounded-2xl text-md border border-gray-500 p-3">
                {
                    isConnected ? (
                        <div className="text-center">
                            Device connected and uploading
                        </div>
                    ) : (
                        <div className="text-center">
                            no device to upload
                            <div className="text-sm text-gray-400">
                                please connect a device to start uploading
                            </div>
                        </div>
                    )
                }
            </div>
        </div>)
}