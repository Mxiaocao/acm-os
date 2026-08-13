import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { App } from "./app/App";
import "./app/app.css";

const root = document.getElementById("root");

if (!root) {
  throw new Error("Missing #root element");
}

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);

if (import.meta.env.VITE_ACM_OS_DESKTOP_E2E === "1") {
  void invoke("desktop_e2e_log", { input: { stage: "main-entry-started" } });
  // @ts-expect-error The desktop E2E driver is a test-only JavaScript asset bundled by Vite.
  void import("../src-tauri/src/desktop_e2e.js");
}
