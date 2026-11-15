import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import ErrorBoundary from "./components/ErrorBoundary";
import { loadPrefs } from "./lib/prefs";
import "./index.css";

// Gelistirmede Tauri disinda (duz tarayici) acilirsa ornek veri donduren taklit yuklenir.
// `import.meta.env.DEV` uretimde false oldugundan bu dal ve dosya pakete girmez.
async function boot() {
  if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) await import("./dev/tauriMock");
  await loadPrefs(); // Hook'lar tercihleri eszamanli okur; once yuklenmeli.
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <ErrorBoundary><App /></ErrorBoundary>
    </React.StrictMode>,
  );
}

void boot();
