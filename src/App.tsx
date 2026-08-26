import { useState } from "react";
import "./App.css";
import NavBar, { type View } from "./components/navBar";
import TitleBar from "./components/titleBar";
import Dashboard from "./pages/dashboard";
import Config from "./pages/config";
import ManualUpload from "./pages/manualUpload";
function App() {
  const [view, setView] = useState<View>("dashboard");

  return (
    <div className="flex flex-col h-full">
      <TitleBar />
      <div className="flex flex-1 min-h-0">
        <NavBar active={view} onSelect={setView} />
        <div className="flex-1 min-w-0 h-full overflow-auto">
          {view === "dashboard" && <Dashboard />}
          {view === "config" && <Config />}
          {view === "manualUpload" && <ManualUpload />}
        </div>
      </div>
    </div>
  );
}

export default App;
