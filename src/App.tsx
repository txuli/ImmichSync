import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import Database from "@tauri-apps/plugin-sql";
import "./App.css";
import NavBar, { type View } from "./components/navBar";
import TitleBar from "./components/titleBar";
import Dashboard from "./pages/dashboard";
import Config from "./pages/config";
import ManualUpload from "./pages/manualUpload";
import NewDevice from "./pages/newDevice";
import Decices from "./pages/devices";
function App() {
  const [view, setView] = useState<View>("dashboard");
  const [newDeviceName, setNewDeviceName] = useState("");
  const [newDevicePath, setNewDevicePath] = useState("");

  useEffect(() => {
   
    Database.load("sqlite:immichsync.db").catch((err) => {
      console.error("Failed to initialize immichsync.db:", err);
    });
  }, []);

  useEffect(() => {
    const unlisten = listen<{ diskName: string; mountPoint: string }>(
      "navigate-new-device",
      (event) => {
        setNewDeviceName(event.payload.diskName);
        setNewDevicePath(event.payload.mountPoint);
        setView("newDevice");
      }
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div className="flex flex-col h-full">
      <TitleBar />
      <div className="flex flex-1 min-h-0">
        <NavBar active={view} onSelect={setView} />
        <div className="flex-1 min-w-0 h-full overflow-auto">
          {view === "dashboard" && <Dashboard />}
          {view === "config" && <Config />}
          {view === "manualUpload" && <ManualUpload />}
          {view === "newDevice" && (
            <NewDevice
              device={newDeviceName}
              mountPoint={newDevicePath}
              onDone={() => setView("dashboard")}
            />
          )}
          {view === "device" && <Decices />}
        </div>
      </div>
    </div>
  );
}

export default App;
