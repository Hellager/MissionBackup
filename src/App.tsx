import { useState, useEffect } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import { listen } from '@tauri-apps/api/event';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import "./App.css";

function App() {
  const [hintMsg, setHintMsg] = useState("");
  const [name, setName] = useState("");

  async function updateAutostart() {
    let autoStartEnabled = await isEnabled();
    if (autoStartEnabled) {
      await disable();
    } else {
      await enable();
    }
    setHintMsg(`Autostart is now ${autoStartEnabled ? 'disabled' : 'enabled'}`);
  }

  useEffect(() => {
    setTimeout(async () => {
      await listen('another_instance', (_) => {
        alert('another_instance');
      });
    }, 0);
  }, []);

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      <div className="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          updateAutostart();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Toggle Autostart</button>
      </form>
      <p>{hintMsg}</p>
    </main>
  );
}

export default App;
