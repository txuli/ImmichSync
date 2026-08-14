import { useState } from "react";
import "./App.css";
import NavBar, { type View } from "./components/navBar";
import Dashboard from "./pages/dashboard";
import Config from "./pages/config";

function App() {
  const [view, setView] = useState<View>("dashboard");

  return (
    <div className="flex h-full">
      <NavBar active={view} onSelect={setView} />
      <div className="flex-1 min-w-0 h-full overflow-auto">
        {view === "dashboard" && <Dashboard />}
        {view === "config" && <Config />}
      </div>
    </div>
  );
}

export default App;
