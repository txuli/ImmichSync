import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import logo from "../assets/logo.svg";

const appWindow = getCurrentWindow();

function MinimizeIcon() {
    return (
        <svg viewBox="0 0 12 12" fill="none" className="size-3.5">
            <path d="M2.5 6h7" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
        </svg>
    );
}

function MaximizeIcon() {
    return (
        <svg viewBox="0 0 12 12" fill="none" className="size-3.5">
            <rect x="2.25" y="2.25" width="7.5" height="7.5" rx="1" stroke="currentColor" strokeWidth="1.3" />
        </svg>
    );
}

function RestoreIcon() {
    return (
        <svg viewBox="0 0 12 12" fill="none" className="size-3.5">
            <rect x="3.5" y="1.5" width="6" height="6" rx="1" stroke="currentColor" strokeWidth="1.2" />
            <path d="M2.5 4.5H2A0.5 0.5 0 0 0 1.5 5v5a0.5 0.5 0 0 0 0.5 0.5h5a0.5 0.5 0 0 0 0.5 -0.5v-0.5" stroke="currentColor" strokeWidth="1.2" strokeLinejoin="round" />
        </svg>
    );
}

function CloseIcon() {
    return (
        <svg viewBox="0 0 12 12" fill="none" className="size-3.5">
            <path d="M2.5 2.5l7 7M9.5 2.5l-7 7" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
        </svg>
    );
}

export default function TitleBar() {
    const [isMaximized, setIsMaximized] = useState(false);

    useEffect(() => {
        appWindow.isMaximized().then(setIsMaximized);
        const unlisten = appWindow.onResized(() => {
            appWindow.isMaximized().then(setIsMaximized);
        });
        return () => {
            unlisten.then((fn) => fn());
        };
    }, []);

    return (
        <div
            data-tauri-drag-region
            className="h-9 shrink-0 w-full flex items-center justify-between bg-[#1A1D24] border-b border-white/5 select-none"
        >
            <div data-tauri-drag-region className="flex items-center gap-2 pl-3 h-full">
                <img src={logo} alt="" className="w-4 h-4 pointer-events-none" />
                <span className="text-xs font-medium text-gray-400 pointer-events-none tracking-wide">
                    Immich Sync
                </span>
            </div>

            <div className="flex items-center gap-1 h-full pr-1.5">
                <button
                    aria-label="Minimize"
                    onClick={() => appWindow.minimize()}
                    className="size-5 p-0 border-0 shadow-none flex items-center justify-center rounded-full bg-transparent text-gray-400 hover:bg-white/10 hover:text-gray-100 transition-colors"
                >
                    <MinimizeIcon />
                </button>
                <button
                    aria-label={isMaximized ? "Restore" : "Maximize"}
                    onClick={() => appWindow.toggleMaximize()}
                    className="size-5 p-0 border-0 shadow-none flex items-center justify-center rounded-full bg-transparent text-gray-400 hover:bg-white/10 hover:text-gray-100 transition-colors"
                >
                    {isMaximized ? <RestoreIcon /> : <MaximizeIcon />}
                </button>
                <button
                    aria-label="Close"
                    onClick={() => appWindow.close()}
                    className="size-5 p-0 border-0 shadow-none flex items-center justify-center rounded-full bg-transparent text-gray-400 hover:bg-[#E5484D] hover:text-white transition-colors"
                >
                    <CloseIcon />
                </button>
            </div>
        </div>
    );
}
