import { useEffect, useState, type ReactElement } from "react"
import { load } from "@tauri-apps/plugin-store"
import logo from "../assets/logo.svg"
import config from "../assets/config.svg"
import dashboard from "../assets/dashboard.svg"
import upload from "../assets/upload.svg"
import device from "../assets/devices.svg"
import { useSyncStore } from "../store/syncStore"
import type { View, StoredFlag } from "../types"
const store = await load('settings.json', { autoSave: true });
interface NavBarProps {
    active: View;
    onSelect: (view: View) => void;
}

function DashboardIcon({ className }: { className?: string }) {
    return (
       <img src={dashboard} alt="dashboard" className={className}/>
    )
}

function ConfigIcon({ className }: { className?: string }) {
    return <img src={config} alt="Config" className={className} />
}
function UploadIcon({ className }: { className?: string }) {
      return <img src={upload} alt="Config" className={className} />
}

function DeviceIcon({ className }: { className?: string }) {
      return <img src={device} alt="Config" className={className} />
}

export default function navBar({ active, onSelect }: NavBarProps) {
    const isConnected = useSyncStore((s) => s.isConnected)
    const [_notifications, setNotifications] = useState(false)

    useEffect(() => {
        async function loadConfig(){
           const notif = await store.get<StoredFlag>('notif')
           setNotifications(notif?.value ?? false)
        }
        loadConfig()

        const unlistenNotif = store.onKeyChange<StoredFlag>('notif', (notif) => {
            setNotifications(notif?.value ?? false)
        })

        return () => {
            unlistenNotif.then((fn) => fn())
        }
    }, [])

    const items: { view: View; label: string; icon: (props: { className?: string }) => ReactElement }[] = [
        { view: "dashboard", label: "Dashboard", icon: DashboardIcon },
        { view: "manualUpload", label: "Manual upload", icon: UploadIcon },
        { view: "config", label: "Config", icon: ConfigIcon },
        { view: "device", label: "Known devices", icon: DeviceIcon },
    ]

    return (
        <div className="bg-[#1A1D24] h-full w-56 shrink-0 flex flex-col border-r border-white/5">
            <div className="flex-1 overflow-auto">
                <div className="pt-6 pb-5 px-4 flex flex-col items-center border-b border-white/5">
                    <img src={logo} alt="Immich Sync" className="w-12 h-12 drop-shadow-[0_0_12px_rgba(91,141,239,0.35)]" />
                    <h1 className="text-lg font-semibold text-center mt-3 tracking-tight text-gray-100">Immich Sync</h1>
                    <p className="text-gray-500 text-center text-xs mt-0.5 tracking-wide uppercase">USB to Immich</p>
                </div>
                <nav className="flex flex-col gap-1 mt-4 px-3">
                    {items.map(({ view, label, icon: Icon }) => {
                        const isActive = active === view
                        return (
                            <button
                                key={view}
                                onClick={() => onSelect(view)}
                                className={`group relative flex items-center gap-3 rounded-lg py-2.5 px-3 text-sm font-medium transition-all duration-150 ${isActive
                                        ? "bg-[#5B8DEF]/10 text-white"
                                        : "text-gray-400 hover:bg-white/5 hover:text-gray-200"
                                    }`}
                            >
                                <span
                                    className={`absolute left-0 top-1/2 -translate-y-1/2 h-5 w-0.75 rounded-full bg-[#5B8DEF] transition-all duration-150 ${isActive ? "opacity-100 scale-y-100" : "opacity-0 scale-y-50"
                                        }`}
                                />
                                <Icon
                                    className={`w-4.5 h-4.5 shrink-0 transition-colors ${isActive ? "text-[#5B8DEF]" : "text-gray-500 group-hover:text-gray-300"
                                        }`}
                                />
                                {label}
                            </button>
                        )
                    })}
                </nav>
            </div>

            <div className="mx-3 mb-4 rounded-xl border border-white/5 bg-[#20232B]/80 backdrop-blur-sm p-3.5 shadow-[0_1px_0_rgba(255,255,255,0.03)]">
                <div className="flex items-center gap-2">
                    <span className="relative flex h-2 w-2">
                        {isConnected && (
                            <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-[#4ADE80] opacity-75" />
                        )}
                        <span className={`relative inline-flex h-2 w-2 rounded-full ${isConnected ? "bg-[#4ADE80]" : "bg-gray-600"}`} />
                    </span>
                    <span className="text-sm text-gray-200 font-medium">
                        {isConnected ? "Device connected" : "No device"}
                    </span>
                </div>
                <p className="text-xs text-gray-500 mt-1 leading-snug">
                    {isConnected ? "Uploading assets to Immich…" : "Connect a device to start uploading."}
                </p>
            </div>
        </div>
    )
}
