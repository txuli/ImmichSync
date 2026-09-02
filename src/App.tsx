import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import "./App.css";
import NavBar, { type View } from "./components/navBar";
import TitleBar from "./components/titleBar";
import Dashboard from "./pages/dashboard";
import Config from "./pages/config";

function App() {
  const [view, setView] = useState<View>("dashboard");
  const [newDeviceName, setNewDeviceName] = useState("");
  const [newDevicePath, setNewDevicePath] = useState("");

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
        </div>
      </div>
    </div>
  );
}

export default App;
