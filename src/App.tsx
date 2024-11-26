import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart';
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [autoStartMsg, setAutoStartMsg] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  async function toggle_autostart() {
    let isAutoStart = await isEnabled();
    if (isAutoStart === true) {
      await disable();
    } else {
      await enable();
    }

    isAutoStart = await isEnabled();
    setAutoStartMsg(`is auto start ${isAutoStart}`)
  }

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
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg}</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          toggle_autostart();
        }}
      >
        <span>AutoStart</span>
        <button type="submit">Toggle</button>
      </form>
      <p>{autoStartMsg}</p>
    </main>
  );
}

export default App;
